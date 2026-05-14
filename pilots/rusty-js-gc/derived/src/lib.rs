//! rusty-js-gc — stop-the-world mark-sweep managed heap. Per
//! specs/rusty-js-gc-design.md.
//!
//! v1 round 3.e.b: standalone Heap<T: Trace> + ObjectId + alloc + mark +
//! sweep. Tested in isolation against a synthetic test-node type. The
//! runtime migration (Value::Object: Rc<RefCell<Object>> -> ObjectId)
//! lands in round 3.e.c after this pilot's algorithms are verified.

/// Handle into a heap. u32 index is dense; the same slot may be reused
/// after collection (v1 doesn't bump generation counters per re-use, so
/// stale handles after sweep silently alias new objects — caller's
/// responsibility per the v1 design's "no incremental, no compaction"
/// commitment).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId(pub u32);

/// Tri-color marking state per spec §III.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color { White, Gray, Black }

/// Heap-managed values implement this trait to declare their out-edges.
/// The mark phase calls trace(ids) on each reached object; the
/// implementation pushes any ObjectIds it holds into `ids`.
pub trait Trace {
    fn trace(&self, ids: &mut Vec<ObjectId>);
}

/// Heap slot: either occupied by a value of type T, or free for reuse.
#[derive(Debug)]
pub enum Slot<T> {
    Object(T),
    Free,
}

pub struct Heap<T: Trace> {
    slots: Vec<Slot<T>>,
    colors: Vec<Color>,
    free_list: Vec<u32>,
    /// Allocations since last collection.
    alloc_count: usize,
    /// Trigger threshold. Adaptive: starts at 1024, doubles after each
    /// cycle until the live set stabilizes.
    threshold: usize,
}

impl<T: Trace> Default for Heap<T> {
    fn default() -> Self { Self::new() }
}

impl<T: Trace> Heap<T> {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            colors: Vec::new(),
            free_list: Vec::new(),
            alloc_count: 0,
            threshold: 1024,
        }
    }

    /// Total slot count (occupied + free).
    pub fn len(&self) -> usize { self.slots.len() }

    /// Allocate a new object. Returns its handle.
    pub fn alloc(&mut self, v: T) -> ObjectId {
        self.alloc_count += 1;
        let id = if let Some(idx) = self.free_list.pop() {
            self.slots[idx as usize] = Slot::Object(v);
            self.colors[idx as usize] = Color::White;
            idx
        } else {
            let idx = self.slots.len() as u32;
            self.slots.push(Slot::Object(v));
            self.colors.push(Color::White);
            idx
        };
        ObjectId(id)
    }

    pub fn get(&self, id: ObjectId) -> Option<&T> {
        match self.slots.get(id.0 as usize) {
            Some(Slot::Object(v)) => Some(v),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, id: ObjectId) -> Option<&mut T> {
        match self.slots.get_mut(id.0 as usize) {
            Some(Slot::Object(v)) => Some(v),
            _ => None,
        }
    }

    pub fn is_free(&self, id: ObjectId) -> bool {
        matches!(self.slots.get(id.0 as usize), Some(Slot::Free) | None)
    }

    /// Free a slot directly. Caller-managed; the GC handles automatic
    /// freeing via sweep.
    pub fn free(&mut self, id: ObjectId) {
        let i = id.0 as usize;
        if i < self.slots.len() {
            self.slots[i] = Slot::Free;
            self.colors[i] = Color::White;
            self.free_list.push(id.0);
        }
    }

    /// Run a full mark-sweep cycle. `roots` enumerates the GC roots
    /// (every ObjectId currently reachable from outside the heap).
    /// Returns the number of slots freed in this cycle.
    pub fn collect(&mut self, roots: impl IntoIterator<Item = ObjectId>) -> usize {
        // 1. Reset all colors to WHITE. (Free slots stay WHITE; doesn't matter.)
        for c in self.colors.iter_mut() {
            *c = Color::White;
        }
        // 2. Mark roots GRAY.
        let mut worklist: Vec<ObjectId> = Vec::new();
        for r in roots {
            let i = r.0 as usize;
            if i < self.slots.len() && matches!(self.slots[i], Slot::Object(_)) && self.colors[i] == Color::White {
                self.colors[i] = Color::Gray;
                worklist.push(r);
            }
        }
        // 3. Trace.
        while let Some(id) = worklist.pop() {
            let i = id.0 as usize;
            self.colors[i] = Color::Black;
            let mut out_edges: Vec<ObjectId> = Vec::new();
            if let Slot::Object(obj) = &self.slots[i] {
                obj.trace(&mut out_edges);
            }
            for e in out_edges {
                let ei = e.0 as usize;
                if ei < self.slots.len() && matches!(self.slots[ei], Slot::Object(_)) && self.colors[ei] == Color::White {
                    self.colors[ei] = Color::Gray;
                    worklist.push(e);
                }
            }
        }
        // 4. Sweep: free WHITE; reset BLACK to WHITE for next cycle.
        let mut freed = 0usize;
        for i in 0..self.slots.len() {
            match (self.colors[i], &self.slots[i]) {
                (Color::White, Slot::Object(_)) => {
                    self.slots[i] = Slot::Free;
                    self.free_list.push(i as u32);
                    freed += 1;
                }
                (Color::Black, _) => self.colors[i] = Color::White,
                _ => {}
            }
        }
        // 5. Reset alloc count; adapt threshold to recent allocation
        // pressure so the next cycle runs at the right time.
        self.alloc_count = 0;
        // Adaptive threshold: aim for 2x the live-object count, with floor.
        let live = self.slots.iter().filter(|s| matches!(s, Slot::Object(_))).count();
        self.threshold = (live * 2).max(1024);
        freed
    }

    /// Run a cycle only if allocation pressure exceeds the threshold.
    pub fn maybe_collect(&mut self, roots: impl IntoIterator<Item = ObjectId>) -> Option<usize> {
        if self.alloc_count >= self.threshold {
            Some(self.collect(roots))
        } else { None }
    }

    /// Number of allocations since last collection.
    pub fn alloc_count(&self) -> usize { self.alloc_count }

    /// Number of currently-occupied slots.
    pub fn live_count(&self) -> usize {
        self.slots.iter().filter(|s| matches!(s, Slot::Object(_))).count()
    }

    /// Number of slots on the free list.
    pub fn free_count(&self) -> usize { self.free_list.len() }
}
