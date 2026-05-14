//! JobQueue + run-loop driver per ECMA-262 §9.4 + WHATWG HTML §8.
//! See specs/rusty-js-runtime-event-loop-design.md.
//!
//! Round Ω.3.f.b scope: JobQueue struct, enqueue API, run_to_completion
//! driver implementing HTML's event-loop algorithm. Host hooks for OS I/O
//! sources land in Ω.3.f.c; Promise reaction routing lands in Ω.3.f.d.

use crate::interp::{Runtime, RuntimeError};
use std::collections::VecDeque;

/// A scheduled unit of work. v1 carries a closure that consumes &mut
/// Runtime. The closure form lets the enqueuer capture whatever values
/// it needs; the run-loop hands the runtime back to the closure at job
/// time.
pub struct Job {
    /// Job label for diagnostic + source-map purposes. Not consulted by
    /// the run-loop.
    pub label: &'static str,
    pub kind: JobKind,
}

pub enum JobKind {
    /// Closure-style job. Captures whatever state the enqueuer needs.
    Closure(Box<dyn FnOnce(&mut Runtime) -> Result<(), RuntimeError>>),
}

impl std::fmt::Debug for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Job {{ label: {:?} }}", self.label)
    }
}

/// FIFO queues per ECMA-262 §9.4.1.
#[derive(Default)]
pub struct JobQueue {
    /// Microtasks — Promise reactions and other "drain between
    /// macrotasks" work. Per §9.4.1: drain to quiescence between every
    /// macrotask boundary; microtasks enqueued during drain process in
    /// the same drain phase.
    microtasks: VecDeque<Job>,
    /// Macrotasks — timer firings, ready I/O completions, host-scheduled
    /// work. Per HTML §8: one macrotask runs to completion per loop
    /// iteration; the next is dequeued after the post-macrotask
    /// microtask drain.
    macrotasks: VecDeque<Job>,
}

impl JobQueue {
    pub fn new() -> Self { Self::default() }
    pub fn microtask_count(&self) -> usize { self.microtasks.len() }
    pub fn macrotask_count(&self) -> usize { self.macrotasks.len() }
    pub fn is_empty(&self) -> bool {
        self.microtasks.is_empty() && self.macrotasks.is_empty()
    }
}

impl Runtime {
    /// Enqueue a microtask. Per ECMA-262 §9.4.1, microtasks are drained
    /// between macrotasks; a microtask enqueued during a drain is
    /// processed in the same drain phase.
    pub fn enqueue_microtask<F>(&mut self, label: &'static str, f: F)
    where F: FnOnce(&mut Runtime) -> Result<(), RuntimeError> + 'static {
        self.job_queue.microtasks.push_back(Job {
            label, kind: JobKind::Closure(Box::new(f)),
        });
    }

    /// Enqueue a macrotask. Per HTML §8, one macrotask runs to
    /// completion per event-loop iteration.
    pub fn enqueue_macrotask<F>(&mut self, label: &'static str, f: F)
    where F: FnOnce(&mut Runtime) -> Result<(), RuntimeError> + 'static {
        self.job_queue.macrotasks.push_back(Job {
            label, kind: JobKind::Closure(Box::new(f)),
        });
    }

    /// HTML event-loop algorithm. Drives the engine to quiescence:
    ///   1. Drain microtasks to quiescence.
    ///   2. Dequeue one macrotask; run it; goto 1.
    ///   3. If no macrotask remains and no I/O is pending, exit.
    ///
    /// Round Ω.3.f.b: phase 3 simply exits when both queues are empty.
    /// Round Ω.3.f.c wires host PollIo to wait for I/O readiness at
    /// idle before exiting.
    pub fn run_to_completion(&mut self) -> Result<(), RuntimeError> {
        // Safety bound. Each iteration advances one macrotask or drains
        // microtasks; pathological enqueue-during-job-loops are bounded
        // by this counter so the engine never hangs unbounded.
        let max_iterations = 10_000_000usize;
        let mut iter = 0;
        loop {
            iter += 1;
            if iter > max_iterations {
                return Err(RuntimeError::TypeError(
                    "run_to_completion: max-iteration safety bound exceeded".into()
                ));
            }
            // Phase 1: drain microtasks to quiescence per §9.4.1.
            while let Some(job) = self.job_queue.microtasks.pop_front() {
                self.run_job(job)?;
            }
            // Phase 2: advance one macrotask.
            if let Some(job) = self.job_queue.macrotasks.pop_front() {
                self.run_job(job)?;
                continue;
            }
            // Phase 3: idle. Round Ω.3.f.b exits immediately; 3.f.c
            // consults host PollIo here.
            return Ok(());
        }
    }

    fn run_job(&mut self, job: Job) -> Result<(), RuntimeError> {
        match job.kind {
            JobKind::Closure(f) => f(self),
        }
    }
}
