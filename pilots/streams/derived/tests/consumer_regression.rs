// Consumer-regression suite for streams.

use rusty_streams::*;

// ────────── undici body streaming — ReadableStream as fetch body ──────
//
// Source: https://github.com/nodejs/undici/blob/main/lib/web/fetch/body.js
//   `extractBody` accepts a ReadableStream and pipes its chunks to the wire.
//   consumer expectation: chunks emerge in enqueue order; close terminates
//   the body cleanly.

#[test]
fn consumer_undici_stream_chunks_in_enqueue_order() {
    let (s, ctrl) = ReadableStream::<Vec<u8>>::manual(8);
    ctrl.enqueue(b"hello ".to_vec()).unwrap();
    ctrl.enqueue(b"world".to_vec()).unwrap();
    ctrl.close().unwrap();
    let mut reader = s.get_reader().unwrap();
    let chunks: Vec<Vec<u8>> = (0..4).filter_map(|_| match reader.read() {
        ReadResult::Chunk(c) => Some(c),
        _ => None,
    }).collect();
    let combined: Vec<u8> = chunks.into_iter().flatten().collect();
    assert_eq!(combined, b"hello world".to_vec());
}

// ────────── Blob.stream() — chunk emission ──────────
//
// Source: WHATWG File API §3 — Blob.prototype.stream returns a
// ReadableStream of Uint8Array chunks. consumer expectation: the stream
// emits the blob's bytes in order, then closes.

#[test]
fn consumer_blob_stream_emits_bytes_then_closes() {
    let payload = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
    let (s, ctrl) = ReadableStream::<u8>::manual(16);
    for b in &payload { ctrl.enqueue(*b).unwrap(); }
    ctrl.close().unwrap();
    let mut reader = s.get_reader().unwrap();
    let mut collected = Vec::new();
    loop {
        match reader.read() {
            ReadResult::Chunk(b) => collected.push(b),
            ReadResult::Done => break,
            _ => panic!("unexpected"),
        }
    }
    assert_eq!(collected, payload);
}

// ────────── postMessage / Worker — tee for fan-out ──────────
//
// Source: HTML §10.5 — postMessage transfers ReadableStream by tee; the
// original branch stays on the source side, the cloned branch transfers
// to the worker. consumer expectation: tee produces independent readers
// of the same content.

#[test]
fn consumer_postmessage_tee_independent_readers() {
    let (s, ctrl) = ReadableStream::<u32>::manual(4);
    ctrl.enqueue(10).unwrap();
    ctrl.enqueue(20).unwrap();
    ctrl.enqueue(30).unwrap();
    let (a, b) = s.tee().unwrap();
    let mut ra = a.get_reader().unwrap();
    let mut rb = b.get_reader().unwrap();
    let mut va = Vec::new();
    let mut vb = Vec::new();
    for _ in 0..4 {
        if let ReadResult::Chunk(c) = ra.read() { va.push(c); }
        if let ReadResult::Chunk(c) = rb.read() { vb.push(c); }
    }
    assert_eq!(va, vec![10, 20, 30]);
    assert_eq!(vb, vec![10, 20, 30]);
}

// ────────── TextDecoderStream pattern — TransformStream consumer ──────
//
// Source: WHATWG Encoding §11 — TextDecoderStream is a TransformStream
// that decodes UTF-8 chunks to strings. consumer expectation: input chunks
// transform to corresponding output chunks via the transformer.

#[test]
fn consumer_textdecoder_stream_pattern() {
    struct AsciiDecoder;
    impl Transformer<Vec<u8>, String> for AsciiDecoder {
        fn transform(&mut self, chunk: Vec<u8>, ctrl: &Controller<String>) {
            ctrl.enqueue(String::from_utf8_lossy(&chunk).into_owned()).unwrap();
        }
    }
    let ts = TransformStream::<Vec<u8>, String>::new(Box::new(AsciiDecoder));
    let (readable, writable) = ts.into_pair();
    let mut writer = writable.get_writer().unwrap();
    writer.write(b"hello ".to_vec()).unwrap();
    writer.write(b"world".to_vec()).unwrap();
    writer.close().unwrap();
    let mut reader = readable.get_reader().unwrap();
    let mut s = String::new();
    while let ReadResult::Chunk(c) = reader.read() { s.push_str(&c); }
    assert_eq!(s, "hello world");
}

