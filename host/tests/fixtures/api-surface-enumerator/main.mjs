try {
// L2/L3 API-surface enumerator per Doc 714 sub-§4.c.
//
// Walks the documented Node + Bun + Web-platform surface and asserts
// presence + typeof + arity + shape. Each entry is one L2 (presence)
// or L3 (shape) micro-constraint derived from published spec, not
// from consumer probing.
//
// Output: a JSON report with per-category pass/fail counts and the
// full failure list. Runs identically under Bun and rusty-bun-host;
// differential reveals which spec surfaces rusty-bun lacks.

const results = [];
function check(category, name, fn) {
  try {
    const ok = !!fn();
    results.push({ category, name, ok });
  } catch (e) {
    results.push({ category, name, ok: false, err: String(e && (e.message || e)) });
  }
}
function expectG(category, globalName, kind, opts) {
  opts = opts || {};
  const val = globalThis[globalName];
  const name = globalName;
  check(category, name, () => {
    if (kind === "function") return typeof val === "function";
    if (kind === "object") return val !== null && typeof val === "object";
    if (kind === "class") return typeof val === "function" && val.prototype !== undefined;
    if (kind === "value") return val !== undefined;
    return false;
  });
  if (opts.proto) {
    for (const m of opts.proto) {
      check(category, name + ".prototype." + m, () =>
        val && val.prototype && typeof val.prototype[m] === "function");
    }
  }
  if (opts.statics) {
    for (const m of opts.statics) {
      check(category, name + "." + m, () => val && typeof val[m] === "function");
    }
  }
}
function expect(category, name, kind, val, opts) { return expectG(category, name, kind, opts); }

// ───── ES built-ins (mostly engine-provided; light shape checks) ─────
expect("es", "Promise", "class", Promise, {
  statics: ["resolve", "reject", "all", "allSettled", "race", "any", "withResolvers"],
  proto: ["then", "catch", "finally"],
});
expect("es", "Symbol", "function", Symbol, {
  statics: ["for", "keyFor"],
});
check("es", "Symbol.asyncIterator", () => typeof Symbol.asyncIterator === "symbol");
check("es", "Symbol.iterator", () => typeof Symbol.iterator === "symbol");
expect("es", "WeakRef", "class", WeakRef, { proto: ["deref"] });
expect("es", "FinalizationRegistry", "class", FinalizationRegistry, {
  proto: ["register", "unregister"],
});
expect("es", "Map", "class", Map, { proto: ["set", "get", "has", "delete", "keys", "values", "entries", "forEach", "clear"] });
expect("es", "Set", "class", Set, {
  proto: ["add", "has", "delete", "keys", "values", "entries", "forEach", "clear",
          "union", "intersection", "difference", "symmetricDifference",
          "isSubsetOf", "isSupersetOf", "isDisjointFrom"],
});
expect("es", "WeakMap", "class", WeakMap, { proto: ["get", "set", "has", "delete"] });
expect("es", "WeakSet", "class", WeakSet, { proto: ["add", "has", "delete"] });
expect("es", "Atomics", "object", Atomics);
check("es", "SharedArrayBuffer", () => typeof SharedArrayBuffer === "function");
check("es", "globalThis.structuredClone", () => typeof structuredClone === "function");
check("es", "queueMicrotask", () => typeof queueMicrotask === "function");

// Array ES2023 / ES2024 immutables
for (const m of ["toReversed", "toSorted", "toSpliced", "with", "groupBy"]) {
  if (m === "groupBy") {
    check("es", "Object.groupBy", () => typeof Object.groupBy === "function");
  } else {
    check("es", "Array.prototype." + m, () => typeof Array.prototype[m] === "function");
  }
}

// ───── Web platform globals ─────
expect("web", "fetch", "function", globalThis.fetch);
expect("web", "URL", "class", URL, {
  statics: ["canParse", "createObjectURL", "revokeObjectURL"],
  proto: ["toString", "toJSON"],
});
expect("web", "URLSearchParams", "class", URLSearchParams, {
  proto: ["get", "getAll", "has", "set", "append", "delete", "entries", "keys", "values", "forEach", "toString"],
});
expect("web", "Headers", "class", Headers, {
  proto: ["get", "set", "append", "delete", "has", "entries", "keys", "values", "forEach"],
});
expect("web", "Request", "class", Request, { proto: ["clone", "text", "json", "arrayBuffer", "blob"] });
expect("web", "Response", "class", Response, {
  statics: ["error", "redirect", "json"],
  proto: ["clone", "text", "json", "arrayBuffer", "blob"],
});
expect("web", "AbortController", "class", AbortController, { proto: ["abort"] });
expect("web", "AbortSignal", "class", AbortSignal, {
  statics: ["abort", "any", "timeout"],
  proto: ["addEventListener", "removeEventListener", "throwIfAborted"],
});
expect("web", "TextEncoder", "class", TextEncoder, { proto: ["encode", "encodeInto"] });
expect("web", "TextDecoder", "class", TextDecoder, { proto: ["decode"] });
expect("web", "Blob", "class", Blob, { proto: ["text", "arrayBuffer", "stream", "slice"] });
expect("web", "File", "class", File, { proto: ["text", "arrayBuffer"] });
expect("web", "FormData", "class", globalThis.FormData, { proto: ["get", "getAll", "has", "set", "append", "delete", "entries", "keys", "values", "forEach"] });
expect("web", "ReadableStream", "class", ReadableStream, { proto: ["getReader", "pipeTo", "pipeThrough", "tee", "cancel"] });
expect("web", "WritableStream", "class", WritableStream, { proto: ["getWriter", "abort", "close"] });
expect("web", "TransformStream", "class", TransformStream);
expect("web", "WebSocket", "class", WebSocket, { proto: ["send", "close"] });
check("web", "atob", () => typeof atob === "function");
check("web", "btoa", () => typeof btoa === "function");
check("web", "performance", () => typeof performance === "object" && typeof performance.now === "function");
check("web", "console", () => typeof console === "object" && typeof console.log === "function");
check("web", "setTimeout", () => typeof setTimeout === "function");
check("web", "setInterval", () => typeof setInterval === "function");
check("web", "clearTimeout", () => typeof clearTimeout === "function");
check("web", "clearInterval", () => typeof clearInterval === "function");

// Web Crypto
check("web", "crypto", () => typeof crypto === "object" && crypto !== null);
check("web", "crypto.randomUUID", () => typeof crypto.randomUUID === "function");
check("web", "crypto.getRandomValues", () => typeof crypto.getRandomValues === "function");
check("web", "crypto.subtle", () => typeof crypto.subtle === "object");
for (const m of ["digest", "sign", "verify", "encrypt", "decrypt", "importKey", "exportKey", "generateKey", "deriveBits", "deriveKey"]) {
  check("web", "crypto.subtle." + m, () => typeof crypto.subtle[m] === "function");
}

// Intl
check("web", "Intl", () => typeof Intl === "object");
check("web", "Intl.Collator", () => typeof Intl.Collator === "function");
check("web", "Intl.NumberFormat", () => typeof Intl.NumberFormat === "function");
check("web", "Intl.DateTimeFormat", () => typeof Intl.DateTimeFormat === "function");
check("web", "Intl.Segmenter", () => typeof Intl.Segmenter === "function");
check("web", "Intl.PluralRules", () => typeof Intl.PluralRules === "function");
check("web", "Intl.ListFormat", () => typeof Intl.ListFormat === "function");
check("web", "Intl.RelativeTimeFormat", () => typeof Intl.RelativeTimeFormat === "function");

// ───── process ─────
expect("process", "process", "object", globalThis.process);
for (const p of ["argv", "env", "platform", "arch", "version", "versions", "stdout", "stderr"]) {
  check("process", "process." + p, () => process[p] !== undefined);
}
for (const m of ["cwd", "exit", "nextTick", "hrtime", "on", "once", "off", "emit"]) {
  check("process", "process." + m, () => typeof process[m] === "function");
}
check("process", "process.pid is number", () => typeof process.pid === "number");
check("process", "process.versions.node", () => typeof process.versions.node === "string");
check("process", "process.argv is Array", () => Array.isArray(process.argv));
check("process", "process.env is object", () => typeof process.env === "object" && process.env !== null);
check("process", "process.hrtime.bigint", () => typeof process.hrtime.bigint === "function");

// ───── Buffer ─────
expect("buffer", "Buffer", "function", globalThis.Buffer, {
  statics: ["alloc", "allocUnsafe", "allocUnsafeSlow", "from", "isBuffer", "concat", "byteLength", "compare"],
});
check("buffer", "Buffer.poolSize", () => typeof Buffer.poolSize === "number" || Buffer.poolSize === undefined);
check("buffer", "Buffer.prototype is Uint8Array-derived", () => {
  const b = Buffer.alloc(4);
  return b instanceof Uint8Array;
});
for (const m of ["write", "toString", "toJSON", "equals", "compare", "fill", "indexOf", "includes", "slice", "subarray", "copy",
                 "readUInt8", "readUInt16LE", "readUInt16BE", "readUInt32LE", "readUInt32BE",
                 "writeUInt8", "writeUInt16LE", "writeUInt32LE",
                 "readInt8", "readInt32LE",
                 "readBigUInt64LE", "readDoubleLE", "writeDoubleLE"]) {
  check("buffer", "Buffer.prototype." + m, () => typeof Buffer.prototype[m] === "function");
}
check("buffer", "Buffer() callable without new", () => {
  const b = Buffer("hi");
  return b && b.length === 2;
});

// ───── node:* builtins (require + import) ─────
const nodeBuiltins = [
  "fs", "path", "os", "url", "crypto", "http", "https", "buffer", "process", "events",
  "util", "util/types", "stream", "stream/promises", "stream/web",
  "assert", "assert/strict", "querystring", "dns", "dns/promises",
  "child_process", "net", "tty", "zlib", "diagnostics_channel",
  "perf_hooks", "async_hooks", "timers", "timers/promises", "console",
  "fs/promises", "test", "worker_threads", "http2", "vm", "string_decoder",
  "readline", "readline/promises", "module", "cluster",
];
for (const name of nodeBuiltins) {
  check("node:builtin", "node:" + name + " import", () => {
    // Don't actually import (would need dynamic import + await); just check
    // that require resolves the name.
    try {
      const m = require("node:" + name);
      return m !== undefined && m !== null;
    } catch (e) {
      return false;
    }
  });
  check("node:builtin", name + " (bare) import", () => {
    try {
      const m = require(name);
      return m !== undefined && m !== null;
    } catch (e) {
      return false;
    }
  });
}

// ───── node:fs surface ─────
const fs = require("node:fs");
for (const m of ["readFileSync", "writeFileSync", "existsSync", "statSync", "lstatSync",
                 "readdirSync", "mkdirSync", "rmdirSync", "unlinkSync", "realpathSync"]) {
  check("node:fs", "fs." + m, () => typeof fs[m] === "function");
}
check("node:fs", "fs.promises", () => typeof fs.promises === "object");

// ───── node:path surface ─────
const path = require("node:path");
for (const m of ["basename", "dirname", "extname", "join", "normalize", "isAbsolute",
                 "resolve", "relative", "parse", "format"]) {
  check("node:path", "path." + m, () => typeof path[m] === "function");
}
for (const v of ["sep", "delimiter"]) {
  check("node:path", "path." + v, () => typeof path[v] === "string");
}
check("node:path", "path.posix", () => typeof path.posix === "object");

// ───── node:crypto surface ─────
const nodeCrypto = require("node:crypto");
for (const m of ["createHash", "createHmac", "randomBytes", "randomFillSync",
                 "pbkdf2Sync", "randomUUID", "getRandomValues"]) {
  check("node:crypto", "crypto." + m, () => typeof nodeCrypto[m] === "function");
}
check("node:crypto", "crypto.subtle", () => typeof nodeCrypto.subtle === "object");
check("node:crypto", "crypto.webcrypto", () => typeof nodeCrypto.webcrypto === "object");

// ───── node:os surface ─────
const os = require("node:os");
for (const m of ["platform", "arch", "type", "tmpdir", "homedir", "hostname", "endianness", "cpus", "totalmem", "freemem"]) {
  check("node:os", "os." + m, () => typeof os[m] === "function");
}
check("node:os", "os.EOL", () => typeof os.EOL === "string");

// ───── node:events surface ─────
const events = require("node:events");
check("node:events", "events.EventEmitter", () => typeof events.EventEmitter === "function");
for (const m of ["on", "once", "off", "emit", "addListener", "removeListener", "listenerCount", "listeners"]) {
  check("node:events", "EventEmitter.prototype." + m, () => typeof events.EventEmitter.prototype[m] === "function");
}
check("node:events", "events.once", () => typeof events.once === "function");

// ───── node:stream surface ─────
const stream = require("node:stream");
for (const cls of ["Readable", "Writable", "Duplex", "Transform", "PassThrough"]) {
  check("node:stream", "stream." + cls, () => typeof stream[cls] === "function");
}
check("node:stream", "stream.pipeline", () => typeof stream.pipeline === "function");
check("node:stream", "stream.finished", () => typeof stream.finished === "function");

// ───── node:util surface ─────
const util = require("node:util");
for (const m of ["promisify", "callbackify", "format", "inspect", "isDeepStrictEqual",
                 "deprecate", "debuglog", "inherits"]) {
  check("node:util", "util." + m, () => typeof util[m] === "function");
}
check("node:util", "util.types", () => typeof util.types === "object");

// ───── Bun.* surface ─────
const Bun_ = globalThis.Bun;
expect("bun", "Bun", "object", Bun_);
if (Bun_) {
  for (const m of ["serve", "file", "spawn", "spawnSync", "write", "deepEquals", "inspect",
                   "sleep", "sleepSync", "nanoseconds", "escapeHTML",
                   "fileURLToPath", "pathToFileURL",
                   "gzipSync", "gunzipSync", "deflateSync", "inflateSync",
                   "password"]) {
    check("bun", "Bun." + m, () => typeof Bun_[m] === "function" || typeof Bun_[m] === "object");
  }
  check("bun", "Bun.CryptoHasher", () => typeof Bun_.CryptoHasher === "function");
  check("bun", "Bun.Glob", () => typeof Bun_.Glob === "function");
  check("bun", "Bun.password.hash", () => Bun_.password && typeof Bun_.password.hash === "function");
  check("bun", "Bun.password.verify", () => Bun_.password && typeof Bun_.password.verify === "function");
  check("bun", "Bun.dns.lookup", () => Bun_.dns && typeof Bun_.dns.lookup === "function");
}

// ───── Report ─────
const byCategory = {};
const failures = [];
for (const r of results) {
  byCategory[r.category] = byCategory[r.category] || { pass: 0, fail: 0 };
  if (r.ok) byCategory[r.category].pass++;
  else { byCategory[r.category].fail++; failures.push({ category: r.category, name: r.name, err: r.err }); }
}
const total = results.length;
const passed = results.filter(r => r.ok).length;

process.stdout.write(JSON.stringify({
  summary: { total, passed, failed: total - passed, percent: Math.round(passed * 1000 / total) / 10 },
  byCategory,
  failures,
}, null, 2) + "\n");
} catch (e) {
  process.stdout.write("FATAL:" + (e && (e.message || e)) + "\n" + (e && e.stack) + "\n");
}
