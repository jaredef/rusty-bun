//! Module Record + linking phases + host hooks. Per
//! specs/rusty-js-runtime-design.md §VI–§VII.
//!
//! Tier-Ω.5.b: ESM module loader. Adds disk-backed module resolution,
//! a module cache keyed by resolved URL, recursive evaluation of
//! cross-module imports, and host-installable built-in dispatch
//! (`node:fs`, `node:path`, ...). Snapshot binding semantics (v1
//! deviation from spec live-binding) — the importer reads the
//! namespace at evaluation entry; mutual cyclic imports may observe
//! Undefined for partially-evaluated dependencies. Documented in
//! pilots/rusty-js-runtime/trajectory.md row Ω.5.b.
//!
//! Out of scope (deferred per Tier-Ω.5.b ceiling):
//! - CJS / `require` interop.
//! - package.json + node_modules walk-up.
//! - Bare specifiers without `node:` prefix.
//! - Dynamic `import()` expression.
//! - `import.meta.url` / `import.meta.dir`.
//! - Live (read-through) bindings.
//! - Top-level await.
//!
//! Tier-Ω.5.h: ESM re-export forms landed. All four shapes are lowered
//! (`export { x } from`, rename, `export * from`, `export * as ns from`)
//! including default<->named conversions. Source modules are loaded
//! eagerly during the link phase so their namespaces are populated when
//! the namespace-build phase reads from them. Snapshot semantics
//! continue to apply — live bindings are still deferred.

use crate::interp::{Frame, Runtime, RuntimeError};
use crate::value::{Object, ObjectRef, Value};
use rusty_js_ast::Module as AstModule;
use rusty_js_bytecode::{CompiledModule, ExportBinding, ImportBindingKind};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleStatus { Unlinked, Linking, Linked, Evaluating, Evaluated, Failed }

/// Tier-Ω.5.j.cjs: ESM vs CJS classification. Drives the evaluation
/// pipeline: ESM goes through `evaluate_module`, CJS goes through the
/// wrapper-synthesis path in `evaluate_cjs_module`. ESM-importing-CJS
/// reads `module.exports` instead of a spec namespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleKind { ESM, CJS }

pub struct ModuleRecord {
    pub url: String,
    pub status: ModuleStatus,
    pub ast: Rc<AstModule>,
    pub bytecode: Rc<CompiledModule>,
    pub namespace: Option<ObjectRef>,
    /// Tier-Ω.5.j.cjs: kind tag. ESM for built-ins and `.mjs` / `.js`
    /// under a `"type":"module"` package.json. CJS otherwise.
    pub kind: ModuleKind,
    /// Tier-Ω.5.j.cjs: for CJS modules, the post-evaluation
    /// `module.exports` value (possibly rebound from the original
    /// `{exports:{}}`). ESM/built-in records leave this as None.
    pub cjs_exports: Option<Value>,
}

/// Tier-Ω.5.j.cjs: classify a resolved URL or a bare `node:*` specifier.
///
///   1. `node:*` → ESM (built-in namespace).
///   2. `.mjs` → ESM.
///   3. `.cjs` → CJS.
///   4. `.js` (or no extension) → walk up looking for the nearest
///      `package.json`. If it parses and contains `"type":"module"`,
///      ESM; otherwise CJS. Missing package.json defaults to CJS.
///   5. Anything else (no extension, no parent walk hit) defaults to
///      CJS — matches Node's heuristic for bare loose files.
pub fn detect_module_kind(resolved_url: &str) -> ModuleKind {
    if resolved_url.starts_with("node:") {
        return ModuleKind::ESM;
    }
    let path_str = match resolved_url.strip_prefix("file://") {
        Some(p) => p,
        None => return ModuleKind::CJS,
    };
    let path = std::path::Path::new(path_str);
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    match ext {
        "mjs" => ModuleKind::ESM,
        "cjs" => ModuleKind::CJS,
        _ => {
            // .js or unknown.
            //
            // Tier-Ω.5.hh: source-sniff for ESM markers BEFORE the
            // package.json walk. Many `.js` files in the parity corpus are
            // ESM-shape (top-level `import`/`export`) but live under a
            // package.json with no `"type":"module"` (or with `"type":
            // "commonjs"` from a transitive dep's package layout). Per the
            // top-of-alphabet conjecture (Doc 714 §VI Consequence 11) +
            // cross-pipeline diagnostic (Doc 720): the kind-detection
            // alphabet's top is the actual ESM-marker presence, not the
            // package.json type field. Closes the 9-package
            // "expected '(' after import" cluster in cjs-wrapper parse.
            if let Ok(head) = read_source_head(path, 65536) {
                if source_has_esm_markers(&head) { return ModuleKind::ESM; }
            }
            // Fall back to the package.json walk.
            let mut cur = path.parent();
            while let Some(d) = cur {
                let candidate = d.join("package.json");
                if candidate.is_file() {
                    if let Ok(text) = std::fs::read_to_string(&candidate) {
                        // Lightweight scan — full JSON parse would pull
                        // a dep. Look for `"type"` then `"module"` or
                        // `"commonjs"` in textual order. False positives
                        // on comments-in-strings are tolerable for v1.
                        if let Some(t) = scan_package_type(&text) {
                            return if t == "module" { ModuleKind::ESM } else { ModuleKind::CJS };
                        }
                        // package.json with no "type" → CJS per Node.
                        return ModuleKind::CJS;
                    }
                }
                cur = d.parent();
            }
            ModuleKind::CJS
        }
    }
}

/// Tier-Ω.5.hh: read the first `n` bytes of `path` for ESM-marker sniffing.
fn read_source_head(path: &std::path::Path, n: usize) -> std::io::Result<String> {
    use std::io::Read;
    let mut f = std::fs::File::open(path)?;
    let mut buf = vec![0u8; n];
    let read = f.read(&mut buf)?;
    buf.truncate(read);
    Ok(String::from_utf8_lossy(&buf).to_string())
}

/// Tier-Ω.5.hh: sniff for ESM markers — top-level `import` or `export`
/// keywords outside strings/comments. v1 heuristic: line-start match
/// after skipping leading whitespace + a small set of common shebang /
/// "use strict" / block-comment prefixes. False positives (`import` in
/// a string at line start) are tolerable for the parity-corpus shape;
/// false negatives (ESM file with import not at line start) are rare.
fn source_has_esm_markers(text: &str) -> bool {
    // Strip a leading shebang line if present.
    let mut t = text;
    if t.starts_with("#!") {
        if let Some(nl) = t.find('\n') { t = &t[nl+1..]; }
    }
    // Look for line-start `import` or `export` keywords with a word
    // boundary after (to avoid matching `importFoo`). Allow leading
    // whitespace per line.
    for line in t.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("import") {
            let rest = &trimmed[6..];
            if rest.is_empty() { return true; }
            let c = rest.chars().next().unwrap();
            // `import` keyword: next char is space, '{', '*', '"', '\''.
            if c.is_whitespace() || c == '{' || c == '*' || c == '"' || c == '\'' || c == '(' {
                // Reject `import(` only if it's specifically dynamic
                // import at expression position. At line-start that's
                // less common; for the kind-sniff we'd accept it but
                // be conservative — `import(` alone doesn't imply ESM.
                if c != '(' { return true; }
            }
        }
        if trimmed.starts_with("export") {
            let rest = &trimmed[6..];
            if rest.is_empty() { return true; }
            let c = rest.chars().next().unwrap();
            if c.is_whitespace() || c == '{' || c == '*' || c == '"' || c == '\'' {
                return true;
            }
        }
    }
    false
}

/// Minimal `"type"` field scan over package.json text. Returns
/// `"module"` or `"commonjs"` if found, else None. Avoids pulling a
/// JSON-parser dep into the runtime crate.
fn scan_package_type(text: &str) -> Option<String> {
    // Tier-Ω.5.tt: scan every `"type"` occurrence and return the first
    // one whose value is `"module"` or `"commonjs"`. Earlier versions
    // returned the first `"type"` regardless — but package.json's
    // funding/repository/bugs blocks all carry `"type":"git"` or
    // `"type":"opencollective"`, and ordering varies. unified +
    // temporal-polyfill + others have a non-top-level `"type"` first.
    let mut cursor = 0usize;
    while let Some(rel) = text[cursor..].find("\"type\"") {
        let key_pos = cursor + rel;
        let after = &text[key_pos + 6..];
        let Some(colon) = after.find(':') else { return None; };
        let after = &after[colon + 1..];
        let after = after.trim_start();
        if let Some(rest) = after.strip_prefix('"') {
            if let Some(end) = rest.find('"') {
                let v = &rest[..end];
                if v == "module" || v == "commonjs" {
                    return Some(v.to_string());
                }
            }
        }
        cursor = key_pos + 6;
    }
    None
}

