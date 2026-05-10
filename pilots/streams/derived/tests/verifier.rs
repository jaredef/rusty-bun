// Verifier for the streams pilot.
//
// CD = runs/2026-05-10-bun-v0.13b-spec-batch/constraints/{readable,
//      writable,transform}stream.constraints.md (sparse: 3 clauses total)
// SPEC EXTRACT = specs/streams.spec.md (44 clauses, primary input)
// SPEC = WHATWG Streams Standard https://streams.spec.whatwg.org/

use rusty_streams::*;

fn read_chunks<T: Clone + 'static>(reader: &mut Reader<T>, max: usize) -> Vec<T> {
    let mut out = Vec::new();
    for _ in 0..max {
        match reader.read() {
            ReadResult::Chunk(c) => out.push(c),
            ReadResult::Done | ReadResult::Pending | ReadResult::Error(_) => break,
        }
    }
    out
}

// ════════════════════ READABLE STREAM ════════════════════

// CD: `expect(typeof ReadableStream).toBe("function")` — class exists.
#[test]
fn cd_readable_class_exists() {
    let (_s, _ctrl) = ReadableStream::<u32>::manual(1);
}

// SPEC: locked false until reader acquired.
#[test]
fn spec_readable_locked_false_initially() {
    let (s, _ctrl) = ReadableStream::<u32>::manual(1);
    assert!(!s.locked());
}

#[test]
fn spec_readable_locked_true_after_get_reader() {
    let (s, _ctrl) = ReadableStream::<u32>::manual(1);
    let _r = s.get_reader().unwrap();
    assert!(s.locked());
}

#[test]
fn spec_readable_get_reader_twice_errors() {
    let (s, _ctrl) = ReadableStream::<u32>::manual(1);
    let _r = s.get_reader().unwrap();
    let r2 = s.get_reader();
    assert!(matches!(r2, Err(StreamError::AlreadyLocked)));
}

#[test]
fn spec_controller_enqueue_then_read() {
    let (s, ctrl) = ReadableStream::<u32>::manual(4);
    ctrl.enqueue(1).unwrap();
    ctrl.enqueue(2).unwrap();
    ctrl.enqueue(3).unwrap();
    let mut r = s.get_reader().unwrap();
    assert!(matches!(r.read(), ReadResult::Chunk(1)));
    assert!(matches!(r.read(), ReadResult::Chunk(2)));
    assert!(matches!(r.read(), ReadResult::Chunk(3)));
}

#[test]
fn spec_controller_close_then_read_yields_done() {
    let (s, ctrl) = ReadableStream::<u32>::manual(4);
    ctrl.enqueue(1).unwrap();
    ctrl.close().unwrap();
    let mut r = s.get_reader().unwrap();
    assert!(matches!(r.read(), ReadResult::Chunk(1)));
    assert!(matches!(r.read(), ReadResult::Done));
}

#[test]
fn spec_controller_enqueue_after_close_errors() {
    let (_s, ctrl) = ReadableStream::<u32>::manual(4);
    ctrl.close().unwrap();
    let r = ctrl.enqueue(1);
    assert!(matches!(r, Err(StreamError::Closed)));
}

#[test]
fn spec_controller_error_propagates_to_reader() {
    let (s, ctrl) = ReadableStream::<u32>::manual(4);
    ctrl.enqueue(7).unwrap();
    ctrl.error("boom");
    let mut r = s.get_reader().unwrap();
    // SPEC: error clears the queue
    assert!(matches!(r.read(), ReadResult::Error(_)));
}

#[test]
fn spec_controller_enqueue_after_error_errors() {
    let (_s, ctrl) = ReadableStream::<u32>::manual(4);
    ctrl.error("boom");
    let r = ctrl.enqueue(1);
    assert!(matches!(r, Err(StreamError::Errored(_))));
}

#[test]
fn spec_pending_when_no_chunk_available() {
    let (s, _ctrl) = ReadableStream::<u32>::manual(4);
    let mut r = s.get_reader().unwrap();
    assert!(matches!(r.read(), ReadResult::Pending));
}

#[test]
fn spec_underlying_source_start_invoked_synchronously() {
    use std::cell::Cell;
    use std::rc::Rc;
    struct CountSource(Rc<Cell<u32>>);
    impl UnderlyingSource<u32> for CountSource {
        fn start(&mut self, _ctrl: &Controller<u32>) {
            self.0.set(self.0.get() + 1);
        }
    }
    let count = Rc::new(Cell::new(0u32));
    let _s = ReadableStream::new(Box::new(CountSource(count.clone())), 1);
    assert_eq!(count.get(), 1);
}

