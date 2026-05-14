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
            // .js or unknown — walk up to find the nearest package.json.
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

/// Minimal `"type"` field scan over package.json text. Returns
/// `"module"` or `"commonjs"` if found, else None. Avoids pulling a
/// JSON-parser dep into the runtime crate.
fn scan_package_type(text: &str) -> Option<String> {
    // Find `"type"` then the next quoted value.
    let key_pos = text.find("\"type\"")?;
    let after = &text[key_pos + 6..];
    let colon = after.find(':')?;
    let after = &after[colon + 1..];
    // Skip whitespace.
    let after = after.trim_start();
    if !after.starts_with('"') { return None; }
    let after = &after[1..];
    let end = after.find('"')?;
    Some(after[..end].to_string())
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
    /// Algorithm (locked by Tier-Ω.5.b design):
    ///   1. `node:foo` → returned unchanged; caller dispatches via the
    ///      ResolveBuiltinModule host hook.
    ///   2. `./`, `../` → resolved relative to dirname(parent path).
    ///      Probes the candidate in order: exact, +.mjs, +.js,
    ///      +/index.mjs, +/index.js. First existing file wins.
    ///   3. `file://...` → already-absolute; probes the same extension list.
    ///   4. Otherwise (bare specifier) → TypeError; node_modules walk is
    ///      deferred to a follow-on round.
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
            "bare specifier '{}' is not supported in v1 \
             (node_modules walk and package.json resolution are deferred)",
            specifier
        )))
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
            reexport_sources: Vec::new(),
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
            .map_err(|e| RuntimeError::CompileError(format!("parse: {}", e.message)))?;
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
            let resolved = Runtime::resolve_module(url, spec)?;
            let is_builtin = resolved.starts_with("node:");
            let ns = if is_builtin {
                self.resolve_builtin_namespace(&resolved)?
            } else {
                self.load_module(&resolved)?
            };
            reexport_namespaces.insert(spec.clone(), ns);
        }

        // Resolve every import to a value vector parallel to
        // bytecode.imports, then write into the frame's local slots
        // before running the body.
        let mut import_values: Vec<(u16, Value)> =
            Vec::with_capacity(bytecode_rc.imports.len());
        for ib in &bytecode_rc.imports {
            let resolved = Runtime::resolve_module(url, &ib.module_request)?;
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
                        Value::Object(oid) => self.object_get(*oid, n),
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
                (ImportBindingKind::Named(n), None) => self.object_get(ns, n),
            };
            import_values.push((ib.slot, v));
        }

        // Build a module frame, pre-populate import slots, run body.
        let mut frame = Frame::new_module(&bytecode_rc);
        for (slot, v) in &import_values {
            frame.write_local(*slot as usize, v.clone());
        }
        self.run_frame_module(&mut frame)?;
        let locals = frame.locals.clone();

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
            reexport_sources: Vec::new(),
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

        // Synthesize the wrapper. The CJS source goes inside a function
        // body whose `default` export is the function. We can then call
        // the function with the synthesized arguments.
        //
        // Note: the leading newline before <source> normalizes line
        // numbers to off-by-one regardless of source content.
        let wrapped = format!(
            "export default (function (exports, module, require, __filename, __dirname) {{\n{}\n}});\n",
            source
        );

        // Parse + compile the wrapper. Reuse the existing ESM pipeline.
        let ast = rusty_js_parser::parse_module(&wrapped)
            .map_err(|e| RuntimeError::CompileError(format!("parse (cjs wrapper): {}", e.message)))?;
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
        // (2) Disk resolution.
        if spec.starts_with("./") || spec.starts_with("../") || spec.starts_with("file://") {
            let resolved = Runtime::resolve_module(parent_url, spec)?;
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
        } else {
            Err(RuntimeError::TypeError(format!(
                "require('{}'): bare specifier resolution (node_modules) is not supported in v1",
                spec
            )))
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
            reexport_sources: Vec::new(),
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
        candidate.join("index.mjs"),
        candidate.join("index.cjs"),
        candidate.join("index.js"),
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
