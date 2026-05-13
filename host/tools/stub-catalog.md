# Stub Catalogue — rusty-bun-host

Per [Doc 716 §VI](https://jaredfoy.com/resolve/doc/716-stubs-as-named-cuts) — the canonical record of substrate stubs in the engagement. Each entry encodes the (substrate node, cut rung, kind, in-degree from sample, known consumers exercising, re-open condition) tuple. Per Doc 716 §V the stub-alphabet is conjecturally bounded at three kinds (K1 throw-on-use, K2 no-op return, K3 hardcoded-sentinel); each entry below classifies into one of those.

Cross-reference: run `host/tools/stub-list.sh` to surface unmarked candidates from the codebase; run `host/tools/substrate-rank.sh` for the in-degree column. The product (in-degree × known-consumer-count) is the closure-priority ranking.

Format per row: `| substrate node | kind | cut rung | in-degree | known consumers | re-open |`. Empty cells mean "none recorded" or "not measured at this scope."

## K1 — throw-on-use

The function/class exists and is callable; calling it throws an explicit "not implemented" / "not supported" error.

| node | rung | in-degree | known consumers | re-open |
|---|---|---|---|---|
| `node:sqlite.DatabaseSync` | L5 | 6 | (undici cache, typescript types — none exercise) | Bun.SQLite full impl OR consumer that needs real sqlite |
| `node:worker_threads.Worker` | L5 | 21 | (workers consumers — none in basket exercise) | Real Worker thread substrate (substantial — separate JS engine per worker) |
| `node:net.createServer` | L5 | 32 | — (http.createServer routes through Bun.serve) | Real socket-server primitive separate from Bun.serve |
| `node:tls.createServer` | L5 | 7 | — (https routes through Π1.4 TLS substrate) | Server-side TLS handshake (we have client-side) |
| `fs.readdirSync` | L5 | 175 | E.20-class file-walking consumers (now retired via basket) | Real readdir Rust binding |
| `fs.readlinkSync` | L5 | 175 | (symlink-aware consumers) | Real readlink |
| `fs/promises.writeFile` | L5 | 46 | (consumers using async fs.writeFile) | Just wrap fs.writeFileSync in Promise |
| `fs/promises.mkdir/rm/stat/readdir/unlink/readlink/rename/open/opendir/symlink/truncate` | L5 | 46 | varies | Per-method wrappers around existing sync surfaces (mostly trivial) |
| `stream.compose` | L5 | 193 | (modern stream composition) | Real compose() per Node 16+ |
| `URL.createObjectURL` | L5 | low | (blob URL consumers) | Real blob registry |

## K2 — no-op return

The function/class exists; calling it returns a benign sentinel (`null`, `undefined`, `false`, empty object, no-op).

| node | rung | in-degree | known consumers | re-open |
|---|---|---|---|---|
| `node:cluster.fork` | L4 | 1 | — (returns null; consumers check isMaster=true and skip) | Real multi-process spawn |
| `node:cluster.{disconnect,setupMaster,...}` | L4 | 1 | — | (Same) |
| `node:inspector.{open,close,Session.*}` | L4 | 1 | (debugger consumers — none in basket) | Real debugger protocol |
| `FinalizationRegistry.{register,unregister}` | L4 | (built-in) | (cache-class consumers — work without finalization) | Real finalization (would need rquickjs hook into GC) |
| `Event.{stopPropagation,stopImmediatePropagation}` | L4 | (built-in) | (DOM-like consumers — work because no bubbling exists) | Real propagation if we ever wire bubbling |
| `MessagePort.postMessage` | L4 | (built-in) | (worker IPC consumers — partial) | Real message ports (would need worker substrate) |
| `BroadcastChannel.{constructor,postMessage,close}` | L4 | (built-in) | (broadcast consumers — none in basket) | Real cross-context channel |
| `fs.{closeSync,ftruncateSync,fsyncSync}` | L4 | 175 | (file-handle consumers, partial) | Real fd-based fs |
| `fs.watchFile / fs.unwatchFile` | L4 | 175 | (polling watchers — chokidar uses inotify directly via fs.watch) | Real polling-based watch (chokidar already retired) |
| `Stats.{isSymbolicLink,isBlockDevice,isCharacterDevice,isFIFO,isSocket}` | L4 | 175 | (Stats consumers — most use isFile/isDirectory only) | Real stat with full mode bits |
| `tty.isatty` | L4 | 18 | (color-detection libs — return false is safe) | Real isatty check via libc |
| `PerformanceObserver.{constructor,observe,disconnect}` | L4 | 25 | (perf-trace consumers — partial) | Real performance entry pipeline |
| `node:readline.emitKeypressEvents` | L4 | 9 | (interactive CLI — none in basket) | Real keypress event pump |
| `Intl.PluralRules.select` (other-fallback) | L5 | varies | (i18n consumers — works for English) | Real CLDR plural-rule tables |

## K3 — hardcoded-sentinel

The function returns a fixed plausibly-correct value; corner-case-discriminating consumers fail to differentiate.

| node | rung | in-degree | known consumers | re-open |
|---|---|---|---|---|
| `Intl.PluralRules.select` (returns "other") | L5 | varies | (non-English-plural consumers) | Real CLDR plural-rule tables per locale |
| `Intl.Segmenter` (code-point granularity) | L5 | (built-in) | (grapheme-cluster consumers fail on emoji ZWJ sequences) | Real CLDR grapheme-break tables |
| `node:cluster.{isMaster:true, isPrimary:true, workers:{}}` | L4 | 1 | (cluster-aware libs that read state-not-call-methods) | Real cluster state |
| `process.{getuid,getgid}: () => -1` | L4 | 66 | (privilege-checking consumers) | Real libc uid/gid (engagement-scope, easy) |
| `process.execPath: ""` (was empty) | L4 | 66 | (sub-process consumers — retired via /proc/self/exe) | Already retired |
| `node:diagnostics_channel.channel(name)` (no-op subscribers) | L4 | 32 | (telemetry consumers — work because they fire-and-forget) | Real channel routing if observability consumer surfaces |
| `node:async_hooks.AsyncLocalStorage` (passthrough store) | L4 | 20 | (request-scoped state consumers — partial) | Real async-context tracking (would need rquickjs hook) |

## Cross-cutting observations

**K1 dominates fs/promises** because that namespace was added as a Promise-wrapping shim after the sync surface stabilized; methods without sync counterparts throw. Closing them is bulk-mechanical: wrap each sync method in `async (...) => sync(...)` if a sync exists, or implement via Rust binding if not.

**K2 dominates worker-class + finalization-class surfaces** because the apparatus made an explicit decision per [Doc 714 sub-§4.b cut-location framework](https://jaredfoy.com/resolve/doc/714-the-rusty-bun-engagement-read-through-the-lattice-extension-basin-expansion-at-the-l2m-saturation-point) to skip multi-process / multi-thread substrate. Closing requires a separate JS engine per worker or rquickjs threading hooks — substantial.

**K3 dominates locale / interactive / privilege surfaces** because those carry CLDR-scale data (locale tables) or system-state (uid/gid) that the apparatus didn't pull into its bounded scope. Closing some (process.getuid via libc) is trivial; closing others (Intl plurals across locales) is data-heavy.

## Priority ranking (operational output 1 per Doc 716 §VI)

Heads of the priority distribution — (in-degree × known-consumer-count) leverage:

1. `fs/promises.writeFile/mkdir/rm/stat/readdir/etc.` — in-degree 46, mechanically closable. Highest leverage per Doc 715 §X.a heavy-tail.
2. `fs.readdirSync` — in-degree 175, already retired via the basket sweep through alternate paths (glob, chokidar via inotify) but the direct surface remains K1.
3. `node:cluster.fork` — low in-degree (1) but high consumer-class (cluster-aware libs). Closure requires real multi-process substrate (out-of-engagement scope).
4. `node:worker_threads.Worker` — in-degree 21, requires separate JS engine per worker. Successor-engagement scope.

## Stub-alphabet stability check (operational output 3 per Doc 716 §VI)

Across this catalogue: only K1, K2, K3 instances. No fourth kind observed. Conjecture from [Doc 716 §V](https://jaredfoy.com/resolve/doc/716-stubs-as-named-cuts) holds at this engagement's scope.

Future engagements applying Pin-Art elsewhere should each maintain their own stub-catalog.md following this schema; the corpus-level conjecture validates by aggregating their evidence.
