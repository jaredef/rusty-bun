// Verifier for the structured-clone pilot. Each test ties to one antichain
// representative from the v0.13b enriched constraint corpus or to a
// HTML §2.10 spec invariant.
//
// CD-SC = runs/2026-05-10-bun-v0.13b-spec-batch/constraints/structuredclone.constraints.md
// SPEC  = https://html.spec.whatwg.org/multipage/structured-data.html

use rusty_structured_clone::*;

fn clone_value(heap: &Heap, root: Value) -> (Heap, Value) {
    structured_clone(heap, &root).expect("clone should succeed")
}

// ─────────── CD-SC / STRU1 antichain reps (cardinality 166) ──────────

// `expect(cloned).toStrictEqual({})` — empty object roundtrip
#[test]
fn cd_stru1_empty_object_roundtrip() {
    let mut heap = Heap::new();
    let v = heap.alloc(HeapObject::Object(Vec::new()));
    let (cloned_heap, cloned) = clone_value(&heap, v);
    let id = match cloned { Value::Ref(id) => id, _ => panic!("expected ref") };
    assert!(matches!(cloned_heap.get(id), HeapObject::Object(e) if e.is_empty()));
}

// `expect(cloned.size).toBe(0)` — Blob preservation
#[test]
fn cd_stru1_empty_blob_size_zero() {
    let mut heap = Heap::new();
    let blob = heap.alloc(HeapObject::Blob {
        bytes: Vec::new(),
        mime_type: String::new(),
    });
    let (cloned_heap, cloned) = clone_value(&heap, blob);
    let id = match cloned { Value::Ref(id) => id, _ => panic!() };
    let HeapObject::Blob { bytes, mime_type } = cloned_heap.get(id) else { panic!("expected blob") };
    assert_eq!(bytes.len(), 0);
    assert_eq!(mime_type, "");
}

// `expect(cloned.file.name).toBe("example.txt")` — File preservation
#[test]
fn cd_stru1_file_name_preserved() {
    let mut heap = Heap::new();
    let file = heap.alloc(HeapObject::File {
        bytes: vec![1, 2, 3],
        mime_type: "text/plain".into(),
        name: "example.txt".into(),
        last_modified: 0,
    });
    // Wrap in {file: <File>} object as the test does
    let wrapper = heap.alloc(HeapObject::Object(vec![("file".into(), file)]));
    let (cloned_heap, cloned) = clone_value(&heap, wrapper);
    let wrapper_id = match cloned { Value::Ref(id) => id, _ => panic!() };
    let HeapObject::Object(entries) = cloned_heap.get(wrapper_id) else { panic!() };
    let file_v = &entries[0].1;
    let file_id = match file_v { Value::Ref(id) => *id, _ => panic!() };
    let HeapObject::File { name, .. } = cloned_heap.get(file_id) else { panic!() };
    assert_eq!(name, "example.txt");
}

// ─────────── CD-SC / STRU2 antichain reps (cardinality 39) ──────────

// `expect(cloned).toBeInstanceOf(Array)` — array class preserved
#[test]
fn cd_stru2_array_class_preserved() {
    let mut heap = Heap::new();
    let arr = heap.alloc(HeapObject::Array(vec![Value::Number(1.0), Value::Number(2.0)]));
    let (cloned_heap, cloned) = clone_value(&heap, arr);
    let id = match cloned { Value::Ref(id) => id, _ => panic!() };
    assert!(matches!(cloned_heap.get(id), HeapObject::Array(_)));
}

// `expect(cloned).toBeInstanceOf(Blob)`
#[test]
fn cd_stru2_blob_class_preserved() {
    let mut heap = Heap::new();
    let blob = heap.alloc(HeapObject::Blob { bytes: vec![1, 2, 3], mime_type: "image/png".into() });
    let (cloned_heap, cloned) = clone_value(&heap, blob);
    let id = match cloned { Value::Ref(id) => id, _ => panic!() };
    assert!(matches!(cloned_heap.get(id), HeapObject::Blob { .. }));
}

