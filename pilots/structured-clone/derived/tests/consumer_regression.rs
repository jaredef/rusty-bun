// Consumer-regression suite for structuredClone.
//
// Each test encodes a documented behavioral expectation from a real consumer
// of structuredClone with cited source. Per Doc 707, each test is a
// bidirectional pin: it constrains the derivation AND surfaces an invariant
// the original implementation is committed to.

use rusty_structured_clone::*;

fn clone(heap: &Heap, root: Value) -> (Heap, Value) {
    structured_clone(heap, &root).expect("clone should succeed")
}

// ─────────── immer — draft-finalize identity preservation ──────────
//
// Source: https://github.com/immerjs/immer/blob/main/src/utils/plugins.ts
//   immer 10+ uses structuredClone for `current()` to detach drafts.
// Consumer expectation: shared references in the source are preserved as
// shared references in the clone — immer's invariant "two draft properties
// pointing to the same source object continue to point to the same cloned
// object" depends on this.

#[test]
fn consumer_immer_shared_reference_survives_clone() {
    let mut heap = Heap::new();
    let shared = heap.alloc(HeapObject::Object(vec![("v".into(), Value::Number(42.0))]));
    let outer = heap.alloc(HeapObject::Object(vec![
        ("a".into(), shared.clone()),
        ("b".into(), shared.clone()),
    ]));
    let (cloned_heap, cloned) = clone(&heap, outer);
    let outer_id = match cloned { Value::Ref(id) => id, _ => panic!() };
    let HeapObject::Object(entries) = cloned_heap.get(outer_id) else { panic!() };
    let id_a = match entries[0].1 { Value::Ref(id) => id, _ => panic!() };
    let id_b = match entries[1].1 { Value::Ref(id) => id, _ => panic!() };
    assert_eq!(id_a, id_b, "immer requires shared-ref identity preservation");
}

// ─────────── @reduxjs/toolkit — listenerMiddleware serializability check ──
//
// Source: https://github.com/reduxjs/redux-toolkit/blob/master/packages/toolkit/src/
//   serializableStateInvariantMiddleware.ts uses structuredClone (or a
//   polyfill) to verify state is serializable; failure to clone = invariant
//   violation.
// Consumer expectation: functions in state throw DataCloneError, which
// the middleware catches and reports.

#[test]
fn consumer_redux_toolkit_function_throws() {
    let heap = Heap::new();
    let r = structured_clone(&heap, &Value::Function);
    assert!(matches!(r, Err(CloneError::NotCloneable("function"))));
}

#[test]
fn consumer_redux_toolkit_object_with_function_throws() {
    let mut heap = Heap::new();
    let obj = heap.alloc(HeapObject::Object(vec![
        ("ok".into(), Value::Number(1.0)),
        ("notOk".into(), Value::Function),
    ]));
    assert!(structured_clone(&heap, &obj).is_err());
}

// ─────────── Worker postMessage — circular reference roundtrip ──────────
//
// Source: HTML §10.5 postMessage spec mandates structuredClone for cross-
// realm message passing. Worker libraries (comlink, etc.) round-trip
// circular structures across threads.
// Reference: https://github.com/GoogleChromeLabs/comlink — the proxy machinery
// builds reference-graphs that may cycle via wrapped proxies.

#[test]
fn consumer_postmessage_circular_reference_roundtrip() {
    let mut heap = Heap::new();
    let id = heap.objects.len();
    heap.objects.push(HeapObject::Object(vec![]));
    heap.objects[id] = HeapObject::Object(vec![("self".into(), Value::Ref(id))]);
    let (cloned_heap, cloned) = clone(&heap, Value::Ref(id));
    let new_id = match cloned { Value::Ref(id) => id, _ => panic!() };
    let HeapObject::Object(entries) = cloned_heap.get(new_id) else { panic!() };
    let self_ref = match entries[0].1 { Value::Ref(id) => id, _ => panic!() };
    assert_eq!(self_ref, new_id);
}

// ─────────── lodash.cloneDeep migration — Map/Set order preservation ──────
//
// Source: many libraries migrating from lodash.cloneDeep to structuredClone
// rely on Map and Set preserving insertion order through the clone. lodash
// preserves it; structuredClone is required to as well per HTML spec.
// https://github.com/lodash/lodash/blob/main/src/_baseClone.js
//   preserves Map.entries iteration order; structuredClone replacements must too.

