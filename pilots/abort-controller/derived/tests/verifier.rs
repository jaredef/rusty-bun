// Verifier for the AbortController + AbortSignal pilot.
//
// CD-AC = runs/2026-05-10-bun-v0.13b-spec-batch/constraints/abortcontroller.constraints.md
// CD-AS = runs/2026-05-10-bun-v0.13b-spec-batch/constraints/abortsignal.constraints.md
// SPEC  = https://dom.spec.whatwg.org/#interface-abortcontroller

use rusty_abort_controller::*;
use std::cell::Cell;
use std::rc::Rc;

// ─────────── CD-AC / ABOR1 antichain reps (cardinality 19) ──────────

// `expect(typeof AbortController !== "undefined").toBe(true)`
#[test]
fn cd_ac_abor1_class_exists() {
    let _ = AbortController::new();
}

// `expect(ac.signal.aborted).toBe(true)` after abort()
#[test]
fn cd_ac_abor1_abort_sets_signal_aborted() {
    let ac = AbortController::new();
    assert!(!ac.signal().aborted());
    ac.abort();
    assert!(ac.signal().aborted());
}

// `expect(ac.signal.reason.code).toBe(20)` — DOMException AbortError code
#[test]
fn cd_ac_abor1_default_reason_code_is_20() {
    let ac = AbortController::new();
    ac.abort();
    let r = ac.signal().reason().expect("aborted signal must have reason");
    assert_eq!(r.code(), 20);
}

// ─────────── CD-AC / ABOR2 antichain rep ──────────

// `expect(ac.signal.reason).toBeInstanceOf(DOMException)`
#[test]
fn cd_ac_abor2_default_reason_is_dom_exception() {
    let ac = AbortController::new();
    ac.abort();
    let r = ac.signal().reason().unwrap();
    assert!(r.is_dom_exception());
}

// ─────────── CD-AS / ABOR1 antichain reps (cardinality 4) ──────────

// `assertEquals(signal.aborted, true)` after AbortSignal.abort()
#[test]
fn cd_as_abor1_abort_static_returns_aborted_signal() {
    let s = AbortSignal::abort_default();
    assert!(s.aborted());
}

// `assertEquals(signal.reason, "hey!")` for AbortSignal.abort("hey!")
#[test]
fn cd_as_abor1_abort_static_with_custom_reason() {
    let s = AbortSignal::abort_with(Reason::Custom("hey!".into()));
    assert_eq!(s.reason(), Some(Reason::Custom("hey!".into())));
}

// `AbortSignal.abort() returns an already-aborted AbortSignal with default reason`
#[test]
fn cd_as_abor1_default_reason_is_abort_error() {
    let s = AbortSignal::abort_default();
    let r = s.reason().unwrap();
    assert!(r.is_dom_exception());
    assert_eq!(r.code(), 20);
}

// ─────────── CD-AS / ABOR2 antichain reps (cardinality 3) ──────────

// `expect(signal.aborted).toBe(true)` after AbortSignal.any() with one already aborted
#[test]
fn cd_as_abor2_any_with_already_aborted_input_is_aborted() {
    let s1 = AbortSignal::abort_with(Reason::Custom("first".into()));
    let any = AbortSignal::any(&[s1]);
    assert!(any.aborted());
    assert_eq!(any.reason(), Some(Reason::Custom("first".into())));
}

// `AbortSignal.any(signals) returns an AbortSignal aborted when any signal aborts`
#[test]
fn cd_as_abor2_any_aborts_when_input_aborts() {
    let ac1 = AbortController::new();
    let ac2 = AbortController::new();
    let any = AbortSignal::any(&[ac1.signal().clone(), ac2.signal().clone()]);
    assert!(!any.aborted());
    ac1.abort();
    assert!(any.aborted());
}

// ─────────── Listener semantics ──────────

#[test]
fn spec_listener_fires_on_abort_transition() {
    let ac = AbortController::new();
    let fired = Rc::new(Cell::new(false));
    let f = fired.clone();
    ac.signal().add_event_listener(move |_reason| {
        f.set(true);
    });
    assert!(!fired.get());
    ac.abort();
    assert!(fired.get());
}

