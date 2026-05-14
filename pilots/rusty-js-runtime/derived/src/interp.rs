//! Bytecode dispatch loop + Runtime + Frame management.
//! Per specs/rusty-js-runtime-design.md §III.

use crate::abstract_ops::*;
use crate::value::{new_upvalue_cell, InternalKind, Object, ObjectRef, UpvalueCell, Value};
use rusty_js_bytecode::{
    op::{decode_i32, decode_u16, op_from_byte, Op},
    CompiledModule,
};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum RuntimeError {
    CompileError(String),
    TypeError(String),
    ReferenceError(String),
    RangeError(String),
    Unimplemented(String),
    /// Thrown JS value bubbling up the call stack.
    Thrown(Value),
}

pub struct Runtime {
    pub globals: HashMap<String, Value>,
    pub last_value: Value,
    pub host_hooks: crate::module::HostHooks,
    /// Tier-Ω.5.b: ESM module cache keyed by resolved URL
    /// (`file://...` for disk-backed modules, `node:foo` for built-ins).
    /// Interior mutability lets `evaluate_module` insert a Linking record
    /// before recursing into imports, so cyclic loads observe the partial
    /// namespace rather than re-entering parse/compile.
    pub modules: HashMap<String, std::rc::Rc<std::cell::RefCell<crate::module::ModuleRecord>>>,
    /// Managed heap. Wired but not yet authoritative for Value::Object;
    /// round 3.e.d migrates Value::Object from Rc<RefCell<Object>> to
    /// ObjectId, at which point this heap becomes the storage for every
    /// allocated Object.
    pub heap: rusty_js_gc::Heap<crate::value::Object>,
    /// Event-loop job queue per ECMA-262 §9.4 + WHATWG HTML §8.
    /// Engine-owned; replaces the pre-Ω rusty-bun-host's mio + JS-side
    /// __keepAlive + __tickKeepAlive split. Per Doc 714 §VI Consequence 5.
    pub job_queue: crate::job_queue::JobQueue,
    /// Promises that have been rejected with no reject handler attached.
    /// Per ECMA-262 §27.2.1.9 HostPromiseRejectionTracker: the host is
    /// notified at end-of-job for any rejection still without a handler.
    /// Drained by `drain_unhandled_rejections()` after run_to_completion.
    pub pending_unhandled: HashSet<rusty_js_gc::ObjectId>,
    /// `this` visible to a native function during its invocation. Set by
    /// call_function before dispatching into a NativeFn; native handlers
    /// read it via `rt.current_this()`. Tier-Ω.5.a: preserves the existing
    /// `Fn(&mut Runtime, &[Value])` NativeFn signature (no cascade through
    /// host-v2/* intrinsics) while still letting Function.prototype.call,
    /// Array.prototype.map's callback dispatch, and the like see a real
    /// receiver. Saved/restored across nested calls.
    pub current_this: Value,
    // ─── Intrinsic prototypes (Tier-Ω.5.a) ───
    //
    // Stashed ObjectIds for the canonical prototype objects. Each
    // Object that ought to inherit from one of these has its `proto`
    // field set at allocation time:
    //   - Ordinary objects -> object_prototype
    //   - Array objects    -> array_prototype
    //   - Function/Closure/BoundFunction -> function_prototype
    //   - Promise          -> promise_prototype
    // Strings + Numbers + Booleans are primitives — their method dispatch
    // routes through these stashes via `Runtime::lookup_method_on_value`
    // without allocating a wrapper.
    pub object_prototype: Option<rusty_js_gc::ObjectId>,
    pub array_prototype: Option<rusty_js_gc::ObjectId>,
    pub function_prototype: Option<rusty_js_gc::ObjectId>,
    pub promise_prototype: Option<rusty_js_gc::ObjectId>,
    pub string_prototype: Option<rusty_js_gc::ObjectId>,
    pub number_prototype: Option<rusty_js_gc::ObjectId>,
    /// Tier-Ω.5.i: %RegExp.prototype% — installed alongside other
    /// intrinsic prototypes; alloc_object auto-wires RegExp objects.
    pub regexp_prototype: Option<rusty_js_gc::ObjectId>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            last_value: Value::Undefined,
            host_hooks: crate::module::HostHooks::default(),
            modules: HashMap::new(),
            heap: rusty_js_gc::Heap::new(),
            job_queue: crate::job_queue::JobQueue::new(),
            pending_unhandled: HashSet::new(),
            current_this: Value::Undefined,
            object_prototype: None,
            array_prototype: None,
            function_prototype: None,
            promise_prototype: None,
            string_prototype: None,
            number_prototype: None,
            regexp_prototype: None,
        }
    }

    /// `this` for the active native call. Returns Undefined outside one.
    pub fn current_this(&self) -> Value { self.current_this.clone() }

    /// Drain promises still rejected with no handler. Caller is the host;
    /// canonical action is print-to-stderr + exit nonzero. Idempotent.
    pub fn drain_unhandled_rejections(&mut self) -> Vec<(rusty_js_gc::ObjectId, Value)> {
        let ids: Vec<_> = self.pending_unhandled.drain().collect();
        ids.into_iter().filter_map(|id| {
            match &self.heap.get(id)?.internal_kind {
                InternalKind::Promise(ps) if matches!(ps.status, crate::value::PromiseStatus::Rejected) => {
                    Some((id, ps.value.clone()))
                }
                _ => None,
            }
        }).collect()
    }

    /// Run a full mark-sweep cycle on the heap with the runtime's
    /// current root set.
    pub fn collect(&mut self) -> usize {
        let roots = self.enumerate_roots();
        self.heap.collect(roots)
    }

    /// Enumerate every ObjectId reachable from the runtime's roots.
    ///
    /// Tracked roots:
    ///   - self.globals.values() — every Value::Object payload
    ///   - self.last_value — if Value::Object
    ///
    /// NOT tracked (3.e.d): the active call-stack frames' operand_stack /
    /// locals / try_stack. Frames are stack-allocated on the Rust call
    /// stack inside run_frame; their values are implicit roots while the
    /// frame is on the stack. This is safe because `collect()` is only
    /// invoked outside a frame's execution (e.g. by tests or external
    /// drivers between top-level run_module calls). When `collect()` is
    /// wired into the dispatch loop at safe points, frame walking will
    /// need to be added — there is no Runtime-side frame_stack field
    /// today (run_frame is called recursively via call_function with
    /// frames living on Rust's stack).
    pub fn enumerate_roots(&self) -> Vec<rusty_js_gc::ObjectId> {
        let mut roots: Vec<rusty_js_gc::ObjectId> = Vec::new();
        for v in self.globals.values() {
            if let Value::Object(id) = v { roots.push(*id); }
        }
        if let Value::Object(id) = &self.last_value { roots.push(*id); }
        roots
    }

    /// Allocate an Object via the managed heap. Returns the ObjectId
    /// handle. Tier-Ω.5.a: if the Object has no explicit proto and an
    /// intrinsic prototype matching its InternalKind has been installed,
    /// the proto is wired automatically. This is the seam through which
    /// prototype-chain method dispatch works without retrofitting every
    /// alloc call-site.
    pub fn alloc_object(&mut self, mut obj: crate::value::Object) -> rusty_js_gc::ObjectId {
        if obj.proto.is_none() {
            obj.proto = match &obj.internal_kind {
                crate::value::InternalKind::Ordinary => self.object_prototype,
                crate::value::InternalKind::Array => self.array_prototype,
                crate::value::InternalKind::Promise(_) => self.promise_prototype,
                crate::value::InternalKind::RegExp(_) => self.regexp_prototype,
                crate::value::InternalKind::Function(_)
                | crate::value::InternalKind::Closure(_)
                | crate::value::InternalKind::BoundFunction(_) => self.function_prototype,
                _ => None,
            };
        }
        self.heap.alloc(obj)
    }

    /// Ergonomic heap accessors. Panic on missing — the migration's
    /// invariant is that every ObjectId in a live Value points to a live
    /// slot. Stale handles after a sweep would be a GC-correctness bug
    /// surfaced loudly here.
    pub fn obj(&self, id: ObjectRef) -> &Object {
        self.heap.get(id).expect("ObjectId points to free/missing slot")
    }
    pub fn obj_mut(&mut self, id: ObjectRef) -> &mut Object {
        self.heap.get_mut(id).expect("ObjectId points to free/missing slot")
    }

    /// OrdinaryGet with prototype walk. Returns Undefined if neither the
    /// object nor any prototype owns the key.
    ///
    /// Tier-Ω.5.a: special-case Array.length — computed from the highest
    /// numeric-indexed own property + 1 (own-only, prototype walk skipped
    /// for this synthetic key). Matches the spec semantics close enough
    /// for the v1 surface without maintaining a separate length slot.
    pub fn object_get(&self, id: ObjectRef, key: &str) -> Value {
        if key == "length" {
            let o = self.obj(id);
            if matches!(o.internal_kind, InternalKind::Array) {
                // If explicit "length" property is set, prefer it; otherwise
                // derive from max numeric index + 1.
                if let Some(d) = o.properties.get("length") {
                    return d.value.clone();
                }
                let mut max: i64 = -1;
                for k in o.properties.keys() {
                    if let Ok(i) = k.parse::<i64>() {
                        if i > max { max = i; }
                    }
                }
                return Value::Number((max + 1) as f64);
            }
        }
        let mut cur = Some(id);
        while let Some(c) = cur {
            let o = self.obj(c);
            if let Some(d) = o.properties.get(key) {
                return d.value.clone();
            }
            cur = o.proto;
        }
        Value::Undefined
    }

    /// Array length helper used by Array.prototype.* methods.
    pub fn array_length(&self, id: ObjectRef) -> usize {
        match self.object_get(id, "length") {
            Value::Number(n) if n.is_finite() && n >= 0.0 => n as usize,
            _ => 0,
        }
    }

    /// OrdinaryDefineOwnProperty — own-key set on the named object.
    pub fn object_set(&mut self, id: ObjectRef, key: String, value: Value) {
        self.obj_mut(id).set_own(key, value);
    }

    /// Typeof with heap deref for Object/function discrimination.
    pub fn type_of_value(&self, v: &Value) -> &'static str {
        match v {
            Value::Object(id) => {
                let o = self.obj(*id);
                if matches!(o.internal_kind,
                    InternalKind::Function(_) | InternalKind::Closure(_) | InternalKind::BoundFunction(_))
                { "function" } else { "object" }
            }
            other => other.type_of(),
        }
    }

    /// Public wrapper: run a module-level Frame. Used by evaluate_module
    /// to drive bytecode execution while retaining access to the post-
    /// execution local-slot values.
    pub fn run_frame_module(&mut self, frame: &mut Frame) -> Result<Value, RuntimeError> {
        self.run_frame(frame)
    }

    /// Execute a compiled module. Returns the terminal stack value (the
    /// last value on the operand stack at module exit) or Undefined.
    pub fn run_module(&mut self, m: &CompiledModule) -> Result<Value, RuntimeError> {
        let mut frame = Frame::new_module(m);
        self.run_frame(&mut frame)
    }

    fn run_frame(&mut self, frame: &mut Frame) -> Result<Value, RuntimeError> {
        // Outer driver: each iteration runs the inner dispatch; if a JS
        // throw bubbles up, walk the try_stack and either resume at a
        // catch handler or re-raise to the caller.
        loop {
            match self.run_frame_inner(frame) {
                Ok(v) => return Ok(v),
                Err(RuntimeError::Thrown(v)) => {
                    if let Some(t) = frame.try_stack.pop() {
                        frame.operand_stack.truncate(t.sp_at_entry);
                        frame.operand_stack.push(v);
                        frame.pc = t.catch_offset;
                        // Continue the outer loop -> re-enter the dispatch.
                    } else {
                        return Err(RuntimeError::Thrown(v));
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }

    fn run_frame_inner(&mut self, frame: &mut Frame) -> Result<Value, RuntimeError> {
        loop {
            let pc = frame.pc;
            if pc >= frame.bytecode.len() {
                return Ok(self.last_value.clone());
            }
            let op_byte = frame.bytecode[pc];
            let op = op_from_byte(op_byte)
                .ok_or_else(|| RuntimeError::Unimplemented(format!("invalid opcode 0x{:02X} @{}", op_byte, pc)))?;
            frame.pc += 1;
            match op {
                // ─── Stack ops ───
                Op::PushNull => frame.push(Value::Null),
                Op::PushUndef => frame.push(Value::Undefined),
                Op::PushTrue => frame.push(Value::Boolean(true)),
                Op::PushFalse => frame.push(Value::Boolean(false)),
                Op::PushI32 => {
                    let v = decode_i32(&frame.bytecode, frame.pc);
                    frame.pc += 4;
                    frame.push(Value::Number(v as f64));
                }
                Op::PushConst => {
                    let idx = decode_u16(&frame.bytecode, frame.pc);
                    frame.pc += 2;
                    let v = self.constant_to_value(frame, idx)?;
                    frame.push(v);
                }
                Op::Pop => { frame.pop()?; }
                Op::Dup => {
                    let top = frame.peek(0)?.clone();
                    frame.push(top);
                }
                Op::Swap => {
                    let len = frame.operand_stack.len();
                    if len < 2 { return Err(RuntimeError::TypeError("stack underflow on Swap".into())); }
                    frame.operand_stack.swap(len - 1, len - 2);
                }

                // ─── Variable / scope ───
                Op::LoadLocal => {
                    let slot = decode_u16(&frame.bytecode, frame.pc) as usize;
                    frame.pc += 2;
                    let v = frame.read_local(slot);
                    frame.push(v);
                }
                Op::StoreLocal => {
                    let slot = decode_u16(&frame.bytecode, frame.pc) as usize;
                    frame.pc += 2;
                    let v = frame.pop()?;
                    frame.write_local(slot, v);
                }
                Op::ResetLocalCell => {
                    // Detach any prior upvalue cell at this slot so the next
                    // CaptureLocal promotes to a fresh cell. Existing closures
                    // that already captured the previous cell retain their
                    // Rc<RefCell<Value>> handle — only the frame's binding to
                    // the cell is cleared. Tier-Ω.5.g.1 per-iteration binding.
                    let slot = decode_u16(&frame.bytecode, frame.pc) as usize;
                    frame.pc += 2;
                    if slot < frame.local_cells.len() {
                        frame.local_cells[slot] = None;
                    }
                }
                Op::LoadGlobal => {
                    let idx = decode_u16(&frame.bytecode, frame.pc);
                    frame.pc += 2;
                    let name = self.constant_name(frame, idx)?;
                    let v = self.globals.get(&name).cloned().unwrap_or(Value::Undefined);
                    frame.last_property_lookup = Some(format!("<global>{}", name));
                    frame.push(v);
                }
                Op::StoreGlobal => {
                    let idx = decode_u16(&frame.bytecode, frame.pc);
                    frame.pc += 2;
                    let name = self.constant_name(frame, idx)?;
                    let v = frame.pop()?;
                    self.globals.insert(name, v);
                }
                Op::LoadUpvalue => {
                    let slot = decode_u16(&frame.bytecode, frame.pc) as usize;
                    frame.pc += 2;
                    let v = frame.upvalues.get(slot)
                        .map(|cell| cell.borrow().clone())
                        .unwrap_or(Value::Undefined);
                    frame.push(v);
                }
                Op::StoreUpvalue => {
                    let slot = decode_u16(&frame.bytecode, frame.pc) as usize;
                    frame.pc += 2;
                    let v = frame.pop()?;
                    if let Some(cell) = frame.upvalues.get(slot) {
                        *cell.borrow_mut() = v;
                    } else {
                        // Out-of-range StoreUpvalue: shouldn't happen for
                        // well-formed bytecode. Extend with a fresh cell so
                        // a later LoadUpvalue at the same slot reads it back.
                        while frame.upvalues.len() <= slot { frame.upvalues.push(new_upvalue_cell(Value::Undefined)); }
                        *frame.upvalues[slot].borrow_mut() = v;
                    }
                }
                Op::CaptureLocal => {
                    // Promote outer-frame slot to a shared cell (idempotent),
                    // then push that cell's Rc into the closure's upvalues.
                    // Binding-shared semantics: outer-frame writes through
                    // the same cell, sibling closures share too.
                    let slot = decode_u16(&frame.bytecode, frame.pc) as usize;
                    frame.pc += 2;
                    let cell = frame.promote_local(slot);
                    let top = match frame.peek(0)? {
                        Value::Object(id) => *id,
                        _ => return Err(RuntimeError::TypeError("CaptureLocal: top of stack is not a closure".into())),
                    };
                    if let InternalKind::Closure(c) = &mut self.obj_mut(top).internal_kind {
                        c.upvalues.push(cell);
                    } else {
                        return Err(RuntimeError::TypeError("CaptureLocal: top is not a closure".into()));
                    }
                }
                Op::CaptureUpvalue => {
                    // Transitive capture: share the Rc<RefCell<Value>> the
                    // enclosing closure already holds. Do NOT deep-copy the
                    // value out and re-wrap — that would break binding
                    // semantics across the three-deep nesting case.
                    let idx = decode_u16(&frame.bytecode, frame.pc) as usize;
                    frame.pc += 2;
                    let cell = frame.upvalues.get(idx)
                        .cloned()
                        .unwrap_or_else(|| new_upvalue_cell(Value::Undefined));
                    let top = match frame.peek(0)? {
                        Value::Object(id) => *id,
                        _ => return Err(RuntimeError::TypeError("CaptureUpvalue: top is not a closure".into())),
                    };
                    if let InternalKind::Closure(c) = &mut self.obj_mut(top).internal_kind {
                        c.upvalues.push(cell);
                    } else {
                        return Err(RuntimeError::TypeError("CaptureUpvalue: top is not a closure".into()));
                    }
                }
                Op::DefineLocal => {
                    let slot = decode_u16(&frame.bytecode, frame.pc) as usize;
                    frame.pc += 2;
                    while frame.locals.len() <= slot { frame.locals.push(Value::Undefined); }
                }

                // ─── Arithmetic ───
                Op::Add => {
                    let r = frame.pop()?; let l = frame.pop()?;
                    frame.push(op_add(&l, &r));
                }
                Op::Sub => {
                    let r = to_number(&frame.pop()?); let l = to_number(&frame.pop()?);
                    frame.push(Value::Number(l - r));
                }
                Op::Mul => {
                    let r = to_number(&frame.pop()?); let l = to_number(&frame.pop()?);
                    frame.push(Value::Number(l * r));
                }
                Op::Div => {
                    let r = to_number(&frame.pop()?); let l = to_number(&frame.pop()?);
                    frame.push(Value::Number(l / r));
                }
                Op::Mod => {
                    let r = to_number(&frame.pop()?); let l = to_number(&frame.pop()?);
                    frame.push(Value::Number(l % r));
                }
                Op::Pow => {
                    let r = to_number(&frame.pop()?); let l = to_number(&frame.pop()?);
                    frame.push(Value::Number(l.powf(r)));
                }
                Op::Neg => {
                    let v = to_number(&frame.pop()?);
                    frame.push(Value::Number(-v));
                }
                Op::Pos => {
                    let v = to_number(&frame.pop()?);
                    frame.push(Value::Number(v));
                }
                Op::Inc => {
                    let v = to_number(&frame.pop()?);
                    frame.push(Value::Number(v + 1.0));
                }
                Op::Dec => {
                    let v = to_number(&frame.pop()?);
                    frame.push(Value::Number(v - 1.0));
                }

                // ─── Comparison / equality ───
                Op::Eq => {
                    let r = frame.pop()?; let l = frame.pop()?;
                    frame.push(Value::Boolean(is_loosely_equal(&l, &r)));
                }
                Op::Ne => {
                    let r = frame.pop()?; let l = frame.pop()?;
                    frame.push(Value::Boolean(!is_loosely_equal(&l, &r)));
                }
                Op::StrictEq => {
                    let r = frame.pop()?; let l = frame.pop()?;
                    frame.push(Value::Boolean(is_strictly_equal(&l, &r)));
                }
                Op::StrictNe => {
                    let r = frame.pop()?; let l = frame.pop()?;
                    frame.push(Value::Boolean(!is_strictly_equal(&l, &r)));
                }
                Op::Lt | Op::Gt | Op::Le | Op::Ge => {
                    let r = frame.pop()?; let l = frame.pop()?;
                    let ord = abstract_relational_compare(&l, &r);
                    let result = match op {
                        Op::Lt => matches!(ord, RelOrder::Less),
                        Op::Gt => matches!(ord, RelOrder::Greater),
                        Op::Le => matches!(ord, RelOrder::Less | RelOrder::Equal),
                        Op::Ge => matches!(ord, RelOrder::Greater | RelOrder::Equal),
                        _ => unreachable!(),
                    };
                    frame.push(Value::Boolean(result));
                }

                // ─── Bitwise / shift ───
                Op::BitAnd => {
                    let r = to_number(&frame.pop()?) as i32;
                    let l = to_number(&frame.pop()?) as i32;
                    frame.push(Value::Number((l & r) as f64));
                }
                Op::BitOr => {
                    let r = to_number(&frame.pop()?) as i32;
                    let l = to_number(&frame.pop()?) as i32;
                    frame.push(Value::Number((l | r) as f64));
                }
                Op::BitXor => {
                    let r = to_number(&frame.pop()?) as i32;
                    let l = to_number(&frame.pop()?) as i32;
                    frame.push(Value::Number((l ^ r) as f64));
                }
                Op::BitNot => {
                    let v = to_number(&frame.pop()?) as i32;
                    frame.push(Value::Number((!v) as f64));
                }
                Op::Shl => {
                    let r = (to_number(&frame.pop()?) as u32) & 0x1F;
                    let l = to_number(&frame.pop()?) as i32;
                    frame.push(Value::Number((l << r) as f64));
                }
                Op::Shr => {
                    let r = (to_number(&frame.pop()?) as u32) & 0x1F;
                    let l = to_number(&frame.pop()?) as i32;
                    frame.push(Value::Number((l >> r) as f64));
                }
                Op::UShr => {
                    let r = (to_number(&frame.pop()?) as u32) & 0x1F;
                    let l = to_number(&frame.pop()?) as u32;
                    frame.push(Value::Number((l >> r) as f64));
                }

                // ─── Logical ───
                Op::Not => {
                    let v = to_boolean(&frame.pop()?);
                    frame.push(Value::Boolean(!v));
                }

                // ─── Unary type / void ───
                Op::Typeof => {
                    let v = frame.pop()?;
                    let t = self.type_of_value(&v);
                    frame.push(Value::String(Rc::new(t.to_string())));
                }
                Op::Void => {
                    let _ = frame.pop()?;
                    frame.push(Value::Undefined);
                }

                // ─── Control flow ───
                Op::Jump => {
                    let disp = decode_i32(&frame.bytecode, frame.pc);
                    frame.pc += 4;
                    frame.pc = (frame.pc as i32 + disp) as usize;
                }
                Op::JumpIfTrue => {
                    let disp = decode_i32(&frame.bytecode, frame.pc);
                    frame.pc += 4;
                    if to_boolean(&frame.pop()?) {
                        frame.pc = (frame.pc as i32 + disp) as usize;
                    }
                }
                Op::JumpIfFalse => {
                    let disp = decode_i32(&frame.bytecode, frame.pc);
                    frame.pc += 4;
                    if !to_boolean(&frame.pop()?) {
                        frame.pc = (frame.pc as i32 + disp) as usize;
                    }
                }
                Op::JumpIfTrueKeep => {
                    let disp = decode_i32(&frame.bytecode, frame.pc);
                    frame.pc += 4;
                    if to_boolean(frame.peek(0)?) {
                        frame.pc = (frame.pc as i32 + disp) as usize;
                    }
                }
                Op::JumpIfFalseKeep => {
                    let disp = decode_i32(&frame.bytecode, frame.pc);
                    frame.pc += 4;
                    if !to_boolean(frame.peek(0)?) {
                        frame.pc = (frame.pc as i32 + disp) as usize;
                    }
                }
                Op::JumpIfNullish => {
                    let disp = decode_i32(&frame.bytecode, frame.pc);
                    frame.pc += 4;
                    let v = frame.pop()?;
                    if matches!(v, Value::Undefined | Value::Null) {
                        frame.pc = (frame.pc as i32 + disp) as usize;
                    }
                }

                // ─── Exception handling (minimal in round 3.d.c) ───
                Op::Throw => {
                    let v = frame.pop()?;
                    return Err(RuntimeError::Thrown(v));
                }
                Op::TryEnter => {
                    // catch_offset is an absolute bytecode offset where
                    // the catch handler begins. Pushed onto frame.try_stack.
                    let catch_off = rusty_js_bytecode::op::decode_u32(&frame.bytecode, frame.pc) as usize;
                    frame.pc += 4;
                    frame.try_stack.push(TryFrame {
                        catch_offset: catch_off,
                        sp_at_entry: frame.operand_stack.len(),
                    });
                }
                Op::TryExit => {
                    frame.try_stack.pop();
                }

                // ─── Returns ───
                Op::Return => {
                    let v = frame.pop()?;
                    self.last_value = v.clone();
                    return Ok(v);
                }
                Op::ReturnUndef => {
                    self.last_value = Value::Undefined;
                    return Ok(Value::Undefined);
                }

                // ─── Object / Array construction ───
                Op::NewObject => {
                    let id = self.alloc_object(Object::new_ordinary());
                    frame.push(Value::Object(id));
                }
                Op::NewArray => {
                    let _hint = decode_u16(&frame.bytecode, frame.pc);
                    frame.pc += 2;
                    let id = self.alloc_object(Object::new_array());
                    frame.push(Value::Object(id));
                }
                Op::InitProp => {
                    let idx = decode_u16(&frame.bytecode, frame.pc);
                    frame.pc += 2;
                    let key = self.constant_name(frame, idx)?;
                    let value = frame.pop()?;
                    let id = match frame.peek(0)? {
                        Value::Object(id) => *id,
                        _ => return Err(RuntimeError::TypeError("InitProp on non-object".into())),
                    };
                    self.object_set(id, key, value);
                }
                Op::InitIndex => {
                    let idx = rusty_js_bytecode::op::decode_u32(&frame.bytecode, frame.pc);
                    frame.pc += 4;
                    let value = frame.pop()?;
                    let id = match frame.peek(0)? {
                        Value::Object(id) => *id,
                        _ => return Err(RuntimeError::TypeError("InitIndex on non-array".into())),
                    };
                    self.object_set(id, idx.to_string(), value);
                }

                // ─── Property access ───
                Op::GetProp => {
                    let idx = decode_u16(&frame.bytecode, frame.pc);
                    frame.pc += 2;
                    let key = self.constant_name(frame, idx)?;
                    let obj_v = frame.pop()?;
                    let v = match &obj_v {
                        Value::Object(id) => self.object_get(*id, &key),
                        Value::String(s) if key == "length" => Value::Number(s.chars().count() as f64),
                        Value::String(_) => {
                            // Primitive string method auto-boxing: route to
                            // %String.prototype% if installed.
                            if let Some(proto) = self.string_prototype {
                                self.object_get(proto, &key)
                            } else { Value::Undefined }
                        }
                        Value::Number(_) => {
                            if let Some(proto) = self.number_prototype {
                                self.object_get(proto, &key)
                            } else { Value::Undefined }
                        }
                        Value::Undefined | Value::Null => {
                            return Err(RuntimeError::TypeError(
                                format!("Cannot read property '{}' of {}", key,
                                    if matches!(obj_v, Value::Undefined) { "undefined" } else { "null" })
                            ));
                        }
                        _ => Value::Undefined,
                    };
                    frame.last_property_lookup = Some(key);
                    frame.push(v);
                }
                Op::SetProp => {
                    let idx = decode_u16(&frame.bytecode, frame.pc);
                    frame.pc += 2;
                    let key = self.constant_name(frame, idx)?;
                    let value = frame.pop()?;
                    let obj_v = frame.pop()?;
                    if let Value::Object(id) = &obj_v {
                        self.object_set(*id, key, value.clone());
                    } else {
                        return Err(RuntimeError::TypeError("SetProp on non-object".into()));
                    }
                    frame.push(value);
                }
                Op::GetIndex => {
                    let key_v = frame.pop()?;
                    let obj_v = frame.pop()?;
                    let key = property_key(&key_v);
                    let v = match obj_v {
                        Value::Object(id) => self.object_get(id, &key),
                        Value::String(s) => {
                            if let Ok(i) = key.parse::<usize>() {
                                s.chars().nth(i)
                                    .map(|c| Value::String(Rc::new(c.to_string())))
                                    .unwrap_or(Value::Undefined)
                            } else if key == "length" {
                                Value::Number(s.chars().count() as f64)
                            } else { Value::Undefined }
                        }
                        Value::Undefined | Value::Null =>
                            return Err(RuntimeError::TypeError("Cannot index undefined/null".into())),
                        _ => Value::Undefined,
                    };
                    frame.push(v);
                }
                Op::SetPrototype => {
                    // Pop [target, proto]; proto on top.
                    let proto_v = frame.pop()?;
                    let target_v = frame.pop()?;
                    let target_id = match target_v {
                        Value::Object(id) => id,
                        _ => return Err(RuntimeError::TypeError(
                            "SetPrototype: target is not an object".into())),
                    };
                    let new_proto = match proto_v {
                        Value::Object(id) => Some(id),
                        Value::Null => None,
                        _ => return Err(RuntimeError::TypeError(
                            "SetPrototype: proto must be Object or Null".into())),
                    };
                    self.obj_mut(target_id).proto = new_proto;
                }
                Op::Instanceof => {
                    // pops [obj, ctor]; ctor on top.
                    let ctor_v = frame.pop()?;
                    let obj_v = frame.pop()?;
                    let result = match (&obj_v, &ctor_v) {
                        (Value::Object(obj_id), Value::Object(ctor_id)) => {
                            // Read ctor.prototype (own + proto-chain), walk obj's proto chain.
                            let proto_v = self.object_get(*ctor_id, "prototype");
                            match proto_v {
                                Value::Object(target_proto) => {
                                    let mut cur = self.obj(*obj_id).proto;
                                    let mut found = false;
                                    while let Some(c) = cur {
                                        if c == target_proto { found = true; break; }
                                        cur = self.obj(c).proto;
                                    }
                                    found
                                }
                                _ => false,
                            }
                        }
                        _ => false,
                    };
                    frame.push(Value::Boolean(result));
                }
                Op::SetIndex => {
                    let value = frame.pop()?;
                    let key_v = frame.pop()?;
                    let obj_v = frame.pop()?;
                    let key = property_key(&key_v);
                    if let Value::Object(id) = &obj_v {
                        self.object_set(*id, key, value.clone());
                    } else {
                        return Err(RuntimeError::TypeError("SetIndex on non-object".into()));
                    }
                    frame.push(value);
                }

                // ─── Closure construction ───
                Op::MakeClosure | Op::MakeArrow => {
                    let idx = decode_u16(&frame.bytecode, frame.pc);
                    frame.pc += 2;
                    let proto = match frame.constants.get(idx) {
                        Some(rusty_js_bytecode::Constant::Function(p)) => p.clone(),
                        _ => return Err(RuntimeError::TypeError("MakeClosure constant is not a function".into())),
                    };
                    let is_arrow = matches!(op, Op::MakeArrow);
                    let proto_rc = Rc::new(*proto);
                    let closure = Object {
                        proto: None,
                        extensible: true,
                        properties: std::collections::HashMap::new(),
                        internal_kind: crate::value::InternalKind::Closure(crate::value::ClosureInternals {
                            proto: proto_rc,
                            upvalues: Vec::new(),
                            is_arrow,
                        }),
                    };
                    let id = self.alloc_object(closure);
                    frame.push(Value::Object(id));
                }

                // ─── Function call ───
                Op::Call => {
                    let n = frame.bytecode[frame.pc] as usize;
                    frame.pc += 1;
                    let mut args = Vec::with_capacity(n);
                    for _ in 0..n {
                        args.push(frame.pop()?);
                    }
                    args.reverse();
                    let callee = frame.pop()?;
                    let callee_hint = frame.last_property_lookup.clone();
                    let result = self.call_function(callee, Value::Undefined, args).map_err(|e| match e {
                        RuntimeError::TypeError(msg) if msg.starts_with("callee is not callable") => {
                            RuntimeError::TypeError(format!("{} (callee='{}')", msg, callee_hint.unwrap_or_else(|| "?".into())))
                        }
                        other => other,
                    })?;
                    frame.push(result);
                }
                Op::CallMethod => {
                    let n = frame.bytecode[frame.pc] as usize;
                    frame.pc += 1;
                    let mut args = Vec::with_capacity(n);
                    for _ in 0..n {
                        args.push(frame.pop()?);
                    }
                    args.reverse();
                    let method = frame.pop()?;
                    let receiver = frame.pop()?;
                    let method_name = frame.last_property_lookup.clone();
                    let result = self.call_function(method, receiver, args).map_err(|e| match e {
                        RuntimeError::TypeError(msg) if msg.starts_with("callee is not callable") => {
                            RuntimeError::TypeError(format!("{} (method='{}')", msg, method_name.unwrap_or_else(|| "?".into())))
                        }
                        other => other,
                    })?;
                    frame.push(result);
                }
                Op::PushThis => {
                    let t = frame.this_value.clone();
                    frame.push(t);
                }
                Op::New => {
                    let n = frame.bytecode[frame.pc] as usize;
                    frame.pc += 1;
                    let mut args = Vec::with_capacity(n);
                    for _ in 0..n {
                        args.push(frame.pop()?);
                    }
                    args.reverse();
                    let callee = frame.pop()?;
                    // Tier-Ω.5.f: consult callee.prototype property to set
                    // the new instance's [[Prototype]]. This is the load-
                    // bearing engine change that makes user-defined classes
                    // (whose prototypes are ordinary objects with method
                    // properties, not intrinsic prototypes) work with `new`.
                    let proto_override = if let Value::Object(cid) = &callee {
                        match self.object_get(*cid, "prototype") {
                            Value::Object(pid) => Some(pid),
                            _ => None,
                        }
                    } else { None };
                    let mut ordinary = Object::new_ordinary();
                    if proto_override.is_some() {
                        ordinary.proto = proto_override;
                    }
                    let this_id = self.alloc_object(ordinary);
                    let this_obj = Value::Object(this_id);
                    let ret = self.call_function(callee, this_obj.clone(), args)?;
                    let result = match ret {
                        Value::Object(_) => ret,
                        _ => this_obj,
                    };
                    frame.push(result);
                }

                // ─── Misc ───
                Op::Nop => {}
                Op::Debugger => {}

                _ => {
                    return Err(RuntimeError::Unimplemented(format!("opcode {:?} not yet handled @{}", op, pc)));
                }
            }
        }
    }

    /// Call a function value. Materializes a new Frame from the callee's
    /// FunctionProto, populates its locals slot 0..N with the arguments,
    /// runs the frame, returns the produced value (or Undefined on ReturnUndef).
    ///
    /// Tier-Ω.5.a: `this` is now threaded — stashed onto
    /// `Runtime::current_this` around NativeFn invocations (saved/restored
    /// across nesting), and set as `Frame::this_value` for closure frames.
    /// BoundFunction unwraps once, prepending bound args and overriding the
    /// caller's `this` with the bound this.
    pub fn call_function(&mut self, callee: Value, this: Value, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let id = match callee {
            Value::Object(id) => id,
            other => return Err(RuntimeError::TypeError(format!("callee is not callable: {:?}", other))),
        };
        // Extract proto-or-native by inspecting the heap object once.
        // BoundFunction: rewrite to its target, prepending bound args.
        let (proto_opt, native_opt, effective_this, effective_args) = {
            let o = self.obj(id);
            match &o.internal_kind {
                crate::value::InternalKind::Closure(c) => (Some(c.proto.clone()), None, this, args),
                crate::value::InternalKind::Function(f) => (None, Some(f.native.clone()), this, args),
                crate::value::InternalKind::BoundFunction(b) => {
                    // One level of unwrap is sufficient for v1; nested
                    // bindings recurse via tail-call into call_function.
                    let target = b.target;
                    let bound_this = b.this.clone();
                    let mut bound_args = b.args.clone();
                    bound_args.extend(args);
                    return self.call_function(Value::Object(target), bound_this, bound_args);
                }
                other => return Err(RuntimeError::TypeError(format!("callee is not callable: Object(kind={})", other.kind_name()))),
            }
        };
        if let Some(native) = native_opt {
            let saved = std::mem::replace(&mut self.current_this, effective_this);
            let result = native(self, &effective_args);
            self.current_this = saved;
            return result;
        }
        let proto = proto_opt.expect("closure branch implies proto");
        let args = effective_args;
        let this = effective_this;
        // Tier-Ω.5.e: binding-shared upvalues. Share the closure's
        // Rc<RefCell<Value>> handles with the inner frame; writes through
        // either side land in the same cell. The outer frame that created
        // the closure shares the cell too via its promoted local slot.
        let upvalues: Vec<UpvalueCell> = {
            let o = self.obj(id);
            match &o.internal_kind {
                crate::value::InternalKind::Closure(c) => c.upvalues.clone(),
                _ => Vec::new(),
            }
        };
        // Tier-Ω.5.l: rest parameter — collect args[rest_slot..] into an
        // Array bound to the rest slot. The Array carries InternalKind::Array
        // so alloc_object auto-wires %Array.prototype%.
        let mut locals: Vec<Value> = Vec::new();
        let rest_slot = proto.rest_param_slot;
        for (i, _) in proto.locals.iter().enumerate() {
            let slot = i as u16;
            if Some(slot) == rest_slot {
                let mut rest = crate::value::Object::new_array();
                let tail: Vec<Value> = if (i as usize) < args.len() {
                    args[i as usize..].to_vec()
                } else { Vec::new() };
                rest.set_own("length".into(), Value::Number(tail.len() as f64));
                for (k, v) in tail.into_iter().enumerate() {
                    rest.set_own(k.to_string(), v);
                }
                let id = self.alloc_object(rest);
                locals.push(Value::Object(id));
            } else if i < args.len() {
                locals.push(args[i].clone());
            } else {
                locals.push(Value::Undefined);
            }
        }
        let mut inner = Frame {
            bytecode: &proto.bytecode,
            constants: &proto.constants,
            locals,
            local_cells: Vec::new(),
            operand_stack: Vec::with_capacity(32),
            pc: 0,
            try_stack: Vec::new(),
            this_value: this,
            upvalues,
            last_property_lookup: None,
        };
        self.run_frame(&mut inner)
    }

    fn constant_to_value(&self, frame: &Frame, idx: u16) -> Result<Value, RuntimeError> {
        match frame.constants.get(idx) {
            Some(rusty_js_bytecode::Constant::Number(n)) => Ok(Value::Number(*n)),
            Some(rusty_js_bytecode::Constant::String(s)) => Ok(Value::String(Rc::new(s.clone()))),
            Some(rusty_js_bytecode::Constant::BigInt(s)) => Ok(Value::BigInt(Rc::new(s.clone()))),
            Some(rusty_js_bytecode::Constant::Regex { .. }) => {
                Err(RuntimeError::Unimplemented("Regex literals not yet supported".into()))
            }
            Some(rusty_js_bytecode::Constant::Function(_)) => {
                // Function constants are not directly Pushable as values;
                // they're consumed by MakeClosure / MakeArrow. Reaching
                // here means the compiler emitted a PushConst on a
                // Function which would be a bug.
                Err(RuntimeError::TypeError("Function constant pushed as a value".into()))
            }
            None => Err(RuntimeError::TypeError(format!("invalid constant index {}", idx))),
        }
    }

    fn constant_name(&self, frame: &Frame, idx: u16) -> Result<String, RuntimeError> {
        match frame.constants.get(idx) {
            Some(rusty_js_bytecode::Constant::String(s)) => Ok(s.clone()),
            _ => Err(RuntimeError::TypeError(format!("constant {} is not a name string", idx))),
        }
    }
}

/// ToPropertyKey per ECMA-262 §7.1.19. v1 simplified: numbers stringify
/// to their canonical decimal form; other primitives ToString-coerce.
fn property_key(v: &Value) -> String {
    match v {
        Value::String(s) => s.as_str().to_string(),
        Value::Number(n) => crate::abstract_ops::number_to_string(*n),
        _ => crate::abstract_ops::to_string(v).as_str().to_string(),
    }
}

pub struct Frame<'a> {
    pub bytecode: &'a [u8],
    pub constants: &'a rusty_js_bytecode::ConstantsPool,
    pub locals: Vec<Value>,
    /// Parallel to `locals`. Tier-Ω.5.e: when a nested closure captures
    /// this frame's local slot `i`, `local_cells[i]` becomes
    /// `Some(Rc<RefCell<Value>>)` and authoritative; `locals[i]` is no
    /// longer read. Lazy in-place promotion (Approach A from the spec
    /// note) keeps unrelated frames on the fast path.
    pub local_cells: Vec<Option<UpvalueCell>>,
    pub operand_stack: Vec<Value>,
    pub pc: usize,
    pub try_stack: Vec<TryFrame>,
    /// `this` for the executing frame. Module frames default to Undefined;
    /// method-call frames receive the receiver. Tier-Ω.5.a.
    pub this_value: Value,
    /// Captured upvalues for this frame as shared binding cells. Closure
    /// frames receive Rc-clones of the closure's upvalue cells so writes
    /// propagate to the outer frame and to sibling closures. Tier-Ω.5.e.
    pub upvalues: Vec<UpvalueCell>,
    /// Diagnostic: name of the property most recently read by Op::GetProp.
    /// Used to enrich "callee is not callable" errors with the method name.
    pub last_property_lookup: Option<String>,
}

#[derive(Debug)]
pub struct TryFrame {
    pub catch_offset: usize,
    pub sp_at_entry: usize,
}

impl<'a> Frame<'a> {
    pub fn new_module(m: &'a CompiledModule) -> Self {
        let mut locals = Vec::new();
        for _ in &m.locals { locals.push(Value::Undefined); }
        Self {
            bytecode: &m.bytecode,
            constants: &m.constants,
            locals,
            local_cells: Vec::new(),
            operand_stack: Vec::with_capacity(32),
            pc: 0,
            try_stack: Vec::new(),
            this_value: Value::Undefined,
            upvalues: Vec::new(),
            last_property_lookup: None,
        }
    }

    /// Read local `slot`. If promoted (a closure captured it), read
    /// through the shared cell; else read the value slot directly.
    pub fn read_local(&self, slot: usize) -> Value {
        if let Some(Some(cell)) = self.local_cells.get(slot) {
            return cell.borrow().clone();
        }
        self.locals.get(slot).cloned().unwrap_or(Value::Undefined)
    }

    /// Write local `slot`. If promoted, write through the shared cell so
    /// nested closures see the update.
    pub fn write_local(&mut self, slot: usize, v: Value) {
        if let Some(Some(cell)) = self.local_cells.get(slot) {
            *cell.borrow_mut() = v;
            return;
        }
        while self.locals.len() <= slot { self.locals.push(Value::Undefined); }
        self.locals[slot] = v;
    }

    /// Promote local `slot` to a shared cell (idempotent). Used when a
    /// nested closure captures the slot — the cell becomes authoritative
    /// for both this frame's reads/writes and the closure's upvalue.
    pub fn promote_local(&mut self, slot: usize) -> UpvalueCell {
        while self.locals.len() <= slot { self.locals.push(Value::Undefined); }
        while self.local_cells.len() <= slot { self.local_cells.push(None); }
        if let Some(cell) = &self.local_cells[slot] {
            return cell.clone();
        }
        let v = std::mem::replace(&mut self.locals[slot], Value::Undefined);
        let cell = new_upvalue_cell(v);
        self.local_cells[slot] = Some(cell.clone());
        cell
    }

    pub fn push(&mut self, v: Value) { self.operand_stack.push(v); }

    pub fn pop(&mut self) -> Result<Value, RuntimeError> {
        self.operand_stack.pop()
            .ok_or_else(|| RuntimeError::TypeError("operand stack underflow".into()))
    }

    pub fn peek(&self, depth: usize) -> Result<&Value, RuntimeError> {
        let len = self.operand_stack.len();
        if depth >= len {
            return Err(RuntimeError::TypeError("operand stack peek underflow".into()));
        }
        Ok(&self.operand_stack[len - 1 - depth])
    }
}
