//! ECMAScript value representation per specs/rusty-js-runtime-design.md §I.
//!
//! v1 simplifications:
//! - Strings stored as Rust String (UTF-8) rather than Vec<u16> (UTF-16
//!   per spec §6.1.4). The mismatch matters for surrogate-pair-aware
//!   indexing/length but doesn't affect most consumer behavior. Migration
//!   to UTF-16 is mechanical when needed.
//! - Round 3.e.d: Object references migrated from Rc<RefCell<Object>> to
//!   ObjectId — Objects live in Runtime.heap. Value::Object payload is
//!   ObjectId (Copy + Eq). Cycles are now reclaimable via rt.collect().

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// A captured-binding cell. Tier-Ω.5.e migrated upvalues from
/// value-snapshot (Vec<Value>) to binding-shared (Vec<UpvalueCell>) per
/// ECMA-262 §8.1 / §10.2: each captured binding is one shared location,
/// shared across the outer frame's slot and every closure that captured
/// it. Writes through any handle are visible to all others.
pub type UpvalueCell = Rc<RefCell<Value>>;

pub fn new_upvalue_cell(v: Value) -> UpvalueCell {
    Rc::new(RefCell::new(v))
}

/// Alias preserving call-site shape. Post-3.e.d this is a heap handle
/// (Copy + Eq), not an Rc<RefCell<...>>.
pub type ObjectRef = rusty_js_gc::ObjectId;

// ──────────────── GC Trace impl ────────────────
//
// Object's out-edges:
//   - proto: Option<ObjectId>
//   - properties.values()'s Value::Object payloads
//   - InternalKind edges:
//       Closure: upvalues' Value::Object payloads
//       BoundFunction: target + this + args
//       Promise: each reaction's chain (always Object) + handler (if Object)
impl rusty_js_gc::Trace for Object {
    fn trace(&self, ids: &mut Vec<rusty_js_gc::ObjectId>) {
        if let Some(p) = self.proto { ids.push(p); }
        for d in self.properties.values() {
            if let Value::Object(id) = &d.value { ids.push(*id); }
        }
        match &self.internal_kind {
            InternalKind::Closure(c) => {
                for cell in &c.upvalues {
                    if let Value::Object(id) = &*cell.borrow() { ids.push(*id); }
                }
            }
            InternalKind::BoundFunction(b) => {
                ids.push(b.target);
                if let Value::Object(id) = &b.this { ids.push(*id); }
                for v in &b.args {
                    if let Value::Object(id) = v { ids.push(*id); }
                }
            }
            InternalKind::Promise(ps) => {
                if let Value::Object(id) = &ps.value { ids.push(*id); }
                for r in &ps.fulfill_reactions {
                    ids.push(r.chain);
                    if let Some(Value::Object(id)) = &r.handler { ids.push(*id); }
                }
                for r in &ps.reject_reactions {
                    ids.push(r.chain);
                    if let Some(Value::Object(id)) = &r.handler { ids.push(*id); }
                }
            }
            _ => {}
        }
    }
}

#[derive(Clone)]
pub enum Value {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(Rc<String>),
    /// BigInt stored as signed decimal-digit string in v1. Arithmetic
    /// defers to a BigInt crate in a follow-on round.
    BigInt(Rc<String>),
    Object(ObjectRef),
}

impl Value {
    pub fn type_of(&self) -> &'static str {
        match self {
            Value::Undefined => "undefined",
            Value::Null => "object",  // per §13.5.3 typeof null is "object"
            Value::Boolean(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::BigInt(_) => "bigint",
            // Post-3.e.d: Value::Object's typeof requires a heap to peek
            // InternalKind. Without a runtime here we report "object";
            // callers that need precise function/object disambiguation
            // should use Runtime::type_of_value (added in 3.e.d).
            Value::Object(_) => "object",
        }
    }

    /// SameValue per spec §7.2.11. Used by Map keys and Set elements.
    pub fn same_value(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Undefined, Value::Undefined) => true,
            (Value::Null, Value::Null) => true,
            (Value::Boolean(x), Value::Boolean(y)) => x == y,
            (Value::Number(x), Value::Number(y)) => {
                if x.is_nan() && y.is_nan() { return true; }
                x.to_bits() == y.to_bits()
            }
            (Value::String(x), Value::String(y)) => x == y,
            (Value::BigInt(x), Value::BigInt(y)) => x == y,
            (Value::Object(x), Value::Object(y)) => x == y,
            _ => false,
        }
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Undefined => write!(f, "undefined"),
            Value::Null => write!(f, "null"),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{:?}", s.as_str()),
            Value::BigInt(s) => write!(f, "{}n", s.as_str()),
            Value::Object(id) => write!(f, "[Object #{}]", id.0),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        Self::same_value(self, other)
    }
}