/// Host-supplied callback kinds. The host installs these to customize the
/// engine's above-spec behavior. Three categories:
///
/// (1) Module-namespace augmentation — Doc 717 Tuple A/B closure point
///     (FinalizeModuleNamespace, installed in round 3.d.f).
///
/// (2) Event-loop integration — Doc 714 §VI Consequence 5: the host
///     supplies OS I/O multiplexing via PollIo, called by the engine at
///     idle (round 3.f.c). The host translates ready I/O events into
///     macrotasks enqueued on the engine's JobQueue.
///
/// (3) Built-in-module dispatch — Tier-Ω.5.b: the host returns a
///     namespace ObjectRef for `node:fs`, `node:path`, etc. The engine
///     consults this hook when resolve_module yields a built-in marker.
pub enum HostHook {
    /// Called between Link and Evaluate. Receives the module's exported
    /// namespace + the AST. The hook can mutate the namespace to add
    /// synthetic bindings (Tuple A: default = namespace; Tuple B: named
    /// exports synthesized from default's own properties).
    FinalizeModuleNamespace(Box<dyn Fn(&mut Runtime, &AstModule, ObjectRef) -> Result<(), RuntimeError>>),
    /// Called at run_to_completion's idle phase (phase 3) when both
    /// microtask and macrotask queues are empty. The host should:
    /// (a) consult its OS I/O multiplexer (mio Poll, libuv, io_uring,
    ///     etc.) for ready events,
    /// (b) translate each ready event into a macrotask enqueued via
    ///     rt.enqueue_macrotask(...),
    /// (c) return true if any work was enqueued (engine loops back to
    ///     phase 1); false if no work pending (engine exits cleanly).
    ///
    /// Default: no hook installed → engine exits at idle.
    PollIo(Box<dyn Fn(&mut Runtime) -> Result<bool, RuntimeError>>),
    /// Tier-Ω.5.b: resolve a `node:*` specifier to a namespace ObjectRef.
    /// The hook receives the specifier (e.g. `"node:fs"`) and returns:
    ///   - Ok(Some(namespace_ref)) → success, engine uses the namespace.
    ///   - Ok(None) → unknown built-in, engine emits a clear TypeError.
    ///   - Err(_) → resolution failure, surfaced as a TypeError.
    /// Installed by host-v2::install_bun_host to route `node:fs`,
    /// `node:path`, `node:os`, `node:process` to the pre-installed
    /// intrinsic objects.
    ResolveBuiltinModule(Box<dyn Fn(&mut Runtime, &str) -> Result<Option<ObjectRef>, RuntimeError>>),
}

#[derive(Default)]
pub struct HostHooks {
    pub finalize_namespace: Option<Box<dyn Fn(&mut Runtime, &AstModule, ObjectRef) -> Result<(), RuntimeError>>>,
    pub poll_io: Option<Box<dyn Fn(&mut Runtime) -> Result<bool, RuntimeError>>>,
    pub resolve_builtin: Option<Box<dyn Fn(&mut Runtime, &str) -> Result<Option<ObjectRef>, RuntimeError>>>,
}

impl Runtime {
    /// Install a host hook. Replaces any previously-installed hook of the
    /// same kind.
    pub fn install_host_hook(&mut self, hook: HostHook) {
        match hook {
            HostHook::FinalizeModuleNamespace(f) => {
                self.host_hooks.finalize_namespace = Some(f);
            }
            HostHook::PollIo(f) => {
                self.host_hooks.poll_io = Some(f);
            }
            HostHook::ResolveBuiltinModule(f) => {
                self.host_hooks.resolve_builtin = Some(f);
            }
        }
    }

    /// Tier-Ω.5.b: resolve a module specifier relative to a parent URL.
    /// Returns either a `file://` URL or a `node:*` built-in marker.
    ///
    /// This is the relative / built-in / absolute path. Bare specifier
    /// resolution requires `&mut self` (for the package.json cache) and
    /// lives on `resolve_module_full`. Callers that don't have a Runtime
    /// in hand can still use this for the non-bare paths.
    ///
    /// Algorithm:
    ///   1. `node:foo` → returned unchanged; caller dispatches via the
    ///      ResolveBuiltinModule host hook.
    ///   2. `./`, `../` → resolved relative to dirname(parent path).
    ///   3. `file://...` → already-absolute; probes the extension list.
    ///   4. Otherwise (bare specifier) → TypeError pointing callers at
    ///      `resolve_module_full`.
    pub fn resolve_module(parent_url: &str, specifier: &str) -> Result<String, RuntimeError> {
        if specifier.starts_with("node:") {
            return Ok(specifier.to_string());
        }
        if specifier.starts_with("./") || specifier.starts_with("../") {
            let parent_path = parent_url.strip_prefix("file://").ok_or_else(|| {
                RuntimeError::TypeError(format!(
                    "relative specifier '{}' requires a file:// parent URL (got '{}')",
                    specifier, parent_url
                ))
            })?;
            let parent = std::path::Path::new(parent_path);
            let parent_dir = parent.parent().unwrap_or_else(|| std::path::Path::new("/"));
            let candidate = parent_dir.join(specifier);
            return probe_with_extensions(&candidate, specifier);
        }
        if let Some(rest) = specifier.strip_prefix("file://") {
            let candidate = std::path::PathBuf::from(rest);
            return probe_with_extensions(&candidate, specifier);
        }
        Err(RuntimeError::TypeError(format!(
            "bare specifier '{}' requires resolve_module_full (caller did not thread the Runtime)",
            specifier
        )))
    }

