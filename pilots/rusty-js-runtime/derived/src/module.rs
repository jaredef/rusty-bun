//! Module Record + linking phases + host hooks. Per
//! specs/rusty-js-runtime-design.md §VI–§VII.
//!
//! v1 round 3.d.f scope:
//! - ModuleRecord (Unlinked → Linking → Linked → Evaluating → Evaluated)
//! - HostFinalizeModuleNamespace hook between Link and Evaluate (Doc 717
//!   Tuple A/B closure point)
//! - SynthesizeNamedExportsFromDefault hook (Doc 717 Tuple B)
//! - evaluate_module(source, url) -> namespace ObjectRef
//!
//! Deferred to a follow-on:
//! - Resolve cross-module imports (requires multi-source loading)
//! - Star re-exports (`export *`)
//! - Cyclic module detection
//!
//! Single-module evaluation works end-to-end: parse + compile + run +
//! build namespace from local exports + call host hooks + return.

use crate::interp::{Runtime, RuntimeError};
use crate::value::{Object, ObjectRef, Value};
use rusty_js_ast::Module as AstModule;
use rusty_js_bytecode::CompiledModule;
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
/// engine's above-spec behavior. Two categories:
///
/// (1) Module-namespace augmentation — Doc 717 Tuple A/B closure point
///     (FinalizeModuleNamespace, installed in round 3.d.f).
///
/// (2) Event-loop integration — Doc 714 §VI Consequence 5: the host
///     supplies OS I/O multiplexing via PollIo, called by the engine at
///     idle (round 3.f.c). The host translates ready I/O events into
///     macrotasks enqueued on the engine's JobQueue.
pub enum HostHook {
    /// Called between Link and Evaluate. Receives the module's exported
    /// namespace + the AST. The hook can mutate the namespace to add
    /// synthetic bindings (Tuple A: default = namespace; Tuple B: named
    /// exports synthesized from default's own properties).
    FinalizeModuleNamespace(Box<dyn Fn(&mut Runtime, &AstModule, &ObjectRef) -> Result<(), RuntimeError>>),
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
}

#[derive(Default)]
pub struct HostHooks {
    pub finalize_namespace: Option<Box<dyn Fn(&mut Runtime, &AstModule, &ObjectRef) -> Result<(), RuntimeError>>>,
    pub poll_io: Option<Box<dyn Fn(&mut Runtime) -> Result<bool, RuntimeError>>>,
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
        }
    }

    /// Evaluate a module: parse + compile + run + build namespace +
    /// invoke HostFinalizeModuleNamespace. Returns the namespace
    /// ObjectRef per spec §16.2.1.10.
    ///
    /// v1 simplification: a module's exports come from the local-slot
    /// table; the namespace's properties are populated from the module's
    /// local_export_entries after evaluation. Indirect (re-export) and
    /// star (`export *`) re-exports are deferred to a follow-on.
    pub fn evaluate_module(&mut self, source: &str, url: &str) -> Result<ObjectRef, RuntimeError> {
        // Parse.
        let ast = rusty_js_parser::parse_module(source)
            .map_err(|e| RuntimeError::CompileError(format!("parse: {}", e.message)))?;
        let ast_rc = Rc::new(ast);
        // Compile.
        let bytecode = rusty_js_bytecode::compile_module(source)
            .map_err(|e| RuntimeError::CompileError(format!("compile: {}", e.message)))?;

        // Evaluate. Capture the post-execution local-slot values so we
        // can drive the namespace from named exports.
        let bytecode_rc = Rc::new(bytecode);
        let (_value, locals) = self.run_module_with_locals(&bytecode_rc)?;

        // Build the namespace from local_export_entries.
        let namespace = Rc::new(RefCell::new(Object::new_module_namespace()));
        for export in &ast_rc.local_export_entries {
            if let (Some(export_name), Some(local_name)) = (&export.export_name, &export.local_name) {
                // Find the slot whose LocalDescriptor.name matches the local_name.
                let slot = bytecode_rc.locals.iter().position(|d| &d.name == local_name);
                let value = match slot {
                    Some(i) => locals.get(i).cloned().unwrap_or(Value::Undefined),
                    None => {
                        // Special: *default* — for `export default <expr>` where
                        // the AST records local_name = "*default*". v1 does not
                        // yet thread the default-expression value into a local;
                        // future Round 3.d.g work.
                        Value::Undefined
                    }
                };
                namespace.borrow_mut().set_own(export_name.clone(), value);
            }
        }

        // Call HostFinalizeModuleNamespace if installed.
        if let Some(hook) = self.host_hooks.finalize_namespace.take() {
            hook(self, &ast_rc, &namespace)?;
            self.host_hooks.finalize_namespace = Some(hook);
        }

        Ok(namespace)
    }

    /// Run a CompiledModule, returning the terminal stack value AND the
    /// frame's final local-slot table (for namespace construction).
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
