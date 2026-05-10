# rusty-bun-host — JS host integration (Tier-H #1+#2)

**Pilots running as a JS runtime.** The host crate embeds rquickjs (a Rust binding for QuickJS) and exposes the rusty-bun derivation pilots to JS code through a working FFI bridge. Per seed §VII Sub-criterion 4 and trajectory §II Tier-H. Iterated across two sessions on 2026-05-10: spike (9 surfaces) → expanded (~25 surfaces across 8 wirings) including JS-class wrappers for stateful types.

## Wired surface

```
console.log / .error / .warn       host primitive (println! / eprintln!)
atob / btoa                        rusty-buffer base64 codec
path.basename / dirname / extname / normalize / isAbsolute / join
                                   rusty-node-path (POSIX subset)
path.sep / path.delimiter          constants
crypto.randomUUID                  rusty-web-crypto UUID v4
crypto.subtle.digestSha256Hex      rusty-web-crypto SHA-256
TextEncoder.prototype.encode       rusty-textencoder (via JS class)
TextDecoder.prototype.decode       rusty-textencoder (via JS class)
Buffer.byteLength / .from / .alloc / .concat /
  .decodeUtf8 / .encodeBase64 / .encodeHex
                                   rusty-buffer
URLSearchParams (full surface)     rusty-urlsearchparams (via JS class +
                                   stateless Rust helpers)
fs.readFileSyncUtf8 / .readFileSyncBytes / .writeFileSync /
  .existsSync / .unlinkSync / .mkdirSyncRecursive / .rmdirSyncRecursive
                                   rusty-node-fs
```

## Verifier results: 33/33

```
JS pure-language sanity                                       2
atob / btoa roundtrip                                         3
path.* (basename/dirname/extname/normalize/isAbsolute/sep/join) 7
crypto.randomUUID format + uniqueness                         2
TextEncoder + TextDecoder (encoding, encode/decode, unicode)  3
Buffer (byteLength, from/concat/alloc, base64/hex)            5
URLSearchParams (construction, getter, toString, sort)        3
fs (write+read roundtrip, existsSync, mkdir+rmdir recursive)  3
crypto.subtle.digestSha256Hex                                 1
Cross-pilot composition (Buffer⨯TextDecoder, URL⨯Buffer,
  fs⨯TextEncoder/Decoder, atob⨯path.basename)                 4
```

## CLI demo

```bash
$ ./target/release/rusty-bun-host host/examples/runtime-demo.js
=== rusty-bun-host runtime demo ===
Hello from rusty-bun-host. 1+2 = 3
btoa('Hello, world!') = SGVsbG8sIHdvcmxkIQ==
atob(...)              = Hello, world!
path.basename('/usr/local/bin/node') = node
path.normalize('/foo/bar//baz/..')   = /foo/bar
crypto.randomUUID()                     = 4bbb788d-b5a7-4c02-9c3e-c039a0901d9c
crypto.subtle.digestSha256Hex('hello')  = 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
encode + decode unicode = héllo, мир! 🌍
Buffer.byteLength('héllo') = 6
Buffer.encodeBase64(Buffer.from('hi')) = aGk=
URLSearchParams.toString() = a=1&a=3&b=2&c=x+y
params.getAll('a')         = [ 1, 3 ]
fs.existsSync(tmp)         = true
fs.readFileSyncUtf8(tmp)   = demo content
after unlink, existsSync   = false

All wired pilots functional from JS.
$ echo $?
0
```

This output came from JS code running through QuickJS, calling into rusty-bun pilot implementations through Rust FFI. Eight wired surfaces composed in a single script.

## Findings

1. **The integration model works.** rquickjs + Rust-side wiring + JS-side calling = working JS execution against pilot implementations. ~440 LOC of FFI glue + ~70 LOC of JS-class setup script = full runtime surface coverage for 8 pilot families.

2. **QuickJS GC requires care with stateful types.** Initial attempt at `URLSearchParams` and `TextEncoder/TextDecoder` used Rust closures that captured `Rc<RefCell<...>>` state and were stored on JS objects. QuickJS' GC doesn't track Rust-side references, so the runtime's drop fired a debug assertion (`list_empty(&rt->gc_obj_list)` failed). **Fix pattern**: stateless Rust helper functions exposed in private `__namespace` globals, plus a JS-side class that holds its own state (in plain JS) and calls into the Rust helpers. No Rust-captured state held by JS objects = no cycles = no GC complaint. Both URLSearchParams and TextEncoder/TextDecoder use this pattern.