    /// Tier-Ω.5.q: full resolver including bare-specifier node_modules
    /// walk-up and package.json `exports` / `main` / `module` / `index.js`
    /// resolution. Caches parsed package.json on the Runtime.
    ///
    /// Algorithm (locked):
    ///   1. `node:`, relative, `file://` → delegate to `resolve_module`.
    ///   2. Bare specifier `pkg[/subpath]`:
    ///      a. Split into pkg_name (one segment, or two for `@scope/pkg`)
    ///         + subpath.
    ///      b. Walk up from `dirname(parent_url)` looking for
    ///         `node_modules/<pkg_name>`. The first hit is the package
    ///         directory.
    ///      c. Read + parse `package.json` (cached on Runtime).
    ///      d. If subpath empty: try `exports."."` (conditional resolve
    ///         per importer_kind), else `module`, else `main`, else
    ///         probe `index.js` family in the package root.
    ///      e. If subpath `./X`: try `exports."./X"` then wildcard
    ///         patterns (`./fp/*` matching `./fp/get`), else probe the
    ///         literal subpath relative to the package root using the
    ///         extension-probe chain.
    ///   3. Anything else → TypeError naming the unresolvable specifier.
    pub fn resolve_module_full(
        &mut self,
        parent_url: &str,
        specifier: &str,
        importer_kind: ModuleKind,
    ) -> Result<String, RuntimeError> {
        // Tier-Ω.5.EEEEEEE: userland-polyfill aliases. Several historically-
        // important npm packages exist as userland reimplementations of
        // Node built-ins so that browser bundlers can substitute them.
        // Real runtimes (Bun, Deno's node compat layer) alias these to the
        // host's built-in equivalent rather than evaluating their source —
        // the source files are heavy (~6000 LOC for readable-stream) and
        // their module-init exercises legacy ES5-prototype patterns that
        // hit subtle correctness gaps. Aliasing closes a cluster of
        // packages whose only fault is depending on these polyfills.
        let aliased = match specifier {
            "readable-stream" | "readable-stream/duplex" | "readable-stream/readable"
            | "readable-stream/writable" | "readable-stream/transform"
            | "readable-stream/passthrough" => Some("node:stream"),
            "safe-buffer" => Some("node:buffer"),
            "stream-browserify" => Some("node:stream"),
            "buffer" if !specifier.starts_with("./") => Some("node:buffer"),
            "events" if !specifier.starts_with("./") => Some("node:events"),
            "util" if !specifier.starts_with("./") => Some("node:util"),
            _ => None,
        };
        if let Some(target) = aliased {
            return Runtime::resolve_module(parent_url, target);
        }
        // Non-bare paths reuse the existing logic.
        if specifier.starts_with("node:")
            || specifier.starts_with("./")
            || specifier.starts_with("../")
            || specifier.starts_with("file://")
        {
            return Runtime::resolve_module(parent_url, specifier);
        }
        // Tier-Ω.5.mm: package internal `imports` field (`#name`). chalk
        // and other packages use this for private subpath imports. Walk
        // up from the importing file looking for the package's
        // package.json with an `imports` map containing the key.
        if specifier.starts_with('#') {
            let parent_path_str = parent_url.strip_prefix("file://").unwrap_or(parent_url);
            let parent_path = std::path::Path::new(parent_path_str);
            let start_dir = parent_path.parent().unwrap_or_else(|| std::path::Path::new("/"));
            let mut cur: Option<&std::path::Path> = Some(start_dir);
            while let Some(d) = cur {
                let candidate = d.join("package.json");
                if candidate.is_file() {
                    let pkg = self.read_package_json(&candidate)?;
                    if let Some(imports) = pkg.raw.get("imports") {
                        if let Some(target) = imports.get(specifier) {
                            if let Some(rel) = resolve_exports_target(target, "", importer_kind) {
                                let pkg_dir = d;
                                let candidate_path = pkg_dir.join(strip_dot_slash(&rel));
                                return probe_with_extensions(&candidate_path, specifier);
                            }
                        }
                    }
                }
                cur = d.parent();
            }
            return Err(RuntimeError::TypeError(format!(
                "package-internal import '{}' not found in any enclosing package.json's `imports` field",
                specifier
            )));
        }
        // Bare specifier — extract pkg_name + subpath.
        let (pkg_name, subpath) = split_bare_specifier(specifier).ok_or_else(|| {
            RuntimeError::TypeError(format!(
                "bare specifier '{}' is malformed (empty or invalid scope/name)",
                specifier
            ))
        })?;

        // Determine the directory to start walking from.
        let parent_path_str = parent_url.strip_prefix("file://").unwrap_or(parent_url);
        let parent_path = std::path::Path::new(parent_path_str);
        let start_dir = if parent_path.is_dir() {
            parent_path.to_path_buf()
        } else {
            parent_path
                .parent()
                .unwrap_or_else(|| std::path::Path::new("/"))
                .to_path_buf()
        };

        // Walk up looking for node_modules/<pkg_name>.
        let pkg_dir = match walk_up_for_pkg(&start_dir, &pkg_name) {
            Some(d) => d,
            None => {
                // Tier-Ω.5.r: node-builtin synonym fallback. Real Node
                // treats a bare specifier matching a known builtin name
                // (e.g. `require("crypto")`, `require("tty")`) as a
                // synonym for `node:<name>`. The node_modules walk has
                // priority — a local `node_modules/crypto` would have
                // already won. We hit this branch only when the walk
                // failed AND the name is a builtin. Returning the
                // `node:` form sends the caller through the existing
                // resolve_builtin_namespace host-hook dispatch.
                if is_node_builtin(&pkg_name) && subpath.is_empty() {
                    return Ok(format!("node:{}", pkg_name));
                }
                // Tier-Ω.5.DDDDDDD: builtin with subpath. `fs/promises`,
                // `stream/web`, `stream/consumers`, `stream/promises`,
                // `dns/promises`, `timers/promises` — Node treats these as
                // sub-namespaces of the corresponding builtin. We route the
                // full `node:fs/promises` form to the builtin host-hook,
                // which maps it back to the `fs` namespace (lib.rs alias
                // table). Rimraf's `import { rm } from 'fs/promises'`
                // and similar consumer-side patterns hit this branch.
                if is_node_builtin(&pkg_name) && !subpath.is_empty() {
                    // subpath is in `./X` form per split_bare_specifier; strip
                    // the leading `./` to produce `node:fs/promises`-style
                    // specifiers that the host-hook alias table recognizes.
                    let tail = subpath.strip_prefix("./").unwrap_or(&subpath);
                    return Ok(format!("node:{}/{}", pkg_name, tail));
                }
                return Err(RuntimeError::TypeError(format!(
                    "bare specifier '{}' not found: walked up from '{}' looking for node_modules/{}",
                    specifier,
                    start_dir.display(),
                    pkg_name
                )));
            }
        };

        // Read + parse package.json (cached).
        let pkg_json_path = pkg_dir.join("package.json");
        let pkg = self.read_package_json(&pkg_json_path)?;

        // Resolve to a candidate path inside the package.
        let candidate = resolve_within_package(&pkg_dir, &pkg, &subpath, importer_kind)
            .ok_or_else(|| {
                RuntimeError::TypeError(format!(
                    "bare specifier '{}' resolved to package '{}' but no entry matched subpath '{}'",
                    specifier,
                    pkg_dir.display(),
                    subpath
                ))
            })?;

        // Tier-Ω.5.FFFFFFFF: JSON modules. The Ω.5.IIIIIII fix made
        // require('*.json') return the parsed data through the CJS path
        // and ESM-default through the namespace; the resolver was still
        // rejecting bare-specifier .json subpaths at this gate. Drop the
        // reject — load_module routes .json through evaluate_json_module.
        // autoprefixer / cssnano / postcss-preset-env all import
        // 'node-releases/data/processed/envs.json' as a bare specifier.
        if candidate.extension().and_then(|s| s.to_str()) == Some("json") {
            if candidate.is_file() {
                let canonical = std::fs::canonicalize(&candidate).map_err(|e| {
                    RuntimeError::TypeError(format!("canonicalize '{}': {}", candidate.display(), e))
                })?;
                return Ok(format!("file://{}", canonical.display()));
            }
        }

        probe_with_extensions(&candidate, specifier)
    }

    /// Tier-Ω.5.q: read+parse package.json with caching. Returns an Rc so
    /// repeated callers share the parsed view without cloning the map.
    pub fn read_package_json(
        &mut self,
        path: &std::path::Path,
    ) -> Result<Rc<ParsedPackageJson>, RuntimeError> {
        let key = path.to_path_buf();
        if let Some(p) = self.pkg_json_cache.get(&key) {
            return Ok(p.clone());
        }
        let text = std::fs::read_to_string(path).map_err(|e| {
            RuntimeError::TypeError(format!(
                "package.json read failed at '{}': {}",
                path.display(),
                e
            ))
        })?;
        let parsed = parse_package_json(&text).map_err(|e| {
            RuntimeError::TypeError(format!(
                "package.json parse failed at '{}': {}",
                path.display(),
                e
            ))
        })?;
        let rc = Rc::new(parsed);
        self.pkg_json_cache.insert(key, rc.clone());
        Ok(rc)
    }

    /// Tier-Ω.5.b: load + evaluate a module from disk, with caching and
    /// snapshot cycle handling. Returns the module's namespace ObjectRef.
    ///
    /// Cycle behavior: if the recursion encounters a module already in
    /// Linking status, it returns the existing (partial) namespace —
    /// fields populated by exports evaluated so far. Mutual cycles where
    /// each module reads the other's exports at top level may observe
    /// Undefined. Spec live-bindings are deferred.
    pub fn load_module(&mut self, url: &str) -> Result<ObjectRef, RuntimeError> {
        if let Some(rec) = self.modules.get(url) {
            let r = rec.borrow();
            if let Some(ns) = r.namespace {
                // Either fully Evaluated, or Linking with a partial ns.
                return Ok(ns);
            }
        }
        if let Some(stripped) = url.strip_prefix("file://") {
            let source = std::fs::read_to_string(stripped).map_err(|e| {
                RuntimeError::TypeError(format!("module load: cannot read '{}': {}", stripped, e))
            })?;
            // Tier-Ω.5.ss: JSON module imports per ESM spec / import-attributes
            // (`import data from "./x.json" with {type:"json"}`). The default
            // export is the parsed JSON value. cli-spinners (ora) depends on
            // this, as do many "data-as-module" packages.
            if stripped.ends_with(".json") {
                return self.evaluate_json_module(&source, url);
            }
            match detect_module_kind(url) {
                ModuleKind::ESM => self.evaluate_module(&source, url),
                ModuleKind::CJS => self.evaluate_cjs_module(&source, url),
            }
        } else {
            Err(RuntimeError::TypeError(format!(
                "load_module: unsupported URL scheme '{}'", url
            )))
        }
    }

    /// Tier-Ω.5.j.cjs: look up the CJS `module.exports` value for a
    /// previously-evaluated CJS module. Returns None for ESM/built-in
    /// records.
    pub fn cjs_exports_of(&self, url: &str) -> Option<Value> {
        self.modules.get(url).and_then(|r| r.borrow().cjs_exports.clone())
    }

