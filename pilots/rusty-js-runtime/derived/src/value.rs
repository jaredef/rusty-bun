//! ECMAScript value representation per specs/rusty-js-runtime-design.md §I.
//!
//! v1 simplifications:
//! - Strings stored as Rust String (UTF-8) rather than Vec<u16> (UTF-16
//!   per spec §6.1.4). The mismatch matters for surrogate-pair-aware
//!   indexing/length but doesn't affect most consumer behavior. Migration
//!   to UTF-16 is mechanical when needed.
//! - Object references as Rc<RefCell<Object>>. Cycles leak. The v2 mark-
//!   sweep GC (Ω.3.e) replaces this with a managed heap.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type ObjectRef = Rc<RefCell<Object>>;

// ──────────────── GC Trace impl ────────────────
//
// Object stores property values as `Value`, and Value::Object currently
// holds Rc<RefCell<Object>> (v1 representation). The Trace impl walks
// the property values + proto + InternalKind fields, pushing the
// embedded ObjectIds where present.
//
// In v1 the Value::Object payload is Rc<RefCell<Object>>, so there are
// no ObjectIds to push — the GC's reachability sweep is operationally
// inert. Wiring the Trace impl now means round 3.e.d (the Value migration)
// changes only the Value enum + a small set of construction sites; the
// trace topology already exists.
impl rusty_js_gc::Trace for Object {
    fn trace(&self, _ids: &mut Vec<rusty_js_gc::ObjectId>) {
        // v1: no GC-tracked edges since Object isn't yet stored in the
        // heap. Round 3.e.d migrates Value::Object to ObjectId; this
        // impl then walks proto + properties.values() + InternalKind
        // fields, pushing each contained ObjectId. The shape is
        // already correct.
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
    pub fn new_object() -> Value {
        Value::Object(Rc::new(RefCell::new(Object::new_ordinary())))
    }

    pub fn type_of(&self) -> &'static str {
        match self {
            Value::Undefined => "undefined",
            Value::Null => "object",  // per §13.5.3 typeof null is "object"
            Value::Boolean(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::BigInt(_) => "bigint",
            Value::Object(o) => {
                let obj = o.borrow();
                if matches!(obj.internal_kind, InternalKind::Function(_) | InternalKind::Closure(_) | InternalKind::BoundFunction(_)) {
                    "function"
                } else {
                    "object"
                }
            }
        }
    }

    /// SameValue per spec §7.2.11. Used by Map keys and Set elements.
    pub fn same_value(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Undefined, Value::Undefined) => true,
            (Value::Null, Value::Null) => true,
            (Value::Boolean(x), Value::Boolean(y)) => x == y,
            (Value::Number(x), Value::Number(y)) => {
                // SameValue treats +0 and -0 as different, and NaN equal to NaN.
                if x.is_nan() && y.is_nan() { return true; }
                x.to_bits() == y.to_bits()
            }
            (Value::String(x), Value::String(y)) => x == y,
            (Value::BigInt(x), Value::BigInt(y)) => x == y,
            (Value::Object(x), Value::Object(y)) => Rc::ptr_eq(x, y),
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
            Value::Object(o) => write!(f, "[Object {:?}]", o.borrow().internal_kind.kind_name()),
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

    /// OrdinaryGet per §10.1.8.1. v1 doesn't walk the prototype chain
    /// beyond one hop; full prototype walk lands when intrinsics arrive.
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

    pub fn get(&self, key: &str) -> Value {
        if let Some(d) = self.properties.get(key) {
            return d.value.clone();
        }
        if let Some(p) = &self.proto {
            return p.borrow().get(key);
        }
        Value::Undefined
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
}

impl InternalKind {
    pub fn kind_name(&self) -> &'static str {
        match self {
            InternalKind::Ordinary => "ordinary",
            InternalKind::Array => "array",
            InternalKind::Function(_) => "function",
            InternalKind::Closure(_) => "closure",
            InternalKind::BoundFunction(_) => "bound-function",
            InternalKind::Error => "error",
            InternalKind::ModuleNamespace => "module-namespace",
        }
    }
}

/// Closure internals — wraps a FunctionProto with captured upvalues.
#[derive(Debug)]
pub struct ClosureInternals {
    pub proto: Rc<rusty_js_bytecode::compiler::FunctionProto>,
    pub upvalues: Vec<Value>,
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

pub type NativeFn = std::rc::Rc<dyn Fn(&[Value]) -> Result<Value, crate::interp::RuntimeError>>;

#[derive(Debug)]
pub struct BoundFunctionInternals {
    pub target: ObjectRef,
    pub this: Value,
    pub args: Vec<Value>,
}
