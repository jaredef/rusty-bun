//! Standalone mark-sweep verification with a synthetic Node type.
//!
//! Each Node carries an optional reference + an optional second
//! reference. Builds graphs (chains, cycles, trees, islands) and asserts
//! the collector frees exactly the right slots.

use rusty_js_gc::{Heap, ObjectId, Trace};

#[derive(Debug, Default)]
struct Node {
    label: String,
    refs: Vec<ObjectId>,
}

impl Trace for Node {
    fn trace(&self, ids: &mut Vec<ObjectId>) {
        ids.extend(self.refs.iter().copied());
    }
}

fn n(label: &str) -> Node {
    Node { label: label.to_string(), refs: Vec::new() }
}

// ─────────── Allocation + free-list ───────────

#[test]
fn alloc_increments_len() {
    let mut h: Heap<Node> = Heap::new();
    let _a = h.alloc(n("a"));
    let _b = h.alloc(n("b"));
    assert_eq!(h.len(), 2);
    assert_eq!(h.live_count(), 2);
    assert_eq!(h.free_count(), 0);
}

#[test]
fn free_pushes_to_free_list() {
    let mut h: Heap<Node> = Heap::new();
    let a = h.alloc(n("a"));
    h.free(a);
    assert_eq!(h.live_count(), 0);
    assert_eq!(h.free_count(), 1);
}

#[test]
fn alloc_reuses_free_slot() {
    let mut h: Heap<Node> = Heap::new();
    let a = h.alloc(n("a"));
    h.free(a);
    let b = h.alloc(n("b"));
    // Reuse: b should land on a's old slot index
    assert_eq!(a.0, b.0);
    assert_eq!(h.free_count(), 0);
}

// ─────────── Mark phase ───────────

#[test]
fn collect_frees_unreachable() {
    let mut h: Heap<Node> = Heap::new();
    let a = h.alloc(n("a"));
    let _b = h.alloc(n("b"));  // unreachable from roots
    let _c = h.alloc(n("c"));
    let freed = h.collect([a]);
    assert_eq!(freed, 2, "b and c should be freed");
    assert_eq!(h.live_count(), 1, "only a remains");
    assert!(!h.is_free(a));
}

#[test]
fn collect_preserves_reachable_chain() {
    // a -> b -> c, root = a, all three reachable
    let mut h: Heap<Node> = Heap::new();
    let a = h.alloc(n("a"));
    let b = h.alloc(n("b"));
    let c = h.alloc(n("c"));
    h.get_mut(a).unwrap().refs.push(b);
    h.get_mut(b).unwrap().refs.push(c);
    let freed = h.collect([a]);
    assert_eq!(freed, 0);
    assert_eq!(h.live_count(), 3);
}

#[test]
fn collect_frees_cycle_when_unreachable() {
    // a -> b -> a (cycle), then drop external root, then sweep.
    // Rc-based heap would leak; mark-sweep frees both.
    let mut h: Heap<Node> = Heap::new();
    let a = h.alloc(n("a"));
    let b = h.alloc(n("b"));
    h.get_mut(a).unwrap().refs.push(b);
    h.get_mut(b).unwrap().refs.push(a);
    // Collect with NO roots — both should be freed.
    let freed = h.collect([]);
    assert_eq!(freed, 2, "cycle should be collected when external root drops");
    assert!(h.is_free(a));
    assert!(h.is_free(b));
}

#[test]
fn collect_preserves_reachable_cycle() {
    // a -> b -> a (cycle), root = a, both reachable
    let mut h: Heap<Node> = Heap::new();
    let a = h.alloc(n("a"));
    let b = h.alloc(n("b"));
    h.get_mut(a).unwrap().refs.push(b);
    h.get_mut(b).unwrap().refs.push(a);
    let freed = h.collect([a]);
    assert_eq!(freed, 0);
    assert_eq!(h.live_count(), 2);
}

#[test]
fn collect_traces_tree() {
    // a -> {b, c}; b -> {d, e}; root = a → all five reachable
    let mut h: Heap<Node> = Heap::new();
    let a = h.alloc(n("a"));
    let b = h.alloc(n("b"));
    let c = h.alloc(n("c"));
    let d = h.alloc(n("d"));
    let e = h.alloc(n("e"));
    h.get_mut(a).unwrap().refs.extend([b, c]);
    h.get_mut(b).unwrap().refs.extend([d, e]);
    let freed = h.collect([a]);
    assert_eq!(freed, 0);
    assert_eq!(h.live_count(), 5);
}

#[test]
fn collect_frees_island() {
    // Two disconnected components: {a -> b} and {c -> d}. Root = a.
    // c and d are an island; both should be freed.
    let mut h: Heap<Node> = Heap::new();
    let a = h.alloc(n("a"));
    let b = h.alloc(n("b"));
    let c = h.alloc(n("c"));
    let d = h.alloc(n("d"));
    h.get_mut(a).unwrap().refs.push(b);
    h.get_mut(c).unwrap().refs.push(d);
    let freed = h.collect([a]);
    assert_eq!(freed, 2, "island {{c, d}} should be freed");
    assert_eq!(h.live_count(), 2);
}

#[test]
fn collect_multiple_roots() {
    let mut h: Heap<Node> = Heap::new();
    let a = h.alloc(n("a"));
    let b = h.alloc(n("b"));
    let _c = h.alloc(n("c"));
    let freed = h.collect([a, b]);
    assert_eq!(freed, 1, "c was unreachable");
    assert_eq!(h.live_count(), 2);
}

#[test]
fn second_collect_preserves_color_state() {
    // After one cycle, all BLACK objects reset to WHITE for the next.
    // Collecting again should produce the same survivors.
    let mut h: Heap<Node> = Heap::new();
    let a = h.alloc(n("a"));
    let b = h.alloc(n("b"));
    h.get_mut(a).unwrap().refs.push(b);
    let f1 = h.collect([a]);
    let f2 = h.collect([a]);
    assert_eq!(f1, 0);
    assert_eq!(f2, 0);
    assert_eq!(h.live_count(), 2);
}

// ─────────── Threshold-driven collection ───────────

#[test]
fn maybe_collect_skips_below_threshold() {
    let mut h: Heap<Node> = Heap::new();
    let _a = h.alloc(n("a"));
    // alloc_count = 1 << threshold (1024)
    let result = h.maybe_collect([]);
    assert!(result.is_none(), "should not collect below threshold");
}

#[test]
fn maybe_collect_runs_at_threshold() {
    let mut h: Heap<Node> = Heap::new();
    // Allocate enough to exceed threshold.
    for i in 0..1500 {
        let _ = h.alloc(Node { label: format!("n{}", i), refs: Vec::new() });
    }
    let result = h.maybe_collect([]);
    assert!(result.is_some(), "should collect at threshold");
    assert_eq!(result.unwrap(), 1500, "all unrooted should free");
}

// ─────────── Slot reuse after collection ───────────

#[test]
fn alloc_after_collection_reuses_slots() {
    let mut h: Heap<Node> = Heap::new();
    let _a = h.alloc(n("a"));
    let _b = h.alloc(n("b"));
    h.collect([]);  // free both
    assert_eq!(h.free_count(), 2);
    let _c = h.alloc(n("c"));
    let _d = h.alloc(n("d"));
    assert_eq!(h.free_count(), 0, "free list should drain on realloc");
    assert_eq!(h.live_count(), 2);
    assert_eq!(h.len(), 2, "no new slots created");
}