    /// Tier-Ω.5.j.cjs: kind of a cached module record.
    pub fn module_kind_of(&self, url: &str) -> Option<ModuleKind> {
        self.modules.get(url).map(|r| r.borrow().kind)
    }

    /// Tier-Ω.5.b: dispatch a `node:*` specifier to the host's
    /// ResolveBuiltinModule hook. Caches the resulting namespace under
    /// the specifier so repeated imports yield identical handles.
    fn resolve_builtin_namespace(&mut self, specifier: &str) -> Result<ObjectRef, RuntimeError> {
        if let Some(rec) = self.modules.get(specifier) {
            if let Some(ns) = rec.borrow().namespace { return Ok(ns); }
        }
        let hook = self.host_hooks.resolve_builtin.take();
        let result = match &hook {
            Some(f) => f(self, specifier),
            None => Ok(None),
        };
        self.host_hooks.resolve_builtin = hook;
        let ns = match result? {
            Some(o) => o,
            None => return Err(RuntimeError::TypeError(format!(
                "unknown built-in module '{}' (no host hook installed or hook returned None)",
                specifier
            ))),
        };
        // Cache a synthetic record so repeated imports share the namespace.
        // We don't have an AST/bytecode for built-ins; store an empty
        // ModuleRecord with namespace set and status Evaluated. The Rc
        // is left orphan-callable from anywhere via self.modules.
        let empty_ast = Rc::new(AstModule {
            span: rusty_js_ast::Span::new(0, 0),
            body: Vec::new(),
            import_entries: Vec::new(),
            local_export_entries: Vec::new(),
            indirect_export_entries: Vec::new(),
            star_export_entries: Vec::new(),
        });
        let empty_bc = Rc::new(CompiledModule {
            bytecode: Vec::new(), constants: Default::default(),
            locals: Vec::new(), source_map: Vec::new(),
            imports: Vec::new(), exports: Vec::new(),
            reexport_sources: Vec::new(), side_effect_imports: Vec::new(),
        });
        self.modules.insert(specifier.to_string(), Rc::new(RefCell::new(ModuleRecord {
            url: specifier.to_string(), status: ModuleStatus::Evaluated,
            ast: empty_ast, bytecode: empty_bc, namespace: Some(ns),
            kind: ModuleKind::ESM, cjs_exports: None,
        })));
        Ok(ns)
    }

    /// Evaluate a module: parse + compile + resolve imports + run +
    /// build namespace + invoke HostFinalizeModuleNamespace. Returns
    /// the namespace ObjectRef per spec §16.2.1.10.
    pub fn evaluate_module(&mut self, source: &str, url: &str) -> Result<ObjectRef, RuntimeError> {
        // Parse + compile.
        let ast = rusty_js_parser::parse_module(source)
            .map_err(|e| RuntimeError::CompileError(format!("parse: {} @byte{} @url={}", e.message, e.span.start, url)))?;
        let ast_rc = Rc::new(ast);
        let bytecode = rusty_js_bytecode::compile_module(source)
            .map_err(|e| RuntimeError::CompileError(format!("compile: {}", e.message)))?;
        let bytecode_rc = Rc::new(bytecode);

        // Pre-allocate this module's namespace + insert a Linking entry
        // into the cache. The slot lets cyclic imports observe the
        // partial namespace; we mutate it in place as exports finalize.
        let namespace = self.alloc_object(Object::new_module_namespace());
        let record = Rc::new(RefCell::new(ModuleRecord {
            url: url.to_string(),
            status: ModuleStatus::Linking,
            ast: ast_rc.clone(),
            bytecode: bytecode_rc.clone(),
            namespace: Some(namespace),
            kind: ModuleKind::ESM,
            cjs_exports: None,
        }));
        self.modules.insert(url.to_string(), record.clone());

        // Tier-Ω.5.h: re-export source dependencies. Load each eagerly so
        // its namespace lives in the module cache by the time the
        // namespace-build phase runs. Build a per-module map from the
        // original specifier text (as it appeared in the source) to the
        // loaded namespace ObjectRef. The namespace-build phase consults
        // this map by specifier — not by resolved URL — because the
        // CompiledModule retains the original specifier strings.
        let mut reexport_namespaces: HashMap<String, ObjectRef> = HashMap::new();
        for spec in &bytecode_rc.reexport_sources {
            let resolved = self.resolve_module_full(url, spec, ModuleKind::ESM)?;
            let is_builtin = resolved.starts_with("node:");
            let ns = if is_builtin {
                self.resolve_builtin_namespace(&resolved)?
            } else {
                self.load_module(&resolved)?
            };
            reexport_namespaces.insert(spec.clone(), ns);
        }
        // Tier-Ω.5.IIIIIIII: evaluate side-effect ImportDeclarations per
        // ECMA-262 §16.2.1.5. Previously `import "X"` was a silent no-op
        // because the compiler tracked only bound imports.
        for spec in &bytecode_rc.side_effect_imports {
            let resolved = self.resolve_module_full(url, spec, ModuleKind::ESM)?;
            let _ns = if resolved.starts_with("node:") {
                self.resolve_builtin_namespace(&resolved)?
            } else {
                self.load_module(&resolved)?
            };
        }

        // Resolve every import to a value vector parallel to
        // bytecode.imports, then write into the frame's local slots
        // before running the body.
        let mut import_values: Vec<(u16, Value)> =
            Vec::with_capacity(bytecode_rc.imports.len());
        for ib in &bytecode_rc.imports {
            let resolved = self.resolve_module_full(url, &ib.module_request, ModuleKind::ESM)?;
            let is_builtin = resolved.starts_with("node:");
            let ns = if is_builtin {
                self.resolve_builtin_namespace(&resolved)?
            } else {
                self.load_module(&resolved)?
            };
            // Tier-Ω.5.j.cjs: if the resolved target is a CJS module,
            // import bindings read `module.exports` per Node interop —
            // default is the raw value, named is a property lookup on
            // the object form, namespace is a synthesized wrapper.
            let cjs_raw = if is_builtin { None } else { self.cjs_exports_of(&resolved) };
            let v = match (&ib.kind, &cjs_raw) {
                (ImportBindingKind::Default, Some(raw)) => raw.clone(),
                (ImportBindingKind::Namespace, Some(raw)) => {
                    Value::Object(self.cjs_namespace_view(raw.clone()))
                }
                (ImportBindingKind::Named(n), Some(raw)) => {
                    match raw {
                        // Tier-Ω.5.aaaaa: dispatch accessor getters when
                        // reading named imports from CJS modules. rxjs
                        // (and any module using __exportStar /
                        // __createBinding) installs its named exports as
                        // Object.defineProperty getters that proxy through
                        // an internal namespace. Without dispatch here,
                        // every such named import resolved to undefined
                        // despite Object.keys listing the name.
                        Value::Object(oid) => {
                            if let Some(getter) = self.find_getter(*oid, n) {
                                self.call_function(getter, Value::Object(*oid), Vec::new())?
                            } else {
                                self.object_get(*oid, n)
                            }
                        }
                        _ => return Err(RuntimeError::TypeError(format!(
                            "named import '{}' from CJS module '{}': module.exports is not an object",
                            n, resolved))),
                    }
                }
                (ImportBindingKind::Default, None) => {
                    // Built-ins follow Node's CJS-interop convention: the
                    // default import is the namespace object itself when
                    // no explicit `default` property exists. Pure-ESM disk
                    // modules require an explicit `export default ...`.
                    let d = self.object_get(ns, "default");
                    if is_builtin && matches!(d, Value::Undefined) {
                        Value::Object(ns)
                    } else { d }
                }
                (ImportBindingKind::Namespace, None) => Value::Object(ns),
                (ImportBindingKind::Named(n), None) => {
                    if let Some(getter) = self.find_getter(ns, n) {
                        self.call_function(getter, Value::Object(ns), Vec::new())?
                    } else {
                        self.object_get(ns, n)
                    }
                }
            };
            import_values.push((ib.slot, v));
        }

        // Tier-Ω.5.r: allocate the synthetic `import.meta` object for this
        // module. The shape is `{ url, dir }` per ECMA-262 §16.2.1.10 plus
        // the Bun-conventional `dir` extension (dirname of url with the
        // file:// prefix stripped). The compiler lowers `import.meta` to
        // Op::PushImportMeta which reads from the frame's import_meta slot.
        let meta_obj = self.alloc_object(Object::new_ordinary());
        self.object_set(meta_obj, "url".to_string(), Value::String(Rc::new(url.to_string())));
        let dir_str = {
            let path = url.strip_prefix("file://").unwrap_or(url);
            let p = std::path::Path::new(path);
            p.parent().map(|d| d.display().to_string()).unwrap_or_default()
        };
        self.object_set(meta_obj, "dir".to_string(), Value::String(Rc::new(dir_str)));

        // Build a module frame, pre-populate import slots, run body.
        let mut frame = Frame::new_module(&bytecode_rc);
        frame.import_meta = Some(meta_obj);
        for (slot, v) in &import_values {
            frame.write_local(*slot as usize, v.clone());
        }
        self.run_frame_module(&mut frame)?;
        // Tier-Ω.5.jjj: read locals through the cell promotion seam.
        // If a slot was promoted to an upvalue cell (because a nested
        // closure captured it), frame.locals[slot] now holds Undefined
        // and the live value lives in frame.local_cells[slot]. The
        // namespace builder previously read locals directly and saw
        // Undefined for every captured export. ufo + many ESM packages
        // with nested closures that capture top-level fn-decls failed
        // this way: shape probe passed (52 keys) but values were
        // Undefined for half the exports.
        let mut locals = frame.locals.clone();
        for (i, slot) in locals.iter_mut().enumerate() {
            if let Some(Some(cell)) = frame.local_cells.get(i) {
                *slot = cell.borrow().clone();
            }
        }

        // Populate the namespace from bytecode.exports. The compiler
        // records (exported, local-slot) pairs; the runtime reads each
        // slot and writes namespace[exported] = value. Undefined slots
        // (e.g. unresolved `export { name }` after a parser-skipped
        // declaration) become namespace[name] = Undefined.
        // Snapshot the source-module namespace property tables before the
        // mutation loop. The borrow checker disallows holding a &Object
        // across object_set, so we extract just the (key, value) pairs we
        // need for Star expansion. For Named / StarAs the read is a single
        // object_get / direct ObjectRef, which is cheap.
        for eb in &bytecode_rc.exports {
            match eb {
                ExportBinding::Local { exported, local } => {
                    let v = locals.get(*local as usize).cloned().unwrap_or(Value::Undefined);
                    self.object_set(namespace, exported.clone(), v);
                }
                ExportBinding::Named { exported, source_specifier, imported } => {
                    // Look up the source namespace loaded earlier. If the
                    // entry is missing the compiler/runtime are out of sync;
                    // treat the export as Undefined to stay closure-shaped.
                    let v = match reexport_namespaces.get(source_specifier) {
                        Some(src_ns) => self.object_get(*src_ns, imported),
                        None => Value::Undefined,
                    };
                    self.object_set(namespace, exported.clone(), v);
                }
                ExportBinding::Star { source_specifier } => {
                    // ECMA-262 §16.2.3.7: `export *` re-exports every name
                    // OTHER than `default`. Snapshot the source namespace's
                    // own properties, then copy each non-`default` entry.
                    let keys_values: Vec<(String, Value)> = match reexport_namespaces.get(source_specifier) {
                        Some(src_ns) => {
                            let o = self.obj(*src_ns);
                            o.properties
                                .iter()
                                .filter(|(k, _)| k.as_str() != "default")
                                .map(|(k, d)| (k.clone(), d.value.clone()))
                                .collect()
                        }
                        None => Vec::new(),
                    };
                    for (k, v) in keys_values {
                        self.object_set(namespace, k, v);
                    }
                }
                ExportBinding::StarAs { exported, source_specifier } => {
                    // `export * as ns from "..."` binds the source's whole
                    // namespace object under `exported`.
                    let v = match reexport_namespaces.get(source_specifier) {
                        Some(src_ns) => Value::Object(*src_ns),
                        None => Value::Undefined,
                    };
                    self.object_set(namespace, exported.clone(), v);
                }
            }
        }

        // Call HostFinalizeModuleNamespace if installed.
        if let Some(hook) = self.host_hooks.finalize_namespace.take() {
            hook(self, &ast_rc, namespace)?;
            self.host_hooks.finalize_namespace = Some(hook);
        }

        // Flip the record to Evaluated.
        record.borrow_mut().status = ModuleStatus::Evaluated;

        Ok(namespace)
    }