#[test]
fn spec_listener_receives_reason() {
    let ac = AbortController::new();
    let received = Rc::new(Cell::new(0u16));
    let r = received.clone();
    ac.signal().add_event_listener(move |reason| {
        r.set(reason.code());
    });
    ac.abort();
    assert_eq!(received.get(), 20); // DOMException AbortError code
}

#[test]
fn spec_abort_is_idempotent_listeners_fire_at_most_once() {
    let ac = AbortController::new();
    let count = Rc::new(Cell::new(0u32));
    let c = count.clone();
    ac.signal().add_event_listener(move |_| {
        c.set(c.get() + 1);
    });
    ac.abort();
    ac.abort();
    ac.abort();
    assert_eq!(count.get(), 1);
}

#[test]
fn spec_abort_idempotent_reason_unchanged() {
    let ac = AbortController::new();
    ac.abort_with(Reason::Custom("first".into()));
    ac.abort_with(Reason::Custom("second".into()));
    assert_eq!(ac.signal().reason(), Some(Reason::Custom("first".into())));
}

#[test]
fn spec_listener_registered_after_abort_fires_immediately() {
    let s = AbortSignal::abort_default();
    let fired = Rc::new(Cell::new(false));
    let f = fired.clone();
    s.add_event_listener(move |_| { f.set(true); });
    assert!(fired.get(), "post-abort listener should fire on registration");
}

// ─────────── throw_if_aborted ──────────

#[test]
fn spec_throw_if_aborted_noop_when_not_aborted() {
    let ac = AbortController::new();
    assert!(ac.signal().throw_if_aborted().is_ok());
}

#[test]
fn spec_throw_if_aborted_returns_reason_when_aborted() {
    let ac = AbortController::new();
    ac.abort_with(Reason::Custom("nope".into()));
    let r = ac.signal().throw_if_aborted();
    assert_eq!(r, Err(Reason::Custom("nope".into())));
}

// ─────────── AbortSignal.any: composability ──────────

#[test]
fn spec_any_with_no_aborted_inputs_starts_unaborted() {
    let ac1 = AbortController::new();
    let ac2 = AbortController::new();
    let any = AbortSignal::any(&[ac1.signal().clone(), ac2.signal().clone()]);
    assert!(!any.aborted());
}

#[test]
fn spec_any_propagates_first_aborter_reason() {
    let ac1 = AbortController::new();
    let ac2 = AbortController::new();
    let any = AbortSignal::any(&[ac1.signal().clone(), ac2.signal().clone()]);
    ac2.abort_with(Reason::Custom("from ac2".into()));
    assert!(any.aborted());
    assert_eq!(any.reason(), Some(Reason::Custom("from ac2".into())));
}

#[test]
fn spec_any_subsequent_input_aborts_dont_re_fire() {
    let ac1 = AbortController::new();
    let ac2 = AbortController::new();
    let any = AbortSignal::any(&[ac1.signal().clone(), ac2.signal().clone()]);
    let count = Rc::new(Cell::new(0u32));
    let c = count.clone();
    any.add_event_listener(move |_| { c.set(c.get() + 1); });
    ac1.abort();
    ac2.abort(); // second input also aborts; result already aborted
    assert_eq!(count.get(), 1);
}

#[test]
fn spec_any_empty_input_is_never_aborted() {
    let any = AbortSignal::any(&[]);
    assert!(!any.aborted());
}

// ─────────── Controller-and-signal share state ──────────

#[test]
fn spec_signal_clone_shares_state_with_original() {
    let ac = AbortController::new();
    let s = ac.signal().clone();
    assert!(!s.aborted());
    ac.abort();
    assert!(s.aborted(), "cloned signal should reflect abort on the controller");
}

#[test]
fn spec_signal_handle_persists_after_controller_drop() {
    let s = {
        let ac = AbortController::new();
        ac.signal().clone()
    };
    // Controller dropped; signal still alive — nothing aborts it because
    // there's no controller left to call abort.
    assert!(!s.aborted());
}