3. **rquickjs `Opt<T>` requires the JS-side to omit the arg, not pass `undefined`.** The TextDecoder constructor passes `this._label` to `__td.decode(bytes, this._label)`. When `this._label` is `undefined` (default-constructed decoder), rquickjs tries to convert undefined → String and fails with "Error converting from js 'undefined' into type 'string'". Fix: branch in JS to omit the arg entirely when the value is undefined.

4. **JS-side classes wrapping Rust-side helpers is the cleanest pattern** for stateful types. The Rust side provides the algorithm; the JS side provides the prototype shape and state-holding. This separation also makes the wired surface easier to extend incrementally.

5. **The example script demonstrates real runtime composition.** Eight distinct surfaces working together in a single JS file with `console.log` output, file I/O, Unicode round-trip, base64 encoding, UUID generation, SHA-256 hashing, and URLSearchParams manipulation. This is operationally what a small JS script targeting Bun would look like.

## What this enables

- **Future Tier-F pilots can be wired into the same host as they land.** The integration model is proven; each new pilot is a small wiring task (~10-30 LOC of FFI per surface, plus a JS-side class for stateful types).
- **Differential testing prep.** Once enough surface is wired, real Bun-using JS scripts can be run under `rusty-bun-host` and compared against `bun` invocations. That's Tier-J in the trajectory.
- **WPT runner candidacy.** `rusty-bun-host <wpt-test.js>` could be wired into the WPT harness. That's Tier-I.

## What's still missing

- **Module loader / resolver** (no `import` / `require`). Single-script eval only.
- **Async / Promise integration**. rquickjs supports it; not wired yet.
- **WebSocket / fetch / streams**. Substantial; multi-pilot Tier-F + Tier-G work.
- **Process / signals / exit-code handling beyond simple eval errors.**
- **Source maps / better error reporting.**
- **The remaining ~70-80% of Bun's API surface** that isn't yet piloted.

## LOC measurement

```
host/src/lib.rs             521 LOC (443 code-only)
host/src/main.rs             40 LOC (CLI entry)
host/tests/integration.rs   ~270 LOC (33 integration tests)
host/examples/*.js          ~80 LOC (2 demo scripts)
```

Per the apparatus pattern, this is the *integration layer*. It does not derive an API surface; it *exposes* surfaces already derived. So the LOC ratio framing from prior pilots doesn't apply directly. The relevant claim is: **~440 code-only LOC of FFI glue exposes 8 pilot families as a working JS runtime.**

## Trajectory advance

Tier-H status (per trajectory §II):
  H.1 ✓ JS engine selection (rquickjs 0.6)
  H.2 ✓ (substantially) Pilots-to-JS FFI: 8 pilot families wired
        (atob/btoa, path, crypto, TextEncoder/Decoder, Buffer,
         URLSearchParams, fs, crypto.subtle); ~10-15 more pilot
         families to wire (Blob, File, structuredClone, AbortController,
         fetch-api, streams, Bun.file/serve/spawn, node-http)
  H.3 ⏳ Module loader + resolver: not started
  H.4 ⏳ globalThis setup: substantially done via H.2
  H.5 ⏳ Console + error reporting: console.log/.error/.warn done; source
        maps / stack-traces not yet

Approximately **40-50% of Tier-H** is complete. The integration model is proven; remaining Tier-H work is incremental against the proven base.

## Files

```
host/
├── AUDIT.md (companion)
├── Cargo.toml             rquickjs + 6 pilot crate deps
├── examples/
│   ├── hello.js           original spike demo
│   └── runtime-demo.js    8-surface composition demo
├── src/
│   ├── lib.rs             ~440 code-only LOC
│   └── main.rs            CLI binary entry
└── tests/
    └── integration.rs     33 JS-driven integration tests
```

## Provenance

- Tool: rquickjs 0.6.2 (Rust binding for QuickJS)
- Pilot dependencies: rusty-buffer, rusty-node-fs, rusty-node-path, rusty-textencoder, rusty-urlsearchparams, rusty-web-crypto
- Result: 33/33 host tests pass; CLI runs runtime-demo.js with exit 0; workspace runner reports 624/0/1 across 72 suites
- Iteration 1 (commit `1c890da`): 9 surfaces, spike scope, GC issues with stateful wirings (avoided in spike)
- Iteration 2 (this commit): 8 pilot families, ~25 surfaces; GC pattern resolved via JS-class + stateless Rust helpers