    /// Tier-Ω.5.j.cjs: evaluate a CJS module from source. Synthesizes a
    /// wrapper function via source-level concatenation, loads it as an
    /// ESM module whose default export is the wrapper, allocates
    /// `module`/`exports`, builds a per-module `require` NativeFn, and
    /// invokes the wrapper. The final `module.exports` becomes the
    /// module's "namespace" for both `require()` and ESM-importing-CJS
    /// callers.
    ///
    /// v1 deviation: line numbers in CJS parse/runtime errors are off by
    /// one line — the synthesized prefix adds one newline. Documented in
    /// pilots/rusty-js-runtime/trajectory.md row Ω.5.j.cjs.
    pub fn evaluate_cjs_module(&mut self, source: &str, url: &str) -> Result<ObjectRef, RuntimeError> {
        // Pre-allocate the placeholder namespace view so cyclic
        // require() observes *something* during evaluation. We'll
        // refresh it after the wrapper returns.
        let placeholder = self.alloc_object(Object::new_module_namespace());
        // Insert a Linking record up-front so a cyclic require returns
        // the partial exports instead of re-entering evaluation.
        let initial_exports_obj = self.alloc_object(Object::new_ordinary());
        let initial_exports = Value::Object(initial_exports_obj);
        let empty_ast = Rc::new(AstModule {
            span: rusty_js_ast::Span::new(0, 0),
            body: Vec::new(),
            import_entries: Vec::new(),
            local_export_entries: Vec::new(),
            indirect_export_entries: Vec::new(),
            star_export_entries: Vec::new(),
        });
        let empty_bc = Rc::new(CompiledModule {
            bytecode: Vec::new(), constants: Default::default(),
            locals: Vec::new(), source_map: Vec::new(),
            imports: Vec::new(), exports: Vec::new(),
            reexport_sources: Vec::new(), side_effect_imports: Vec::new(),
        });
        let record = Rc::new(RefCell::new(ModuleRecord {
            url: url.to_string(),
            status: ModuleStatus::Linking,
            ast: empty_ast,
            bytecode: empty_bc,
            namespace: Some(placeholder),
            kind: ModuleKind::CJS,
            cjs_exports: Some(initial_exports.clone()),
        }));
        self.modules.insert(url.to_string(), record.clone());

        // Tier-Ω.5.UUUUUUU: strip a leading hashbang line (`#!/usr/bin/env node`)
        // before synthesizing the CJS wrapper. The hashbang is valid only as
        // the very first characters of a source per ECMA §12.5 HashbangComment;
        // once we prepend the wrapper, the `#!` is no longer at offset 0 and
        // the lexer errors on `#` (invalid identifier). clack / nx / many CLI
        // packages have shebangs at the top of their entry .js files.
        let source_no_shebang = if source.starts_with("#!") {
            match source.find('\n') {
                Some(nl) => &source[nl + 1..],
                None => "",
            }
        } else {
            source
        };

        // Synthesize the wrapper. The CJS source goes inside a function
        // body whose `default` export is the function. We can then call
        // the function with the synthesized arguments.
        //
        // Note: the leading newline before <source> normalizes line
        // numbers to off-by-one regardless of source content.
        let wrapped = format!(
            "export default (function (exports, module, require, __filename, __dirname) {{\n{}\n}});\n",
            source_no_shebang
        );

        // Parse + compile the wrapper. Reuse the existing ESM pipeline.
        let ast = rusty_js_parser::parse_module(&wrapped)
            .map_err(|e| RuntimeError::CompileError(format!("parse (cjs wrapper): {} @byte{} @url={}", e.message, e.span.start, url)))?;
        let _ast_rc = Rc::new(ast);
        let bytecode = rusty_js_bytecode::compile_module(&wrapped)
            .map_err(|e| RuntimeError::CompileError(format!("compile (cjs wrapper): {}", e.message)))?;
        let bytecode_rc = Rc::new(bytecode);

        // Run the wrapper's outer module body. No imports/re-exports
        // are expected (the wrapper itself never uses them), so the
        // import-resolution and reexport-loading loops are noops here;
        // we skip them to keep the path linear.
        let mut frame = Frame::new_module(&bytecode_rc);
        self.run_frame_module(&mut frame)?;
        let locals = frame.locals.clone();

        // Find the wrapper function: it's the `default` local. The
        // compiler stored it under the "<module.default>" slot whose
        // index is recorded in the exports list.
        let wrapper_fn: Value = bytecode_rc.exports.iter().find_map(|eb| {
            if let rusty_js_bytecode::ExportBinding::Local { exported, local } = eb {
                if exported == "default" {
                    return locals.get(*local as usize).cloned();
                }
            }
            None
        }).unwrap_or(Value::Undefined);

        // Build synthesized __filename / __dirname.
        let (filename, dirname) = filename_dirname_from_url(url);
        let filename_v = Value::String(Rc::new(filename));
        let dirname_v = Value::String(Rc::new(dirname));

        // Build the per-module require NativeFn. Captures the URL.
        let require_url = url.to_string();
        let require_fn: crate::value::NativeFn = Rc::new(move |rt, args| {
            let spec = match args.first() {
                Some(Value::String(s)) => s.as_str().to_string(),
                _ => return Err(RuntimeError::TypeError(
                    "require: argument must be a string specifier".into())),
            };
            rt.cjs_require(&require_url, &spec)
        });
        let require_obj = Object {
            proto: None,
            extensible: true,
            properties: std::collections::HashMap::new(),
            internal_kind: crate::value::InternalKind::Function(
                crate::value::FunctionInternals {
                    name: "require".to_string(),
                    native: require_fn,
                }
            ),
        };
        let require_id = self.alloc_object(require_obj);
        // Tier-Ω.5.YYYYYYY: require.resolve(specifier) returns the resolved
        // module URL string (not the module's value). gulp-uglify / metro /
        // metro-config / many Node toolchains read require.resolve at
        // module-init for path lookups. Real Node resolves through Module's
        // _resolveFilename; for load-test purposes return a synthetic URL
        // pointing to a plausible resolved location.
        let require_resolve_url = url.to_string();
        let require_resolve_fn: crate::value::NativeFn = std::rc::Rc::new(move |rt, args| {
            let spec = match args.first() {
                Some(Value::String(s)) => s.as_str().to_string(),
                _ => return Err(RuntimeError::TypeError(
                    "require.resolve: argument must be a string specifier".into())),
            };
            // Best-effort: ask the resolver for the URL, fallback to spec.
            let resolved = rt.resolve_module_full(&require_resolve_url, &spec, ModuleKind::CJS)
                .unwrap_or(spec.clone());
            // Strip file:// prefix per Node's resolve-returns-filesystem-path convention.
            let path = resolved.strip_prefix("file://").unwrap_or(&resolved).to_string();
            Ok(Value::String(std::rc::Rc::new(path)))
        });
        let require_resolve_obj = Object {
            proto: None, extensible: true, properties: std::collections::HashMap::new(),
            internal_kind: crate::value::InternalKind::Function(
                crate::value::FunctionInternals {
                    name: "resolve".to_string(), native: require_resolve_fn,
                }
            ),
        };
        let require_resolve_id = self.alloc_object(require_resolve_obj);
        // .paths(spec) returns an array of candidate node_modules dirs.
        let require_paths_fn: crate::value::NativeFn = std::rc::Rc::new(|rt, _args| {
            let arr = rt.alloc_object(Object::new_array());
            rt.object_set(arr, "length".into(), Value::Number(0.0));
            Ok(Value::Object(arr))
        });
        let require_paths_obj = Object {
            proto: None, extensible: true, properties: std::collections::HashMap::new(),
            internal_kind: crate::value::InternalKind::Function(
                crate::value::FunctionInternals {
                    name: "paths".to_string(), native: require_paths_fn,
                }
            ),
        };
        let require_paths_id = self.alloc_object(require_paths_obj);
        self.object_set(require_resolve_id, "paths".into(), Value::Object(require_paths_id));
        self.object_set(require_id, "resolve".into(), Value::Object(require_resolve_id));
        // require.cache — empty module cache for compatibility probes.
        let require_cache = self.alloc_object(Object::new_ordinary());
        self.object_set(require_id, "cache".into(), Value::Object(require_cache));
        // require.extensions — legacy, returns empty object.
        let require_extensions = self.alloc_object(Object::new_ordinary());
        self.object_set(require_id, "extensions".into(), Value::Object(require_extensions));
        let require_v = Value::Object(require_id);

        // Build the `module` object: { exports: <initial_exports_obj> }.
        let module_id = self.alloc_object(Object::new_ordinary());
        self.object_set(module_id, "exports".to_string(), initial_exports.clone());
        let module_v = Value::Object(module_id);

        // Call the wrapper with the synthesized argument tuple.
        let _ = self.call_function(
            wrapper_fn,
            Value::Undefined,
            vec![
                initial_exports.clone(),
                module_v.clone(),
                require_v,
                filename_v,
                dirname_v,
            ],
        )?;

        // After the call, the canonical exports value is
        // `module.exports` (which may have been rebound).
        let final_exports = self.object_get(module_id, "exports");

        // Update the record + refresh the placeholder namespace view.
        {
            let mut r = record.borrow_mut();
            r.cjs_exports = Some(final_exports.clone());
            r.status = ModuleStatus::Evaluated;
        }

        // Refresh the placeholder namespace view in place so any cached
        // ObjectRef (e.g. an ESM importer holding `ns`) sees the final
        // exports shape.
        self.populate_cjs_namespace_view(placeholder, &final_exports);

        Ok(placeholder)
    }

