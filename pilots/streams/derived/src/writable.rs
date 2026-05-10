// WritableStream — WHATWG Streams Standard §5.

use std::cell::RefCell;
use std::rc::Rc;

use crate::readable::StreamError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WState {
    Writable,
    Closing,
    Closed,
    Errored,
}

struct WInner {
    state: WState,
    error: Option<String>,
    locked: bool,
}

impl WInner {
    fn new() -> Self {
        Self { state: WState::Writable, error: None, locked: false }
    }
}

pub trait UnderlyingSink<T> {
    fn start(&mut self) {}
    /// SPEC: write callback returns success or an error reason. Pilot uses
    /// Result so the verifier can probe error propagation.
    fn write(&mut self, chunk: T) -> Result<(), String>;
    fn close(&mut self) -> Result<(), String> { Ok(()) }
    fn abort(&mut self, _reason: Option<String>) {}
}

type SinkRef<T> = Rc<RefCell<Box<dyn UnderlyingSink<T>>>>;

pub struct WritableStream<T> {
    inner: Rc<RefCell<WInner>>,
    sink: SinkRef<T>,
}

impl<T: 'static> WritableStream<T> {
    pub fn new(sink: Box<dyn UnderlyingSink<T>>) -> Self {
        let inner = Rc::new(RefCell::new(WInner::new()));
        let sink: SinkRef<T> = Rc::new(RefCell::new(sink));
        sink.borrow_mut().start();
        Self { inner, sink }
    }

    /// SPEC §5.locked.
    pub fn locked(&self) -> bool { self.inner.borrow().locked }

    /// SPEC §5.abort.
    pub fn abort(&self, reason: Option<String>) -> Result<(), StreamError> {
        if self.inner.borrow().locked { return Err(StreamError::AlreadyLocked); }
        let mut inner = self.inner.borrow_mut();
        inner.state = WState::Errored;
        inner.error = Some(reason.clone().unwrap_or_default());
        drop(inner);
        self.sink.borrow_mut().abort(reason);
        Ok(())
    }

    /// SPEC §5.close.
    pub fn close(&self) -> Result<(), StreamError> {
        if self.inner.borrow().locked { return Err(StreamError::AlreadyLocked); }
        self.do_close()
    }

    fn do_close(&self) -> Result<(), StreamError> {
        let state = self.inner.borrow().state;
        match state {
            WState::Closed | WState::Closing => Err(StreamError::Closed),
            WState::Errored => Err(StreamError::Errored(
                self.inner.borrow().error.clone().unwrap_or_default()
            )),
            WState::Writable => {
                self.inner.borrow_mut().state = WState::Closing;
                let r = self.sink.borrow_mut().close();
                match r {
                    Ok(()) => {
                        self.inner.borrow_mut().state = WState::Closed;
                        Ok(())
                    }
                    Err(msg) => {
                        let mut i = self.inner.borrow_mut();
                        i.state = WState::Errored;
                        i.error = Some(msg.clone());
                        Err(StreamError::Errored(msg))
                    }
                }
            }
        }
    }

    /// SPEC §5.getWriter: locks the stream + returns a Writer.
    pub fn get_writer(&self) -> Result<Writer<T>, StreamError> {
        let mut inner = self.inner.borrow_mut();
        if inner.locked { return Err(StreamError::AlreadyLocked); }
        inner.locked = true;
        Ok(Writer {
            inner: self.inner.clone(),
            sink: self.sink.clone(),
            released: false,
        })
    }
}

// ─────────── Writer ────────────

pub struct Writer<T> {
    inner: Rc<RefCell<WInner>>,
    sink: SinkRef<T>,
    released: bool,
}

impl<T: 'static> Writer<T> {
    /// SPEC §5.writer.write.
    pub fn write(&mut self, chunk: T) -> Result<(), StreamError> {
        if self.released { return Err(StreamError::ReleasedReader); }
        let state = self.inner.borrow().state;
        match state {
            WState::Closed | WState::Closing => Err(StreamError::Closed),
            WState::Errored => Err(StreamError::Errored(
                self.inner.borrow().error.clone().unwrap_or_default()
            )),
            WState::Writable => {
                let r = self.sink.borrow_mut().write(chunk);
                if let Err(msg) = r {
                    let mut i = self.inner.borrow_mut();
                    i.state = WState::Errored;
                    i.error = Some(msg.clone());
                    return Err(StreamError::Errored(msg));
                }
                Ok(())
            }
        }
    }

    /// SPEC §5.writer.close.
    pub fn close(&mut self) -> Result<(), StreamError> {
        if self.released { return Err(StreamError::ReleasedReader); }
        let state = self.inner.borrow().state;
        match state {
            WState::Writable => {
                self.inner.borrow_mut().state = WState::Closing;
                let r = self.sink.borrow_mut().close();
                match r {
                    Ok(()) => {
                        let mut i = self.inner.borrow_mut();
                        i.state = WState::Closed;
                        i.locked = false;
                        self.released = true;
                        Ok(())
                    }
                    Err(msg) => {
                        let mut i = self.inner.borrow_mut();
                        i.state = WState::Errored;
                        i.error = Some(msg.clone());
                        Err(StreamError::Errored(msg))
                    }
                }
            }
            _ => Err(StreamError::Closed),
        }
    }

    pub fn abort(&mut self, reason: Option<String>) -> Result<(), StreamError> {
        if self.released { return Err(StreamError::ReleasedReader); }
        let mut i = self.inner.borrow_mut();
        i.state = WState::Errored;
        i.error = Some(reason.clone().unwrap_or_default());
        i.locked = false;
        drop(i);
        self.sink.borrow_mut().abort(reason);
        self.released = true;
        Ok(())
    }

    pub fn release_lock(&mut self) -> Result<(), StreamError> {
        if self.released { return Err(StreamError::ReleasedReader); }
        self.inner.borrow_mut().locked = false;
        self.released = true;
        Ok(())
    }
}
