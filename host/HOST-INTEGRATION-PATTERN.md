# Host integration pattern

Reference document for wiring rusty-bun pilots into the JS host (`rusty-bun-host`). Per [seed §III.A8](../seed.md) (architecture decision) and [seed §IV.M6](../seed.md) (host-wirability future-move discipline). Future pilot wirings should follow this document.

The pattern was crystallized after the iteration documented in [host/RUN-NOTES.md](RUN-NOTES.md) where two technical findings emerged: (1) QuickJS' GC does not track Rust-captured state stored on JS objects, and (2) rquickjs `Opt<T>` requires JS-side argument omission rather than passing `undefined`. Both findings constrain how pilots are wired.

## Three patterns by pilot shape

### Pattern 1 — Pure-value pilot API

Use when the pilot's Rust API is a free function or static method that takes and returns owned values.

**Examples:** `atob`, `btoa`, `path.basename`, `path.dirname`, `crypto.randomUUID`, `crypto.subtle.digestSha256Hex`, `fs.readFileSyncUtf8`, `fs.existsSync`.

**Pattern:**
```rust
fn wire_my_pilot<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    global.set(
        "myFunction",
        Function::new(ctx.clone(), |arg: String| -> String {
            rusty_my_pilot::my_function(&arg)
        })?,
    )?;
    Ok(())
}
```

No state. No JS-side wrapper class needed. Optional args use `Opt<T>`:
```rust
Function::new(ctx.clone(), |arg: String, opt_arg: Opt<String>| -> String {
    rusty_my_pilot::my_function(&arg, opt_arg.0.as_deref())
})?
```

If a JS caller is going to invoke this and might pass `undefined` for the optional arg, the JS-side must branch (see Pattern 3 finding 2).

### Pattern 2 — Namespaced object of pure-value functions

Use when several related functions share a namespace (`path.*`, `fs.*`, `crypto.*`, `Buffer.*`).

**Examples:** all the `path.*` methods, `fs.*` methods, `Buffer.*` static methods.

**Pattern:**
```rust
fn wire_my_namespace<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    let ns = Object::new(ctx.clone())?;
    ns.set("first", Function::new(ctx.clone(), |x: String| -> String {
        rusty_pilot::first(&x)
    })?)?;
    ns.set("second", Function::new(ctx.clone(), |x: u32| -> u32 {
        rusty_pilot::second(x)
    })?)?;
    ns.set("constant", "/")?;  // string constants OK directly
    global.set("myNamespace", ns)?;
    Ok(())
}
```

Identical to Pattern 1 in stateless-ness; the namespace is just an Object with multiple Function members.

### Pattern 3 — Stateful types via JS-class wrapping stateless Rust helpers

Use when the pilot's API is a class with instance state (URLSearchParams, TextEncoder, TextDecoder, future Blob, File, Headers, Request, Response, AbortController, etc).

**The naive approach DOES NOT WORK.** Storing Rust closures that capture `Rc<RefCell<State>>` as methods on JS objects breaks QuickJS' GC. The runtime drop fires `list_empty(&rt->gc_obj_list)` debug assertion because QuickJS does not track Rust-side references that JS objects hold transitively.

**The correct pattern:**

a. **Rust side:** expose stateless helper functions in a private `__namespace` global. The functions take plain pairs/arrays as input and return plain pairs/arrays as output.

```rust
fn wire_my_class_static<'js>(ctx: &rquickjs::Ctx<'js>, global: &Object<'js>) -> JsResult<()> {
    let ns = Object::new(ctx.clone())?;
    ns.set("parse", Function::new(ctx.clone(), |input: String| -> Vec<Vec<String>> {
        // Parse input; return as plain JS-array-of-arrays (Vec<Vec<String>>)
        rusty_pilot::parse(&input).into_iter()
            .map(|(k, v)| vec![k, v])
            .collect()
    })?)?;
    ns.set("serialize", Function::new(ctx.clone(), |pairs: Vec<Vec<String>>| -> String {
        // Reconstruct from JS-array-of-arrays
        let pair_refs: Vec<(&str, &str)> = pairs.iter()
            .filter_map(|p| if p.len() >= 2 { Some((p[0].as_str(), p[1].as_str())) } else { None })
            .collect();
        rusty_pilot::serialize(&pair_refs)
    })?)?;
    global.set("__myclass", ns)?;
    Ok(())
}
```

b. **JS side:** install a class via `ctx.eval()` that holds its own pure-JS state and calls into the Rust helpers.

```rust
fn install_my_class_js<'js>(ctx: &rquickjs::Ctx<'js>) -> JsResult<()> {
    ctx.eval::<(), _>(r#"
        globalThis.MyClass = class MyClass {
            constructor(init) {
                this._pairs = typeof init === "string"
                    ? __myclass.parse(init)
                    : [];
            }
            append(k, v) { this._pairs.push([String(k), String(v)]); }
            toString() { return __myclass.serialize(this._pairs); }
        };
    "#)?;
    Ok(())
}
```

c. **Wire both** in `wire_globals`:
```rust
wire_my_class_static(&ctx, &global)?;
install_my_class_js(&ctx)?;
```

The JS class holds state in a plain `this._pairs` array. The Rust helpers operate on plain `Vec<Vec<String>>`. No Rust-side state is held by JS objects; QuickJS' GC sees only JS-side state, which it can drop cleanly.

**Active examples:** URLSearchParams, TextEncoder, TextDecoder.

### Pattern 4 — Spec-formalization pilot, JS-side instantiation