    /// Tier-Ω.5.j.cjs: build a fresh namespace view ObjectRef from a
    /// CJS module.exports value. Used by ESM `import * as X from
    /// "./lib.cjs"` to satisfy the spec namespace shape.
    pub fn cjs_namespace_view(&mut self, exports: Value) -> ObjectRef {
        let ns = self.alloc_object(Object::new_module_namespace());
        self.populate_cjs_namespace_view(ns, &exports);
        ns
    }

    /// Tier-Ω.5.ss: JSON module — produce a namespace with a single
    /// `default` export holding the parsed value, mirroring Node's
    /// import-attributes JSON module semantics.
    pub fn evaluate_json_module(&mut self, source: &str, url: &str) -> Result<ObjectRef, RuntimeError> {
        let value = crate::intrinsics::json_parse(self, source).map_err(|e| {
            RuntimeError::CompileError(format!("parse (json module): {:?} @url={}", e, url))
        })?;
        let ns = self.alloc_object(Object::new_module_namespace());
        self.object_set(ns, "default".to_string(), value.clone());
        // Tier-Ω.5.IIIIIII: also mirror the parsed value's own properties
        // onto the namespace so `import { name } from "./x.json"` works
        // alongside the default-import. AND set cjs_exports = parsed value
        // so `require("./x.json")` returns the *data*, not the namespace
        // object (which would have only `default` as a key — statuses/
        // express's CJS read of statuses/codes.json was hitting exactly
        // this: Object.keys(codes) returned ['default'] instead of the
        // numeric status codes, then codes[code].toLowerCase() failed).
        if let Value::Object(oid) = &value {
            let pairs: Vec<(String, Value)> = self.obj(*oid).properties.iter()
                .map(|(k, d)| (k.clone(), d.value.clone())).collect();
            for (k, v) in pairs {
                self.object_set(ns, k, v);
            }
        }
        let empty_ast = Rc::new(AstModule {
            span: rusty_js_ast::Span::new(0, 0),
            body: Vec::new(),
            import_entries: Vec::new(),
            local_export_entries: Vec::new(),
            indirect_export_entries: Vec::new(),
            star_export_entries: Vec::new(),
        });
        let empty_bc = Rc::new(CompiledModule {
            bytecode: Vec::new(), constants: Default::default(),
            locals: Vec::new(), source_map: Vec::new(),
            imports: Vec::new(), exports: Vec::new(),
            reexport_sources: Vec::new(), side_effect_imports: Vec::new(),
        });
        self.modules.insert(url.to_string(), Rc::new(RefCell::new(ModuleRecord {
            url: url.to_string(), status: ModuleStatus::Evaluated,
            ast: empty_ast, bytecode: empty_bc, namespace: Some(ns),
            kind: ModuleKind::ESM, cjs_exports: Some(value),
        })));
        Ok(ns)
    }

