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
//! - Re-exports (`export { x } from "..."`, `export * from "..."`).

use crate::interp::{Frame, Runtime, RuntimeError};
use crate::value::{Object, ObjectRef, Value};
use rusty_js_ast::Module as AstModule;
use rusty_js_bytecode::{CompiledModule, ImportBindingKind};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleStatus { Unlinked, Linking, Linked, Evaluating, Evaluated, Failed }

pub struct ModuleRecord {
    pub url: String,
    pub status: ModuleStatus,
    pub ast: Rc<AstModule>,
    pub bytecode: Rc<CompiledModule>,
    pub namespace: Option<ObjectRef>,
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
            self.evaluate_module(&source, url)
        } else {
            Err(RuntimeError::TypeError(format!(
                "load_module: unsupported URL scheme '{}'", url
            )))
        }
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
        });
        self.modules.insert(specifier.to_string(), Rc::new(RefCell::new(ModuleRecord {
            url: specifier.to_string(), status: ModuleStatus::Evaluated,
            ast: empty_ast, bytecode: empty_bc, namespace: Some(ns),
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
        }));
        self.modules.insert(url.to_string(), record.clone());

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
            let v = match &ib.kind {
                ImportBindingKind::Default => {
                    // Built-ins follow Node's CJS-interop convention: the
                    // default import is the namespace object itself when
                    // no explicit `default` property exists. Pure-ESM disk
                    // modules require an explicit `export default ...`.
                    let d = self.object_get(ns, "default");
                    if is_builtin && matches!(d, Value::Undefined) {
                        Value::Object(ns)
                    } else { d }
                }
                ImportBindingKind::Namespace => Value::Object(ns),
                ImportBindingKind::Named(n) => self.object_get(ns, n),
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
        for eb in &bytecode_rc.exports {
            let v = locals.get(eb.local as usize).cloned().unwrap_or(Value::Undefined);
            self.object_set(namespace, eb.exported.clone(), v);
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
        with_suffix(candidate, ".js"),
        candidate.join("index.mjs"),
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
fn with_suffix(p: &std::path::Path, suffix: &str) -> std::path::PathBuf {
    let mut s = p.as_os_str().to_owned();
    s.push(suffix);
    std::path::PathBuf::from(s)
}
