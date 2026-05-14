//! Cycle-collection test for the round 3.e.d Value::Object -> ObjectId
//! migration. Proves that the mark-sweep GC reclaims cycles that the
//! pre-migration Rc-based representation would have leaked.

use rusty_js_runtime::value::{Object, Value};
use rusty_js_runtime::Runtime;

#[test]
fn cycle_collection_reclaims_unreachable_pair() {
    let mut rt = Runtime::new();

    // Build a 2-object cycle: a.b = b; b.a = a.
    let a = rt.alloc_object(Object::new_ordinary());
    let b = rt.alloc_object(Object::new_ordinary());
    rt.object_set(a, "b".into(), Value::Object(b));
    rt.object_set(b, "a".into(), Value::Object(a));

    // Make them reachable via globals first.
    rt.globals.insert("a".into(), Value::Object(a));
    rt.globals.insert("b".into(), Value::Object(b));

    let before_anchor = rt.heap.live_count();
    // Confirm the GC keeps them while they are reachable.
    let freed_when_reachable = rt.collect();
    assert_eq!(freed_when_reachable, 0,
        "reachable cycle should not be freed (freed={})", freed_when_reachable);
    let after_anchor = rt.heap.live_count();
    assert_eq!(before_anchor, after_anchor);

    // Drop the root references — the cycle is now unreachable but still
    // references itself. The pre-migration Rc<RefCell> representation
    // would leak here. The GC should reclaim both slots.
    rt.globals.remove("a");
    rt.globals.remove("b");

    let live_before = rt.heap.live_count();
    let freed = rt.collect();
    let live_after = rt.heap.live_count();

    println!("gc_cycle: live_before={} freed={} live_after={}",
        live_before, freed, live_after);

    assert!(live_before - live_after >= 2,
        "cycle should drop live_count by at least 2 (before={}, after={}, freed={})",
        live_before, live_after, freed);
    assert!(freed >= 2,
        "collect() should report at least 2 freed slots, got {}", freed);
}
