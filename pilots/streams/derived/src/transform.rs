// TransformStream — WHATWG Streams Standard §6.
//
// Pilot implementation: a TransformStream owns a paired (writable, readable)
// pair connected by a transformer callback. Writing to the writable side
// invokes transform(chunk) which may enqueue 0+ output chunks on the
// readable side via the transform-controller.

use std::cell::RefCell;
use std::rc::Rc;

use crate::readable::{Controller as RController, ReadableStream, StreamError};
use crate::writable::{UnderlyingSink, WritableStream};

pub trait Transformer<I, O> {
    fn start(&mut self, _ctrl: &RController<O>) {}
    /// SPEC: invoked per input chunk; may enqueue zero or more output chunks.
    fn transform(&mut self, chunk: I, ctrl: &RController<O>);
    /// SPEC: called when the writable side closes; flush remaining state.
    fn flush(&mut self, _ctrl: &RController<O>) {}
}

pub struct TransformStream<I, O> {
    readable: ReadableStream<O>,
    writable: WritableStream<I>,
}

impl<I: 'static, O: Clone + 'static> TransformStream<I, O> {
    pub fn new(transformer: Box<dyn Transformer<I, O>>) -> Self {
        let (readable, ctrl) = ReadableStream::<O>::manual(1);
        let ctrl = Rc::new(ctrl);
        let transformer = Rc::new(RefCell::new(transformer));

        // Run start(transformer) synchronously per spec.
        transformer.borrow_mut().start(&ctrl);

        let sink: Box<dyn UnderlyingSink<I>> = Box::new(TransformSink {
            transformer: transformer.clone(),
            ctrl: ctrl.clone(),
        });
        let writable = WritableStream::new(sink);

        Self { readable, writable }
    }

    /// SPEC §6.readable.
    pub fn readable(&self) -> &ReadableStream<O> { &self.readable }

    /// SPEC §6.writable.
    pub fn writable(&self) -> &WritableStream<I> { &self.writable }

    /// Owned getters for chained-test usage (the verifier sometimes needs
    /// to consume the readable side after dropping the TransformStream).
    pub fn into_pair(self) -> (ReadableStream<O>, WritableStream<I>) {
        (self.readable, self.writable)
    }
}

struct TransformSink<I, O> {
    transformer: Rc<RefCell<Box<dyn Transformer<I, O>>>>,
    ctrl: Rc<RController<O>>,
}

impl<I, O> UnderlyingSink<I> for TransformSink<I, O> {
    fn write(&mut self, chunk: I) -> Result<(), String> {
        self.transformer.borrow_mut().transform(chunk, &self.ctrl);
        Ok(())
    }
    fn close(&mut self) -> Result<(), String> {
        self.transformer.borrow_mut().flush(&self.ctrl);
        let _ = self.ctrl.close();
        Ok(())
    }
    fn abort(&mut self, reason: Option<String>) {
        self.ctrl.error(reason.unwrap_or_else(|| "aborted".into()));
    }
}

#[allow(dead_code)]
fn _ensure_stream_error_used(_: StreamError) {} // hush unused-import warning