// `expect(cloned.file).toBeInstanceOf(File)`
#[test]
fn cd_stru2_file_class_preserved() {
    let mut heap = Heap::new();
    let file = heap.alloc(HeapObject::File {
        bytes: vec![],
        mime_type: "".into(),
        name: "x".into(),
        last_modified: 0,
    });
    let (cloned_heap, cloned) = clone_value(&heap, file);
    let id = match cloned { Value::Ref(id) => id, _ => panic!() };
    assert!(matches!(cloned_heap.get(id), HeapObject::File { .. }));
}

// ─────────── CD-SC / STRU3 antichain reps (null / undefined preserved) ──

// `expect(cloned[0].a).toBeNull()`
#[test]
fn cd_stru3_null_property_preserved() {
    let mut heap = Heap::new();
    let inner = heap.alloc(HeapObject::Object(vec![
        ("a".into(), Value::Null),
        ("b".into(), Value::Undefined),
    ]));
    let outer = heap.alloc(HeapObject::Array(vec![inner]));
    let (cloned_heap, cloned) = clone_value(&heap, outer);
    let outer_id = match cloned { Value::Ref(id) => id, _ => panic!() };
    let HeapObject::Array(items) = cloned_heap.get(outer_id) else { panic!() };
    let inner_id = match items[0] { Value::Ref(id) => id, _ => panic!() };
    let HeapObject::Object(entries) = cloned_heap.get(inner_id) else { panic!() };
    assert_eq!(entries[0].1, Value::Null);
    assert_eq!(entries[1].1, Value::Undefined);
}

// ─────────── CD-SC / STRU4 antichain reps (DataCloneError) ──────────

#[test]
fn cd_stru4_function_throws() {
    let heap = Heap::new();
    let r = structured_clone(&heap, &Value::Function);
    assert!(matches!(r, Err(CloneError::NotCloneable("function"))));
}

#[test]
fn cd_stru4_noncloneable_throws() {
    let heap = Heap::new();
    let r = structured_clone(&heap, &Value::NonCloneable);
    assert!(matches!(r, Err(CloneError::NotCloneable(_))));
}

// `structuredClone throws DataCloneError on values containing non-cloneable references`
#[test]
fn cd_stru4_object_containing_function_throws() {
    let mut heap = Heap::new();
    let obj = heap.alloc(HeapObject::Object(vec![
        ("ok".into(), Value::Number(1.0)),
        ("bad".into(), Value::Function),
    ]));
    let r = structured_clone(&heap, &obj);
    assert!(matches!(r, Err(CloneError::NotCloneable("function"))));
}

// ─────────── CD-SC / STRU5 antichain (spec extracts) ──────────

#[test]
fn spec_clones_primitives_by_value() {
    for v in [
        Value::Null,
        Value::Undefined,
        Value::Boolean(true),
        Value::Number(3.14),
        Value::BigInt(1_000_000_000_000_000_000_i128),
        Value::String("hello".into()),
    ] {
        let heap = Heap::new();
        let (_, cloned) = structured_clone(&heap, &v).unwrap();
        assert_eq!(v, cloned);
    }
}

#[test]
fn spec_clones_date_with_same_time_value() {
    let mut heap = Heap::new();
    let d = heap.alloc(HeapObject::Date(1_700_000_000_000));
    let (cloned_heap, cloned) = clone_value(&heap, d);
    let id = match cloned { Value::Ref(id) => id, _ => panic!() };
    assert_eq!(*cloned_heap.get(id), HeapObject::Date(1_700_000_000_000));
}

#[test]
fn spec_clones_regexp_with_source_and_flags() {
    let mut heap = Heap::new();
    let re = heap.alloc(HeapObject::RegExp {
        source: "abc.*".into(),
        flags: "gi".into(),
    });
    let (cloned_heap, cloned) = clone_value(&heap, re);
    let id = match cloned { Value::Ref(id) => id, _ => panic!() };
    let HeapObject::RegExp { source, flags } = cloned_heap.get(id) else { panic!() };
    assert_eq!(source, "abc.*");
    assert_eq!(flags, "gi");
}