// ────────── pipeTo pattern — manual pipe loop ──────────
//
// Source: many libraries (eg. node-stream-piper) ship pipeTo polyfills.
// consumer expectation: a manual loop reading from a ReadableStream and
// writing to a WritableStream produces correct end-to-end transfer.

#[test]
fn consumer_pipe_to_pattern_manual_loop() {
    let (src, ctrl) = ReadableStream::<u32>::manual(4);
    ctrl.enqueue(1).unwrap();
    ctrl.enqueue(2).unwrap();
    ctrl.enqueue(3).unwrap();
    ctrl.close().unwrap();

    let collected = std::rc::Rc::new(std::cell::RefCell::new(Vec::<u32>::new()));
    struct Sink(std::rc::Rc<std::cell::RefCell<Vec<u32>>>);
    impl UnderlyingSink<u32> for Sink {
        fn write(&mut self, chunk: u32) -> Result<(), String> {
            self.0.borrow_mut().push(chunk); Ok(())
        }
    }
    let dst = WritableStream::new(Box::new(Sink(collected.clone())));
    let mut reader = src.get_reader().unwrap();
    let mut writer = dst.get_writer().unwrap();
    loop {
        match reader.read() {
            ReadResult::Chunk(c) => writer.write(c).unwrap(),
            ReadResult::Done => break,
            _ => panic!(),
        }
    }
    writer.close().unwrap();
    assert_eq!(*collected.borrow(), vec![1, 2, 3]);
}

// ────────── error propagation — fetch body cancellation ──────────
//
// Source: https://github.com/nodejs/undici/blob/main/lib/web/fetch/body.js
//   when the consumer cancels mid-stream, undici expects the underlying
//   source to be notified. consumer expectation: reader.cancel propagates.

#[test]
fn consumer_undici_cancel_propagates_to_source() {
    use std::cell::Cell;
    use std::rc::Rc;
    struct CancellableSource {
        cancelled: Rc<Cell<bool>>,
    }
    impl UnderlyingSource<u32> for CancellableSource {
        fn cancel(&mut self, _: Option<String>) {
            self.cancelled.set(true);
        }
    }
    let cancelled = Rc::new(Cell::new(false));
    let s = ReadableStream::new(
        Box::new(CancellableSource { cancelled: cancelled.clone() }),
        1,
    );
    let mut r = s.get_reader().unwrap();
    r.cancel(Some("user cancel".into())).unwrap();
    assert!(cancelled.get());
}

// ────────── error in source.error → reader sees error ──────────

#[test]
fn consumer_source_error_visible_to_reader() {
    let (s, ctrl) = ReadableStream::<u32>::manual(4);
    ctrl.enqueue(1).unwrap();
    ctrl.error("upstream failed");
    let mut r = s.get_reader().unwrap();
    let mut got_error = false;
    for _ in 0..4 {
        if let ReadResult::Error(_) = r.read() { got_error = true; break; }
    }
    assert!(got_error);
}

// ────────── WPT streams test corpus (transcribed) ──────────
//
// Source: web-platform-tests/wpt/streams/

#[test]
fn wpt_streams_constructor_no_args() {
    let (s, _ctrl) = ReadableStream::<u32>::manual(1);
    assert!(!s.locked());
}

#[test]
fn wpt_streams_pull_called_lazily() {
    use std::cell::Cell;
    use std::rc::Rc;
    struct LazySource {
        pull_count: Rc<Cell<u32>>,
    }
    impl UnderlyingSource<u32> for LazySource {
        fn pull(&mut self, ctrl: &Controller<u32>) {
            self.pull_count.set(self.pull_count.get() + 1);
            ctrl.enqueue(99).unwrap();
        }
    }
    let pulls = Rc::new(Cell::new(0u32));
    let s = ReadableStream::new(
        Box::new(LazySource { pull_count: pulls.clone() }),
        1,
    );
    // Construction should NOT invoke pull (only start, which we don't define here).
    // Pull is invoked once we start reading.
    assert_eq!(pulls.get(), 0);
    let mut r = s.get_reader().unwrap();
    let _ = r.read();
    assert!(pulls.get() >= 1);
}
