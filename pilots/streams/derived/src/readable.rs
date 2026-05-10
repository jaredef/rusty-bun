// ReadableStream — WHATWG Streams Standard §4.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum StreamError {
    AlreadyLocked,
    NotLocked,
    Closed,
    Errored(String),
    ReleasedReader,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Readable,
    Closed,
    Errored,
}

#[derive(Debug)]
pub enum ReadResult<T> {
    Chunk(T),
    Done,
    Pending,
    Error(String),
}

struct Inner<T> {
    queue: VecDeque<T>,
    state: State,
    error: Option<String>,
    locked: bool,
    high_water_mark: usize,
    /// Bookkeeping for tee branches: when both branches have been
    /// cancelled, propagate to source.
    cancel_count: u8,
    cancel_target: u8,
}

impl<T> Inner<T> {
    fn new(hwm: usize) -> Self {
        Self {
            queue: VecDeque::new(),
            state: State::Readable,
            error: None,
            locked: false,
            high_water_mark: hwm,
            cancel_count: 0,
            cancel_target: 1,
        }
    }
}

pub trait UnderlyingSource<T> {
    /// SPEC: optional; called once when the stream is constructed. Pilot
    /// invokes synchronously at construction.
    fn start(&mut self, _ctrl: &Controller<T>) {}
    /// SPEC: called when the stream wants more data. Pilot invokes when the
    /// queue drops below the high-water-mark.
    fn pull(&mut self, _ctrl: &Controller<T>) {}
    /// SPEC: called when the stream is cancelled.
    fn cancel(&mut self, _reason: Option<String>) {}
}

/// Default no-op source for streams whose chunks are enqueued externally.
pub struct ManualSource;
impl<T> UnderlyingSource<T> for ManualSource {}

type SourceRef<T> = Rc<RefCell<Box<dyn UnderlyingSource<T>>>>;

pub struct ReadableStream<T> {
    inner: Rc<RefCell<Inner<T>>>,
    source: SourceRef<T>,
}

impl<T: Clone + 'static> ReadableStream<T> {
    pub fn new(source: Box<dyn UnderlyingSource<T>>, high_water_mark: usize) -> Self {
        let inner = Rc::new(RefCell::new(Inner::new(high_water_mark)));
        let source: SourceRef<T> = Rc::new(RefCell::new(source));
        // SPEC §4.constructor: invoke start synchronously.
        let ctrl = Controller { inner: inner.clone() };
        source.borrow_mut().start(&ctrl);
        Self { inner, source }
    }

    pub fn manual(high_water_mark: usize) -> (Self, Controller<T>) {
        let inner = Rc::new(RefCell::new(Inner::new(high_water_mark)));
        let stream = Self {
            inner: inner.clone(),
            source: Rc::new(RefCell::new(Box::new(ManualSource) as Box<dyn UnderlyingSource<T>>)),
        };
        (stream, Controller { inner })
    }

    /// SPEC §4.locked.
    pub fn locked(&self) -> bool { self.inner.borrow().locked }

    /// SPEC §4.cancel.
    pub fn cancel(&self, reason: Option<String>) -> Result<(), StreamError> {
        if self.inner.borrow().locked { return Err(StreamError::AlreadyLocked); }
        self.do_cancel(reason);
        Ok(())
    }

    fn do_cancel(&self, reason: Option<String>) {
        let mut inner = self.inner.borrow_mut();
        inner.state = State::Closed;
        inner.queue.clear();
        drop(inner);
        self.source.borrow_mut().cancel(reason);
    }

    /// SPEC §4.getReader: returns a default reader, locks the stream.
    pub fn get_reader(&self) -> Result<Reader<T>, StreamError> {
        let mut inner = self.inner.borrow_mut();
        if inner.locked { return Err(StreamError::AlreadyLocked); }
        inner.locked = true;
        Ok(Reader {
            inner: self.inner.clone(),
            source: self.source.clone(),
            released: false,
        })
    }

    /// SPEC §4.tee. Pilot semantics: branches share a snapshot queue and a
    /// coordinated cancellation count. Both must cancel for source.cancel
    /// to fire.
    pub fn tee(&self) -> Result<(ReadableStream<T>, ReadableStream<T>), StreamError> {
        let mut inner = self.inner.borrow_mut();
        if inner.locked { return Err(StreamError::AlreadyLocked); }
        inner.locked = true;
        let snapshot: Vec<T> = inner.queue.iter().cloned().collect();
        let state = inner.state;
        let error = inner.error.clone();
        let hwm = inner.high_water_mark;
        drop(inner);

        let make_branch = || {
            let inner = Rc::new(RefCell::new(Inner::new(hwm)));
            {
                let mut i = inner.borrow_mut();
                for c in &snapshot { i.queue.push_back(c.clone()); }
                i.state = state;
                i.error = error.clone();
                i.cancel_target = 2;
            }
            ReadableStream {
                inner,
                source: Rc::new(RefCell::new(
                    Box::new(ManualSource) as Box<dyn UnderlyingSource<T>>
                )),
            }
        };
        Ok((make_branch(), make_branch()))
    }
}