#[test]
fn spec_underlying_source_pull_invoked_when_queue_empty() {
    use std::cell::Cell;
    use std::rc::Rc;
    struct PullSource {
        pulls: Rc<Cell<u32>>,
        chunks_remaining: Rc<Cell<u32>>,
    }
    impl UnderlyingSource<u32> for PullSource {
        fn pull(&mut self, ctrl: &Controller<u32>) {
            self.pulls.set(self.pulls.get() + 1);
            if self.chunks_remaining.get() > 0 {
                ctrl.enqueue(42).unwrap();
                self.chunks_remaining.set(self.chunks_remaining.get() - 1);
            } else {
                ctrl.close().unwrap();
            }
        }
    }
    let pulls = Rc::new(Cell::new(0u32));
    let remaining = Rc::new(Cell::new(2u32));
    let s = ReadableStream::new(Box::new(PullSource {
        pulls: pulls.clone(),
        chunks_remaining: remaining.clone(),
    }), 1);
    let mut r = s.get_reader().unwrap();
    let chunks = read_chunks(&mut r, 4);
    assert_eq!(chunks, vec![42, 42]);
    assert!(pulls.get() >= 2, "pull must be called per chunk; got {}", pulls.get());
}

#[test]
fn spec_reader_release_lock_unlocks_stream() {
    let (s, _ctrl) = ReadableStream::<u32>::manual(1);
    let mut r = s.get_reader().unwrap();
    r.release_lock().unwrap();
    assert!(!s.locked());
}

#[test]
fn spec_reader_after_release_errors_on_read() {
    let (s, _ctrl) = ReadableStream::<u32>::manual(1);
    let mut r = s.get_reader().unwrap();
    r.release_lock().unwrap();
    assert!(matches!(r.read(), ReadResult::Error(_)));
}

// ════════════════════ TEE ════════════════════

#[test]
fn spec_tee_returns_two_streams() {
    let (s, ctrl) = ReadableStream::<u32>::manual(4);
    ctrl.enqueue(1).unwrap();
    ctrl.enqueue(2).unwrap();
    ctrl.enqueue(3).unwrap();
    let (a, b) = s.tee().unwrap();
    let mut ra = a.get_reader().unwrap();
    let mut rb = b.get_reader().unwrap();
    assert_eq!(read_chunks(&mut ra, 4), vec![1, 2, 3]);
    assert_eq!(read_chunks(&mut rb, 4), vec![1, 2, 3]);
}

#[test]
fn spec_tee_locks_original() {
    let (s, _ctrl) = ReadableStream::<u32>::manual(1);
    let _branches = s.tee().unwrap();
    assert!(s.locked());
}

#[test]
fn spec_tee_branches_independent_state() {
    let (s, ctrl) = ReadableStream::<u32>::manual(4);
    ctrl.enqueue(1).unwrap();
    ctrl.enqueue(2).unwrap();
    let (a, b) = s.tee().unwrap();
    let mut ra = a.get_reader().unwrap();
    // Drain a; b must still have everything
    assert!(matches!(ra.read(), ReadResult::Chunk(1)));
    let mut rb = b.get_reader().unwrap();
    assert!(matches!(rb.read(), ReadResult::Chunk(1)));
}

// ════════════════════ WRITABLE STREAM ════════════════════

struct CollectingSink {
    chunks: std::rc::Rc<std::cell::RefCell<Vec<u32>>>,
    closed: std::rc::Rc<std::cell::Cell<bool>>,
}
impl UnderlyingSink<u32> for CollectingSink {
    fn write(&mut self, chunk: u32) -> Result<(), String> {
        self.chunks.borrow_mut().push(chunk);
        Ok(())
    }
    fn close(&mut self) -> Result<(), String> {
        self.closed.set(true);
        Ok(())
    }
}

#[test]
fn spec_writable_class_exists() {
    let _ = WritableStream::new(Box::new(CollectingSink {
        chunks: Default::default(),
        closed: Default::default(),
    }));
}

#[test]
fn spec_writer_write_invokes_sink() {
    let chunks = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let closed = std::rc::Rc::new(std::cell::Cell::new(false));
    let s = WritableStream::new(Box::new(CollectingSink {
        chunks: chunks.clone(),
        closed: closed.clone(),
    }));
    let mut w = s.get_writer().unwrap();
    w.write(1).unwrap();
    w.write(2).unwrap();
    w.write(3).unwrap();
    assert_eq!(*chunks.borrow(), vec![1, 2, 3]);
}

#[test]
fn spec_writer_close_invokes_sink_close() {
    let chunks = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let closed = std::rc::Rc::new(std::cell::Cell::new(false));
    let s = WritableStream::new(Box::new(CollectingSink {
        chunks: chunks.clone(),
        closed: closed.clone(),
    }));
    let mut w = s.get_writer().unwrap();
    w.write(1).unwrap();
    w.close().unwrap();
    assert!(closed.get());
}

