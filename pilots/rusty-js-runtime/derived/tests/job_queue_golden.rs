//! JobQueue + run_to_completion tests per ECMA-262 §9.4.1 + HTML §8.
//!
//! Each test verifies one ordering invariant the engine must enforce.
//! The tests use a shared Rc<RefCell<Vec<&'static str>>> log inside
//! closures to record the order jobs run; the test asserts the log
//! sequence.

use rusty_js_runtime::Runtime;
use std::cell::RefCell;
use std::rc::Rc;

fn log_setup() -> Rc<RefCell<Vec<&'static str>>> {
    Rc::new(RefCell::new(Vec::new()))
}

fn push_log(log: &Rc<RefCell<Vec<&'static str>>>, s: &'static str) {
    log.borrow_mut().push(s);
}

// ─────────── Empty queue ───────────

#[test]
fn run_to_completion_empty_exits_immediately() {
    let mut rt = Runtime::new();
    rt.run_to_completion().expect("empty queue should exit cleanly");
}

#[test]
fn empty_queue_reports_empty() {
    let rt = Runtime::new();
    assert!(rt.job_queue.is_empty());
    assert_eq!(rt.job_queue.microtask_count(), 0);
    assert_eq!(rt.job_queue.macrotask_count(), 0);
}

// ─────────── Microtask drain ordering ───────────

#[test]
fn microtasks_run_fifo() {
    let mut rt = Runtime::new();
    let log = log_setup();
    let l1 = log.clone();
    let l2 = log.clone();
    let l3 = log.clone();
    rt.enqueue_microtask("a", move |_rt| { push_log(&l1, "a"); Ok(()) });
    rt.enqueue_microtask("b", move |_rt| { push_log(&l2, "b"); Ok(()) });
    rt.enqueue_microtask("c", move |_rt| { push_log(&l3, "c"); Ok(()) });
    rt.run_to_completion().unwrap();
    assert_eq!(*log.borrow(), vec!["a", "b", "c"]);
}

#[test]
fn microtask_can_enqueue_more_microtasks_in_same_drain() {
    // Per §9.4.1: microtasks enqueued during a drain process in the
    // same drain. Verify by enqueuing a microtask from a microtask;
    // both should run before the test exits.
    let mut rt = Runtime::new();
    let log = log_setup();
    let l_outer = log.clone();
    let l_inner = log.clone();
    rt.enqueue_microtask("outer", move |rt| {
        push_log(&l_outer, "outer");
        let l_inner = l_inner.clone();
        rt.enqueue_microtask("inner", move |_rt| {
            push_log(&l_inner, "inner");
            Ok(())
        });
        Ok(())
    });
    rt.run_to_completion().unwrap();
    assert_eq!(*log.borrow(), vec!["outer", "inner"]);
}

// ─────────── Macrotask phase ordering ───────────

#[test]
fn macrotasks_run_fifo_one_at_a_time() {
    let mut rt = Runtime::new();
    let log = log_setup();
    let l1 = log.clone();
    let l2 = log.clone();
    rt.enqueue_macrotask("m1", move |_rt| { push_log(&l1, "m1"); Ok(()) });
    rt.enqueue_macrotask("m2", move |_rt| { push_log(&l2, "m2"); Ok(()) });
    rt.run_to_completion().unwrap();
    assert_eq!(*log.borrow(), vec!["m1", "m2"]);
}

#[test]
fn microtask_drains_between_macrotasks() {
    // The critical invariant: microtasks drain to quiescence between
    // every macrotask. If a macrotask enqueues both a microtask and a
    // macrotask, the microtask runs FIRST, then the next macrotask.
    let mut rt = Runtime::new();
    let log = log_setup();
    let l_m1 = log.clone();
    let l_m2 = log.clone();
    let l_micro = log.clone();
    let l_micro_handle = log.clone();
    let l_m2_handle = log.clone();
    rt.enqueue_macrotask("m1", move |rt| {
        push_log(&l_m1, "m1");
        // From inside m1, enqueue a microtask AND a macrotask.
        let l_micro = l_micro_handle.clone();
        rt.enqueue_microtask("micro", move |_rt| {
            push_log(&l_micro, "micro-from-m1");
            Ok(())
        });
        let l_m2 = l_m2_handle.clone();
        rt.enqueue_macrotask("m2", move |_rt| {
            push_log(&l_m2, "m2");
            Ok(())
        });
        Ok(())
    });
    let _ = l_m2; let _ = l_micro;
    rt.run_to_completion().unwrap();
    // Expected order: m1, then microtask drain (micro-from-m1), then m2
    assert_eq!(*log.borrow(), vec!["m1", "micro-from-m1", "m2"]);
}