impl<T> std::fmt::Debug for ReadableStream<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReadableStream")
            .field("locked", &self.inner.borrow().locked)
            .field("state", &self.inner.borrow().state)
            .finish()
    }
}

// ─────────── Reader ────────────

pub struct Reader<T> {
    inner: Rc<RefCell<Inner<T>>>,
    source: SourceRef<T>,
    released: bool,
}

impl<T: Clone + 'static> Reader<T> {
    /// SPEC §4.read. Pure-Rust analog returns a synchronous ReadResult
    /// rather than a Promise.
    pub fn read(&mut self) -> ReadResult<T> {
        if self.released { return ReadResult::Error("released reader".into()); }
        let chunk = self.inner.borrow_mut().queue.pop_front();
        if let Some(c) = chunk {
            let need_pull = {
                let i = self.inner.borrow();
                i.queue.len() < i.high_water_mark && i.state == State::Readable
            };
            if need_pull {
                let ctrl = Controller { inner: self.inner.clone() };
                self.source.borrow_mut().pull(&ctrl);
            }
            return ReadResult::Chunk(c);
        }
        let state = self.inner.borrow().state;
        match state {
            State::Closed => ReadResult::Done,
            State::Errored => {
                let msg = self.inner.borrow().error.clone().unwrap_or_default();
                ReadResult::Error(msg)
            }
            State::Readable => {
                let ctrl = Controller { inner: self.inner.clone() };
                self.source.borrow_mut().pull(&ctrl);
                if let Some(c) = self.inner.borrow_mut().queue.pop_front() {
                    return ReadResult::Chunk(c);
                }
                let s = self.inner.borrow().state;
                match s {
                    State::Closed => ReadResult::Done,
                    State::Errored => {
                        let msg = self.inner.borrow().error.clone().unwrap_or_default();
                        ReadResult::Error(msg)
                    }
                    State::Readable => ReadResult::Pending,
                }
            }
        }
    }

    /// SPEC §4.reader.cancel.
    pub fn cancel(&mut self, reason: Option<String>) -> Result<(), StreamError> {
        if self.released { return Err(StreamError::ReleasedReader); }
        let mut inner = self.inner.borrow_mut();
        let target = inner.cancel_target;
        inner.cancel_count += 1;
        let propagate = inner.cancel_count >= target;
        inner.state = State::Closed;
        inner.queue.clear();
        inner.locked = false;
        drop(inner);
        if propagate {
            self.source.borrow_mut().cancel(reason);
        }
        self.released = true;
        Ok(())
    }

    /// SPEC §4.reader.releaseLock.
    pub fn release_lock(&mut self) -> Result<(), StreamError> {
        if self.released { return Err(StreamError::ReleasedReader); }
        self.inner.borrow_mut().locked = false;
        self.released = true;
        Ok(())
    }
}

// ─────────── Controller ────────────

pub struct Controller<T> {
    inner: Rc<RefCell<Inner<T>>>,
}

impl<T> Controller<T> {
    /// SPEC §4.controller.enqueue.
    pub fn enqueue(&self, chunk: T) -> Result<(), StreamError> {
        let mut inner = self.inner.borrow_mut();
        match inner.state {
            State::Closed => Err(StreamError::Closed),
            State::Errored => Err(StreamError::Errored(
                inner.error.clone().unwrap_or_default()
            )),
            State::Readable => {
                inner.queue.push_back(chunk);
                Ok(())
            }
        }
    }

    /// SPEC §4.controller.close.
    pub fn close(&self) -> Result<(), StreamError> {
        let mut inner = self.inner.borrow_mut();
        if inner.state == State::Closed { return Err(StreamError::Closed); }
        inner.state = State::Closed;
        Ok(())
    }

    /// SPEC §4.controller.error.
    pub fn error(&self, reason: impl Into<String>) {
        let mut inner = self.inner.borrow_mut();
        inner.state = State::Errored;
        inner.error = Some(reason.into());
        inner.queue.clear();
    }

    pub fn desired_size(&self) -> i64 {
        let inner = self.inner.borrow();
        inner.high_water_mark as i64 - inner.queue.len() as i64
    }
}