#[test]
fn spec_clones_map_preserving_entry_order() {
    let mut heap = Heap::new();
    let m = heap.alloc(HeapObject::Map(vec![
        (Value::String("a".into()), Value::Number(1.0)),
        (Value::String("b".into()), Value::Number(2.0)),
        (Value::String("c".into()), Value::Number(3.0)),
    ]));
    let (cloned_heap, cloned) = clone_value(&heap, m);
    let id = match cloned { Value::Ref(id) => id, _ => panic!() };
    let HeapObject::Map(entries) = cloned_heap.get(id) else { panic!() };
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].0, Value::String("a".into()));
    assert_eq!(entries[2].0, Value::String("c".into()));
}

#[test]
fn spec_clones_set_preserving_entry_order() {
    let mut heap = Heap::new();
    let s = heap.alloc(HeapObject::Set(vec![
        Value::Number(3.0), Value::Number(1.0), Value::Number(2.0),
    ]));
    let (cloned_heap, cloned) = clone_value(&heap, s);
    let id = match cloned { Value::Ref(id) => id, _ => panic!() };
    let HeapObject::Set(items) = cloned_heap.get(id) else { panic!() };
    assert_eq!(items, &vec![Value::Number(3.0), Value::Number(1.0), Value::Number(2.0)]);
}

#[test]
fn spec_clones_plain_objects_recursively() {
    let mut heap = Heap::new();
    let inner = heap.alloc(HeapObject::Object(vec![
        ("x".into(), Value::Number(1.0)),
    ]));
    let outer = heap.alloc(HeapObject::Object(vec![
        ("inner".into(), inner),
    ]));
    let (cloned_heap, cloned) = clone_value(&heap, outer);
    let outer_id = match cloned { Value::Ref(id) => id, _ => panic!() };
    let HeapObject::Object(entries) = cloned_heap.get(outer_id) else { panic!() };
    let inner_id = match &entries[0].1 { Value::Ref(id) => *id, _ => panic!() };
    let HeapObject::Object(inner_entries) = cloned_heap.get(inner_id) else { panic!() };
    assert_eq!(inner_entries[0].1, Value::Number(1.0));
}

#[test]
fn spec_clones_arrays_recursively() {
    let mut heap = Heap::new();
    let inner = heap.alloc(HeapObject::Array(vec![Value::Number(1.0), Value::Number(2.0)]));
    let outer = heap.alloc(HeapObject::Array(vec![inner, Value::Number(3.0)]));
    let (cloned_heap, cloned) = clone_value(&heap, outer);
    let outer_id = match cloned { Value::Ref(id) => id, _ => panic!() };
    let HeapObject::Array(items) = cloned_heap.get(outer_id) else { panic!() };
    let inner_id = match items[0] { Value::Ref(id) => id, _ => panic!() };
    let HeapObject::Array(inner_items) = cloned_heap.get(inner_id) else { panic!() };
    assert_eq!(inner_items, &vec![Value::Number(1.0), Value::Number(2.0)]);
}

#[test]
fn spec_clones_arraybuffer_with_byte_content() {
    let mut heap = Heap::new();
    let buf = heap.alloc(HeapObject::ArrayBuffer(vec![10, 20, 30, 40]));
    let (cloned_heap, cloned) = clone_value(&heap, buf);
    let id = match cloned { Value::Ref(id) => id, _ => panic!() };
    let HeapObject::ArrayBuffer(bytes) = cloned_heap.get(id) else { panic!() };
    assert_eq!(bytes, &vec![10, 20, 30, 40]);
}

#[test]
fn spec_clones_typedarray_attached_to_cloned_buffer() {
    let mut heap = Heap::new();
    let buf = heap.alloc(HeapObject::ArrayBuffer(vec![0xAA; 16]));
    let buf_id = match buf { Value::Ref(id) => id, _ => panic!() };
    let view = heap.alloc(HeapObject::TypedArrayView {
        buffer: buf_id,
        byte_offset: 4,
        length: 2,
        kind: TypedArrayKind::Uint32,
    });
    let (cloned_heap, cloned) = clone_value(&heap, view);
    let view_id = match cloned { Value::Ref(id) => id, _ => panic!() };
    let HeapObject::TypedArrayView { buffer, byte_offset, length, kind } = cloned_heap.get(view_id) else { panic!() };
    assert_eq!(*byte_offset, 4);
    assert_eq!(*length, 2);
    assert_eq!(*kind, TypedArrayKind::Uint32);
    // The cloned view's buffer must point to the cloned ArrayBuffer
    let HeapObject::ArrayBuffer(bytes) = cloned_heap.get(*buffer) else { panic!() };
    assert_eq!(bytes.len(), 16);
}

