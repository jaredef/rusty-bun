# rusty-bun-host — JS host integration spike (Tier-H #1+#2 partial)

**First operational instance of rusty-bun pilots running under a real JS engine.**

Per the [trajectory](../trajectory.md) §II Tier-H and the [seed](../seed.md) §VII Sub-criterion 4 (JS host integration), this crate embeds rquickjs (a Rust binding for QuickJS) and exposes existing pilots to JS code. **For the first time in the engagement, the pilots are callable from JS.**

Prior to this spike, the 16 pilots produced 591 Rust tests against Rust modules. Zero JS code had executed against any of them. This spike changes that: 15 of those tests are now **JS-side** tests running through the embedded engine, and a CLI binary `rusty-bun-host` accepts a `<script.js>` arg and runs the file with the wired pilots in `globalThis`.

## Pipeline

```
existing pilot crates (16 of them, building on rusty-blob, rusty-buffer,
  rusty-node-path, rusty-textencoder, rusty-urlsearchparams, rusty-web-crypto,
  ...)
       │
       ▼
new host crate at host/
  - rquickjs 0.6 dep (production-tested QuickJS Rust binding)
  - wire_globals(): installs atob, btoa, path.*, crypto.* into globalThis
  - eval_string / eval_bool / eval_i64 helpers for tests
  - CLI binary `rusty-bun-host <script.js>`
       │
       ▼
host/tests/integration.rs   15 tests; JS code through embedded host
host/examples/hello.js      end-to-end demo via CLI
       │
       ▼
15/15 pass + CLI runs example with exit 0 = host integration operational
```

## What's wired into globalThis

```
atob(base64String) -> string
btoa(byteString) -> string

path.basename(p, ext?)
path.dirname(p)
path.extname(p)
path.normalize(p)
path.isAbsolute(p)
path.join(a, b?)
path.sep ("/")
path.delimiter (":")

crypto.randomUUID()
```

Spike scope: enough surface to demonstrate the integration model works. Future sessions extend to TextEncoder/TextDecoder, URLSearchParams, Buffer, Blob, structuredClone, AbortController, fetch-api types — every existing pilot becomes a wiring target.

## Verifier results: 15/15

```
JS pure-language sanity                                   2 (1+2+3, array.join)
btoa / atob roundtrip + known values                      3
path.basename, dirname, extname, normalize, isAbsolute    7
path.sep constant                                         1
crypto.randomUUID format + uniqueness                     2
Compositional: chain pilots from JS                       1 (atob+path.basename)
```

The compositional test is significant: it exercises **two pilots from a single JS expression**:
```js
const cipher = btoa("/usr/local/bin/node");
const recovered = atob(cipher);
path.basename(recovered);  // → "node"
```

This is the first cross-pilot composition the apparatus has produced *from JS code*. Until this spike, all cross-pilot composition was Rust-side (e.g., the File pilot composing with Blob via path-dependency).

## CLI demo

```bash
$ ./target/release/rusty-bun-host host/examples/hello.js
$ echo $?
0
```

The example script runs btoa/atob roundtrip + path manipulation + crypto.randomUUID + cross-pilot composition. Exit code 0 = all JS-side assertions passed.

## What this spike proves

1. **The integration model works.** rquickjs + Rust-side wiring + JS-side calling = working JS execution against pilot implementations. The shape generalizes; future sessions add wirings for the un-wired pilots without changing the integration model.

2. **The pilots are runtime-shaped.** The pilots' Rust APIs translate to JS-host objects with negligible adaptation cost. atob/btoa wrap the rusty-buffer base64 codec; path.* wraps rusty-node-path 1:1; crypto.randomUUID wraps rusty-web-crypto's UUID generator.

3. **The workspace runner picks up host tests automatically.** `./bin/run-pilots.sh` now reports 606 tests (591 prior + 15 host integration) across 72 suites, all passing.

4. **The integration is small.** ~150 LOC of FFI glue for 9 surfaces wired. The integration cost per surface is low.

## What this spike does NOT prove

- The wired surface set covers a working runtime. We have ~5% of the surface a real Bun-using app would touch.
- Module loading works. The CLI evaluates a single script; no `import` / `require` resolution.
- The full pilots' API works through the bridge. Only spike-scope subsets are wired (e.g., path's join takes 1-2 args, not spread).
- Async / Promise integration. rquickjs supports async; the spike doesn't exercise it.
- WebSocket / fetch / streams integration with transport. Out of spike scope.

The spike is the *first* of approximately **30+ wiring + integration tasks** that Tier-H requires. Each subsequent task (wire TextEncoder, wire Buffer, wire URLSearchParams, wire fetch, ...) is incremental against this proven base.

## Trajectory advance

Per the trajectory's Tier-H:
- **H.1 — JS engine selection.** ✓ DONE. Selected rquickjs 0.6 (production-tested, well-maintained, smaller than Boa, has Rust-idiomatic API).
- **H.2 — Pilots-to-JS FFI.** Partial. 9 surfaces wired (atob, btoa, path.*, crypto.randomUUID). Many more pending; pattern established.
- **H.3 — Module loader + resolver.** Not started.
- **H.4 — globalThis setup.** Partial via H.2.
- **H.5 — Console + error reporting.** Not started.

Approximately **20-30% of Tier-H** is complete after this spike, on the most load-bearing axes (engine selection + integration model + first pilot wiring set). The remaining Tier-H work is incremental.

## Files

```
host/
├── AUDIT.md (this file's companion)
├── Cargo.toml             rquickjs + 3 pilot crate path-deps
├── examples/
│   └── hello.js           runnable demo
├── src/
│   ├── lib.rs             ~140 LOC: new_runtime, wire_globals, eval helpers
│   └── main.rs            CLI binary entry
└── tests/
    └── integration.rs     15 JS-driven integration tests
```

## Provenance

- Tool: rquickjs 0.6 (Rust binding for QuickJS), pulled from crates.io.
- Substrate: rusty-buffer (Pilot 10), rusty-node-path (Pilot 8), rusty-web-crypto (Pilot 16).
- Result: 15/15 host tests pass; CLI runs example with exit 0; workspace runner reports 606/0/1 across 72 suites.
- This is the first operational instance of rusty-bun pilots running under a real JS engine.