#[derive(Debug)]
pub struct Object {
    pub proto: Option<ObjectRef>,
    pub extensible: bool,
    pub properties: HashMap<String, PropertyDescriptor>,
    pub internal_kind: InternalKind,
}

impl Object {
    pub fn new_ordinary() -> Self {
        Self {
            proto: None,
            extensible: true,
            properties: HashMap::new(),
            internal_kind: InternalKind::Ordinary,
        }
    }

    pub fn new_array() -> Self {
        Self {
            proto: None,
            extensible: true,
            properties: HashMap::new(),
            internal_kind: InternalKind::Array,
        }
    }

    /// OrdinaryGet per §10.1.8.1. Own-property only. Prototype-chain
    /// walk moved to Runtime::object_get (proto deref requires heap).
    pub fn get_own(&self, key: &str) -> Option<&PropertyDescriptor> {
        self.properties.get(key)
    }

    /// OrdinaryDefineOwnProperty per §10.1.6.1 (simplified — full
    /// invariants check lands with intrinsics).
    pub fn set_own(&mut self, key: String, value: Value) {
        self.properties.insert(key, PropertyDescriptor {
            value,
            writable: true,
            enumerable: true,
            configurable: true,
        });
    }
}

#[derive(Debug, Clone)]
pub struct PropertyDescriptor {
    pub value: Value,
    pub writable: bool,
    pub enumerable: bool,
    pub configurable: bool,
}

#[derive(Debug)]
pub enum InternalKind {
    Ordinary,
    Array,
    Function(FunctionInternals),
    Closure(ClosureInternals),
    BoundFunction(BoundFunctionInternals),
    Error,
    ModuleNamespace,
    /// Promise per ECMA-262 §27.2.
    Promise(PromiseState),
    /// Regular expression object per ECMA-262 §22.2. Tier-Ω.5.i.
    RegExp(RegExpInternals),
}

/// RegExp instance internals. `source` and `flags` retain the original JS
/// spelling for the .source / .flags accessor surface. `compiled` is the
/// Rust `regex` crate compilation of the translated pattern — None when
/// the pattern uses features the Rust crate does not support (lookbehind,
/// backreferences); methods then throw a TypeError on call rather than
/// panicking. `last_index` backs the stateful exec/test path under the
/// 'g' flag per §22.2.5.2.
#[derive(Debug)]
pub struct RegExpInternals {
    pub source: Rc<String>,
    pub flags: Rc<String>,
    pub compiled: Option<regex::Regex>,
    pub last_index: usize,
}

#[derive(Debug)]
pub struct PromiseState {
    pub status: PromiseStatus,
    pub value: Value,
    pub fulfill_reactions: Vec<PromiseReaction>,
    pub reject_reactions: Vec<PromiseReaction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromiseStatus { Pending, Fulfilled, Rejected }

#[derive(Debug)]
pub struct PromiseReaction {
    pub handler: Option<Value>,
    /// Chained Promise to resolve with the handler's result.
    pub chain: ObjectRef,
}

impl InternalKind {
    pub fn kind_name(&self) -> &'static str {
        match self {
            InternalKind::Ordinary => "ordinary",
            InternalKind::Array => "array",
            InternalKind::Function(_) => "function",
            InternalKind::Promise(_) => "promise",
            InternalKind::Closure(_) => "closure",
            InternalKind::BoundFunction(_) => "bound-function",
            InternalKind::Error => "error",
            InternalKind::ModuleNamespace => "module-namespace",
            InternalKind::RegExp(_) => "regexp",
        }
    }
}

/// Closure internals — wraps a FunctionProto with captured upvalues.
#[derive(Debug)]
pub struct ClosureInternals {
    pub proto: Rc<rusty_js_bytecode::compiler::FunctionProto>,
    /// Tier-Ω.5.e: shared-binding upvalues. Each cell is shared with the
    /// outer frame's promoted local slot and with any sibling closures
    /// that captured the same binding.
    pub upvalues: Vec<UpvalueCell>,
    pub is_arrow: bool,
}

/// Native function (intrinsic) backed by a Rust callback.
pub struct FunctionInternals {
    pub name: String,
    pub native: NativeFn,
}

impl std::fmt::Debug for FunctionInternals {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FunctionInternals {{ name: {:?} }}", self.name)
    }
}

pub type NativeFn = std::rc::Rc<dyn Fn(&mut crate::interp::Runtime, &[Value]) -> Result<Value, crate::interp::RuntimeError>>;

#[derive(Debug)]
pub struct BoundFunctionInternals {
    pub target: ObjectRef,
    pub this: Value,
    pub args: Vec<Value>,
}