// ─────────── Identity preservation (the load-bearing property) ─────────

#[test]
fn spec_preserves_shared_reference_identity_within_call() {
    let mut heap = Heap::new();
    let shared = heap.alloc(HeapObject::Object(vec![("k".into(), Value::Number(42.0))]));
    let outer = heap.alloc(HeapObject::Array(vec![shared.clone(), shared.clone()]));
    let (cloned_heap, cloned) = clone_value(&heap, outer);
    let outer_id = match cloned { Value::Ref(id) => id, _ => panic!() };
    let HeapObject::Array(items) = cloned_heap.get(outer_id) else { panic!() };
    // Both array slots must reference the SAME id in the cloned heap.
    let id_a = match items[0] { Value::Ref(id) => id, _ => panic!() };
    let id_b = match items[1] { Value::Ref(id) => id, _ => panic!() };
    assert_eq!(id_a, id_b, "shared reference identity must be preserved");
}

#[test]
fn spec_preserves_circular_references() {
    // Build a self-referential object: { self: <self> }
    let mut heap = Heap::new();
    let self_id = heap.objects.len();
    heap.objects.push(HeapObject::Object(vec![]));
    heap.objects[self_id] = HeapObject::Object(vec![("self".into(), Value::Ref(self_id))]);
    let root = Value::Ref(self_id);
    let (cloned_heap, cloned) = clone_value(&heap, root);
    let cloned_id = match cloned { Value::Ref(id) => id, _ => panic!() };
    let HeapObject::Object(entries) = cloned_heap.get(cloned_id) else { panic!() };
    let self_ref_id = match entries[0].1 { Value::Ref(id) => id, _ => panic!() };
    assert_eq!(self_ref_id, cloned_id, "cloned self-reference must point to the cloned object");
}

#[test]
fn spec_circular_via_indirection() {
    // A → B → A (mutual circular)
    let mut heap = Heap::new();
    let a_id = heap.objects.len();
    heap.objects.push(HeapObject::Object(vec![]));
    let b_id = heap.objects.len();
    heap.objects.push(HeapObject::Object(vec![]));
    heap.objects[a_id] = HeapObject::Object(vec![("b".into(), Value::Ref(b_id))]);
    heap.objects[b_id] = HeapObject::Object(vec![("a".into(), Value::Ref(a_id))]);
    let (cloned_heap, cloned) = clone_value(&heap, Value::Ref(a_id));
    let new_a = match cloned { Value::Ref(id) => id, _ => panic!() };
    let HeapObject::Object(a_entries) = cloned_heap.get(new_a) else { panic!() };
    let new_b = match a_entries[0].1 { Value::Ref(id) => id, _ => panic!() };
    let HeapObject::Object(b_entries) = cloned_heap.get(new_b) else { panic!() };
    let back_to_a = match b_entries[0].1 { Value::Ref(id) => id, _ => panic!() };
    assert_eq!(back_to_a, new_a, "B's back-reference must point to the cloned A");
}

#[test]
fn spec_clone_produces_independent_target() {
    // After cloning, mutating the source must NOT affect the clone.
    let mut heap = Heap::new();
    let arr = heap.alloc(HeapObject::Array(vec![Value::Number(1.0)]));
    let (cloned_heap, cloned) = clone_value(&heap, arr.clone());
    // Mutate source:
    let src_id = match arr { Value::Ref(id) => id, _ => panic!() };
    if let HeapObject::Array(items) = heap.get_mut(src_id) {
        items.push(Value::Number(99.0));
    }
    // Clone should still have 1 item.
    let new_id = match cloned { Value::Ref(id) => id, _ => panic!() };
    let HeapObject::Array(items) = cloned_heap.get(new_id) else { panic!() };
    assert_eq!(items.len(), 1);
}
