// Simulated-derivation of AbortController + AbortSignal (DOM §3.3).
//
// Inputs:
//   AUDIT — pilots/abort-controller/AUDIT.md
//   SPEC  — https://dom.spec.whatwg.org/#interface-abortcontroller
//   CD    — runs/2026-05-10-bun-v0.13b-spec-batch/constraints/
//             abortcontroller.constraints.md (3 props / 21 clauses)
//             abortsignal.constraints.md (7 props / 12 clauses)
//
// First pilot needing interior mutability + shared ownership. JS's
// controller-and-signal pair share private state; Rust analog is
// `Rc<RefCell<SignalInner>>` shared between AbortController and the
// AbortSignal it owns.

use std::cell::RefCell;
use std::rc::Rc;

// ─────────── Reason ────────────
//
// CD: `expect(ac.signal.reason).toBeInstanceOf(DOMException)` and
//     `expect(ac.signal.reason.code).toBe(20)`.
// SPEC §3.3: when abort() is called with no argument, the reason is a
// DOMException with name "AbortError" and code 20.

#[derive(Debug, Clone, PartialEq)]
pub enum Reason {
    /// Default DOMException analog. Spec says this is a DOMException with
    /// name "AbortError" and legacy code 20.
    AbortError,
    /// Custom user-supplied reason (any string in the pilot scope; JS would
    /// allow any value).
    Custom(String),
}

impl Reason {
    /// SPEC: DOMException AbortError legacy code. Bun test asserts code === 20.
    pub fn code(&self) -> u16 {
        match self {
            Reason::AbortError => 20,
            Reason::Custom(_) => 0,
        }
    }

    pub fn is_dom_exception(&self) -> bool {
        matches!(self, Reason::AbortError)
    }
}

// ─────────── Inner shared state ────────────

#[derive(Default)]
struct SignalInner {
    aborted: bool,
    reason: Option<Reason>,
    /// SPEC: listeners fire at most once on the abort transition.
    listeners: Vec<Box<dyn FnOnce(&Reason)>>,
}

// ─────────── AbortSignal ────────────

#[derive(Clone)]
pub struct AbortSignal {
    inner: Rc<RefCell<SignalInner>>,
}

impl AbortSignal {
    /// Construct a non-aborted signal. Used internally by AbortController and
    /// AbortSignal::any.
    fn new() -> Self {
        Self { inner: Rc::new(RefCell::new(SignalInner::default())) }
    }

    /// SPEC §3.3.AbortSignal.abort: returns an already-aborted signal.
    /// CD ABOR1 antichain: `AbortSignal.abort()` ⇒ `signal.aborted === true`.
    pub fn abort_default() -> Self {
        let s = Self::new();
        s.inner.borrow_mut().aborted = true;
        s.inner.borrow_mut().reason = Some(Reason::AbortError);
        s
    }

    pub fn abort_with(reason: Reason) -> Self {
        let s = Self::new();
        s.inner.borrow_mut().aborted = true;
        s.inner.borrow_mut().reason = Some(reason);
        s
    }

    /// SPEC §3.3.AbortSignal.any: returns a signal aborted when any input
    /// signal aborts. If any input is already aborted at construction, the
    /// result is constructed already-aborted.
    pub fn any(signals: &[AbortSignal]) -> Self {
        // Check for already-aborted inputs first.
        for s in signals {
            let inner = s.inner.borrow();
            if inner.aborted {
                let result = Self::new();
                let reason = inner.reason.clone().unwrap_or(Reason::AbortError);
                result.inner.borrow_mut().aborted = true;
                result.inner.borrow_mut().reason = Some(reason);
                return result;
            }
        }
        // Otherwise: register a listener on each input that aborts the result.
        let result = Self::new();
        for s in signals {
            let result_clone = result.clone();
            s.add_event_listener(move |reason| {
                // The result may already be aborted by another listener firing.
                let mut r = result_clone.inner.borrow_mut();
                if !r.aborted {
                    r.aborted = true;
                    r.reason = Some(reason.clone());
                    let listeners: Vec<_> = r.listeners.drain(..).collect();
                    drop(r);
                    let final_reason = result_clone.inner.borrow().reason.clone().unwrap();
                    for cb in listeners { cb(&final_reason); }
                }
            });
        }
        result
    }

    /// SPEC §3.3: the aborted getter.
    pub fn aborted(&self) -> bool {
        self.inner.borrow().aborted
    }

    /// SPEC §3.3: the reason getter — Some(reason) when aborted, None otherwise.
    pub fn reason(&self) -> Option<Reason> {
        self.inner.borrow().reason.clone()
    }

    /// SPEC §3.3.throwIfAborted: returns Err(reason) when aborted; Ok(())
    /// otherwise. (JS equivalent throws; Rust's idiomatic translation is
    /// Result.)
    pub fn throw_if_aborted(&self) -> Result<(), Reason> {
        let inner = self.inner.borrow();
        if inner.aborted {
            Err(inner.reason.clone().unwrap_or(Reason::AbortError))
        } else {
            Ok(())
        }
    }

    /// SPEC §3.3: addEventListener("abort", callback). Pilot exposes this
    /// directly. If the signal is already aborted at registration time, the
    /// callback fires immediately (per the spec's microtask semantics — pilot
    /// runs synchronously).
    pub fn add_event_listener<F>(&self, callback: F)
    where F: FnOnce(&Reason) + 'static
    {
        let mut inner = self.inner.borrow_mut();
        if inner.aborted {
            let reason = inner.reason.clone().unwrap_or(Reason::AbortError);
            drop(inner);
            callback(&reason);
        } else {
            inner.listeners.push(Box::new(callback));
        }
    }

    /// Internal: fire the abort transition. Idempotent — second and later
    /// calls are no-ops per the spec.
    fn fire_abort(&self, reason: Reason) {
        let mut inner = self.inner.borrow_mut();
        if inner.aborted { return; }
        inner.aborted = true;
        inner.reason = Some(reason.clone());
        let listeners: Vec<_> = inner.listeners.drain(..).collect();
        drop(inner);
        for cb in listeners { cb(&reason); }
    }
}

// ─────────── AbortController ────────────

pub struct AbortController {
    signal: AbortSignal,
}

impl AbortController {
    /// SPEC §3.3.AbortController constructor: creates a non-aborted signal.
    /// CD ABOR3: `AbortController is defined as a global constructor`.
    pub fn new() -> Self {
        Self { signal: AbortSignal::new() }
    }

    /// SPEC §3.3.signal getter: returns the controller's associated signal.
    pub fn signal(&self) -> &AbortSignal { &self.signal }

    /// SPEC §3.3.abort: aborts the signal with the default reason.
    /// CD ABOR1 antichain: `expect(ac.signal.aborted).toBe(true)` after
    /// `ac.abort()`.
    pub fn abort(&self) {
        self.signal.fire_abort(Reason::AbortError);
    }

    /// SPEC §3.3.abort(reason): aborts with a custom reason.
    pub fn abort_with(&self, reason: Reason) {
        self.signal.fire_abort(reason);
    }
}

impl Default for AbortController {
    fn default() -> Self { Self::new() }
}
