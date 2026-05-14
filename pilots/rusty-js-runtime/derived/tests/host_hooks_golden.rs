//! PollIo host-hook integration tests. Simulates a host wiring an
//! OS-I/O multiplexer to the engine's run_to_completion: the engine
//! consults PollIo at idle; the host enqueues macrotasks; the engine
//! drives them; loops until quiescent.
//!
//! Round Ω.3.f.c: validates the host-hook event-loop integration
//! contract per design spec §II piece 2.

use rusty_js_runtime::{HostHook, Runtime};
use std::cell::RefCell;
use std::rc::Rc;

// ─────────── Single PollIo call enqueues a macrotask ───────────

#[test]
fn poll_io_can_enqueue_macrotask_then_exit() {
    let mut rt = Runtime::new();
    let log = Rc::new(RefCell::new(Vec::<&'static str>::new()));
    let calls = Rc::new(RefCell::new(0u32));

    let calls_h = calls.clone();
    let log_h = log.clone();
    rt.install_host_hook(HostHook::PollIo(Box::new(move |rt| {
        let mut n = calls_h.borrow_mut();
        *n += 1;
        if *n == 1 {
            // First call: enqueue a macrotask, return true (more work).
            let lh = log_h.clone();
            rt.enqueue_macrotask("io_event", move |_rt| {
                lh.borrow_mut().push("io_event");
                Ok(())
            });
            Ok(true)
        } else {
            // Subsequent calls: no work pending; exit.
            Ok(false)
        }
    })));

    rt.run_to_completion().unwrap();
    assert_eq!(*log.borrow(), vec!["io_event"]);
    assert_eq!(*calls.borrow(), 2, "PollIo called once to enqueue + once to confirm idle");
}

// ─────────── PollIo not called when work is pending ───────────

#[test]
fn poll_io_not_called_when_macrotasks_remain() {
    // If the engine has pre-existing macrotasks, PollIo is not consulted
    // until those drain.
    let mut rt = Runtime::new();
    let poll_calls = Rc::new(RefCell::new(0u32));
    let pc = poll_calls.clone();

    rt.install_host_hook(HostHook::PollIo(Box::new(move |_rt| {
        *pc.borrow_mut() += 1;
        Ok(false)
    })));

    let log = Rc::new(RefCell::new(0u32));
    let log_h = log.clone();
    rt.enqueue_macrotask("m1", move |_rt| {
        *log_h.borrow_mut() += 1;
        Ok(())
    });

    rt.run_to_completion().unwrap();
    assert_eq!(*log.borrow(), 1);
    // PollIo called once (after m1 drained, to confirm no more work).
    assert_eq!(*poll_calls.borrow(), 1);
}

// ─────────── Repeated I/O readiness simulating a server ───────────

#[test]
fn poll_io_drives_an_event_stream() {
    // Simulates a 3-event I/O stream: the host's PollIo returns 3
    // events in sequence, then signals exit.
    let mut rt = Runtime::new();
    let log = Rc::new(RefCell::new(Vec::<&'static str>::new()));
    let events_remaining = Rc::new(RefCell::new(3i32));

    let er = events_remaining.clone();
    let log_h = log.clone();
    rt.install_host_hook(HostHook::PollIo(Box::new(move |rt| {
        let mut remaining = er.borrow_mut();
        if *remaining > 0 {
            *remaining -= 1;
            let lh = log_h.clone();
            rt.enqueue_macrotask("io_event", move |_rt| {
                lh.borrow_mut().push("event");
                Ok(())
            });
            Ok(true)
        } else {
            Ok(false)
        }
    })));

    rt.run_to_completion().unwrap();
    assert_eq!(*log.borrow(), vec!["event", "event", "event"]);
}

// ─────────── PollIo can enqueue microtask + macrotask ───────────

#[test]
fn poll_io_can_enqueue_microtask_too() {
    // The host might want to enqueue a microtask in response to an
    // I/O event (e.g., to resolve a Promise). Engine should drain
    // microtasks before the next macrotask phase.
    let mut rt = Runtime::new();
    let log = Rc::new(RefCell::new(Vec::<&'static str>::new()));
    let fired = Rc::new(RefCell::new(false));

    let f = fired.clone();
    let l = log.clone();
    rt.install_host_hook(HostHook::PollIo(Box::new(move |rt| {
        if *f.borrow() { return Ok(false); }
        *f.borrow_mut() = true;
        let l_micro = l.clone();
        let l_macro = l.clone();
        rt.enqueue_macrotask("macro", move |_rt| {
            l_macro.borrow_mut().push("macro");
            Ok(())
        });
        rt.enqueue_microtask("micro", move |_rt| {
            l_micro.borrow_mut().push("micro");
            Ok(())
        });
        Ok(true)
    })));

    rt.run_to_completion().unwrap();
    // Microtask drains first (between PollIo enqueue and macro), then macro.
    assert_eq!(*log.borrow(), vec!["micro", "macro"]);
}

// ─────────── PollIo error propagates ───────────

#[test]
fn poll_io_error_propagates() {
    let mut rt = Runtime::new();
    rt.install_host_hook(HostHook::PollIo(Box::new(|_rt| {
        Err(rusty_js_runtime::RuntimeError::TypeError("io failure".into()))
    })));
    let result = rt.run_to_completion();
    assert!(result.is_err());
}

// ─────────── No host hook installed ───────────

#[test]
fn no_host_hook_exits_at_idle() {
    let mut rt = Runtime::new();
    // No PollIo hook installed; no jobs.
    rt.run_to_completion().expect("should exit immediately");
}

// ─────────── Composite: existing macrotask drains, then PollIo fires once ───────────

#[test]
fn pre_existing_macrotask_drains_then_poll_io_fires() {
    let mut rt = Runtime::new();
    let log = Rc::new(RefCell::new(Vec::<&'static str>::new()));
    let polled = Rc::new(RefCell::new(false));

    let p_handle = polled.clone();
    let log_p = log.clone();
    rt.install_host_hook(HostHook::PollIo(Box::new(move |rt| {
        if *p_handle.borrow() { return Ok(false); }
        *p_handle.borrow_mut() = true;
        let lh = log_p.clone();
        rt.enqueue_macrotask("polled_event", move |_rt| {
            lh.borrow_mut().push("polled_event");
            Ok(())
        });
        Ok(true)
    })));

    let log_pre = log.clone();
    rt.enqueue_macrotask("pre_existing", move |_rt| {
        log_pre.borrow_mut().push("pre_existing");
        Ok(())
    });

    rt.run_to_completion().unwrap();
    assert_eq!(*log.borrow(), vec!["pre_existing", "polled_event"]);
}
