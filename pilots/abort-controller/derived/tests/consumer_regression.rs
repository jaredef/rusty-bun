// Consumer-regression suite for AbortController + AbortSignal.
//
// Each test encodes a documented behavioral expectation from a real
// consumer. Per Doc 707, each test is a bidirectional pin.

use rusty_abort_controller::*;
use std::cell::Cell;
use std::rc::Rc;

// ─────────── node-fetch / undici — fetch(url, {signal}) ──────────
//
// Source: https://github.com/nodejs/undici/blob/main/lib/web/fetch/index.js
//   fetch's `signal.aborted` check runs at multiple points (connect, redirect,
//   body-read). consumer expectation: aborting via controller propagates to
//   signal.aborted synchronously and listeners fire immediately.

#[test]
fn consumer_node_fetch_signal_aborted_visible_after_controller_abort() {
    let ac = AbortController::new();
    let signal = ac.signal().clone();
    assert!(!signal.aborted());
    ac.abort();
    // node-fetch's check is signal.aborted in a hot loop; must be true
    // immediately after abort returns.
    assert!(signal.aborted());
}

#[test]
fn consumer_node_fetch_listener_receives_abort_synchronously() {
    let ac = AbortController::new();
    let fired = Rc::new(Cell::new(false));
    let f = fired.clone();
    ac.signal().add_event_listener(move |_| { f.set(true); });
    ac.abort();
    // node-fetch installs cleanup listeners; expects them to run before
    // ac.abort() returns control to the caller (synchronous in this scope).
    assert!(fired.get());
}

// ─────────── abort-controller polyfill (pre-Node-16) ──────────
//
// Source: https://github.com/mysticatea/abort-controller/blob/master/src/abort-controller.ts
//   the polyfill version exposed by NodeJS core in 14.17+. consumers expect
//   `controller.abort(reason)` to populate `signal.reason` exactly with the
//   passed reason value.

#[test]
fn consumer_polyfill_custom_reason_preserved() {
    let ac = AbortController::new();
    ac.abort_with(Reason::Custom("user cancelled".into()));
    assert_eq!(ac.signal().reason(), Some(Reason::Custom("user cancelled".into())));
}

// ─────────── p-cancelable — Promise cancellation idiom ──────────
//
// Source: https://github.com/sindresorhus/p-cancelable/blob/main/source/index.ts
//   cancellation invokes signal.aborted check; the library throws the
//   abort reason. Consumer expects throwIfAborted to surface the reason
//   when called after abort.

#[test]
fn consumer_p_cancelable_throw_if_aborted_yields_reason() {
    let ac = AbortController::new();
    ac.abort_with(Reason::Custom("cancelled".into()));
    let r = ac.signal().throw_if_aborted();
    assert_eq!(r, Err(Reason::Custom("cancelled".into())));
}

#[test]
fn consumer_p_cancelable_throw_if_aborted_noop_when_not_aborted() {
    let ac = AbortController::new();
    assert!(ac.signal().throw_if_aborted().is_ok());
}

// ─────────── AbortSignal.timeout / AbortSignal.any — fetch combinators ────
//
// Source: https://github.com/nodejs/undici/blob/main/lib/web/fetch/util.js
//   `combineSignals` uses AbortSignal.any to merge user-supplied signal with
//   request-internal timeout signal. when either fires, fetch aborts.

#[test]
fn consumer_undici_any_combines_user_and_timeout_signals() {
    let user_ac = AbortController::new();
    let timeout_ac = AbortController::new();
    let combined = AbortSignal::any(&[
        user_ac.signal().clone(),
        timeout_ac.signal().clone(),
    ]);
    assert!(!combined.aborted());
    timeout_ac.abort();
    assert!(combined.aborted(), "any() must abort when timeout signal fires");
}

#[test]
fn consumer_undici_any_with_already_aborted_user_returns_aborted() {
    let user_ac = AbortController::new();
    user_ac.abort_with(Reason::Custom("user cancelled".into()));
    let timeout_ac = AbortController::new();
    let combined = AbortSignal::any(&[
        user_ac.signal().clone(),
        timeout_ac.signal().clone(),
    ]);
    assert!(combined.aborted());
    assert_eq!(combined.reason(), Some(Reason::Custom("user cancelled".into())));
}

// ─────────── AsyncIterator + abort — modern async consumers ──────────
//
// Source: https://github.com/sindresorhus/p-event/blob/main/source/index.ts
//   p-event with abort handles late-registration: a listener added AFTER
//   the signal is already aborted must fire immediately (microtask in JS,
//   synchronous in pilot).

#[test]
fn consumer_p_event_late_listener_fires_immediately() {
    let s = AbortSignal::abort_default();
    let fired = Rc::new(Cell::new(false));
    let f = fired.clone();
    s.add_event_listener(move |_| { f.set(true); });
    assert!(fired.get());
}

// ─────────── Idempotency — every consumer relies on this ──────────
//
// Source: spec + every observed consumer. Repeated abort() calls must NOT
// re-fire listeners. node-fetch in particular registers cleanup once and
// would double-cleanup if listeners fired twice.

#[test]
fn consumer_idempotent_abort_listener_fires_at_most_once() {
    let ac = AbortController::new();
    let count = Rc::new(Cell::new(0u32));
    let c = count.clone();
    ac.signal().add_event_listener(move |_| { c.set(c.get() + 1); });
    ac.abort();
    ac.abort();
    ac.abort();
    assert_eq!(count.get(), 1);
}

// ─────────── DOMException AbortError — error-handling consumers ──────────
//
// Source: https://developer.mozilla.org/en-US/docs/Web/API/AbortSignal/reason
//   default reason is DOMException with name "AbortError" and legacy code 20.
//   Many consumers branch on `err.name === "AbortError"` or `err.code === 20`
//   to distinguish abort-from-other errors.

#[test]
fn consumer_dom_exception_default_reason_code_20() {
    let ac = AbortController::new();
    ac.abort();
    let r = ac.signal().reason().unwrap();
    assert_eq!(r.code(), 20);
    assert!(r.is_dom_exception());
}