    fn populate_cjs_namespace_view(&mut self, ns: ObjectRef, exports: &Value) {
        match exports {
            Value::Object(oid) => {
                // Mirror own properties + a `default` pointer at the
                // exports value itself.
                let pairs: Vec<(String, Value)> = self
                    .obj(*oid)
                    .properties
                    .iter()
                    .map(|(k, d)| (k.clone(), d.value.clone()))
                    .collect();
                for (k, v) in pairs {
                    self.object_set(ns, k, v);
                }
                self.object_set(ns, "default".to_string(), exports.clone());
            }
            _ => {
                // Non-object exports: only `default` is meaningful.
                self.object_set(ns, "default".to_string(), exports.clone());
            }
        }
    }

    /// Tier-Ω.5.j.cjs: implement `require(spec)` from a CJS module
    /// whose URL is `parent_url`. Resolution order:
    ///   1. Built-in (host hook). Tries the spec as-is, then `node:` +
    ///      spec if no prefix is present. Lets `require("fs")` work.
    ///   2. Relative `./` / `../` / absolute `file://` → existing
    ///      resolve_module pipeline.
    ///   3. Otherwise → TypeError (bare specifier; node_modules walk
    ///      deferred).
    pub fn cjs_require(&mut self, parent_url: &str, spec: &str) -> Result<Value, RuntimeError> {
        // (1) Built-in dispatch — direct or via node: prefix.
        let builtin_attempts: Vec<String> = if spec.starts_with("node:") {
            vec![spec.to_string()]
        } else if spec.starts_with("./") || spec.starts_with("../") || spec.starts_with("file://") {
            Vec::new()
        } else {
            vec![spec.to_string(), format!("node:{}", spec)]
        };
        for cand in &builtin_attempts {
            // Probe the hook by attempting resolution. Cache hits are
            // fine: resolve_builtin_namespace will reuse them.
            let probe = self.try_resolve_builtin(cand);
            if let Ok(Some(ns)) = probe {
                return Ok(Value::Object(ns));
            }
        }
        // (2) Disk resolution — relative, absolute, OR bare via Ω.5.q.
        let resolved = self.resolve_module_full(parent_url, spec, ModuleKind::CJS)?;
        // Tier-Ω.5.EEEEEEE: resolver may rewrite to `node:` (userland-polyfill
        // alias path); re-try the builtin dispatch on the rewritten target.
        if resolved.starts_with("node:") {
            if let Ok(Some(ns)) = self.try_resolve_builtin(&resolved) {
                return Ok(Value::Object(ns));
            }
        }
        // Cache check first.
        if let Some(rec) = self.modules.get(&resolved) {
            let r = rec.borrow();
            if let Some(raw) = r.cjs_exports.clone() {
                return Ok(raw);
            }
            if let Some(ns) = r.namespace {
                return Ok(Value::Object(ns));
            }
        }
        // Load. For CJS the return is module.exports; for ESM,
        // the namespace object.
        let ns = self.load_module(&resolved)?;
        match self.cjs_exports_of(&resolved) {
            Some(v) => Ok(v),
            None => Ok(Value::Object(ns)),
        }
    }

    /// Tier-Ω.5.j.cjs: side-effect-free probe of the
    /// ResolveBuiltinModule host hook. Returns Ok(Some(ns)) if a
    /// built-in matches; Ok(None) otherwise.
    fn try_resolve_builtin(&mut self, spec: &str) -> Result<Option<ObjectRef>, RuntimeError> {
        // Cache hit?
        if let Some(rec) = self.modules.get(spec) {
            if let Some(ns) = rec.borrow().namespace { return Ok(Some(ns)); }
        }
        let hook = self.host_hooks.resolve_builtin.take();
        let result = match &hook {
            Some(f) => f(self, spec),
            None => Ok(None),
        };
        self.host_hooks.resolve_builtin = hook;
        let ns = match result? {
            Some(o) => o,
            None => return Ok(None),
        };
        // Cache.
        let empty_ast = Rc::new(AstModule {
            span: rusty_js_ast::Span::new(0, 0),
            body: Vec::new(),
            import_entries: Vec::new(),
            local_export_entries: Vec::new(),
            indirect_export_entries: Vec::new(),
            star_export_entries: Vec::new(),
        });
        let empty_bc = Rc::new(CompiledModule {
            bytecode: Vec::new(), constants: Default::default(),
            locals: Vec::new(), source_map: Vec::new(),
            imports: Vec::new(), exports: Vec::new(),
            reexport_sources: Vec::new(), side_effect_imports: Vec::new(),
        });
        self.modules.insert(spec.to_string(), Rc::new(RefCell::new(ModuleRecord {
            url: spec.to_string(), status: ModuleStatus::Evaluated,
            ast: empty_ast, bytecode: empty_bc, namespace: Some(ns),
            kind: ModuleKind::ESM, cjs_exports: None,
        })));
        Ok(Some(ns))
    }

    /// Run a CompiledModule, returning the terminal stack value AND the
    /// frame's final local-slot table (for namespace construction).
    /// Retained for callers that bypass the disk-loader pipeline (tests +
    /// the run_module convenience).
    pub fn run_module_with_locals(
        &mut self,
        m: &CompiledModule,
    ) -> Result<(Value, Vec<Value>), RuntimeError> {
        let mut frame = crate::interp::Frame::new_module(m);
        let v = self.run_frame_module(&mut frame)?;
        Ok((v, frame.locals.clone()))
    }
}

impl Object {
    pub fn new_module_namespace() -> Self {
        Self {
            proto: None,
            extensible: false,
            properties: HashMap::new(),
            internal_kind: crate::value::InternalKind::ModuleNamespace,
        }
    }
}

/// Probe a candidate filesystem path against the v1 extension list:
///   {path}, {path}.mjs, {path}.js, {path}/index.mjs, {path}/index.js.
/// Returns the first hit as a canonical file:// URL.
fn probe_with_extensions(candidate: &std::path::Path, original: &str) -> Result<String, RuntimeError> {
    let attempts: Vec<std::path::PathBuf> = vec![
        candidate.to_path_buf(),
        with_suffix(candidate, ".mjs"),
        with_suffix(candidate, ".cjs"),
        with_suffix(candidate, ".js"),
        with_suffix(candidate, ".json"),
        candidate.join("index.mjs"),
        candidate.join("index.cjs"),
        candidate.join("index.js"),
        // Tier-Ω.5.AAAAAAAA: probe index.json as well per Node's CJS
        // resolution algorithm (require() of a directory with no main
        // falls back to index.js, then index.json). spdx-license-ids and
        // other data-only packages ship just index.json.
        candidate.join("index.json"),
    ];
    for p in &attempts {
        if p.is_file() {
            let canonical = std::fs::canonicalize(p).map_err(|e| {
                RuntimeError::TypeError(format!("canonicalize '{}': {}", p.display(), e))
            })?;
            return Ok(format!("file://{}", canonical.display()));
        }
    }
    Err(RuntimeError::TypeError(format!(
        "module not found: '{}' (tried {:?})",
        original,
        attempts.iter().map(|p| p.display().to_string()).collect::<Vec<_>>()
    )))
}

/// Append a literal suffix to a path's final component (no semantic
/// interpretation — distinct from `Path::with_extension`, which replaces
/// the existing extension and would convert `./util` → `./util.mjs`
/// correctly but `./util.foo` → `./util.mjs` (wrong). We want the
/// concat-after form per the locked design.
/// Tier-Ω.5.j.cjs: derive __filename + __dirname from a `file://` URL.
/// For URLs without the prefix, returns the URL itself as __filename
/// and an empty __dirname.
fn filename_dirname_from_url(url: &str) -> (String, String) {
    let path = url.strip_prefix("file://").unwrap_or(url);
    let p = std::path::Path::new(path);
    let dir = p.parent().map(|d| d.display().to_string()).unwrap_or_default();
    (path.to_string(), dir)
}

fn with_suffix(p: &std::path::Path, suffix: &str) -> std::path::PathBuf {
    let mut s = p.as_os_str().to_owned();
    s.push(suffix);
    std::path::PathBuf::from(s)
}

// ────────────────────────────────────────────────────────────────────────
// Tier-Ω.5.q: bare-specifier resolution helpers.
// ────────────────────────────────────────────────────────────────────────

/// Parsed view of a package.json sufficient for module resolution.
/// Fields are extracted lazily-but-once via serde_json::Value; we keep
/// the raw Value around so resolvers can walk the conditional-exports
/// tree without re-parsing.
pub struct ParsedPackageJson {
    pub raw: serde_json::Value,
    pub name: Option<String>,
    pub main: Option<String>,
    pub module_field: Option<String>,
    pub type_field: Option<String>,
}