When a pilot's Rust crate models an algorithm against a custom representation (e.g., structured-clone's `Heap`/`Value`, streams' generic `ReadableStream<T>` with `Rc<RefCell>` state machines) and the algorithm operates on values the JS engine already has natively (Date, RegExp, Map, Set, ArrayBuffer, TypedArrays, Promises, async iterators), wire the surface as a JS-side reimplementation against the same constraint set the pilot was derived from. The pilot's Rust crate stays the canonical algorithmic reference (verifier-tests, doc citations, ratio anchor); the host's JS implementation is a sibling instantiation.

Apply when ALL of:
1. The Rust API takes/returns a custom representation rather than primitive values.
2. The algorithm is pure value-recursion plus memo (no I/O, no Rust-only crypto, etc.).
3. The JS engine's built-in types are sufficient for the operands.

**Active examples:** structuredClone, ReadableStream/WritableStream/TransformStream.

## Sync-or-async user callbacks

When invoking user-supplied callbacks that the spec declares MAY be sync OR async (ReadableStream's `start`/`pull`/`cancel`, WritableStream's `start`/`write`/`close`/`abort`, TransformStream's `transform`/`flush`), DO NOT blanket-wrap with `await`. Per [bug-catcher E.6](../bun-bug-catcher.md), wrapping a sync callback in `async () => await fn()` introduces an extra microtask boundary that, under rquickjs/QuickJS, drops the resumption of awaiters resolved synchronously inside the user callback.

**Wrong:**
```js
Promise.resolve().then(async () => {
    await userCallback();  // breaks if userCallback is sync and resolves a pending awaiter
});
```

**Right:**
```js
Promise.resolve().then(() => {
    const r = userCallback();
    if (r && typeof r.then === "function") {
        r.then(onResolve, onReject);
    } else {
        onResolve();
    }
});
```

## rquickjs argument-omission rule

When a JS class's method delegates to a Rust function with `Opt<T>` args, **the JS class MUST branch on `undefined` and omit the arg**, not pass undefined directly.

**Wrong (triggers conversion error):**
```js
class MyClass {
    method(arg) { return __myclass.method(arg); }  // arg may be undefined → fails
}
```

**Right:**
```js
class MyClass {
    method(arg) {
        if (arg === undefined) return __myclass.method();
        return __myclass.method(arg);
    }
}
```

rquickjs' `Opt<T>` accepts JS arity-omission, not undefined-as-value.

## Type translation cheat sheet

| Rust type | JS-side type | Notes |
|---|---|---|
| `String` | string | exact |
| `&str` | not directly bindable; use String | clone at the boundary |
| `i64` / `u64` | number | precision lost above 2^53 |
| `i32` / `u32` / `usize` | number | safe range |
| `f64` | number | exact |
| `bool` | boolean | exact |
| `Vec<u8>` | Array of numbers | becomes JS array; consider Uint8Array for true byte-array semantics (not yet wired) |
| `Vec<Vec<String>>` | Array of [string, string] arrays | works for pair-list passing |
| `Option<T>` | T or null/undefined | use `Opt<T>` for JS-omission semantics |
| `Result<T, E>` where E: Display | T or thrown error | `JsResult<T>` plus `rquickjs::Error::new_from_js_message` |

## Testing pattern

Every wired pilot adds at least one test to `host/tests/integration.rs` exercising the pilot from JS code:

```rust
#[test]
fn js_my_pilot_basic() {
    let r = eval_string(r#"
        // JS code that exercises the wired surface
        myFunction("input")
    "#).unwrap();
    assert_eq!(r, "expected output");
}
```

For composition tests (multiple pilots in one JS expression), name `js_compose_<surfaces>_<scenario>` per existing convention.

## Sequence for adding a new pilot to the host

1. Verify the pilot's Rust API is host-wirable per seed §IV.M6 (pure-value or splittable into stateless helpers).
2. Add the pilot's crate to `host/Cargo.toml` `[dependencies]`.
3. Implement a `wire_<name>` function in `host/src/lib.rs` per the matching pattern.
4. Call `wire_<name>` from `wire_globals`.
5. For stateful types: add `install_<name>_class_js` and call after the `wire_<name>_static`.
6. Add 3-5 integration tests to `host/tests/integration.rs`.
7. Add the pilot to `host/examples/runtime-demo.js` for the CLI demo.
8. Run `./bin/run-pilots.sh`; expect tests pass count to increase by N tests, suites by 1+.
9. Update `host/RUN-NOTES.md`'s wired-surface list and tests-per-suite count.

## Files

```
host/
├── HOST-INTEGRATION-PATTERN.md   ← this file
├── RUN-NOTES.md                  ← run record + technical findings
├── Cargo.toml                    ← workspace member with rquickjs + pilot deps
├── examples/                     ← runnable JS demos
├── src/
│   ├── lib.rs                    ← wire_globals + per-pilot wiring functions
│   └── main.rs                   ← CLI entry (rusty-bun-host <script.js>)
└── tests/
    └── integration.rs            ← JS-driven integration tests
```

## Provenance

- Pattern emerged: 2026-05-10 across two host iterations
  - Iteration 1 (commit `1c890da`): 9 surfaces, spike scope, GC issues with stateful wirings (sidestepped in spike)
  - Iteration 2 (commit `474cf29`): 8 pilot families, GC pattern resolved via JS-class + stateless-helper
- Formal integration into apparatus: this commit
- Future iterations should reference this document when adding new wirings
