// Simulated-derivation of structuredClone.
//
// Inputs:
//   AUDIT — pilots/structured-clone/AUDIT.md
//   SPEC  — https://html.spec.whatwg.org/multipage/structured-data.html#structured-clone
//   CD    — runs/2026-05-10-bun-v0.13b-spec-batch/constraints/
//           structuredclone.constraints.md (5 properties / 227 clauses)
//
// The spec defines structured-clone as a two-phase algorithm:
//   StructuredSerialize(value, memory)      — Value → SerializedRecord
//   StructuredDeserialize(record, realm)    — SerializedRecord → Value
// The pilot mirrors this exactly. Identity + circular-reference handling
// fall out of the index-based serialization. See AUDIT.md "Approach" §.

use std::collections::HashMap;

// ─────────────────────── Value type universe ──────────────────────────────
//
// Heap is the Rust analog of a JS realm: it owns all reference values. Plain
// values (primitives) are stored inline; reference values (Object, Array,
// Map, Set, ArrayBuffer, etc.) are stored as Heap entries and referenced by
// id. This is what enables identity/circular-reference preservation.

pub type ValueId = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    // Primitives — cloned by value automatically.
    Null,
    Undefined,
    Boolean(bool),
    Number(f64),
    BigInt(i128),
    String(String),
    // Reference values — actual contents in Heap; this carries the id.
    Ref(ValueId),
    // Function — opaque; not cloneable per SPEC.
    Function,
    // Non-cloneable marker for testing the error path.
    NonCloneable,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HeapObject {
    Object(Vec<(String, Value)>),
    Array(Vec<Value>),
    Date(i64),                        // epoch ms
    RegExp { source: String, flags: String },
    Map(Vec<(Value, Value)>),
    Set(Vec<Value>),
    ArrayBuffer(Vec<u8>),
    TypedArrayView { buffer: ValueId, byte_offset: usize, length: usize, kind: TypedArrayKind },
    /// Pilot-scope simplified Blob: bytes + MIME type, satisfies the
    /// constraint-doc reps (`cloned.size === 0`, `cloned instanceof Blob`).
    Blob { bytes: Vec<u8>, mime_type: String },
    /// Pilot-scope simplified File: extends Blob with name + lastModified.
    File { bytes: Vec<u8>, mime_type: String, name: String, last_modified: i64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypedArrayKind {
    Int8,
    Uint8,
    Uint8Clamped,
    Int16,
    Uint16,
    Int32,
    Uint32,
    Float32,
    Float64,
    BigInt64,
    BigUint64,
}

#[derive(Debug, Default)]
pub struct Heap {
    /// Public so verifier tests can construct cyclic graphs by pre-allocating
    /// placeholder slots and patching them. The structured-clone algorithm
    /// itself doesn't need this access — but tests modeling JS programs that
    /// build cycles do.
    pub objects: Vec<HeapObject>,
}

impl Heap {
    pub fn new() -> Self { Self { objects: Vec::new() } }

    pub fn alloc(&mut self, obj: HeapObject) -> Value {
        let id = self.objects.len();
        self.objects.push(obj);
        Value::Ref(id)
    }

    pub fn get(&self, id: ValueId) -> &HeapObject {
        &self.objects[id]
    }

    pub fn get_mut(&mut self, id: ValueId) -> &mut HeapObject {
        &mut self.objects[id]
    }
}

// ──────────────────────── SerializedRecord ────────────────────────────────
//
// Flat representation of a Value graph. Each record carries primitive
// payload OR a reference to another record by index. Cycles are inherently
// supported because back-references are just indices.

#[derive(Debug, Clone, PartialEq)]
pub enum SerializedRecord {
    Null,
    Undefined,
    Boolean(bool),
    Number(f64),
    BigInt(i128),
    String(String),
    Date(i64),
    RegExp { source: String, flags: String },
    Map { entries: Vec<(SerializedSlot, SerializedSlot)> },
    Set { entries: Vec<SerializedSlot> },
    Array { items: Vec<SerializedSlot> },
    Object { entries: Vec<(String, SerializedSlot)> },
    ArrayBuffer { bytes: Vec<u8> },
    TypedArrayView { buffer: SerializedSlot, byte_offset: usize, length: usize, kind: TypedArrayKind },
    Blob { bytes: Vec<u8>, mime_type: String },
    File { bytes: Vec<u8>, mime_type: String, name: String, last_modified: i64 },
}

/// A slot is either an inline primitive or a back-reference to a record.
#[derive(Debug, Clone, PartialEq)]
pub enum SerializedSlot {
    Inline(InlinePrimitive),
    Ref(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum InlinePrimitive {
    Null,
    Undefined,
    Boolean(bool),
    Number(f64),
    BigInt(i128),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SerializedScript {
    pub records: Vec<SerializedRecord>,
    pub root: SerializedSlot,
}

// ─────────────────────────── Errors ───────────────────────────────────────
//
// SPEC §2.10.1: "DataCloneError" DOMException is the only error structured-
// clone raises. CD STRU4 lists three triggers: functions, DOM nodes,
// non-cloneable references.

#[derive(Debug, Clone, PartialEq)]
pub enum CloneError {
    /// Attempted to clone a Function (or DOM-node-like opaque platform type).
    NotCloneable(&'static str),
}

// ───────────────────────── Phase 1: serialize ────────────────────────────
//
// SPEC §2.10.1.StructuredSerializeInternal. memory maps already-visited
// reference values to their assigned record indices, so the second visit
// emits a back-reference instead of re-serializing.

pub fn structured_serialize(heap: &Heap, root: &Value) -> Result<SerializedScript, CloneError> {
    let mut ser = Serializer { heap, records: Vec::new(), memory: HashMap::new() };
    let root_slot = ser.serialize_value(root)?;
    Ok(SerializedScript { records: ser.records, root: root_slot })
}

struct Serializer<'a> {
    heap: &'a Heap,
    records: Vec<SerializedRecord>,
    memory: HashMap<ValueId, usize>,
}

impl<'a> Serializer<'a> {
    fn serialize_value(&mut self, v: &Value) -> Result<SerializedSlot, CloneError> {
        match v {
            Value::Null => Ok(SerializedSlot::Inline(InlinePrimitive::Null)),
            Value::Undefined => Ok(SerializedSlot::Inline(InlinePrimitive::Undefined)),
            Value::Boolean(b) => Ok(SerializedSlot::Inline(InlinePrimitive::Boolean(*b))),
            Value::Number(n) => Ok(SerializedSlot::Inline(InlinePrimitive::Number(*n))),
            Value::BigInt(b) => Ok(SerializedSlot::Inline(InlinePrimitive::BigInt(*b))),
            Value::String(s) => Ok(SerializedSlot::Inline(InlinePrimitive::String(s.clone()))),
            Value::Function => Err(CloneError::NotCloneable("function")),
            Value::NonCloneable => Err(CloneError::NotCloneable("non-cloneable marker")),
            Value::Ref(id) => self.serialize_ref(*id),
        }
    }

    fn serialize_ref(&mut self, id: ValueId) -> Result<SerializedSlot, CloneError> {
        // SPEC: if memory[value] exists, return that index — the cycle/shared
        // reference is preserved as a back-reference.
        if let Some(&idx) = self.memory.get(&id) {
            return Ok(SerializedSlot::Ref(idx));
        }
        // SPEC: register memory[value] = next_index BEFORE recursing into
        // children, so a cycle that loops back finds the slot already
        // assigned. The actual record is filled in after the recursive walk.
        let idx = self.records.len();
        self.memory.insert(id, idx);
        // Reserve the slot with a placeholder; we'll overwrite it.
        self.records.push(SerializedRecord::Null);

        let record = self.serialize_heap_object(id)?;
        self.records[idx] = record;
        Ok(SerializedSlot::Ref(idx))
    }

    fn serialize_heap_object(&mut self, id: ValueId) -> Result<SerializedRecord, CloneError> {
        // Clone the heap object's structural shape locally so we can hold
        // immutable refs from `self.heap` while the recursive walk also
        // borrows `self`. (Heap-clone is necessary because we're walking
        // children; an alternative would be a snapshot pass first.)
        let obj = self.heap.get(id).clone();
        Ok(match obj {
            HeapObject::Object(entries) => {
                let mut out = Vec::with_capacity(entries.len());
                for (k, v) in entries {
                    out.push((k, self.serialize_value(&v)?));
                }
                SerializedRecord::Object { entries: out }
            }
            HeapObject::Array(items) => {
                let mut out = Vec::with_capacity(items.len());
                for v in items {
                    out.push(self.serialize_value(&v)?);
                }
                SerializedRecord::Array { items: out }
            }
            HeapObject::Date(ms) => SerializedRecord::Date(ms),
            HeapObject::RegExp { source, flags } => SerializedRecord::RegExp { source, flags },
            HeapObject::Map(entries) => {
                let mut out = Vec::with_capacity(entries.len());
                for (k, v) in entries {
                    out.push((self.serialize_value(&k)?, self.serialize_value(&v)?));
                }
                SerializedRecord::Map { entries: out }
            }
            HeapObject::Set(items) => {
                let mut out = Vec::with_capacity(items.len());
                for v in items {
                    out.push(self.serialize_value(&v)?);
                }
                SerializedRecord::Set { entries: out }
            }
            HeapObject::ArrayBuffer(bytes) => SerializedRecord::ArrayBuffer { bytes },
            HeapObject::TypedArrayView { buffer, byte_offset, length, kind } => {
                let buf_slot = self.serialize_ref(buffer)?;
                SerializedRecord::TypedArrayView { buffer: buf_slot, byte_offset, length, kind }
            }
            HeapObject::Blob { bytes, mime_type } => {
                SerializedRecord::Blob { bytes, mime_type }
            }
            HeapObject::File { bytes, mime_type, name, last_modified } => {
                SerializedRecord::File { bytes, mime_type, name, last_modified }
            }
        })
    }
}

// ───────────────────────── Phase 2: deserialize ──────────────────────────
//
// SPEC §2.10.2.StructuredDeserialize. Walks the record vector once,
// allocating Heap slots in order. Back-references are resolved through a
// records-index → ValueId map. The forward-reference pre-allocation is
// symmetric to Phase 1: allocate the heap slot before recursing into
// the record's children.

pub fn structured_deserialize(script: &SerializedScript) -> (Heap, Value) {
    let mut heap = Heap::new();
    let mut record_to_id: Vec<Option<ValueId>> = vec![None; script.records.len()];
    let root = deserialize_slot(&script.root, &script.records, &mut heap, &mut record_to_id);
    (heap, root)
}

fn deserialize_slot(
    slot: &SerializedSlot,
    records: &[SerializedRecord],
    heap: &mut Heap,
    record_to_id: &mut [Option<ValueId>],
) -> Value {
    match slot {
        SerializedSlot::Inline(p) => match p {
            InlinePrimitive::Null => Value::Null,
            InlinePrimitive::Undefined => Value::Undefined,
            InlinePrimitive::Boolean(b) => Value::Boolean(*b),
            InlinePrimitive::Number(n) => Value::Number(*n),
            InlinePrimitive::BigInt(i) => Value::BigInt(*i),
            InlinePrimitive::String(s) => Value::String(s.clone()),
        },
        SerializedSlot::Ref(idx) => {
            if let Some(id) = record_to_id[*idx] {
                return Value::Ref(id);
            }
            // Pre-allocate a placeholder slot so cycles back into this
            // record find a stable id.
            let id = heap.objects.len();
            heap.objects.push(HeapObject::Object(Vec::new())); // placeholder
            record_to_id[*idx] = Some(id);
            let obj = build_heap_object(&records[*idx], records, heap, record_to_id);
            heap.objects[id] = obj;
            Value::Ref(id)
        }
    }
}

fn build_heap_object(
    rec: &SerializedRecord,
    records: &[SerializedRecord],
    heap: &mut Heap,
    record_to_id: &mut [Option<ValueId>],
) -> HeapObject {
    match rec {
        SerializedRecord::Null
        | SerializedRecord::Undefined
        | SerializedRecord::Boolean(_)
        | SerializedRecord::Number(_)
        | SerializedRecord::BigInt(_)
        | SerializedRecord::String(_) => {
            // These never reach the heap-build path (they're inline primitives).
            unreachable!("primitive record encountered at heap-object slot")
        }
        SerializedRecord::Object { entries } => {
            let mut out = Vec::with_capacity(entries.len());
            for (k, slot) in entries {
                out.push((k.clone(), deserialize_slot(slot, records, heap, record_to_id)));
            }
            HeapObject::Object(out)
        }
        SerializedRecord::Array { items } => {
            let mut out = Vec::with_capacity(items.len());
            for slot in items {
                out.push(deserialize_slot(slot, records, heap, record_to_id));
            }
            HeapObject::Array(out)
        }
        SerializedRecord::Date(ms) => HeapObject::Date(*ms),
        SerializedRecord::RegExp { source, flags } => HeapObject::RegExp {
            source: source.clone(),
            flags: flags.clone(),
        },
        SerializedRecord::Map { entries } => {
            let mut out = Vec::with_capacity(entries.len());
            for (k, v) in entries {
                let kv = deserialize_slot(k, records, heap, record_to_id);
                let vv = deserialize_slot(v, records, heap, record_to_id);
                out.push((kv, vv));
            }
            HeapObject::Map(out)
        }
        SerializedRecord::Set { entries } => {
            let mut out = Vec::with_capacity(entries.len());
            for slot in entries {
                out.push(deserialize_slot(slot, records, heap, record_to_id));
            }
            HeapObject::Set(out)
        }
        SerializedRecord::ArrayBuffer { bytes } => HeapObject::ArrayBuffer(bytes.clone()),
        SerializedRecord::TypedArrayView { buffer, byte_offset, length, kind } => {
            let buf_value = deserialize_slot(buffer, records, heap, record_to_id);
            let buffer_id = match buf_value {
                Value::Ref(id) => id,
                _ => unreachable!("typed array buffer must be a Ref"),
            };
            HeapObject::TypedArrayView {
                buffer: buffer_id,
                byte_offset: *byte_offset,
                length: *length,
                kind: *kind,
            }
        }
        SerializedRecord::Blob { bytes, mime_type } => HeapObject::Blob {
            bytes: bytes.clone(),
            mime_type: mime_type.clone(),
        },
        SerializedRecord::File { bytes, mime_type, name, last_modified } => HeapObject::File {
            bytes: bytes.clone(),
            mime_type: mime_type.clone(),
            name: name.clone(),
            last_modified: *last_modified,
        },
    }
}

// ─────────────────────── Public entry point ──────────────────────────────
//
// CD STRU3: structuredClone is exposed as a function. Pilot's signature
// takes a Heap reference (the source realm) and a Value (the root); returns
// a fresh Heap (the target realm) and a fresh Value rooted in it.

pub fn structured_clone(heap: &Heap, root: &Value) -> Result<(Heap, Value), CloneError> {
    let script = structured_serialize(heap, root)?;
    Ok(structured_deserialize(&script))
}