fn parse_package_json(text: &str) -> Result<ParsedPackageJson, String> {
    let raw: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("JSON parse: {}", e))?;
    let name = raw.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
    let main = raw.get("main").and_then(|v| v.as_str()).map(|s| s.to_string());
    let module_field = raw.get("module").and_then(|v| v.as_str()).map(|s| s.to_string());
    let type_field = raw.get("type").and_then(|v| v.as_str()).map(|s| s.to_string());
    Ok(ParsedPackageJson { raw, name, main, module_field, type_field })
}

/// Tier-Ω.5.r: known Node builtin module names. Used by
/// resolve_module_full to treat a bare `require("crypto")` as a synonym
/// for `require("node:crypto")` when no node_modules/<name> exists.
/// The host hook decides whether each name actually has a stub — names
/// without a hook return Ok(None) and the caller surfaces a clean
/// "unknown built-in" error.
fn is_node_builtin(name: &str) -> bool {
    matches!(name,
        "assert" | "async_hooks" | "buffer" | "child_process" | "cluster"
        | "console" | "constants" | "crypto" | "dgram" | "diagnostics_channel"
        | "dns" | "domain" | "events" | "fs" | "http" | "http2" | "https"
        | "inspector" | "module" | "net" | "os" | "path" | "perf_hooks"
        | "process" | "punycode" | "querystring" | "readline" | "repl"
        | "stream" | "string_decoder" | "sys" | "timers" | "tls" | "trace_events"
        | "tty" | "url" | "util" | "v8" | "vm" | "wasi" | "worker_threads"
        | "zlib"
    )
}

/// Split a bare specifier into (pkg_name, subpath). Returns None on
/// empty / malformed input.
///
/// Examples:
///   "react"          → ("react", "")
///   "lodash/fp/get"  → ("lodash", "./fp/get")
///   "@org/pkg"       → ("@org/pkg", "")
///   "@org/pkg/sub"   → ("@org/pkg", "./sub")
fn split_bare_specifier(specifier: &str) -> Option<(String, String)> {
    if specifier.is_empty() { return None; }
    if specifier.starts_with('@') {
        let mut parts = specifier.splitn(3, '/');
        let scope = parts.next()?;
        let name = parts.next()?;
        if scope.len() < 2 || name.is_empty() { return None; }
        let pkg = format!("{}/{}", scope, name);
        let subpath = match parts.next() {
            Some(rest) if !rest.is_empty() => format!("./{}", rest),
            _ => String::new(),
        };
        Some((pkg, subpath))
    } else {
        let mut parts = specifier.splitn(2, '/');
        let name = parts.next()?;
        if name.is_empty() { return None; }
        let subpath = match parts.next() {
            Some(rest) if !rest.is_empty() => format!("./{}", rest),
            _ => String::new(),
        };
        Some((name.to_string(), subpath))
    }
}

/// Walk up from `start_dir` (inclusive) looking for
/// `node_modules/<pkg_name>` as a directory. Returns the package
/// directory if found.
fn walk_up_for_pkg(start_dir: &std::path::Path, pkg_name: &str) -> Option<std::path::PathBuf> {
    let mut cur: Option<&std::path::Path> = Some(start_dir);
    while let Some(d) = cur {
        let candidate = d.join("node_modules").join(pkg_name);
        if candidate.is_dir() {
            return Some(candidate);
        }
        cur = d.parent();
    }
    None
}

/// Resolve a subpath inside a package's directory using the
/// package.json fields. Returns the candidate path (without extension
/// probing — caller runs `probe_with_extensions`).
fn resolve_within_package(
    pkg_dir: &std::path::Path,
    pkg: &ParsedPackageJson,
    subpath: &str,
    importer_kind: ModuleKind,
) -> Option<std::path::PathBuf> {
    let exports = pkg.raw.get("exports");

    // ── Empty subpath: main entry ────────────────────────────────────
    if subpath.is_empty() {
        if let Some(exp) = exports {
            // exports may be a string, an array, or a map keyed by "."
            // or by conditions.
            // If it's a string/array, treat as the "." target.
            if exp.is_string() || exp.is_array() {
                if let Some(rel) = resolve_exports_target(exp, "", importer_kind) {
                    return Some(pkg_dir.join(strip_dot_slash(&rel)));
                }
            } else if let Some(map) = exp.as_object() {
                // If the map's keys look like subpaths (start with "."),
                // look up ".". Otherwise the map is a top-level
                // conditional-exports object for ".".
                let keys_are_subpath_style = map.keys().any(|k| k.starts_with('.'));
                if keys_are_subpath_style {
                    if let Some(target) = map.get(".") {
                        if let Some(rel) = resolve_exports_target(target, "", importer_kind) {
                            return Some(pkg_dir.join(strip_dot_slash(&rel)));
                        }
                    }
                } else if let Some(rel) = resolve_exports_target(exp, "", importer_kind) {
                    return Some(pkg_dir.join(strip_dot_slash(&rel)));
                }
            }
        }
        // ESM importer prefers `module` field for dual packages, CJS
        // prefers `main`. Either way fall back to `main` then index.
        if matches!(importer_kind, ModuleKind::ESM) {
            if let Some(m) = &pkg.module_field {
                return Some(pkg_dir.join(strip_dot_slash(m)));
            }
        }
        if let Some(m) = &pkg.main {
            return Some(pkg_dir.join(strip_dot_slash(m)));
        }
        return Some(pkg_dir.join("index"));
    }

    // ── Subpath import: subpath looks like "./X" ─────────────────────
    if let Some(exp) = exports {
        if let Some(map) = exp.as_object() {
            // Direct match.
            if let Some(target) = map.get(subpath) {
                if let Some(rel) = resolve_exports_target(target, "", importer_kind) {
                    return Some(pkg_dir.join(strip_dot_slash(&rel)));
                }
            }
            // Wildcard pattern match: find any key with a single '*'.
            for (k, v) in map.iter() {
                if let Some(star_pos) = k.find('*') {
                    let prefix = &k[..star_pos];
                    let suffix = &k[star_pos + 1..];
                    if subpath.starts_with(prefix) && subpath.ends_with(suffix)
                        && subpath.len() >= prefix.len() + suffix.len()
                    {
                        let captured = &subpath[prefix.len()..subpath.len() - suffix.len()];
                        if let Some(rel) = resolve_exports_target(v, captured, importer_kind) {
                            return Some(pkg_dir.join(strip_dot_slash(&rel)));
                        }
                    }
                }
            }
        }
    }
    // Fallback: literal subpath relative to package root.
    Some(pkg_dir.join(strip_dot_slash(subpath)))
}

/// Strip a leading "./" from a relative path so PathBuf::join treats
/// it as truly relative to the package directory.
fn strip_dot_slash(s: &str) -> &str {
    s.strip_prefix("./").unwrap_or(s)
}

/// Resolve a single `exports` target value against an optional
/// wildcard capture and the importer's module kind. Returns the
/// relative path (e.g. "./dist/index.js") or None if no condition
/// matched.
///
/// Conditions checked in priority order for ESM:  import, default.
/// For CJS:  require, default.
/// Conditions ignored:  browser, types, react-native, deno, worker.
fn resolve_exports_target(
    target: &serde_json::Value,
    capture: &str,
    importer_kind: ModuleKind,
) -> Option<String> {
    match target {
        serde_json::Value::String(s) => {
            Some(substitute_wildcard(s, capture))
        }
        serde_json::Value::Array(arr) => {
            // First entry that resolves wins (per Node spec).
            for item in arr {
                if let Some(r) = resolve_exports_target(item, capture, importer_kind) {
                    return Some(r);
                }
            }
            None
        }
        serde_json::Value::Object(map) => {
            // Conditional-exports map. Priority: importer-specific then
            // generic conditions.
            let priority: &[&str] = match importer_kind {
                ModuleKind::ESM => &["import", "node", "default"],
                ModuleKind::CJS => &["require", "node", "default"],
            };
            for cond in priority {
                if let Some(v) = map.get(*cond) {
                    if let Some(r) = resolve_exports_target(v, capture, importer_kind) {
                        return Some(r);
                    }
                }
            }
            None
        }
        serde_json::Value::Null => None,
        _ => None,
    }
}

/// Substitute the wildcard capture into a target path. The target's
/// single `*` (if any) is replaced with `capture`. Targets with no `*`
/// are returned verbatim.
fn substitute_wildcard(target: &str, capture: &str) -> String {
    if capture.is_empty() || !target.contains('*') {
        return target.to_string();
    }
    target.replacen('*', capture, 1)
}