#[test]
fn microtasks_drain_before_first_macrotask() {
    // If both queues have pre-existing entries, microtasks drain first
    // (per the run-loop's phase ordering).
    let mut rt = Runtime::new();
    let log = log_setup();
    let l_micro = log.clone();
    let l_macro = log.clone();
    rt.enqueue_macrotask("macro1", move |_rt| { push_log(&l_macro, "macro1"); Ok(()) });
    rt.enqueue_microtask("micro1", move |_rt| { push_log(&l_micro, "micro1"); Ok(()) });
    rt.run_to_completion().unwrap();
    assert_eq!(*log.borrow(), vec!["micro1", "macro1"]);
}

// ─────────── Nested enqueue patterns ───────────

#[test]
fn macrotask_can_enqueue_nested_microtasks() {
    // m1 enqueues micro_a; micro_a enqueues micro_b; both micros run
    // before m2.
    let mut rt = Runtime::new();
    let log = log_setup();
    let l_m1 = log.clone();
    let l_a_handle = log.clone();
    let l_b_handle = log.clone();
    let l_m2 = log.clone();
    rt.enqueue_macrotask("m1", move |rt| {
        push_log(&l_m1, "m1");
        let l_a = l_a_handle.clone();
        let l_b = l_b_handle.clone();
        rt.enqueue_microtask("a", move |rt| {
            push_log(&l_a, "a");
            let l_b = l_b.clone();
            rt.enqueue_microtask("b", move |_rt| {
                push_log(&l_b, "b");
                Ok(())
            });
            Ok(())
        });
        Ok(())
    });
    rt.enqueue_macrotask("m2", move |_rt| { push_log(&l_m2, "m2"); Ok(()) });
    rt.run_to_completion().unwrap();
    assert_eq!(*log.borrow(), vec!["m1", "a", "b", "m2"]);
}

#[test]
fn deep_microtask_chain_drains_all() {
    // A microtask enqueues another, which enqueues another, ... drains all.
    let mut rt = Runtime::new();
    let log = log_setup();
    let l_log = log.clone();
    // Use a Cell to share count across closure boundaries.
    let counter = Rc::new(RefCell::new(0u32));
    fn enqueue_next(rt: &mut Runtime, counter: Rc<RefCell<u32>>, log: Rc<RefCell<Vec<&'static str>>>) {
        let c = counter.clone();
        let l = log.clone();
        rt.enqueue_microtask("chain", move |rt| {
            let mut count = c.borrow_mut();
            *count += 1;
            if *count >= 10 { return Ok(()); }
            drop(count);
            let l_push = l.clone();
            push_log(&l_push, "chain");
            enqueue_next(rt, c.clone(), l.clone());
            Ok(())
        });
    }
    enqueue_next(&mut rt, counter.clone(), l_log.clone());
    rt.run_to_completion().unwrap();
    assert!(*counter.borrow() >= 10);
}

// ─────────── Job-error propagation ───────────

#[test]
fn microtask_error_propagates_out() {
    let mut rt = Runtime::new();
    rt.enqueue_microtask("err", |_rt| {
        Err(rusty_js_runtime::RuntimeError::TypeError("intentional".into()))
    });
    let result = rt.run_to_completion();
    assert!(result.is_err());
}

#[test]
fn macrotask_error_propagates_out() {
    let mut rt = Runtime::new();
    rt.enqueue_macrotask("err", |_rt| {
        Err(rusty_js_runtime::RuntimeError::TypeError("intentional".into()))
    });
    let result = rt.run_to_completion();
    assert!(result.is_err());
}

// ─────────── Safety bound ───────────

// (Not exercised in v1 tests — the max-iteration bound is 10M, well
// beyond what a normal test would trigger.)
