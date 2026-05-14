//! Bytecode dispatch loop + Runtime + Frame management.
//! Per specs/rusty-js-runtime-design.md §III.

use crate::abstract_ops::*;
use crate::value::{InternalKind, Object, ObjectRef, Value};
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
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            last_value: Value::Undefined,
            host_hooks: crate::module::HostHooks::default(),
            heap: rusty_js_gc::Heap::new(),
            job_queue: crate::job_queue::JobQueue::new(),
            pending_unhandled: HashSet::new(),
        }
    }

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
    /// handle.
    pub fn alloc_object(&mut self, obj: crate::value::Object) -> rusty_js_gc::ObjectId {
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
    pub fn object_get(&self, id: ObjectRef, key: &str) -> Value {
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
                    let v = frame.locals.get(slot).cloned().unwrap_or(Value::Undefined);
                    frame.push(v);
                }
                Op::StoreLocal => {
                    let slot = decode_u16(&frame.bytecode, frame.pc) as usize;
                    frame.pc += 2;
                    let v = frame.pop()?;
                    while frame.locals.len() <= slot { frame.locals.push(Value::Undefined); }
                    frame.locals[slot] = v;
                }
                Op::LoadGlobal => {
                    let idx = decode_u16(&frame.bytecode, frame.pc);
                    frame.pc += 2;
                    let name = self.constant_name(frame, idx)?;
                    let v = self.globals.get(&name).cloned().unwrap_or(Value::Undefined);
                    frame.push(v);
                }
                Op::StoreGlobal => {
                    let idx = decode_u16(&frame.bytecode, frame.pc);
                    frame.pc += 2;
                    let name = self.constant_name(frame, idx)?;
                    let v = frame.pop()?;
                    self.globals.insert(name, v);
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
                        Value::Undefined | Value::Null => {
                            return Err(RuntimeError::TypeError(
                                format!("Cannot read property '{}' of {}", key,
                                    if matches!(obj_v, Value::Undefined) { "undefined" } else { "null" })
                            ));
                        }
                        _ => Value::Undefined,
                    };
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
                    let result = self.call_function(callee, Value::Undefined, args)?;
                    frame.push(result);
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
                    let this_id = self.alloc_object(Object::new_ordinary());
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
    pub fn call_function(&mut self, callee: Value, _this: Value, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let id = match callee {
            Value::Object(id) => id,
            _ => return Err(RuntimeError::TypeError("callee is not callable".into())),
        };
        // Extract proto-or-native by inspecting the heap object once.
        let (proto_opt, native_opt) = {
            let o = self.obj(id);
            match &o.internal_kind {
                crate::value::InternalKind::Closure(c) => (Some(c.proto.clone()), None),
                crate::value::InternalKind::Function(f) => (None, Some(f.native.clone())),
                _ => return Err(RuntimeError::TypeError("callee is not callable".into())),
            }
        };
        if let Some(native) = native_opt {
            return native(self, &args);
        }
        let proto = proto_opt.expect("closure branch implies proto");
        // Build the inner frame's locals: arguments first, then space for
        // additional locals beyond params.
        let mut locals: Vec<Value> = Vec::new();
        for (i, _) in proto.locals.iter().enumerate() {
            if i < args.len() {
                locals.push(args[i].clone());
            } else {
                locals.push(Value::Undefined);
            }
        }
        // If fewer locals than args (e.g. extra args), trim — JS permits.
        let mut inner = Frame {
            bytecode: &proto.bytecode,
            constants: &proto.constants,
            locals,
            operand_stack: Vec::with_capacity(32),
            pc: 0,
            try_stack: Vec::new(),
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
    pub operand_stack: Vec<Value>,
    pub pc: usize,
    pub try_stack: Vec<TryFrame>,
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
            operand_stack: Vec::with_capacity(32),
            pc: 0,
            try_stack: Vec::new(),
        }
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