#[test]
fn spec_writer_after_close_errors_on_write() {
    let chunks = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let closed = std::rc::Rc::new(std::cell::Cell::new(false));
    let s = WritableStream::new(Box::new(CollectingSink {
        chunks: chunks.clone(),
        closed: closed.clone(),
    }));
    let mut w = s.get_writer().unwrap();
    w.close().unwrap();
    let r = w.write(99);
    assert!(matches!(r, Err(_)));
}

#[test]
fn spec_writable_locked_after_get_writer() {
    let s = WritableStream::new(Box::new(CollectingSink {
        chunks: Default::default(),
        closed: Default::default(),
    }));
    let _w = s.get_writer().unwrap();
    assert!(s.locked());
}

#[test]
fn spec_writable_get_writer_twice_errors() {
    let s = WritableStream::new(Box::new(CollectingSink {
        chunks: Default::default(),
        closed: Default::default(),
    }));
    let _w = s.get_writer().unwrap();
    let w2 = s.get_writer();
    assert!(matches!(w2, Err(StreamError::AlreadyLocked)));
}

#[test]
fn spec_writer_abort_propagates() {
    use std::cell::Cell;
    use std::rc::Rc;
    struct AbortableSink {
        aborted: Rc<Cell<bool>>,
    }
    impl UnderlyingSink<u32> for AbortableSink {
        fn write(&mut self, _: u32) -> Result<(), String> { Ok(()) }
        fn abort(&mut self, _: Option<String>) { self.aborted.set(true); }
    }
    let aborted = Rc::new(Cell::new(false));
    let s = WritableStream::new(Box::new(AbortableSink { aborted: aborted.clone() }));
    let mut w = s.get_writer().unwrap();
    w.abort(Some("user cancel".into())).unwrap();
    assert!(aborted.get());
}

#[test]
fn spec_writer_sink_error_transitions_to_errored() {
    struct FailingSink;
    impl UnderlyingSink<u32> for FailingSink {
        fn write(&mut self, _: u32) -> Result<(), String> { Err("write failed".into()) }
    }
    let s = WritableStream::new(Box::new(FailingSink));
    let mut w = s.get_writer().unwrap();
    let r = w.write(1);
    assert!(matches!(r, Err(StreamError::Errored(_))));
}

// ════════════════════ TRANSFORM STREAM ════════════════════

struct DoubleTransformer;
impl Transformer<u32, u32> for DoubleTransformer {
    fn transform(&mut self, chunk: u32, ctrl: &Controller<u32>) {
        ctrl.enqueue(chunk * 2).unwrap();
    }
}

#[test]
fn spec_transform_class_exists() {
    let _ = TransformStream::<u32, u32>::new(Box::new(DoubleTransformer));
}

#[test]
fn spec_transform_pipeline_basic() {
    let ts = TransformStream::<u32, u32>::new(Box::new(DoubleTransformer));
    let (readable, writable) = ts.into_pair();
    let mut writer = writable.get_writer().unwrap();
    writer.write(1).unwrap();
    writer.write(2).unwrap();
    writer.write(3).unwrap();
    let mut reader = readable.get_reader().unwrap();
    assert_eq!(read_chunks(&mut reader, 8), vec![2, 4, 6]);
}

#[test]
fn spec_transform_flush_called_on_close() {
    use std::cell::Cell;
    use std::rc::Rc;
    struct FlushTransformer {
        flushed: Rc<Cell<bool>>,
    }
    impl Transformer<u32, u32> for FlushTransformer {
        fn transform(&mut self, chunk: u32, ctrl: &Controller<u32>) {
            ctrl.enqueue(chunk).unwrap();
        }
        fn flush(&mut self, _: &Controller<u32>) {
            self.flushed.set(true);
        }
    }
    let flushed = Rc::new(Cell::new(false));
    let ts = TransformStream::<u32, u32>::new(Box::new(FlushTransformer {
        flushed: flushed.clone(),
    }));
    let (_readable, writable) = ts.into_pair();
    let mut writer = writable.get_writer().unwrap();
    writer.write(1).unwrap();
    writer.close().unwrap();
    assert!(flushed.get());
}

#[test]
fn spec_transform_after_close_readable_yields_done() {
    let ts = TransformStream::<u32, u32>::new(Box::new(DoubleTransformer));
    let (readable, writable) = ts.into_pair();
    let mut writer = writable.get_writer().unwrap();
    writer.write(5).unwrap();
    writer.close().unwrap();
    let mut reader = readable.get_reader().unwrap();
    assert_eq!(read_chunks(&mut reader, 4), vec![10]);
    assert!(matches!(reader.read(), ReadResult::Done));
}