#[test]
fn consumer_lodash_migration_map_order_preserved() {
    let mut heap = Heap::new();
    let m = heap.alloc(HeapObject::Map(vec![
        (Value::String("first".into()), Value::Number(1.0)),
        (Value::String("second".into()), Value::Number(2.0)),
        (Value::String("third".into()), Value::Number(3.0)),
    ]));
    let (cloned_heap, cloned) = clone(&heap, m);
    let id = match cloned { Value::Ref(id) => id, _ => panic!() };
    let HeapObject::Map(entries) = cloned_heap.get(id) else { panic!() };
    let names: Vec<&Value> = entries.iter().map(|(k, _)| k).collect();
    assert_eq!(names[0], &Value::String("first".into()));
    assert_eq!(names[2], &Value::String("third".into()));
}

#[test]
fn consumer_lodash_migration_set_order_preserved() {
    let mut heap = Heap::new();
    let s = heap.alloc(HeapObject::Set(vec![
        Value::Number(7.0), Value::Number(3.0), Value::Number(11.0),
    ]));
    let (cloned_heap, cloned) = clone(&heap, s);
    let id = match cloned { Value::Ref(id) => id, _ => panic!() };
    let HeapObject::Set(items) = cloned_heap.get(id) else { panic!() };
    assert_eq!(items, &vec![Value::Number(7.0), Value::Number(3.0), Value::Number(11.0)]);
}

// ─────────── ArrayBuffer/TypedArray library consumers ──────────
//
// Source: numjs, ml-matrix, gl-matrix all clone TypedArray-backed structures.
// Consumer expectation: a TypedArrayView in the source whose buffer is the
// same as another View's buffer must, after clone, share its (cloned) buffer
// with the (cloned) other View.

#[test]
fn consumer_typed_array_views_share_cloned_buffer() {
    let mut heap = Heap::new();
    let buf = heap.alloc(HeapObject::ArrayBuffer(vec![0; 32]));
    let buf_id = match buf { Value::Ref(id) => id, _ => panic!() };
    let v1 = heap.alloc(HeapObject::TypedArrayView {
        buffer: buf_id, byte_offset: 0, length: 4, kind: TypedArrayKind::Uint32,
    });
    let v2 = heap.alloc(HeapObject::TypedArrayView {
        buffer: buf_id, byte_offset: 16, length: 4, kind: TypedArrayKind::Uint32,
    });
    let outer = heap.alloc(HeapObject::Array(vec![v1, v2]));
    let (cloned_heap, cloned) = clone(&heap, outer);
    let outer_id = match cloned { Value::Ref(id) => id, _ => panic!() };
    let HeapObject::Array(items) = cloned_heap.get(outer_id) else { panic!() };
    let v1_id = match items[0] { Value::Ref(id) => id, _ => panic!() };
    let v2_id = match items[1] { Value::Ref(id) => id, _ => panic!() };
    let HeapObject::TypedArrayView { buffer: b1, .. } = cloned_heap.get(v1_id) else { panic!() };
    let HeapObject::TypedArrayView { buffer: b2, .. } = cloned_heap.get(v2_id) else { panic!() };
    assert_eq!(b1, b2, "two views over the same buffer must share the cloned buffer");
}

// ─────────── WPT structured-clone test corpus ──────────
//
// Source: web-platform-tests/wpt/structured-clone/

#[test]
fn wpt_structured_clone_date_time_preserved() {
    let mut heap = Heap::new();
    let d = heap.alloc(HeapObject::Date(1_700_000_000_000));
    let (cloned_heap, cloned) = clone(&heap, d);
    let id = match cloned { Value::Ref(id) => id, _ => panic!() };
    assert_eq!(*cloned_heap.get(id), HeapObject::Date(1_700_000_000_000));
}

#[test]
fn wpt_structured_clone_regexp_source_and_flags_preserved() {
    let mut heap = Heap::new();
    let re = heap.alloc(HeapObject::RegExp {
        source: "^(?:abc|def)$".into(), flags: "gimsuy".into(),
    });
    let (cloned_heap, cloned) = clone(&heap, re);
    let id = match cloned { Value::Ref(id) => id, _ => panic!() };
    let HeapObject::RegExp { source, flags } = cloned_heap.get(id) else { panic!() };
    assert_eq!(source, "^(?:abc|def)$");
    assert_eq!(flags, "gimsuy");
}

#[test]
fn wpt_structured_clone_array_buffer_byte_content() {
    let mut heap = Heap::new();
    let buf = heap.alloc(HeapObject::ArrayBuffer((0u8..=255).collect()));
    let (cloned_heap, cloned) = clone(&heap, buf);
    let id = match cloned { Value::Ref(id) => id, _ => panic!() };
    let HeapObject::ArrayBuffer(bytes) = cloned_heap.get(id) else { panic!() };
    assert_eq!(bytes.len(), 256);
    assert_eq!(bytes[0], 0);
    assert_eq!(bytes[255], 255);
}
