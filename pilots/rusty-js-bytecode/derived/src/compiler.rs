//! Compiler from rusty_js_ast typed AST to bytecode. Per design spec §IV–§V.
//!
//! v1 (round 3.c.b): single-pass walk of expressions + minimal statement
//! support (ExpressionStatement + Return). Variable references compile to
//! LOAD_GLOBAL by default; local scope resolution + upvalue binding land in
//! round 3.c.c. Control-flow opcodes land in 3.c.c, function/closure in 3.c.d.

use crate::constants::{Constant, ConstantsPool};
use crate::op::*;
use rusty_js_ast::*;

#[derive(Debug, Clone)]
pub struct CompileError {
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct LocalDescriptor {
    pub name: String,
    pub kind: VariableKind,
    pub depth: u32,
}

#[derive(Debug, Clone)]
pub struct UpvalueDescriptor {
    pub source: UpvalueSource,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpvalueSource {
    Local(u16),
    Upvalue(u16),
}

#[derive(Debug, Clone)]
pub struct FunctionProto {
    pub bytecode: Vec<u8>,
    pub constants: ConstantsPool,
    pub params: u16,
    pub locals: Vec<LocalDescriptor>,
    pub upvalues: Vec<UpvalueDescriptor>,
    /// Tier-Ω.5.l: if the last parameter is a rest parameter (`...name`),
    /// this is its local slot index. The runtime collects all arguments
    /// from this index onward into a single Array bound to this slot.
    /// None for ordinary parameter lists.
    pub rest_param_slot: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct CompiledModule {
    pub bytecode: Vec<u8>,
    pub constants: ConstantsPool,
    pub locals: Vec<LocalDescriptor>,
    pub source_map: Vec<(usize, Span)>,
    /// Tier-Ω.5.b: ESM static imports. Each entry binds a local slot to a
    /// value drawn from another module's namespace. The runtime resolves
    /// `module_request` and populates `slot` BEFORE running the module body.
    pub imports: Vec<ImportBinding>,
    /// Tier-Ω.5.b: ESM static exports. After running the module body, the
    /// runtime reads each `local` slot and writes it to namespace[`exported`].
    /// `default` exports use the synthetic local "<module.default>".
    pub exports: Vec<ExportBinding>,
    /// Tier-Ω.5.h: ESM re-export source dependencies. Each entry is the
    /// `from "..."` specifier of an `export ... from "..."` form. The
    /// runtime loads these modules eagerly (like ImportDeclaration sources)
    /// so their namespaces are populated in the module cache before the
    /// namespace-build phase reads from them.
    pub reexport_sources: Vec<String>,
}

/// One ESM import binding. Compiled from ImportDeclaration entries.
#[derive(Debug, Clone)]
pub struct ImportBinding {
    /// Local-slot index this binding writes to.
    pub slot: u16,
    /// Specifier from `from "..."`. Either `node:*` or a relative path
    /// after Tier-Ω.5.b's resolver.
    pub module_request: String,
    /// What to read from the imported module's namespace.
    pub kind: ImportBindingKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportBindingKind {
    /// `import x from "..."` — read namespace["default"].
    Default,
    /// `import * as x from "..."` — bind the namespace object itself.
    Namespace,
    /// `import { name } from "..."` — read namespace[name].
    Named(String),
}

/// One ESM export binding. Compiled from ExportDeclaration entries.
///
/// Tier-Ω.5.h widened this from a single (exported, local) struct to a
/// four-variant enum to accommodate the four re-export forms. The
/// runtime's namespace-build phase iterates these and reads from either
/// the local-slot table (Local) or from a previously-loaded source
/// module's namespace (Named / Star / StarAs). Snapshot semantics: source
/// modules are loaded eagerly during evaluate_module's link phase so
/// their namespaces are populated by the time the namespace-build phase
/// runs. Cyclic re-exports may observe partial namespaces (v1 deviation
/// from spec live-bindings — see module.rs banner).
#[derive(Debug, Clone)]
pub enum ExportBinding {
    /// `export { x }` (no `from`) and `export default ...` — the namespace
    /// entry is populated from a local slot in this module's frame.
    Local {
        /// Name as it appears in the namespace.
        exported: String,
        /// Local-slot index whose value populates namespace[`exported`].
        local: u16,
    },
    /// `export { x } from "..."` or `export { x as y } from "..."`. The
    /// runtime reads `source_specifier`'s namespace at `imported`, writes
    /// the value to this module's namespace under `exported`. Either
    /// name may be `"default"` to express default<->named conversions.
    Named {
        exported: String,
        source_specifier: String,
        imported: String,
    },
    /// `export * from "..."`. The runtime iterates the source's namespace
    /// own properties and copies each (except `"default"` per spec
    /// §16.2.3.7) into this namespace under the same name.
    Star { source_specifier: String },
    /// `export * as ns from "..."`. The runtime writes the source's whole
    /// namespace object to this namespace under `exported`.
    StarAs { exported: String, source_specifier: String },
}

pub struct Compiler {
    bytecode: Vec<u8>,
    constants: ConstantsPool,
    locals: Vec<LocalDescriptor>,
    source_map: Vec<(usize, Span)>,
    /// Stack of loop frames. Each frame collects patch sites for break
    /// jumps and the bytecode offset of the loop's continue target.
    /// Push on loop entry, pop on loop exit.
    loop_stack: Vec<LoopFrame>,
    /// Tier-Ω.5.o: frames for LabelledStatement wrapping non-loop bodies
    /// (e.g. `outer: { ... break outer; }`). Loop labels live on the
    /// LoopFrame's `label` field instead.
    label_stack: Vec<LabelFrame>,
    /// Tier-Ω.5.o: pending label name to attach to the next pushed
    /// LoopFrame. Set by compile_stmt(Stmt::Labelled { body: <loop> })
    /// and cleared at frame-push by the loop's compile site.
    pending_label: Option<String>,
    /// Tier-Ω.5.c: each enclosing-function level's locals + accumulated
    /// upvalues, walked when resolving identifiers inside nested functions.
    /// Innermost outer is at the back. Empty at the top-level module.
    enclosing: Vec<EnclosingFrame>,
    /// This proto's own upvalue descriptors (only meaningful when this
    /// Compiler is compiling a nested function, i.e. enclosing.is_empty()
    /// is false).
    upvalues: Vec<UpvalueDescriptor>,
    /// Tier-Ω.5.f: class lowering context. Pushed when entering a class
    /// constructor / instance method / static method body. Read by
    /// Expr::Super and super(...) / super.method() lowerings to resolve
    /// the synthetic hidden bindings (`<super.ctor>` / `<super.proto>`)
    /// allocated by the class-emission site.
    class_stack: Vec<ClassFrame>,
    /// Counter for synthesizing unique local names across nested classes.
    class_seq: u32,
    /// Tier-Ω.5.b: ESM import bindings collected from the module's
    /// ImportDeclarations. Each binding allocates a local slot at the
    /// pre-body lowering step; references to the local name resolve to
    /// that slot via resolve_local. The runtime populates these slots
    /// from the imported module's namespace before run_frame_module.
    imports: Vec<ImportBinding>,
    /// Tier-Ω.5.b: ESM export bindings populated as ExportDeclarations are
    /// lowered. Filled lazily at compile time (Named export specifiers
    /// resolve their `local` -> slot at end-of-module).
    exports: Vec<ExportBinding>,
    /// Tier-Ω.5.h: re-export source dependencies (`from "..."` specifiers).
    reexport_sources: Vec<String>,
    /// Tier-Ω.5.b: snapshot of named local-or-default exports seen so far,
    /// pending slot lookup. For `export { name }` the slot is the local
    /// previously declared by `const name = ...` / `function name() {}`.
    /// Resolved at the end of compile_module.
    pending_named_exports: Vec<(String, String)>, // (exported, local_name)
}

#[derive(Debug, Clone)]
struct ClassFrame {
    /// Synthetic outer-local name holding the parent constructor (None
    /// when the class has no `extends` clause — super-references are a
    /// compile-time error in that case).
    super_ctor_name: Option<String>,
    /// Synthetic outer-local name holding the parent prototype.
    super_proto_name: Option<String>,
    /// True inside the constructor body (only place where bare `super(...)`
    /// is valid). False inside instance / static methods.
    in_constructor: bool,
    /// True for static methods — bare `super(...)` not allowed; super.x
    /// resolves to the parent constructor, not the parent prototype.
    is_static: bool,
}

#[derive(Debug, Clone)]
struct EnclosingFrame {
    locals: Vec<LocalDescriptor>,
    /// Upvalues that this enclosing frame itself captured. Needed when an
    /// inner function references a name owned by an even-outer level — the
    /// intermediate frames each get a transitive upvalue.
    upvalues: Vec<UpvalueDescriptor>,
}

fn emit_captures(buf: &mut Vec<u8>, captures: &[UpvalueDescriptor]) {
    for u in captures {
        match u.source {
            UpvalueSource::Local(slot) => {
                encode_op(buf, Op::CaptureLocal);
                encode_u16(buf, slot);
            }
            UpvalueSource::Upvalue(idx) => {
                encode_op(buf, Op::CaptureUpvalue);
                encode_u16(buf, idx);
            }
        }
    }
}

fn add_upvalue_to(table: &mut Vec<UpvalueDescriptor>, src: UpvalueSource, name: String) -> u16 {
    if let Some(i) = table.iter().position(|u| u.source == src) {
        return i as u16;
    }
    let idx = table.len() as u16;
    table.push(UpvalueDescriptor { source: src, name });
    idx
}

#[derive(Debug)]
struct LoopFrame {
    /// Bytecode offset where `continue` should jump to. For while / do-while
    /// this is fixed up front. For C-style for, the target is the update
    /// position which isn't known when the body compiles — `continue` then
    /// records a patch site in continue_patches instead of emitting a
    /// back-jump immediately.
    continue_target: usize,
    /// True while continue_target is provisional; `continue` records a
    /// patch site instead of emitting a known back-jump.
    continue_pending: bool,
    /// Operand-byte offsets of unresolved continue forward-jumps.
    continue_patches: Vec<usize>,
    /// Operand-byte offsets of unresolved break forward-jumps.
    break_patches: Vec<usize>,
    /// Tier-Ω.5.m: true for switch frames. `break` still targets this
    /// frame, but `continue` skips past it to the enclosing loop —
    /// switch is a break-only construct per ECMA-262 §14.12.4.
    is_switch: bool,
    /// Tier-Ω.5.o: label name attached to this frame by an enclosing
    /// LabelledStatement. `break LABEL` / `continue LABEL` match the
    /// innermost frame with this label. None for unlabelled loops.
    label: Option<String>,
}

/// Tier-Ω.5.o: frame for a LabelledStatement wrapping a non-loop body.
/// Only `break LABEL` targets it; `continue LABEL` matches loop frames.
#[derive(Debug)]
struct LabelFrame {
    label: String,
    break_patches: Vec<usize>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            bytecode: Vec::new(),
            constants: ConstantsPool::new(),
            locals: Vec::new(),
            source_map: Vec::new(),
            loop_stack: Vec::new(), label_stack: Vec::new(), pending_label: None,
            enclosing: Vec::new(),
            upvalues: Vec::new(),
            class_stack: Vec::new(),
            class_seq: 0,
            imports: Vec::new(),
            exports: Vec::new(),
            reexport_sources: Vec::new(),
            pending_named_exports: Vec::new(),
        }
    }

    pub fn compile_module(&mut self, m: &Module) -> Result<CompiledModule, CompileError> {
        // Tier-Ω.5.b phase A: pre-allocate locals for every import binding
        // so references to imported names in the body resolve to LoadLocal
        // (not LoadGlobal). The runtime populates these slots before
        // run_frame_module by reading from each module-request's namespace.
        for item in &m.body {
            if let ModuleItem::Import(imp) = item {
                let module_request = imp.specifier.value.clone();
                if let Some(def) = &imp.default_binding {
                    let slot = self.alloc_local(LocalDescriptor {
                        name: def.name.clone(), kind: VariableKind::Const, depth: 0,
                    });
                    self.imports.push(ImportBinding {
                        slot, module_request: module_request.clone(),
                        kind: ImportBindingKind::Default,
                    });
                }
                if let Some(ns) = &imp.namespace_binding {
                    let slot = self.alloc_local(LocalDescriptor {
                        name: ns.name.clone(), kind: VariableKind::Const, depth: 0,
                    });
                    self.imports.push(ImportBinding {
                        slot, module_request: module_request.clone(),
                        kind: ImportBindingKind::Namespace,
                    });
                }
                for spec in &imp.named_imports {
                    let imported_name = match &spec.imported {
                        ModuleExportName::Ident(b) => b.name.clone(),
                        ModuleExportName::String { value, .. } => value.clone(),
                    };
                    let slot = self.alloc_local(LocalDescriptor {
                        name: spec.local.name.clone(), kind: VariableKind::Const, depth: 0,
                    });
                    self.imports.push(ImportBinding {
                        slot, module_request: module_request.clone(),
                        kind: ImportBindingKind::Named(imported_name),
                    });
                }
            }
        }

        // Phase B: walk the body in order. Imports already lowered.
        // Statements compile normally. Exports are recorded for phase C
        // (default-export expressions are lowered inline into a synthetic
        // "<module.default>" local).
        for item in &m.body {
            match item {
                ModuleItem::Import(_) => { /* lowered in phase A */ }
                ModuleItem::Statement(s) => self.compile_stmt(s)?,
                ModuleItem::Export(e) => self.compile_export(e)?,
            }
        }

        // Phase C: resolve pending named-export specifiers to slot indices.
        // For `export { name }` after a local declaration, the slot is the
        // local previously bound by the declaration.
        for (exported, local_name) in std::mem::take(&mut self.pending_named_exports) {
            if let Some(slot) = self.resolve_local(&local_name) {
                self.exports.push(ExportBinding::Local { exported, local: slot });
            }
            // Silently drop unresolved names; the namespace builder yields
            // Undefined for missing exports.
        }

        encode_op(&mut self.bytecode, Op::ReturnUndef);
        Ok(CompiledModule {
            bytecode: std::mem::take(&mut self.bytecode),
            constants: std::mem::take(&mut self.constants),
            locals: std::mem::take(&mut self.locals),
            source_map: std::mem::take(&mut self.source_map),
            imports: std::mem::take(&mut self.imports),
            exports: std::mem::take(&mut self.exports),
            reexport_sources: std::mem::take(&mut self.reexport_sources),
        })
    }

    /// Tier-Ω.5.b: lower one ExportDeclaration. Named local exports are
    /// recorded for end-of-module slot resolution. Default exports lower
    /// the underlying expression / hoistable-function / class to bytecode
    /// and store the result in the synthetic "<module.default>" local.
    /// Re-export forms (StarFrom / StarAsFrom / Named-with-source) are
    /// deferred to a follow-on round per the scope ceiling.
    fn compile_export(&mut self, e: &ExportDeclaration) -> Result<(), CompileError> {
        match e {
            ExportDeclaration::Named { specifiers, source: None, .. } => {
                for spec in specifiers {
                    let local_name = match &spec.local {
                        ModuleExportName::Ident(b) => b.name.clone(),
                        ModuleExportName::String { value, .. } => value.clone(),
                    };
                    let exported_name = match &spec.exported {
                        ModuleExportName::Ident(b) => b.name.clone(),
                        ModuleExportName::String { value, .. } => value.clone(),
                    };
                    self.pending_named_exports.push((exported_name, local_name));
                }
            }
            // Tier-Ω.5.h: re-export forms. Each records its source-module
            // specifier in `reexport_sources` so the runtime loads the
            // dependency eagerly, then emits one or more ExportBinding
            // entries that the namespace-build phase resolves against the
            // source module's namespace.
            ExportDeclaration::Named { source: Some(src), specifiers, .. } => {
                let source_specifier = src.value.clone();
                if !self.reexport_sources.iter().any(|s| s == &source_specifier) {
                    self.reexport_sources.push(source_specifier.clone());
                }
                for spec in specifiers {
                    let imported = match &spec.local {
                        ModuleExportName::Ident(b) => b.name.clone(),
                        ModuleExportName::String { value, .. } => value.clone(),
                    };
                    let exported = match &spec.exported {
                        ModuleExportName::Ident(b) => b.name.clone(),
                        ModuleExportName::String { value, .. } => value.clone(),
                    };
                    self.exports.push(ExportBinding::Named {
                        exported, source_specifier: source_specifier.clone(), imported,
                    });
                }
            }
            ExportDeclaration::StarFrom { source, .. } => {
                let source_specifier = source.value.clone();
                if !self.reexport_sources.iter().any(|s| s == &source_specifier) {
                    self.reexport_sources.push(source_specifier.clone());
                }
                self.exports.push(ExportBinding::Star { source_specifier });
            }
            ExportDeclaration::StarAsFrom { source, exported, .. } => {
                let source_specifier = source.value.clone();
                if !self.reexport_sources.iter().any(|s| s == &source_specifier) {
                    self.reexport_sources.push(source_specifier.clone());
                }
                let exported_name = match exported {
                    ModuleExportName::Ident(b) => b.name.clone(),
                    ModuleExportName::String { value, .. } => value.clone(),
                };
                self.exports.push(ExportBinding::StarAs {
                    exported: exported_name, source_specifier,
                });
            }
            ExportDeclaration::Default { body, span } => {
                // Synthesize a local slot for the default binding. Reuse
                // across modules with multiple defaults isn't legal ECMAScript,
                // but we accept duplicate slot allocation (the last write wins).
                let slot = self.alloc_local(LocalDescriptor {
                    name: "<module.default>".to_string(),
                    kind: VariableKind::Const, depth: 0,
                });
                match body {
                    DefaultExportBody::Expression { expr } => {
                        self.compile_expr(expr)?;
                    }
                    DefaultExportBody::HoistableFunction { name, is_async, is_generator, params, body } => {
                        let proto = self.compile_function_proto(name.clone(), *is_async, *is_generator, params, body)?;
                        let captures = proto.upvalues.clone();
                        let idx = self.constants.intern(Constant::Function(Box::new(proto)));
                        encode_op(&mut self.bytecode, Op::MakeClosure);
                        encode_u16(&mut self.bytecode, idx);
                        emit_captures(&mut self.bytecode, &captures);
                    }
                    DefaultExportBody::Class { .. } => {
                        return Err(self.err(*span, "export default class not yet supported"));
                    }
                }
                encode_op(&mut self.bytecode, Op::StoreLocal);
                encode_u16(&mut self.bytecode, slot);
                self.exports.push(ExportBinding::Local {
                    exported: "default".to_string(), local: slot,
                });
            }
            ExportDeclaration::Declaration { names, .. } => {
                // The decl's body was already consumed by the parser, but
                // its names are recorded. Since the parser currently skips
                // the decl's bytecode (only its declarator names are kept),
                // these slots may not exist. Try to resolve; silently drop
                // unresolved entries — they manifest as Undefined in the
                // namespace, which mirrors the parser's body-opaque mode.
                for n in names {
                    self.pending_named_exports.push((n.name.clone(), n.name.clone()));
                }
            }
        }
        Ok(())
    }

    fn compile_stmt(&mut self, s: &Stmt) -> Result<(), CompileError> {
        let span = s.span();
        self.record_span(span);
        match s {
            Stmt::Expression { expr, .. } => {
                self.compile_expr(expr)?;
                encode_op(&mut self.bytecode, Op::Pop);
            }
            Stmt::Return { argument, .. } => {
                if let Some(e) = argument {
                    self.compile_expr(e)?;
                    encode_op(&mut self.bytecode, Op::Return);
                } else {
                    encode_op(&mut self.bytecode, Op::ReturnUndef);
                }
            }
            Stmt::Empty { .. } => {}
            Stmt::Block { body, .. } => {
                for s in body { self.compile_stmt(s)?; }
            }
            Stmt::Variable(v) => {
                for d in &v.declarators {
                    match &d.target {
                        rusty_js_ast::BindingPattern::Identifier(id) => {
                            // Allocate a local slot for the binding.
                            let slot = self.alloc_local(LocalDescriptor {
                                name: id.name.clone(),
                                kind: v.kind,
                                depth: 0,
                            });
                            if let Some(init) = &d.init {
                                self.compile_expr(init)?;
                            } else {
                                encode_op(&mut self.bytecode, Op::PushUndef);
                            }
                            encode_op(&mut self.bytecode, Op::StoreLocal);
                            encode_u16(&mut self.bytecode, slot);
                        }
                        pat @ (rusty_js_ast::BindingPattern::Array(_)
                              | rusty_js_ast::BindingPattern::Object(_)) => {
                            // Tier-Ω.5.g.3: destructure declarator. Evaluate
                            // init into a hidden source slot, allocate every
                            // bound name as a local under v.kind, then walk
                            // the pattern.
                            for id in pat.collect_names() {
                                self.alloc_local(LocalDescriptor {
                                    name: id.name.clone(),
                                    kind: v.kind,
                                    depth: 0,
                                });
                            }
                            let src_slot = self.alloc_temp("<destr.src>");
                            if let Some(init) = &d.init {
                                self.compile_expr(init)?;
                            } else {
                                encode_op(&mut self.bytecode, Op::PushUndef);
                            }
                            encode_op(&mut self.bytecode, Op::StoreLocal);
                            encode_u16(&mut self.bytecode, src_slot);
                            self.emit_destructure(pat, src_slot)?;
                        }
                    }
                }
            }
            Stmt::Throw { argument, .. } => {
                self.compile_expr(argument)?;
                encode_op(&mut self.bytecode, Op::Throw);
            }
            Stmt::Debugger { .. } => {
                encode_op(&mut self.bytecode, Op::Debugger);
            }
            Stmt::If { test, consequent, alternate, .. } => {
                self.compile_expr(test)?;
                let jump_if_false = self.emit_jump(Op::JumpIfFalse);
                self.compile_stmt(consequent)?;
                if let Some(alt) = alternate {
                    let jump_end = self.emit_jump(Op::Jump);
                    self.patch_jump(jump_if_false);
                    self.compile_stmt(alt)?;
                    self.patch_jump(jump_end);
                } else {
                    self.patch_jump(jump_if_false);
                }
            }
            Stmt::While { test, body, .. } => {
                let loop_start = self.bytecode.len();
                self.loop_stack.push(LoopFrame {
                    continue_target: loop_start, continue_pending: false,
                    continue_patches: Vec::new(), break_patches: Vec::new(), is_switch: false, label: self.pending_label.take(),
                });
                self.compile_expr(test)?;
                let jump_if_false = self.emit_jump(Op::JumpIfFalse);
                self.compile_stmt(body)?;
                self.emit_back_jump(loop_start);
                self.patch_jump(jump_if_false);
                let frame = self.loop_stack.pop().unwrap();
                for site in frame.break_patches { self.patch_jump_at(site); }
            }
            Stmt::DoWhile { body, test, .. } => {
                let loop_start = self.bytecode.len();
                self.loop_stack.push(LoopFrame {
                    continue_target: 0, continue_pending: true,
                    continue_patches: Vec::new(), break_patches: Vec::new(), is_switch: false, label: self.pending_label.take(),
                });
                self.compile_stmt(body)?;
                let test_pos = self.bytecode.len();
                // Finalize continue target to test_pos and patch any
                // pending continue sites.
                {
                    let frame = self.loop_stack.last_mut().unwrap();
                    frame.continue_target = test_pos;
                    frame.continue_pending = false;
                }
                let patches = std::mem::take(&mut self.loop_stack.last_mut().unwrap().continue_patches);
                for site in patches { self.patch_jump_at(site); }
                self.compile_expr(test)?;
                let jump_back = self.emit_jump(Op::JumpIfTrue);
                self.patch_jump_to(jump_back, loop_start);
                let frame = self.loop_stack.pop().unwrap();
                for site in frame.break_patches { self.patch_jump_at(site); }
            }
            Stmt::For { init, test, update, body, .. } => {
                if let Some(init) = init {
                    match init {
                        ForInit::Variable(v) => self.compile_stmt(&Stmt::Variable(v.clone()))?,
                        ForInit::Expression(e) => {
                            self.compile_expr(e)?;
                            encode_op(&mut self.bytecode, Op::Pop);
                        }
                    }
                }
                let test_pos = self.bytecode.len();
                self.loop_stack.push(LoopFrame {
                    continue_target: 0, continue_pending: true,
                    continue_patches: Vec::new(), break_patches: Vec::new(), is_switch: false, label: self.pending_label.take(),
                });
                let jump_if_false = if let Some(t) = test {
                    self.compile_expr(t)?;
                    Some(self.emit_jump(Op::JumpIfFalse))
                } else { None };
                self.compile_stmt(body)?;
                let update_pos = self.bytecode.len();
                // Finalize continue target and patch pending continue sites.
                {
                    let frame = self.loop_stack.last_mut().unwrap();
                    frame.continue_target = update_pos;
                    frame.continue_pending = false;
                }
                let patches = std::mem::take(&mut self.loop_stack.last_mut().unwrap().continue_patches);
                for site in patches { self.patch_jump_at(site); }
                if let Some(u) = update {
                    self.compile_expr(u)?;
                    encode_op(&mut self.bytecode, Op::Pop);
                }
                self.emit_back_jump(test_pos);
                if let Some(j) = jump_if_false { self.patch_jump(j); }
                let frame = self.loop_stack.pop().unwrap();
                for site in frame.break_patches { self.patch_jump_at(site); }
            }
            Stmt::ForOf { left, right, body, await_, .. } => {
                if *await_ {
                    return Err(self.err(span, "for-await-of not yet supported"));
                }
                // Allocate hidden slot for the iterator and a binding slot
                // for the loop variable.
                let iter_slot = self.alloc_local(LocalDescriptor {
                    name: "<iter>".into(), kind: VariableKind::Let, depth: 0,
                });
                // Per ECMA-262 §14.7.5.5, `let`/`const` heads receive a fresh
                // binding per iteration; `var` heads remain function-scoped
                // and share a single slot across iterations. We track this
                // with `per_iter_fresh`: when true, emit Op::ResetLocalCell at
                // iteration entry so closures captured in iteration N keep
                // their handle to that iteration's cell. Tier-Ω.5.g.1.
                // Returns (slot_to_store_value_into, destructure_pattern_or_none, per_iter_fresh).
                // When destructure_pattern is Some, the body prologue will
                // run the pattern lowering using slot_to_store_value_into
                // as the hidden source.
                let (bind_slot, destr_pat, per_iter_fresh): (u16, Option<rusty_js_ast::BindingPattern>, bool) = match left {
                    rusty_js_ast::ForBinding::Decl { kind, target, .. } => {
                        match target {
                            rusty_js_ast::BindingPattern::Identifier(id) => {
                                let s = self.alloc_local(LocalDescriptor {
                                    name: id.name.clone(), kind: *kind, depth: 0,
                                });
                                let fresh = matches!(kind, VariableKind::Let | VariableKind::Const);
                                (s, None, fresh)
                            }
                            pat @ (rusty_js_ast::BindingPattern::Array(_)
                                  | rusty_js_ast::BindingPattern::Object(_)) => {
                                // Allocate every bound name as a local under kind,
                                // then a hidden source slot for the per-iter value.
                                for id in pat.collect_names() {
                                    self.alloc_local(LocalDescriptor {
                                        name: id.name.clone(), kind: *kind, depth: 0,
                                    });
                                }
                                let s = self.alloc_temp("<forof.src>");
                                let fresh = matches!(kind, VariableKind::Let | VariableKind::Const);
                                (s, Some(pat.clone()), fresh)
                            }
                        }
                    }
                    rusty_js_ast::ForBinding::Pattern(pat) => {
                        match pat {
                            rusty_js_ast::BindingPattern::Identifier(id) => {
                                if let Some(s) = self.resolve_local(&id.name) { (s, None, false) }
                                else {
                                    let s = self.alloc_local(LocalDescriptor {
                                        name: id.name.clone(), kind: VariableKind::Let, depth: 0,
                                    });
                                    (s, None, false)
                                }
                            }
                            other => {
                                // Standalone pattern in for-of head (no var/let/const).
                                // Bound names assumed to already exist or are
                                // freshly let-allocated here.
                                for id in other.collect_names() {
                                    if self.resolve_local(&id.name).is_none() {
                                        self.alloc_local(LocalDescriptor {
                                            name: id.name.clone(), kind: VariableKind::Let, depth: 0,
                                        });
                                    }
                                }
                                let s = self.alloc_temp("<forof.src>");
                                (s, Some(other.clone()), false)
                            }
                        }
                    }
                };
                // Compute iterable[@@iterator]() and store into iter_slot.
                self.compile_expr(right)?;
                encode_op(&mut self.bytecode, Op::Dup);
                let iter_key = self.constants.intern(Constant::String("@@iterator".into()));
                encode_op(&mut self.bytecode, Op::GetProp);
                encode_u16(&mut self.bytecode, iter_key);
                encode_op(&mut self.bytecode, Op::CallMethod);
                encode_u8(&mut self.bytecode, 0);
                encode_op(&mut self.bytecode, Op::StoreLocal);
                encode_u16(&mut self.bytecode, iter_slot);

                let loop_start = self.bytecode.len();
                self.loop_stack.push(LoopFrame {
                    continue_target: loop_start, continue_pending: false,
                    continue_patches: Vec::new(), break_patches: Vec::new(), is_switch: false, label: self.pending_label.take(),
                });
                // result = iter.next()
                encode_op(&mut self.bytecode, Op::LoadLocal);
                encode_u16(&mut self.bytecode, iter_slot);
                encode_op(&mut self.bytecode, Op::Dup);
                let next_key = self.constants.intern(Constant::String("next".into()));
                encode_op(&mut self.bytecode, Op::GetProp);
                encode_u16(&mut self.bytecode, next_key);
                encode_op(&mut self.bytecode, Op::CallMethod);
                encode_u8(&mut self.bytecode, 0);
                // [result]
                encode_op(&mut self.bytecode, Op::Dup);
                let done_key = self.constants.intern(Constant::String("done".into()));
                encode_op(&mut self.bytecode, Op::GetProp);
                encode_u16(&mut self.bytecode, done_key);
                // [result, done] — JumpIfTrue pops done
                let j_done = self.emit_jump(Op::JumpIfTrue);
                // [result]
                let value_key = self.constants.intern(Constant::String("value".into()));
                encode_op(&mut self.bytecode, Op::GetProp);
                encode_u16(&mut self.bytecode, value_key);
                // Per-iteration fresh binding for let/const heads: detach the
                // previous iteration's upvalue cell from this frame slot so
                // the body's CaptureLocal promotes to a new one. ECMA-262
                // §14.7.5.5 / Tier-Ω.5.g.1.
                if per_iter_fresh {
                    encode_op(&mut self.bytecode, Op::ResetLocalCell);
                    encode_u16(&mut self.bytecode, bind_slot);
                }
                encode_op(&mut self.bytecode, Op::StoreLocal);
                encode_u16(&mut self.bytecode, bind_slot);
                if let Some(pat) = &destr_pat {
                    self.emit_destructure(pat, bind_slot)?;
                }
                self.compile_stmt(body)?;
                self.emit_back_jump(loop_start);
                self.patch_jump(j_done);
                // At the exit, the result object is on the stack — pop it.
                encode_op(&mut self.bytecode, Op::Pop);
                let frame = self.loop_stack.pop().unwrap();
                for site in frame.break_patches { self.patch_jump_at(site); }
            }
            Stmt::Break { label, .. } => {
                match label {
                    None => {
                        if let Some(frame) = self.loop_stack.last_mut() {
                            let patch_site = encode_op(&mut self.bytecode, Op::Jump);
                            encode_i32(&mut self.bytecode, 0);
                            frame.break_patches.push(patch_site);
                        } else {
                            return Err(self.err(span, "break outside of loop"));
                        }
                    }
                    Some(name) => {
                        // Tier-Ω.5.o: labelled break — walk loop_stack
                        // top-down for a frame whose `label` matches, then
                        // fall back to label_stack (labelled non-loop).
                        let needle = name.name.clone();
                        if let Some(idx) = self.loop_stack.iter()
                            .rposition(|f| f.label.as_deref() == Some(needle.as_str()))
                        {
                            let patch_site = encode_op(&mut self.bytecode, Op::Jump);
                            encode_i32(&mut self.bytecode, 0);
                            self.loop_stack[idx].break_patches.push(patch_site);
                        } else if let Some(idx) = self.label_stack.iter()
                            .rposition(|f| f.label == needle)
                        {
                            let patch_site = encode_op(&mut self.bytecode, Op::Jump);
                            encode_i32(&mut self.bytecode, 0);
                            self.label_stack[idx].break_patches.push(patch_site);
                        } else {
                            return Err(self.err(span,
                                &format!("break label '{}' not found in enclosing scopes", needle)));
                        }
                    }
                }
            }
            Stmt::FunctionDecl { name, is_async, is_generator, params, body, .. } => {
                let proto = self.compile_function_proto(name.clone(), *is_async, *is_generator, params, body)?;
                let captures = proto.upvalues.clone();
                let idx = self.constants.intern(Constant::Function(Box::new(proto)));
                encode_op(&mut self.bytecode, Op::MakeClosure);
                encode_u16(&mut self.bytecode, idx);
                emit_captures(&mut self.bytecode, &captures);
                // Bind to a local slot under the function's name.
                if let Some(n) = name {
                    let slot = self.alloc_local(LocalDescriptor {
                        name: n.name.clone(),
                        kind: VariableKind::Var,  // functions are var-scoped per spec
                        depth: 0,
                    });
                    encode_op(&mut self.bytecode, Op::StoreLocal);
                    encode_u16(&mut self.bytecode, slot);
                } else {
                    encode_op(&mut self.bytecode, Op::Pop);
                }
            }
            Stmt::Try { block, handler, finalizer, .. } => {
                // v1 minimal: encode TRY_ENTER with catch offset, compile block,
                // TRY_EXIT, jump past handler/finalizer; emit handler/finalizer
                // bodies. No exception-value binding to catch parameter yet
                // (would require a CATCH_BIND opcode). Body content compiles
                // normally.
                let try_enter = self.bytecode.len();
                encode_op(&mut self.bytecode, Op::TryEnter);
                let catch_off_patch = self.bytecode.len();
                encode_u32(&mut self.bytecode, 0);
                self.compile_stmt(block)?;
                encode_op(&mut self.bytecode, Op::TryExit);
                let jump_to_end = self.emit_jump(Op::Jump);
                // Patch the catch offset to point here (start of handler).
                let catch_pos = self.bytecode.len();
                let _ = try_enter;
                self.bytecode[catch_off_patch..catch_off_patch + 4]
                    .copy_from_slice(&(catch_pos as u32).to_le_bytes());
                if let Some(h) = handler {
                    // Binding the catch param to a local: v1 pops the thrown
                    // value into a fresh slot if param present, else discards.
                    if let Some(p) = &h.param {
                        let slot = self.alloc_local(LocalDescriptor {
                            name: p.name.clone(),
                            kind: VariableKind::Let,
                            depth: 0,
                        });
                        encode_op(&mut self.bytecode, Op::StoreLocal);
                        encode_u16(&mut self.bytecode, slot);
                    } else {
                        encode_op(&mut self.bytecode, Op::Pop);
                    }
                    self.compile_stmt(&h.body)?;
                }
                self.patch_jump(jump_to_end);
                if let Some(fin) = finalizer {
                    self.compile_stmt(fin)?;
                }
            }
            Stmt::Continue { label, .. } => {
                // Find the target loop frame (skipping switch frames per
                // §14.12.4). Unlabelled: innermost loop. Labelled: nearest
                // loop frame whose `label` matches — switch frames are
                // skipped on the way up; labelled non-loop frames cannot
                // be `continue`d into and are skipped silently (a label
                // attached to a block doesn't support continue).
                let loop_idx = match label {
                    None => self.loop_stack.iter().rposition(|f| !f.is_switch),
                    Some(name) => {
                        let needle = name.name.clone();
                        let r = self.loop_stack.iter()
                            .rposition(|f| !f.is_switch && f.label.as_deref() == Some(needle.as_str()));
                        if r.is_none() {
                            return Err(self.err(span,
                                &format!("continue label '{}' does not match an enclosing loop", needle)));
                        }
                        r
                    }
                };
                let Some(idx) = loop_idx else {
                    return Err(self.err(span, "continue outside of loop"));
                };
                let pending = self.loop_stack[idx].continue_pending;
                if pending {
                    let patch_site = encode_op(&mut self.bytecode, Op::Jump);
                    encode_i32(&mut self.bytecode, 0);
                    self.loop_stack[idx].continue_patches.push(patch_site);
                } else {
                    let target = self.loop_stack[idx].continue_target;
                    self.emit_back_jump(target);
                }
            }
            Stmt::ClassDecl { name, super_class, members, span } => {
                // Lower the class definition; result (the constructor function)
                // is left on the stack, then bound to a local under `name`.
                self.compile_class(*span, name.as_ref(), super_class.as_ref(), members)?;
                if let Some(n) = name {
                    let slot = self.alloc_local(LocalDescriptor {
                        name: n.name.clone(),
                        kind: VariableKind::Let,
                        depth: 0,
                    });
                    encode_op(&mut self.bytecode, Op::StoreLocal);
                    encode_u16(&mut self.bytecode, slot);
                } else {
                    encode_op(&mut self.bytecode, Op::Pop);
                }
            }
            Stmt::Switch { discriminant, cases, .. } => {
                // Tier-Ω.5.m: switch lowering per ECMA-262 §14.12.4.
                // 1. Spill the discriminant into a hidden local so the
                //    per-case StrictEq compares always use the same value.
                let disc_slot = self.alloc_temp("<switch.disc>");
                self.compile_expr(discriminant)?;
                encode_op(&mut self.bytecode, Op::StoreLocal);
                encode_u16(&mut self.bytecode, disc_slot);

                // 2. Dispatch chain. For each non-default case, emit a
                //    StrictEq test that conditionally jumps to that case's
                //    body. Record one patch site per case (None for the
                //    default — its body label is patched via default_jump).
                let mut case_body_patches: Vec<Option<usize>> = Vec::with_capacity(cases.len());
                let mut default_idx: Option<usize> = None;
                for (i, case) in cases.iter().enumerate() {
                    match &case.test {
                        Some(val) => {
                            encode_op(&mut self.bytecode, Op::LoadLocal);
                            encode_u16(&mut self.bytecode, disc_slot);
                            self.compile_expr(val)?;
                            encode_op(&mut self.bytecode, Op::StrictEq);
                            let j = self.emit_jump(Op::JumpIfTrue);
                            case_body_patches.push(Some(j));
                        }
                        None => {
                            if default_idx.is_some() {
                                return Err(self.err(span, "switch has more than one default clause"));
                            }
                            default_idx = Some(i);
                            // Body label patched after default fall-through
                            // jump below.
                            case_body_patches.push(None);
                        }
                    }
                }

                // 3. If no case matched: jump to default body (if any) or
                //    past the switch end.
                let default_jump = self.emit_jump(Op::Jump);

                // 4. Push a switch frame so `break` targets the end. We
                //    leave continue_pending=false and continue_target=0:
                //    Continue handling skips switch frames explicitly.
                self.loop_stack.push(LoopFrame {
                    continue_target: 0, continue_pending: false,
                    continue_patches: Vec::new(), break_patches: Vec::new(),
                    is_switch: true, label: None,
                });

                // 5. Emit each case body in textual order. Patch its
                //    dispatch site (or default_jump for the default case)
                //    to the body start so fall-through flows naturally
                //    into the next body.
                for (i, case) in cases.iter().enumerate() {
                    let body_start = self.bytecode.len();
                    match case_body_patches[i] {
                        Some(p) => self.patch_jump_to(p, body_start),
                        None => self.patch_jump_to(default_jump, body_start),
                    }
                    for s in &case.consequent {
                        self.compile_stmt(s)?;
                    }
                }

                // 6. End label. If no default clause existed, the
                //    default_jump still needs a target — wire it to here.
                if default_idx.is_none() {
                    self.patch_jump(default_jump);
                }
                let frame = self.loop_stack.pop().unwrap();
                for site in frame.break_patches { self.patch_jump_at(site); }
            }
            Stmt::ForIn { left, right, body, .. } => {
                // Tier-Ω.5.m: for-in lowering. Spec deviations:
                //  - Own enumerable string keys only (no proto-chain walk).
                //  - No Symbol-key exclusion (we don't ship real Symbols).
                //  - Enumeration order matches Object.keys (integer-like
                //    indices in ascending order, then string keys in
                //    insertion order, per ECMA-262 §7.3.22).
                //
                // Lower as: keys = Object.keys(obj); for (i=0; i<keys.length; i++)
                //   bind = keys[i]; body.
                let keys_slot = self.alloc_temp("<forin.keys>");
                let len_slot = self.alloc_temp("<forin.len>");
                let idx_slot = self.alloc_temp("<forin.idx>");

                // Decide the per-iteration binding slot (and per_iter_fresh
                // for let/const heads, mirroring Ω.5.g.1 for-of semantics).
                // ForBinding::Pattern with non-Identifier is deferred.
                let (bind_slot, per_iter_fresh): (u16, bool) = match left {
                    rusty_js_ast::ForBinding::Decl { kind, target, .. } => {
                        match target {
                            rusty_js_ast::BindingPattern::Identifier(id) => {
                                let s = self.alloc_local(LocalDescriptor {
                                    name: id.name.clone(), kind: *kind, depth: 0,
                                });
                                let fresh = matches!(kind, VariableKind::Let | VariableKind::Const);
                                (s, fresh)
                            }
                            _ => return Err(self.err(
                                span,
                                "for-in with destructure head not yet supported",
                            )),
                        }
                    }
                    rusty_js_ast::ForBinding::Pattern(pat) => {
                        match pat {
                            rusty_js_ast::BindingPattern::Identifier(id) => {
                                if let Some(s) = self.resolve_local(&id.name) { (s, false) }
                                else {
                                    let s = self.alloc_local(LocalDescriptor {
                                        name: id.name.clone(), kind: VariableKind::Let, depth: 0,
                                    });
                                    (s, false)
                                }
                            }
                            _ => return Err(self.err(
                                span,
                                "for-in with destructure head not yet supported",
                            )),
                        }
                    }
                };

                // keys = Object.keys(<right>)
                let obj_name = self.constants.intern(Constant::String("Object".into()));
                encode_op(&mut self.bytecode, Op::LoadGlobal);
                encode_u16(&mut self.bytecode, obj_name);
                encode_op(&mut self.bytecode, Op::Dup);
                let keys_key = self.constants.intern(Constant::String("keys".into()));
                encode_op(&mut self.bytecode, Op::GetProp);
                encode_u16(&mut self.bytecode, keys_key);
                self.compile_expr(right)?;
                encode_op(&mut self.bytecode, Op::CallMethod);
                encode_u8(&mut self.bytecode, 1);
                encode_op(&mut self.bytecode, Op::StoreLocal);
                encode_u16(&mut self.bytecode, keys_slot);

                // len = keys.length
                encode_op(&mut self.bytecode, Op::LoadLocal);
                encode_u16(&mut self.bytecode, keys_slot);
                let len_key = self.constants.intern(Constant::String("length".into()));
                encode_op(&mut self.bytecode, Op::GetProp);
                encode_u16(&mut self.bytecode, len_key);
                encode_op(&mut self.bytecode, Op::StoreLocal);
                encode_u16(&mut self.bytecode, len_slot);

                // i = 0
                encode_op(&mut self.bytecode, Op::PushI32);
                encode_i32(&mut self.bytecode, 0);
                encode_op(&mut self.bytecode, Op::StoreLocal);
                encode_u16(&mut self.bytecode, idx_slot);

                // loop_start: if (i >= len) break
                let loop_start = self.bytecode.len();
                self.loop_stack.push(LoopFrame {
                    continue_target: 0, continue_pending: true,
                    continue_patches: Vec::new(), break_patches: Vec::new(),
                    is_switch: false, label: self.pending_label.take(),
                });
                encode_op(&mut self.bytecode, Op::LoadLocal);
                encode_u16(&mut self.bytecode, idx_slot);
                encode_op(&mut self.bytecode, Op::LoadLocal);
                encode_u16(&mut self.bytecode, len_slot);
                encode_op(&mut self.bytecode, Op::Lt);
                let j_done = self.emit_jump(Op::JumpIfFalse);

                // bind = keys[i]
                if per_iter_fresh {
                    encode_op(&mut self.bytecode, Op::ResetLocalCell);
                    encode_u16(&mut self.bytecode, bind_slot);
                }
                encode_op(&mut self.bytecode, Op::LoadLocal);
                encode_u16(&mut self.bytecode, keys_slot);
                encode_op(&mut self.bytecode, Op::LoadLocal);
                encode_u16(&mut self.bytecode, idx_slot);
                encode_op(&mut self.bytecode, Op::GetIndex);
                encode_op(&mut self.bytecode, Op::StoreLocal);
                encode_u16(&mut self.bytecode, bind_slot);

                self.compile_stmt(body)?;

                // continue target: i++
                let cont_pos = self.bytecode.len();
                {
                    let frame = self.loop_stack.last_mut().unwrap();
                    frame.continue_target = cont_pos;
                    frame.continue_pending = false;
                }
                let patches = std::mem::take(&mut self.loop_stack.last_mut().unwrap().continue_patches);
                for site in patches { self.patch_jump_at(site); }
                encode_op(&mut self.bytecode, Op::LoadLocal);
                encode_u16(&mut self.bytecode, idx_slot);
                encode_op(&mut self.bytecode, Op::PushI32);
                encode_i32(&mut self.bytecode, 1);
                encode_op(&mut self.bytecode, Op::Add);
                encode_op(&mut self.bytecode, Op::StoreLocal);
                encode_u16(&mut self.bytecode, idx_slot);
                self.emit_back_jump(loop_start);
                self.patch_jump(j_done);
                let frame = self.loop_stack.pop().unwrap();
                for site in frame.break_patches { self.patch_jump_at(site); }
            }
            Stmt::Labelled { label, body, .. } => {
                // Tier-Ω.5.o: LabelledStatement. If the body is a loop,
                // the label rides on the loop's LoopFrame (via
                // pending_label) and break/continue resolve there. For a
                // non-loop body, push a LabelFrame so labelled `break`
                // still works; labelled `continue` is rejected at the
                // continue site.
                let is_loop_body = matches!(&**body,
                    Stmt::While { .. } | Stmt::DoWhile { .. }
                    | Stmt::For { .. } | Stmt::ForIn { .. } | Stmt::ForOf { .. });
                if is_loop_body {
                    self.pending_label = Some(label.name.clone());
                    self.compile_stmt(body)?;
                    // pending_label is consumed by the loop's frame-push.
                } else {
                    self.label_stack.push(LabelFrame {
                        label: label.name.clone(),
                        break_patches: Vec::new(),
                    });
                    self.compile_stmt(body)?;
                    let frame = self.label_stack.pop().unwrap();
                    for site in frame.break_patches { self.patch_jump_at(site); }
                }
            }
            other => {
                let tag = match other {
                    Stmt::Opaque { .. } => "Opaque",
                    _ => "<other>",
                };
                return Err(self.err(span, &format!("statement form not yet supported in compiler v1: {}", tag)));
            }
        }
        Ok(())
    }

    // ───────────────── Tier-Ω.5.g.3: destructuring lowering ─────────────────

    /// Emit bytecode that destructures the value currently in `src_slot`
    /// into the bindings named by `pat`. Pattern leaves (Identifier) emit
    /// LoadLocal+StoreLocal into the leaf binding's slot, which was
    /// pre-allocated by the caller via pat.collect_names().
    fn emit_destructure(
        &mut self,
        pat: &rusty_js_ast::BindingPattern,
        src_slot: u16,
    ) -> Result<(), CompileError> {
        match pat {
            rusty_js_ast::BindingPattern::Identifier(id) => {
                let slot = self.resolve_local(&id.name)
                    .expect("destructure leaf: binding slot pre-allocated by caller");
                encode_op(&mut self.bytecode, Op::LoadLocal);
                encode_u16(&mut self.bytecode, src_slot);
                encode_op(&mut self.bytecode, Op::StoreLocal);
                encode_u16(&mut self.bytecode, slot);
            }
            rusty_js_ast::BindingPattern::Array(arr) => {
                for (i, slot_opt) in arr.elements.iter().enumerate() {
                    let Some(elem) = slot_opt else { continue; };
                    // value = src[i]
                    encode_op(&mut self.bytecode, Op::LoadLocal);
                    encode_u16(&mut self.bytecode, src_slot);
                    encode_op(&mut self.bytecode, Op::PushI32);
                    encode_i32(&mut self.bytecode, i as i32);
                    encode_op(&mut self.bytecode, Op::GetIndex);
                    self.emit_element_with_default(&elem.target, elem.default.as_ref())?;
                }
                if let Some(rest_pat) = &arr.rest {
                    // value = __destr_array_rest(src, start)
                    let name_idx = self.constants.intern(Constant::String("__destr_array_rest".into()));
                    encode_op(&mut self.bytecode, Op::LoadGlobal);
                    encode_u16(&mut self.bytecode, name_idx);
                    encode_op(&mut self.bytecode, Op::LoadLocal);
                    encode_u16(&mut self.bytecode, src_slot);
                    encode_op(&mut self.bytecode, Op::PushI32);
                    encode_i32(&mut self.bytecode, arr.elements.len() as i32);
                    encode_op(&mut self.bytecode, Op::Call);
                    encode_u8(&mut self.bytecode, 2);
                    // No default for rest.
                    self.emit_element_with_default(rest_pat, None)?;
                }
            }
            rusty_js_ast::BindingPattern::Object(obj) => {
                let mut static_excluded: Vec<String> = Vec::new();
                for prop in &obj.properties {
                    // Push src on stack and get the property.
                    encode_op(&mut self.bytecode, Op::LoadLocal);
                    encode_u16(&mut self.bytecode, src_slot);
                    match &prop.key {
                        rusty_js_ast::PropertyKey::Identifier(id) => {
                            let k = self.constants.intern(Constant::String(id.name.clone()));
                            encode_op(&mut self.bytecode, Op::GetProp);
                            encode_u16(&mut self.bytecode, k);
                            static_excluded.push(id.name.clone());
                        }
                        rusty_js_ast::PropertyKey::String(s) => {
                            let k = self.constants.intern(Constant::String((**s).clone()));
                            encode_op(&mut self.bytecode, Op::GetProp);
                            encode_u16(&mut self.bytecode, k);
                            static_excluded.push((**s).clone());
                        }
                        rusty_js_ast::PropertyKey::Number(n) => {
                            let name = if n.fract() == 0.0 { format!("{}", *n as i64) } else { format!("{}", n) };
                            let k = self.constants.intern(Constant::String(name.clone()));
                            encode_op(&mut self.bytecode, Op::GetProp);
                            encode_u16(&mut self.bytecode, k);
                            static_excluded.push(name);
                        }
                        rusty_js_ast::PropertyKey::Computed(expr) => {
                            // src[expr]
                            self.compile_expr(expr)?;
                            encode_op(&mut self.bytecode, Op::GetIndex);
                            // Computed key excludes nothing reliably; rest
                            // pattern with computed keys above it isn't
                            // well-supported in our subset.
                        }
                    }
                    self.emit_element_with_default(&prop.value.target, prop.value.default.as_ref())?;
                }
                if let Some(rest_id) = &obj.rest {
                    // Call shape: [callee, src, excluded_array]
                    let name_idx = self.constants.intern(Constant::String("__destr_object_rest".into()));
                    encode_op(&mut self.bytecode, Op::LoadGlobal);
                    encode_u16(&mut self.bytecode, name_idx);
                    encode_op(&mut self.bytecode, Op::LoadLocal);
                    encode_u16(&mut self.bytecode, src_slot);
                    encode_op(&mut self.bytecode, Op::NewArray);
                    encode_u16(&mut self.bytecode, static_excluded.len() as u16);
                    for (i, k) in static_excluded.iter().enumerate() {
                        let idx = self.constants.intern(Constant::String(k.clone()));
                        encode_op(&mut self.bytecode, Op::PushConst);
                        encode_u16(&mut self.bytecode, idx);
                        encode_op(&mut self.bytecode, Op::InitIndex);
                        encode_u32(&mut self.bytecode, i as u32);
                    }
                    encode_op(&mut self.bytecode, Op::Call);
                    encode_u8(&mut self.bytecode, 2);
                    let slot = self.resolve_local(&rest_id.name)
                        .expect("object-rest binding slot pre-allocated by caller");
                    encode_op(&mut self.bytecode, Op::StoreLocal);
                    encode_u16(&mut self.bytecode, slot);
                }
            }
        }
        Ok(())
    }

    /// Consume the value currently on top of the operand stack, apply an
    /// optional default if it is === undefined, then bind it into `target`.
    /// For non-Identifier targets, spills into a fresh hidden local and
    /// recurses.
    fn emit_element_with_default(
        &mut self,
        target: &rusty_js_ast::BindingPattern,
        default: Option<&Expr>,
    ) -> Result<(), CompileError> {
        if let Some(def_expr) = default {
            // Dup; PushUndef; StrictEq; JumpIfFalse skip_default; Pop; <default>; skip:
            encode_op(&mut self.bytecode, Op::Dup);
            encode_op(&mut self.bytecode, Op::PushUndef);
            encode_op(&mut self.bytecode, Op::StrictEq);
            let j_skip = self.emit_jump(Op::JumpIfFalse);
            encode_op(&mut self.bytecode, Op::Pop);
            self.compile_expr(def_expr)?;
            self.patch_jump(j_skip);
        }
        match target {
            rusty_js_ast::BindingPattern::Identifier(id) => {
                let slot = self.resolve_local(&id.name)
                    .expect("destructure leaf: binding slot pre-allocated by caller");
                encode_op(&mut self.bytecode, Op::StoreLocal);
                encode_u16(&mut self.bytecode, slot);
            }
            nested => {
                let tmp = self.alloc_temp("<destr.tmp>");
                encode_op(&mut self.bytecode, Op::StoreLocal);
                encode_u16(&mut self.bytecode, tmp);
                self.emit_destructure(nested, tmp)?;
            }
        }
        Ok(())
    }

    /// Emit a forward jump with placeholder operand; return the operand
    /// offset for later patching via `patch_jump`.
    fn emit_jump(&mut self, op: Op) -> usize {
        encode_op(&mut self.bytecode, op);
        let operand_off = self.bytecode.len();
        encode_i32(&mut self.bytecode, 0);
        operand_off
    }

    /// Patch a forward-jump's operand so the jump targets the current
    /// bytecode offset (i.e., the place where emission has currently
    /// advanced to).
    fn patch_jump(&mut self, operand_off: usize) {
        let here = self.bytecode.len() as i32;
        let from = (operand_off + 4) as i32;
        let disp = here - from;
        self.bytecode[operand_off..operand_off + 4].copy_from_slice(&disp.to_le_bytes());
    }

    fn patch_jump_at(&mut self, operand_off: usize) {
        self.patch_jump(operand_off);
    }

    /// Patch a forward-jump to a specific absolute target offset.
    fn patch_jump_to(&mut self, operand_off: usize, target: usize) {
        let from = (operand_off + 4) as i32;
        let disp = target as i32 - from;
        self.bytecode[operand_off..operand_off + 4].copy_from_slice(&disp.to_le_bytes());
    }

    /// Emit an unconditional backward Jump to the given absolute offset.
    fn emit_back_jump(&mut self, target: usize) {
        encode_op(&mut self.bytecode, Op::Jump);
        let from = (self.bytecode.len() + 4) as i32;
        let disp = target as i32 - from;
        encode_i32(&mut self.bytecode, disp);
    }

    /// Allocate a local-slot for a binding. Returns the slot index.
    fn alloc_local(&mut self, desc: LocalDescriptor) -> u16 {
        let idx = self.locals.len();
        assert!(idx < u16::MAX as usize, "too many locals");
        self.locals.push(desc);
        idx as u16
    }

    /// Resolve an identifier to a local-slot index, if any.
    fn resolve_local(&self, name: &str) -> Option<u16> {
        for (i, l) in self.locals.iter().enumerate().rev() {
            if l.name == name { return Some(i as u16); }
        }
        None
    }

    /// Tier-Ω.5.c: resolve an identifier to an upvalue slot in this proto.
    /// Walks the enclosing chain bottom-up. If the name resolves to a local
    /// in an outer frame, an upvalue is created in this proto (and in every
    /// intermediate enclosing frame as a transitive upvalue).
    ///
    /// Returns the upvalue index in `self.upvalues` (0-based).
    fn resolve_upvalue(&mut self, name: &str) -> Option<u16> {
        if self.enclosing.is_empty() { return None; }
        // Walk from innermost enclosing (back) to outermost (front).
        // Innermost-first lets us emit the chain of transitive upvalues
        // in the right order.
        let levels = self.enclosing.len();
        for depth in (0..levels).rev() {
            // Check locals of this enclosing level.
            let local_slot = self.enclosing[depth].locals.iter().enumerate().rev()
                .find(|(_, l)| l.name == name).map(|(i, _)| i as u16);
            if let Some(slot) = local_slot {
                // Build the upvalue chain top-down from `depth` toward
                // current proto. Topmost (this proto) ends up referencing
                // an Upvalue of the immediate parent unless `depth` is
                // levels-1 (immediate parent), in which case it references
                // a Local.
                let mut src = UpvalueSource::Local(slot);
                let name_s = name.to_string();
                for d in (depth + 1)..levels {
                    let up_idx = add_upvalue_to(&mut self.enclosing[d].upvalues, src, name_s.clone());
                    src = UpvalueSource::Upvalue(up_idx);
                }
                let idx = add_upvalue_to(&mut self.upvalues, src, name.to_string());
                return Some(idx);
            }
            // Else check upvalues of this enclosing level — name might be
            // already-captured at this depth from an even-outer level.
            let up_at_depth = self.enclosing[depth].upvalues.iter().enumerate()
                .find(|(_, u)| u.name == name).map(|(i, _)| i as u16);
            if let Some(up_idx) = up_at_depth {
                let mut src = UpvalueSource::Upvalue(up_idx);
                for d in (depth + 1)..levels {
                    let i = add_upvalue_to(&mut self.enclosing[d].upvalues, src, name.to_string());
                    src = UpvalueSource::Upvalue(i);
                }
                let idx = add_upvalue_to(&mut self.upvalues, src, name.to_string());
                return Some(idx);
            }
        }
        None
    }

    fn compile_expr(&mut self, e: &Expr) -> Result<(), CompileError> {
        self.record_span(e.span());
        match e {
            Expr::NullLiteral { .. } => { encode_op(&mut self.bytecode, Op::PushNull); }
            Expr::BoolLiteral { value, .. } => {
                encode_op(&mut self.bytecode, if *value { Op::PushTrue } else { Op::PushFalse });
            }
            Expr::NumberLiteral { value, .. } => {
                // Integer-fast-path: if the number fits in i32 exactly, emit PushI32.
                if value.fract() == 0.0 && *value >= i32::MIN as f64 && *value <= i32::MAX as f64 {
                    let iv = *value as i32;
                    encode_op(&mut self.bytecode, Op::PushI32);
                    encode_i32(&mut self.bytecode, iv);
                } else {
                    let idx = self.constants.intern(Constant::Number(*value));
                    encode_op(&mut self.bytecode, Op::PushConst);
                    encode_u16(&mut self.bytecode, idx);
                }
            }
            Expr::StringLiteral { value, .. } => {
                let idx = self.constants.intern(Constant::String(value.clone()));
                encode_op(&mut self.bytecode, Op::PushConst);
                encode_u16(&mut self.bytecode, idx);
            }
            Expr::BigIntLiteral { digits, .. } => {
                let idx = self.constants.intern(Constant::BigInt(digits.clone()));
                encode_op(&mut self.bytecode, Op::PushConst);
                encode_u16(&mut self.bytecode, idx);
            }
            Expr::Identifier { name, .. } => {
                if let Some(slot) = self.resolve_local(name) {
                    encode_op(&mut self.bytecode, Op::LoadLocal);
                    encode_u16(&mut self.bytecode, slot);
                } else if let Some(up) = self.resolve_upvalue(name) {
                    encode_op(&mut self.bytecode, Op::LoadUpvalue);
                    encode_u16(&mut self.bytecode, up);
                } else {
                    let name_idx = self.constants.intern(Constant::String(name.clone()));
                    encode_op(&mut self.bytecode, Op::LoadGlobal);
                    encode_u16(&mut self.bytecode, name_idx);
                }
            }
            Expr::Unary { operator, argument, .. } => {
                self.compile_expr(argument)?;
                let op = match operator {
                    UnaryOp::Plus => Op::Pos,
                    UnaryOp::Minus => Op::Neg,
                    UnaryOp::BitNot => Op::BitNot,
                    UnaryOp::LogicalNot => Op::Not,
                    UnaryOp::Typeof => Op::Typeof,
                    UnaryOp::Void => Op::Void,
                    UnaryOp::Delete => Op::Delete,
                    UnaryOp::Await => return Err(self.err(e.span(), "await not yet supported")),
                };
                encode_op(&mut self.bytecode, op);
            }
            Expr::Binary { operator, left, right, .. } => {
                match operator {
                    BinaryOp::LogicalAnd => {
                        // emit left; JumpIfFalseKeep end; Pop; emit right; end:
                        self.compile_expr(left)?;
                        let j = self.emit_jump(Op::JumpIfFalseKeep);
                        encode_op(&mut self.bytecode, Op::Pop);
                        self.compile_expr(right)?;
                        self.patch_jump(j);
                    }
                    BinaryOp::LogicalOr => {
                        self.compile_expr(left)?;
                        let j = self.emit_jump(Op::JumpIfTrueKeep);
                        encode_op(&mut self.bytecode, Op::Pop);
                        self.compile_expr(right)?;
                        self.patch_jump(j);
                    }
                    BinaryOp::NullishCoalesce => {
                        // Push LHS. Dup. JumpIfNullish to fallback (pops the
                        // top copy; the remaining LHS is the result). Else
                        // fall-through: same — Pop the dup, then we want LHS
                        // as result. Use the cleaner form:
                        //   emit LHS                            [a]
                        //   Dup                                 [a, a]
                        //   JumpIfNullish fb (pops top)          [a]   (jumps if nullish)
                        //   Jump end                            [a]
                        //   fb: Pop                              []
                        //       emit RHS                         [b]
                        //   end:                                 [result]
                        self.compile_expr(left)?;
                        encode_op(&mut self.bytecode, Op::Dup);
                        let j_fb = self.emit_jump(Op::JumpIfNullish);
                        let j_end = self.emit_jump(Op::Jump);
                        self.patch_jump(j_fb);
                        encode_op(&mut self.bytecode, Op::Pop);
                        self.compile_expr(right)?;
                        self.patch_jump(j_end);
                    }
                    _ => {
                        self.compile_expr(left)?;
                        self.compile_expr(right)?;
                        let op = match operator {
                            BinaryOp::Add => Op::Add, BinaryOp::Sub => Op::Sub,
                            BinaryOp::Mul => Op::Mul, BinaryOp::Div => Op::Div,
                            BinaryOp::Mod => Op::Mod, BinaryOp::Pow => Op::Pow,
                            BinaryOp::Shl => Op::Shl, BinaryOp::Shr => Op::Shr,
                            BinaryOp::UShr => Op::UShr,
                            BinaryOp::Lt => Op::Lt, BinaryOp::Gt => Op::Gt,
                            BinaryOp::Le => Op::Le, BinaryOp::Ge => Op::Ge,
                            BinaryOp::Eq => Op::Eq, BinaryOp::Ne => Op::Ne,
                            BinaryOp::StrictEq => Op::StrictEq, BinaryOp::StrictNe => Op::StrictNe,
                            BinaryOp::Instanceof => Op::Instanceof, BinaryOp::In => Op::In,
                            BinaryOp::BitAnd => Op::BitAnd, BinaryOp::BitOr => Op::BitOr,
                            BinaryOp::BitXor => Op::BitXor,
                            _ => unreachable!(),
                        };
                        encode_op(&mut self.bytecode, op);
                    }
                }
            }
            Expr::Parenthesized { expr, .. } => self.compile_expr(expr)?,
            Expr::Conditional { test, consequent, alternate, .. } => {
                self.compile_expr(test)?;
                let j_else = self.emit_jump(Op::JumpIfFalse);
                self.compile_expr(consequent)?;
                let j_end = self.emit_jump(Op::Jump);
                self.patch_jump(j_else);
                self.compile_expr(alternate)?;
                self.patch_jump(j_end);
            }
            Expr::Sequence { expressions, .. } => {
                // Tier-Ω.5.n: comma expression `a, b, c`. Evaluate each;
                // discard all but the last; final value remains on stack.
                let n = expressions.len();
                if n == 0 {
                    encode_op(&mut self.bytecode, Op::PushUndef);
                } else {
                    for (i, ex) in expressions.iter().enumerate() {
                        self.compile_expr(ex)?;
                        if i + 1 < n {
                            encode_op(&mut self.bytecode, Op::Pop);
                        }
                    }
                }
            }
            Expr::Assign { operator, target, value, .. } => {
                self.compile_assign(e.span(), *operator, target, value)?;
            }
            Expr::This { .. } => {
                // Tier-Ω.5.a: this now threads through the frame.
                encode_op(&mut self.bytecode, Op::PushThis);
            }
            Expr::Member { object, property, optional: _, .. } => {
                // Tier-Ω.5.f: super.x read — load from the parent prototype
                // (or parent constructor in a static context). The lookup
                // does NOT thread `this` for a bare member read; only when
                // wrapped in a Call does receiver-as-this matter.
                if matches!(object.as_ref(), Expr::Super { .. }) {
                    self.compile_super_member_load(e.span(), property)?;
                    return Ok(());
                }
                self.compile_expr(object)?;
                match property.as_ref() {
                    MemberProperty::Identifier { name, .. } => {
                        let idx = self.constants.intern(Constant::String(name.clone()));
                        encode_op(&mut self.bytecode, Op::GetProp);
                        encode_u16(&mut self.bytecode, idx);
                    }
                    MemberProperty::Computed { expr, .. } => {
                        self.compile_expr(expr)?;
                        encode_op(&mut self.bytecode, Op::GetIndex);
                    }
                    MemberProperty::Private { name, .. } => {
                        let idx = self.constants.intern(Constant::String(format!("#{}", name)));
                        encode_op(&mut self.bytecode, Op::GetProp);
                        encode_u16(&mut self.bytecode, idx);
                    }
                }
            }
            Expr::Call { callee, arguments, optional: _, .. } => {
                let n = arguments.len();
                if n > 255 {
                    return Err(self.err(e.span(), "too many call arguments (>255)"));
                }
                // Tier-Ω.5.f: super(...) call inside a derived-class
                // constructor. Lowers to a method-call on the parent
                // constructor with the current `this` as receiver.
                if matches!(callee.as_ref(), Expr::Super { .. }) {
                    self.compile_super_call(e.span(), arguments)?;
                    return Ok(());
                }
                // Tier-Ω.5.f: super.method(...) call inside an instance or
                // static method. Lowers to a method-call on the parent
                // prototype's (or parent constructor's) named slot with
                // the current `this` as receiver.
                if let Expr::Member { object, property, .. } = callee.as_ref() {
                    if matches!(object.as_ref(), Expr::Super { .. }) {
                        self.compile_super_member_call(e.span(), property, arguments)?;
                        return Ok(());
                    }
                }
                // Tier-Ω.5.a: when callee is a MemberExpression, emit a
                // method-call form so `this` threads as the receiver.
                let is_method = matches!(callee.as_ref(), Expr::Member { .. });
                let has_spread = Self::args_has_spread(arguments);
                if is_method {
                    if let Expr::Member { object, property, optional: _, .. } = callee.as_ref() {
                        if has_spread {
                            // Tier-Ω.5.k: lower `obj.f(...args)` as
                            //   __apply(method, receiver, argsArray)
                            // Stack:
                            //   LoadGlobal __apply        [__apply]
                            //   <object>                  [__apply, recv]
                            //   Dup                       [__apply, recv, recv]
                            //   GetProp/GetIndex name     [__apply, recv, method]
                            //   Swap                      [__apply, method, recv]
                            //   <argsArray>               [__apply, method, recv, arr]
                            //   Call 3                    [result]
                            let apply_name = self.constants.intern(
                                Constant::String("__apply".to_string()));
                            encode_op(&mut self.bytecode, Op::LoadGlobal);
                            encode_u16(&mut self.bytecode, apply_name);
                            self.compile_expr(object)?;
                            encode_op(&mut self.bytecode, Op::Dup);
                            match property.as_ref() {
                                MemberProperty::Identifier { name, .. } => {
                                    let idx = self.constants.intern(Constant::String(name.clone()));
                                    encode_op(&mut self.bytecode, Op::GetProp);
                                    encode_u16(&mut self.bytecode, idx);
                                }
                                MemberProperty::Computed { expr, .. } => {
                                    self.compile_expr(expr)?;
                                    encode_op(&mut self.bytecode, Op::GetIndex);
                                }
                                MemberProperty::Private { name, .. } => {
                                    let idx = self.constants.intern(
                                        Constant::String(format!("#{}", name)));
                                    encode_op(&mut self.bytecode, Op::GetProp);
                                    encode_u16(&mut self.bytecode, idx);
                                }
                            }
                            encode_op(&mut self.bytecode, Op::Swap);
                            self.emit_args_array(arguments)?;
                            encode_op(&mut self.bytecode, Op::Call);
                            encode_u8(&mut self.bytecode, 3);
                        } else {
                            // Push receiver, then method (looked up via GetProp /
                            // GetIndex), then args, then CallMethod n.
                            self.compile_expr(object)?;
                            // Duplicate receiver so we can use it for the method
                            // lookup without losing it for the CallMethod consumer.
                            encode_op(&mut self.bytecode, Op::Dup);
                            match property.as_ref() {
                                MemberProperty::Identifier { name, .. } => {
                                    let idx = self.constants.intern(Constant::String(name.clone()));
                                    encode_op(&mut self.bytecode, Op::GetProp);
                                    encode_u16(&mut self.bytecode, idx);
                                }
                                MemberProperty::Computed { expr, .. } => {
                                    self.compile_expr(expr)?;
                                    encode_op(&mut self.bytecode, Op::GetIndex);
                                }
                                MemberProperty::Private { name, .. } => {
                                    let idx = self.constants.intern(Constant::String(format!("#{}", name)));
                                    encode_op(&mut self.bytecode, Op::GetProp);
                                    encode_u16(&mut self.bytecode, idx);
                                }
                            }
                            // Now stack: [receiver, method]. Compile args.
                            for a in arguments {
                                match a {
                                    Argument::Expr(e) => self.compile_expr(e)?,
                                    Argument::Spread { .. } => unreachable!(),
                                }
                            }
                            encode_op(&mut self.bytecode, Op::CallMethod);
                            encode_u8(&mut self.bytecode, n as u8);
                        }
                    }
                } else if has_spread {
                    // Tier-Ω.5.k: lower `f(...args)` as
                    //   __apply(callee, undefined, argsArray)
                    let apply_name = self.constants.intern(
                        Constant::String("__apply".to_string()));
                    encode_op(&mut self.bytecode, Op::LoadGlobal);
                    encode_u16(&mut self.bytecode, apply_name);
                    self.compile_expr(callee)?;
                    encode_op(&mut self.bytecode, Op::PushUndef);
                    self.emit_args_array(arguments)?;
                    encode_op(&mut self.bytecode, Op::Call);
                    encode_u8(&mut self.bytecode, 3);
                } else {
                    self.compile_expr(callee)?;
                    for a in arguments {
                        match a {
                            Argument::Expr(e) => self.compile_expr(e)?,
                            Argument::Spread { .. } => unreachable!(),
                        }
                    }
                    encode_op(&mut self.bytecode, Op::Call);
                    encode_u8(&mut self.bytecode, n as u8);
                }
            }
            Expr::New { callee, arguments, .. } => {
                let n = arguments.len();
                if n > 255 {
                    return Err(self.err(e.span(), "too many new arguments (>255)"));
                }
                if Self::args_has_spread(arguments) {
                    // Tier-Ω.5.k: lower `new C(...args)` as
                    //   __construct(callee, argsArray)
                    let ctor_name = self.constants.intern(
                        Constant::String("__construct".to_string()));
                    encode_op(&mut self.bytecode, Op::LoadGlobal);
                    encode_u16(&mut self.bytecode, ctor_name);
                    self.compile_expr(callee)?;
                    self.emit_args_array(arguments)?;
                    encode_op(&mut self.bytecode, Op::Call);
                    encode_u8(&mut self.bytecode, 2);
                } else {
                    self.compile_expr(callee)?;
                    for a in arguments {
                        match a {
                            Argument::Expr(e) => self.compile_expr(e)?,
                            Argument::Spread { .. } => unreachable!(),
                        }
                    }
                    encode_op(&mut self.bytecode, Op::New);
                    encode_u8(&mut self.bytecode, n as u8);
                }
            }
            Expr::Array { elements, .. } => {
                let has_spread = elements.iter().any(|el| matches!(el, ArrayElement::Spread { .. }));
                if !has_spread {
                    let len = elements.len();
                    encode_op(&mut self.bytecode, Op::NewArray);
                    encode_u16(&mut self.bytecode, len.min(u16::MAX as usize) as u16);
                    let mut idx = 0u32;
                    for el in elements {
                        match el {
                            ArrayElement::Elision { .. } => { idx += 1; }
                            ArrayElement::Expr(ex) => {
                                self.compile_expr(ex)?;
                                encode_op(&mut self.bytecode, Op::InitIndex);
                                encode_u32(&mut self.bytecode, idx);
                                idx += 1;
                            }
                            ArrayElement::Spread { .. } => unreachable!(),
                        }
                    }
                } else {
                    // Tier-Ω.5.l: array literal with spread. Build incrementally
                    // via __array_push_single / __array_extend, matching the
                    // shape of emit_args_array (Ω.5.k).
                    encode_op(&mut self.bytecode, Op::NewArray);
                    encode_u16(&mut self.bytecode, 0);
                    let push_name = self.constants.intern(Constant::String("__array_push_single".to_string()));
                    let extend_name = self.constants.intern(Constant::String("__array_extend".to_string()));
                    for el in elements {
                        match el {
                            ArrayElement::Elision { .. } => {
                                encode_op(&mut self.bytecode, Op::LoadGlobal);
                                encode_u16(&mut self.bytecode, push_name);
                                encode_op(&mut self.bytecode, Op::Swap);
                                encode_op(&mut self.bytecode, Op::PushUndef);
                                encode_op(&mut self.bytecode, Op::Call);
                                encode_u8(&mut self.bytecode, 2);
                            }
                            ArrayElement::Expr(ex) => {
                                encode_op(&mut self.bytecode, Op::LoadGlobal);
                                encode_u16(&mut self.bytecode, push_name);
                                encode_op(&mut self.bytecode, Op::Swap);
                                self.compile_expr(ex)?;
                                encode_op(&mut self.bytecode, Op::Call);
                                encode_u8(&mut self.bytecode, 2);
                            }
                            ArrayElement::Spread { expr, .. } => {
                                encode_op(&mut self.bytecode, Op::LoadGlobal);
                                encode_u16(&mut self.bytecode, extend_name);
                                encode_op(&mut self.bytecode, Op::Swap);
                                self.compile_expr(expr)?;
                                encode_op(&mut self.bytecode, Op::Call);
                                encode_u8(&mut self.bytecode, 2);
                            }
                        }
                    }
                }
            }
            Expr::Object { properties, .. } => {
                encode_op(&mut self.bytecode, Op::NewObject);
                for p in properties {
                    match p {
                        ObjectProperty::Property { key, value, .. } => {
                            match key {
                                ObjectKey::Identifier { name, .. } | ObjectKey::String { value: name, .. } => {
                                    self.compile_expr(value)?;
                                    let idx = self.constants.intern(Constant::String(name.clone()));
                                    encode_op(&mut self.bytecode, Op::InitProp);
                                    encode_u16(&mut self.bytecode, idx);
                                }
                                ObjectKey::Number { value: num, .. } => {
                                    self.compile_expr(value)?;
                                    let name = if num.fract() == 0.0 {
                                        format!("{}", *num as i64)
                                    } else { format!("{}", num) };
                                    let idx = self.constants.intern(Constant::String(name));
                                    encode_op(&mut self.bytecode, Op::InitProp);
                                    encode_u16(&mut self.bytecode, idx);
                                }
                                ObjectKey::Computed { expr: key_expr, .. } => {
                                    // Tier-Ω.5.o: computed object key `{[k]: v}`.
                                    // Stack: [target] -> Dup -> [target, target]
                                    // -> compile key -> [target, target, key]
                                    // -> compile value -> [target, target, key, value]
                                    // -> SetIndex -> [target, value]
                                    // -> Pop -> [target].
                                    encode_op(&mut self.bytecode, Op::Dup);
                                    self.compile_expr(key_expr)?;
                                    self.compile_expr(value)?;
                                    encode_op(&mut self.bytecode, Op::SetIndex);
                                    encode_op(&mut self.bytecode, Op::Pop);
                                }
                            }
                        }
                        ObjectProperty::Spread { expr, .. } => {
                            // Tier-Ω.5.k: lower `{...src}` as
                            //   Dup; LoadGlobal __object_spread; Swap;
                            //   <compile src>; Call 2; Pop
                            // Pre: [target]. Post: [target]. The helper
                            // copies own-enumerable props of src into
                            // target left-to-right and returns target.
                            encode_op(&mut self.bytecode, Op::Dup);
                            let helper = self.constants.intern(
                                Constant::String("__object_spread".to_string()));
                            encode_op(&mut self.bytecode, Op::LoadGlobal);
                            encode_u16(&mut self.bytecode, helper);
                            encode_op(&mut self.bytecode, Op::Swap);
                            self.compile_expr(expr)?;
                            encode_op(&mut self.bytecode, Op::Call);
                            encode_u8(&mut self.bytecode, 2);
                            encode_op(&mut self.bytecode, Op::Pop);
                        }
                    }
                }
            }
            Expr::Function { name, is_async, is_generator, params, body, .. } => {
                let proto = self.compile_function_proto(name.clone(), *is_async, *is_generator, params, body)?;
                let captures = proto.upvalues.clone();
                let idx = self.constants.intern(Constant::Function(Box::new(proto)));
                encode_op(&mut self.bytecode, Op::MakeClosure);
                encode_u16(&mut self.bytecode, idx);
                emit_captures(&mut self.bytecode, &captures);
            }
            Expr::Arrow { is_async, params, body, .. } => {
                let body_stmts: Vec<Stmt> = match body {
                    ArrowBody::Block(stmts) => stmts.clone(),
                    ArrowBody::Expression(expr) => vec![Stmt::Return {
                        argument: Some((**expr).clone()),
                        span: expr.span(),
                    }],
                };
                let proto = self.compile_function_proto(None, *is_async, false, params, &body_stmts)?;
                let captures = proto.upvalues.clone();
                let idx = self.constants.intern(Constant::Function(Box::new(proto)));
                encode_op(&mut self.bytecode, Op::MakeArrow);
                encode_u16(&mut self.bytecode, idx);
                emit_captures(&mut self.bytecode, &captures);
            }
            Expr::Update { operator, argument, prefix, .. } => {
                self.compile_update(e.span(), *operator, argument, *prefix)?;
            }
            Expr::Class { name, super_class, members, span } => {
                // Class expression: lower exactly like ClassDecl but leave
                // the constructor on the stack as the expression's value.
                self.compile_class(
                    *span,
                    name.as_ref(),
                    super_class.as_ref().map(|b| b.as_ref()),
                    members,
                )?;
            }
            Expr::Super { span } => {
                return Err(self.err(*span,
                    "bare `super` reference is only valid as `super(...)` or `super.method(...)`"));
            }
            Expr::TemplateLiteral { quasis, expressions, .. } => {
                // Tier-Ω.5.g.3: lower to left-to-right Add chain. op_add
                // coerces non-string operands when the LHS is a String, so
                // explicit ToString is unnecessary: the first quasi (a
                // String constant) seeds the chain, after which every Add
                // produces a String result.
                debug_assert_eq!(quasis.len(), expressions.len() + 1);
                let first = self.constants.intern(Constant::String((**quasis.first().unwrap()).clone()));
                encode_op(&mut self.bytecode, Op::PushConst);
                encode_u16(&mut self.bytecode, first);
                for (i, expr) in expressions.iter().enumerate() {
                    self.compile_expr(expr)?;
                    encode_op(&mut self.bytecode, Op::Add);
                    let q = self.constants.intern(Constant::String((*quasis[i + 1]).clone()));
                    encode_op(&mut self.bytecode, Op::PushConst);
                    encode_u16(&mut self.bytecode, q);
                    encode_op(&mut self.bytecode, Op::Add);
                }
            }
            Expr::RegExp { pattern, flags, .. } => {
                // Tier-Ω.5.i: lower regex literal to a call into the hidden
                // global `__createRegExp(pattern, flags)`. Avoids adding a
                // new opcode; trades one bytecode slot for two symbol-table
                // lookups at install_intrinsics time. The runtime helper
                // allocates an Object with InternalKind::RegExp wired to
                // %RegExp.prototype% via the alloc-time proto seam.
                let helper_name = self.constants.intern(Constant::String("__createRegExp".to_string()));
                encode_op(&mut self.bytecode, Op::LoadGlobal);
                encode_u16(&mut self.bytecode, helper_name);
                let pat_idx = self.constants.intern(Constant::String((**pattern).clone()));
                encode_op(&mut self.bytecode, Op::PushConst);
                encode_u16(&mut self.bytecode, pat_idx);
                let flags_idx = self.constants.intern(Constant::String((**flags).clone()));
                encode_op(&mut self.bytecode, Op::PushConst);
                encode_u16(&mut self.bytecode, flags_idx);
                encode_op(&mut self.bytecode, Op::Call);
                encode_u8(&mut self.bytecode, 2u8);
            }
            other => {
                let tag = match other {
                    Expr::Sequence { .. } => "Sequence",
                    Expr::Conditional { .. } => "Conditional",
                    Expr::MetaProperty { .. } => "MetaProperty",
                    Expr::Opaque { .. } => "Opaque",
                    Expr::Class { .. } => "ClassExpression",
                    Expr::Super { .. } => "Super(standalone)",
                    Expr::Function { .. } => "Function",
                    Expr::Arrow { .. } => "Arrow",
                    _ => "<other>",
                };
                return Err(self.err(e.span(), &format!("expression form not yet supported in compiler v1: {}", tag)));
            }
        }
        Ok(())
    }

    /// Compile a nested function body into a FunctionProto. Tier-Ω.5.c
    /// threads the outer-scope chain in so identifiers in the body that
    /// resolve to an enclosing local are captured as upvalues.
    fn compile_function_proto(
        &mut self,
        _name: Option<BindingIdentifier>,
        _is_async: bool,
        _is_generator: bool,
        params: &[Parameter],
        body: &[Stmt],
    ) -> Result<FunctionProto, CompileError> {
        // Build the sub-compiler's enclosing chain from self's enclosing
        // plus self's own locals/upvalues snapshot. The snapshot is
        // immutable from the sub's perspective EXCEPT the sub may
        // back-fill upvalues into intermediate frames (handled by writing
        // back to self after the sub finishes).
        let mut sub_enclosing: Vec<EnclosingFrame> = self.enclosing.iter().cloned().collect();
        sub_enclosing.push(EnclosingFrame {
            locals: self.locals.clone(),
            upvalues: self.upvalues.clone(),
        });
        let mut sub = Compiler {
            bytecode: Vec::new(),
            constants: ConstantsPool::new(),
            locals: Vec::new(),
            source_map: Vec::new(),
            loop_stack: Vec::new(), label_stack: Vec::new(), pending_label: None,
            enclosing: sub_enclosing,
            upvalues: Vec::new(),
            class_stack: self.class_stack.clone(),
            class_seq: self.class_seq,
            imports: Vec::new(),
            exports: Vec::new(),
            reexport_sources: Vec::new(),
            pending_named_exports: Vec::new(),
        };
        let param_count = params.len() as u16;
        // Tier-Ω.5.l: track the rest-parameter slot. Per spec only the
        // last parameter can be a rest parameter; the runtime uses this
        // to collect `args[slot..]` into an Array at call time.
        let mut rest_param_slot: Option<u16> = None;
        // Allocate one local per parameter position (slots 0..N receive
        // the args at call time per Runtime::call_function). For destructure
        // params, the param slot is the hidden source and additional locals
        // for inner names are allocated below; a prologue then runs the
        // pattern lowering at function entry.
        let mut destr_prologue: Vec<(rusty_js_ast::BindingPattern, u16, Option<Expr>)> = Vec::new();
        for (i, p) in params.iter().enumerate() {
            match &p.target {
                rusty_js_ast::BindingPattern::Identifier(n) => {
                    let slot = sub.alloc_local(LocalDescriptor {
                        name: n.name.clone(),
                        kind: VariableKind::Let,
                        depth: 0,
                    });
                    if p.rest {
                        rest_param_slot = Some(slot);
                    }
                    if p.default.is_some() {
                        destr_prologue.push((
                            rusty_js_ast::BindingPattern::Identifier(n.clone()),
                            i as u16,
                            p.default.clone(),
                        ));
                    }
                }
                pat @ (rusty_js_ast::BindingPattern::Array(_)
                      | rusty_js_ast::BindingPattern::Object(_)) => {
                    // Hidden source slot in the param position.
                    sub.alloc_local(LocalDescriptor {
                        name: format!("<param${}>", i),
                        kind: VariableKind::Let,
                        depth: 0,
                    });
                    // Inner names.
                    for id in pat.collect_names() {
                        sub.alloc_local(LocalDescriptor {
                            name: id.name.clone(),
                            kind: VariableKind::Let,
                            depth: 0,
                        });
                    }
                    destr_prologue.push((pat.clone(), i as u16, p.default.clone()));
                }
            }
        }
        // Emit per-parameter default-application + destructure prologue.
        for (pat, slot, default) in &destr_prologue {
            if let Some(def_expr) = default {
                // if args[slot] === undefined: args[slot] = default
                encode_op(&mut sub.bytecode, Op::LoadLocal);
                encode_u16(&mut sub.bytecode, *slot);
                encode_op(&mut sub.bytecode, Op::PushUndef);
                encode_op(&mut sub.bytecode, Op::StrictEq);
                let j_skip = sub.emit_jump(Op::JumpIfFalse);
                sub.compile_expr(def_expr)?;
                encode_op(&mut sub.bytecode, Op::StoreLocal);
                encode_u16(&mut sub.bytecode, *slot);
                sub.patch_jump(j_skip);
            }
            if !matches!(pat, rusty_js_ast::BindingPattern::Identifier(_)) {
                sub.emit_destructure(pat, *slot)?;
            }
        }
        for s in body { sub.compile_stmt(s)?; }
        encode_op(&mut sub.bytecode, Op::ReturnUndef);

        // Back-propagate any new upvalues the sub added to intermediate
        // enclosing frames. The innermost enclosing-of-sub is this proto
        // itself, so its upvalues -> self.upvalues. Even-outer frames -> self.enclosing[i].
        let mut frames = sub.enclosing;
        let inner = frames.pop().expect("sub had at least one enclosing");
        self.upvalues = inner.upvalues;
        for (i, ef) in frames.into_iter().enumerate() {
            self.enclosing[i].upvalues = ef.upvalues;
        }

        Ok(FunctionProto {
            bytecode: sub.bytecode,
            constants: sub.constants,
            params: param_count,
            locals: sub.locals,
            upvalues: sub.upvalues,
            rest_param_slot,
        })
    }

    fn record_span(&mut self, span: Span) {
        let off = self.bytecode.len();
        if self.source_map.last().map_or(true, |&(_, s)| s != span) {
            self.source_map.push((off, span));
        }
    }

    fn err(&self, span: Span, msg: &str) -> CompileError {
        CompileError { span, message: msg.to_string() }
    }

    // ───────────────── Tier-Ω.5.k: spread-argument lowering ─────────────────

    /// True if any argument is a spread (`...x`). Drives the choice between
    /// the direct Op::Call/Op::CallMethod/Op::New emit path and the
    /// __apply / __construct helper path.
    fn args_has_spread(arguments: &[Argument]) -> bool {
        arguments.iter().any(|a| matches!(a, Argument::Spread { .. }))
    }

    /// Emit code that builds a fresh Array containing the call arguments,
    /// with spread elements expanded via @@iterator. Stack delta: pushes
    /// one Array.
    fn emit_args_array(&mut self, arguments: &[Argument]) -> Result<(), CompileError> {
        encode_op(&mut self.bytecode, Op::NewArray);
        encode_u16(&mut self.bytecode, 0);
        let push_name = self.constants.intern(
            Constant::String("__array_push_single".to_string()));
        let extend_name = self.constants.intern(
            Constant::String("__array_extend".to_string()));
        for a in arguments {
            match a {
                Argument::Expr(expr) => {
                    // Pre: [.., arr]. Post: [.., arr].
                    encode_op(&mut self.bytecode, Op::LoadGlobal);
                    encode_u16(&mut self.bytecode, push_name);
                    encode_op(&mut self.bytecode, Op::Swap);
                    self.compile_expr(expr)?;
                    encode_op(&mut self.bytecode, Op::Call);
                    encode_u8(&mut self.bytecode, 2);
                }
                Argument::Spread { expr, .. } => {
                    encode_op(&mut self.bytecode, Op::LoadGlobal);
                    encode_u16(&mut self.bytecode, extend_name);
                    encode_op(&mut self.bytecode, Op::Swap);
                    self.compile_expr(expr)?;
                    encode_op(&mut self.bytecode, Op::Call);
                    encode_u8(&mut self.bytecode, 2);
                }
            }
        }
        Ok(())
    }

    // ───────────────── Tier-Ω.5.d: compound assignment + update ─────────────────

    /// Map a compound AssignOp (e.g. AddAssign) to its arithmetic/bitwise
    /// binary opcode. Returns None for the plain `=` form and for the three
    /// short-circuit logical/nullish variants, which are lowered separately.
    fn assign_op_binop(op: AssignOp) -> Option<Op> {
        Some(match op {
            AssignOp::AddAssign    => Op::Add,
            AssignOp::SubAssign    => Op::Sub,
            AssignOp::MulAssign    => Op::Mul,
            AssignOp::DivAssign    => Op::Div,
            AssignOp::ModAssign    => Op::Mod,
            AssignOp::PowAssign    => Op::Pow,
            AssignOp::ShlAssign    => Op::Shl,
            AssignOp::ShrAssign    => Op::Shr,
            AssignOp::UShrAssign   => Op::UShr,
            AssignOp::BitAndAssign => Op::BitAnd,
            AssignOp::BitOrAssign  => Op::BitOr,
            AssignOp::BitXorAssign => Op::BitXor,
            AssignOp::Assign
            | AssignOp::LogicalAndAssign
            | AssignOp::LogicalOrAssign
            | AssignOp::NullishAssign => return None,
        })
    }

    fn alloc_temp(&mut self, name: &str) -> u16 {
        self.alloc_local(LocalDescriptor {
            name: name.to_string(),
            kind: VariableKind::Let,
            depth: 0,
        })
    }

    /// Emit load/store for a bare identifier resolved against locals,
    /// upvalues, then globals (in that order).
    fn emit_load_ident(&mut self, name: &str) {
        if let Some(s) = self.resolve_local(name) {
            encode_op(&mut self.bytecode, Op::LoadLocal);
            encode_u16(&mut self.bytecode, s);
        } else if let Some(u) = self.resolve_upvalue(name) {
            encode_op(&mut self.bytecode, Op::LoadUpvalue);
            encode_u16(&mut self.bytecode, u);
        } else {
            let idx = self.constants.intern(Constant::String(name.to_string()));
            encode_op(&mut self.bytecode, Op::LoadGlobal);
            encode_u16(&mut self.bytecode, idx);
        }
    }

    fn emit_store_ident(&mut self, name: &str) {
        if let Some(s) = self.resolve_local(name) {
            encode_op(&mut self.bytecode, Op::StoreLocal);
            encode_u16(&mut self.bytecode, s);
        } else if let Some(u) = self.resolve_upvalue(name) {
            encode_op(&mut self.bytecode, Op::StoreUpvalue);
            encode_u16(&mut self.bytecode, u);
        } else {
            let idx = self.constants.intern(Constant::String(name.to_string()));
            encode_op(&mut self.bytecode, Op::StoreGlobal);
            encode_u16(&mut self.bytecode, idx);
        }
    }

    fn compile_assign(
        &mut self,
        span: Span,
        operator: AssignOp,
        target: &Expr,
        value: &Expr,
    ) -> Result<(), CompileError> {
        // ── Plain assignment: pre-existing semantics, fast path. ──
        if matches!(operator, AssignOp::Assign) {
            return self.compile_plain_assign(span, target, value);
        }

        // ── Logical / nullish: short-circuit lowering. ──
        if matches!(operator, AssignOp::LogicalAndAssign
                            | AssignOp::LogicalOrAssign
                            | AssignOp::NullishAssign) {
            return self.compile_logical_assign(span, operator, target, value);
        }

        // ── Arithmetic / bitwise compound: read-modify-write. ──
        let binop = Self::assign_op_binop(operator)
            .expect("non-logical compound assign must map to a binop");

        match target {
            Expr::Identifier { name, .. } => {
                self.emit_load_ident(name);          // [old]
                self.compile_expr(value)?;            // [old, v]
                encode_op(&mut self.bytecode, binop); // [new]
                encode_op(&mut self.bytecode, Op::Dup); // [new, new]
                self.emit_store_ident(name);          // [new]
            }
            Expr::Member { object, property, .. } => {
                self.compile_compound_member(span, &**object, property, value, binop)?;
            }
            _ => return Err(self.err(span, "complex assignment target not yet supported")),
        }
        Ok(())
    }

    fn compile_plain_assign(
        &mut self,
        span: Span,
        target: &Expr,
        value: &Expr,
    ) -> Result<(), CompileError> {
        match target {
            Expr::Identifier { name, .. } => {
                self.compile_expr(value)?;
                encode_op(&mut self.bytecode, Op::Dup);
                self.emit_store_ident(name);
            }
            Expr::Member { object, property, .. } => {
                self.compile_expr(object)?;
                match property.as_ref() {
                    MemberProperty::Identifier { name, .. } => {
                        self.compile_expr(value)?;
                        let idx = self.constants.intern(Constant::String(name.clone()));
                        encode_op(&mut self.bytecode, Op::SetProp);
                        encode_u16(&mut self.bytecode, idx);
                    }
                    MemberProperty::Computed { expr, .. } => {
                        self.compile_expr(expr)?;
                        self.compile_expr(value)?;
                        encode_op(&mut self.bytecode, Op::SetIndex);
                    }
                    MemberProperty::Private { name, .. } => {
                        self.compile_expr(value)?;
                        let idx = self.constants.intern(Constant::String(format!("#{}", name)));
                        encode_op(&mut self.bytecode, Op::SetProp);
                        encode_u16(&mut self.bytecode, idx);
                    }
                }
            }
            _ => return Err(self.err(span, "complex assignment target not yet supported")),
        }
        Ok(())
    }

    /// Compound assignment with a `MemberExpression` target. Spills the
    /// object (and, for computed/index, the key) into temporary locals so
    /// each sub-expression is evaluated exactly once.
    fn compile_compound_member(
        &mut self,
        span: Span,
        object: &Expr,
        property: &MemberProperty,
        value: &Expr,
        binop: Op,
    ) -> Result<(), CompileError> {
        let tmp_obj = self.alloc_temp("<compound.obj>");
        self.compile_expr(object)?;
        encode_op(&mut self.bytecode, Op::StoreLocal);
        encode_u16(&mut self.bytecode, tmp_obj);

        match property {
            MemberProperty::Identifier { name, .. } => {
                let key_idx = self.constants.intern(Constant::String(name.clone()));
                // read old
                encode_op(&mut self.bytecode, Op::LoadLocal);
                encode_u16(&mut self.bytecode, tmp_obj);
                encode_op(&mut self.bytecode, Op::GetProp);
                encode_u16(&mut self.bytecode, key_idx);
                // compute new
                self.compile_expr(value)?;
                encode_op(&mut self.bytecode, binop);
                // write: [obj, new] then SetProp → [new]
                let tmp_new = self.alloc_temp("<compound.new>");
                encode_op(&mut self.bytecode, Op::StoreLocal);
                encode_u16(&mut self.bytecode, tmp_new);
                encode_op(&mut self.bytecode, Op::LoadLocal);
                encode_u16(&mut self.bytecode, tmp_obj);
                encode_op(&mut self.bytecode, Op::LoadLocal);
                encode_u16(&mut self.bytecode, tmp_new);
                encode_op(&mut self.bytecode, Op::SetProp);
                encode_u16(&mut self.bytecode, key_idx);
            }
            MemberProperty::Computed { expr, .. } => {
                let tmp_key = self.alloc_temp("<compound.key>");
                self.compile_expr(expr)?;
                encode_op(&mut self.bytecode, Op::StoreLocal);
                encode_u16(&mut self.bytecode, tmp_key);
                // read old
                encode_op(&mut self.bytecode, Op::LoadLocal);
                encode_u16(&mut self.bytecode, tmp_obj);
                encode_op(&mut self.bytecode, Op::LoadLocal);
                encode_u16(&mut self.bytecode, tmp_key);
                encode_op(&mut self.bytecode, Op::GetIndex);
                // compute new
                self.compile_expr(value)?;
                encode_op(&mut self.bytecode, binop);
                let tmp_new = self.alloc_temp("<compound.new>");
                encode_op(&mut self.bytecode, Op::StoreLocal);
                encode_u16(&mut self.bytecode, tmp_new);
                // write
                encode_op(&mut self.bytecode, Op::LoadLocal);
                encode_u16(&mut self.bytecode, tmp_obj);
                encode_op(&mut self.bytecode, Op::LoadLocal);
                encode_u16(&mut self.bytecode, tmp_key);
                encode_op(&mut self.bytecode, Op::LoadLocal);
                encode_u16(&mut self.bytecode, tmp_new);
                encode_op(&mut self.bytecode, Op::SetIndex);
            }
            MemberProperty::Private { name, .. } => {
                let key_idx = self.constants.intern(Constant::String(format!("#{}", name)));
                encode_op(&mut self.bytecode, Op::LoadLocal);
                encode_u16(&mut self.bytecode, tmp_obj);
                encode_op(&mut self.bytecode, Op::GetProp);
                encode_u16(&mut self.bytecode, key_idx);
                self.compile_expr(value)?;
                encode_op(&mut self.bytecode, binop);
                let tmp_new = self.alloc_temp("<compound.new>");
                encode_op(&mut self.bytecode, Op::StoreLocal);
                encode_u16(&mut self.bytecode, tmp_new);
                encode_op(&mut self.bytecode, Op::LoadLocal);
                encode_u16(&mut self.bytecode, tmp_obj);
                encode_op(&mut self.bytecode, Op::LoadLocal);
                encode_u16(&mut self.bytecode, tmp_new);
                encode_op(&mut self.bytecode, Op::SetProp);
                encode_u16(&mut self.bytecode, key_idx);
            }
        }
        let _ = span;
        Ok(())
    }

    /// Logical / nullish compound assignment. Short-circuits: only
    /// evaluates the RHS and performs the store when the LHS reads as
    /// (truthy / falsy / nullish) appropriate for the operator.
    fn compile_logical_assign(
        &mut self,
        span: Span,
        operator: AssignOp,
        target: &Expr,
        value: &Expr,
    ) -> Result<(), CompileError> {
        // For an identifier target the lowering is:
        //
        //   LoadX                 [x]
        //   Dup                   [x, x]
        //   J<short-circuit> end  (pops top; keeps the other x as result on the
        //                          short-circuit branch)
        //   Pop                   []      (drop the kept copy; we'll replace)
        //   <eval value>          [v]
        //   Dup                   [v, v]
        //   StoreX                [v]
        //   end:                  [result]
        //
        // The trick: the `keep` jump opcodes (JumpIfTrueKeep/JumpIfFalseKeep)
        // keep on jump-taken and pop on fall-through. JumpIfNullish always
        // pops; for ??= we instead route via an unconditional Jump on the
        // not-nullish branch (matching the existing ?? lowering above).

        match target {
            Expr::Identifier { name, .. } => {
                self.emit_load_ident(name);
                encode_op(&mut self.bytecode, Op::Dup);
                let j_end = match operator {
                    AssignOp::LogicalAndAssign => {
                        // assign if truthy → short-circuit-end on falsy
                        Some(self.emit_jump(Op::JumpIfFalseKeep))
                    }
                    AssignOp::LogicalOrAssign => {
                        // assign if falsy → short-circuit-end on truthy
                        Some(self.emit_jump(Op::JumpIfTrueKeep))
                    }
                    AssignOp::NullishAssign => None, // handled below with custom flow
                    _ => unreachable!(),
                };

                if let Some(j) = j_end {
                    // assign branch
                    encode_op(&mut self.bytecode, Op::Pop);
                    self.compile_expr(value)?;
                    encode_op(&mut self.bytecode, Op::Dup);
                    self.emit_store_ident(name);
                    self.patch_jump(j);
                } else {
                    // NullishAssign: pattern matches the `??` operator in compile_expr.
                    //   [x, x] JumpIfNullish do_assign  (pops top)  → [x]
                    //   Jump end
                    //   do_assign: Pop → []; eval v; Dup; Store     → [v]
                    //   end:                                         → [result]
                    let j_assign = self.emit_jump(Op::JumpIfNullish);
                    let j_end2 = self.emit_jump(Op::Jump);
                    self.patch_jump(j_assign);
                    encode_op(&mut self.bytecode, Op::Pop);
                    self.compile_expr(value)?;
                    encode_op(&mut self.bytecode, Op::Dup);
                    self.emit_store_ident(name);
                    self.patch_jump(j_end2);
                }
            }
            Expr::Member { object, property, .. } => {
                self.compile_logical_assign_member(span, operator, object, property, value)?;
            }
            _ => return Err(self.err(span, "complex assignment target not yet supported")),
        }
        Ok(())
    }

    fn compile_logical_assign_member(
        &mut self,
        span: Span,
        operator: AssignOp,
        object: &Expr,
        property: &MemberProperty,
        value: &Expr,
    ) -> Result<(), CompileError> {
        // Spill object (and key) once, read old, branch, then write iff
        // the short-circuit predicate selects the assign path.
        let tmp_obj = self.alloc_temp("<lcompound.obj>");
        self.compile_expr(object)?;
        encode_op(&mut self.bytecode, Op::StoreLocal);
        encode_u16(&mut self.bytecode, tmp_obj);

        // After this block, the old-value is on the stack as the result
        // on the short-circuit (keep-old) path. We'll then branch.
        enum Key { Static(u16), Computed(u16), Private(u16) }
        let key = match property {
            MemberProperty::Identifier { name, .. } => {
                let idx = self.constants.intern(Constant::String(name.clone()));
                encode_op(&mut self.bytecode, Op::LoadLocal);
                encode_u16(&mut self.bytecode, tmp_obj);
                encode_op(&mut self.bytecode, Op::GetProp);
                encode_u16(&mut self.bytecode, idx);
                Key::Static(idx)
            }
            MemberProperty::Computed { expr, .. } => {
                let tmp_key = self.alloc_temp("<lcompound.key>");
                self.compile_expr(expr)?;
                encode_op(&mut self.bytecode, Op::StoreLocal);
                encode_u16(&mut self.bytecode, tmp_key);
                encode_op(&mut self.bytecode, Op::LoadLocal);
                encode_u16(&mut self.bytecode, tmp_obj);
                encode_op(&mut self.bytecode, Op::LoadLocal);
                encode_u16(&mut self.bytecode, tmp_key);
                encode_op(&mut self.bytecode, Op::GetIndex);
                Key::Computed(tmp_key)
            }
            MemberProperty::Private { name, .. } => {
                let idx = self.constants.intern(Constant::String(format!("#{}", name)));
                encode_op(&mut self.bytecode, Op::LoadLocal);
                encode_u16(&mut self.bytecode, tmp_obj);
                encode_op(&mut self.bytecode, Op::GetProp);
                encode_u16(&mut self.bytecode, idx);
                Key::Private(idx)
            }
        };
        // stack: [old]
        encode_op(&mut self.bytecode, Op::Dup);
        // stack: [old, old]

        let j_skip_assign = match operator {
            AssignOp::LogicalAndAssign => Some(self.emit_jump(Op::JumpIfFalseKeep)),
            AssignOp::LogicalOrAssign  => Some(self.emit_jump(Op::JumpIfTrueKeep)),
            AssignOp::NullishAssign    => None,
            _ => unreachable!(),
        };

        // Emit one "assign branch": pop the kept old copy, eval RHS, write
        // through the member. Leaves the new value on the stack.
        let emit_assign_branch = |c: &mut Self, value: &Expr, key: &Key, tmp_obj: u16| -> Result<(), CompileError> {
            encode_op(&mut c.bytecode, Op::Pop);
            c.compile_expr(value)?;
            let tmp_new = c.alloc_temp("<lcompound.new>");
            encode_op(&mut c.bytecode, Op::StoreLocal);
            encode_u16(&mut c.bytecode, tmp_new);
            match key {
                Key::Static(idx) => {
                    encode_op(&mut c.bytecode, Op::LoadLocal);
                    encode_u16(&mut c.bytecode, tmp_obj);
                    encode_op(&mut c.bytecode, Op::LoadLocal);
                    encode_u16(&mut c.bytecode, tmp_new);
                    encode_op(&mut c.bytecode, Op::SetProp);
                    encode_u16(&mut c.bytecode, *idx);
                }
                Key::Computed(tmp_key) => {
                    encode_op(&mut c.bytecode, Op::LoadLocal);
                    encode_u16(&mut c.bytecode, tmp_obj);
                    encode_op(&mut c.bytecode, Op::LoadLocal);
                    encode_u16(&mut c.bytecode, *tmp_key);
                    encode_op(&mut c.bytecode, Op::LoadLocal);
                    encode_u16(&mut c.bytecode, tmp_new);
                    encode_op(&mut c.bytecode, Op::SetIndex);
                }
                Key::Private(idx) => {
                    encode_op(&mut c.bytecode, Op::LoadLocal);
                    encode_u16(&mut c.bytecode, tmp_obj);
                    encode_op(&mut c.bytecode, Op::LoadLocal);
                    encode_u16(&mut c.bytecode, tmp_new);
                    encode_op(&mut c.bytecode, Op::SetProp);
                    encode_u16(&mut c.bytecode, *idx);
                }
            }
            Ok(())
        };

        if let Some(j) = j_skip_assign {
            emit_assign_branch(self, value, &key, tmp_obj)?;
            self.patch_jump(j);
        } else {
            let j_assign = self.emit_jump(Op::JumpIfNullish);
            let j_end = self.emit_jump(Op::Jump);
            self.patch_jump(j_assign);
            emit_assign_branch(self, value, &key, tmp_obj)?;
            self.patch_jump(j_end);
        }
        let _ = span;
        Ok(())
    }

    /// Compile a prefix or postfix update expression. Handles identifier,
    /// static member, computed member, and private member targets.
    fn compile_update(
        &mut self,
        span: Span,
        operator: UpdateOp,
        argument: &Expr,
        prefix: bool,
    ) -> Result<(), CompileError> {
        let op = match operator {
            UpdateOp::Inc => Op::Inc,
            UpdateOp::Dec => Op::Dec,
        };
        match argument {
            Expr::Identifier { name, .. } => {
                self.emit_load_ident(name);              // [old]
                if !prefix {
                    encode_op(&mut self.bytecode, Op::Dup); // [old, old]
                }
                encode_op(&mut self.bytecode, op);        // prefix:[new]  postfix:[old, new]
                if prefix {
                    encode_op(&mut self.bytecode, Op::Dup); // [new, new]
                }
                // Store consumes top: prefix leaves [new]; postfix leaves [old].
                self.emit_store_ident(name);
            }
            Expr::Member { object, property, .. } => {
                let tmp_obj = self.alloc_temp("<update.obj>");
                self.compile_expr(object)?;
                encode_op(&mut self.bytecode, Op::StoreLocal);
                encode_u16(&mut self.bytecode, tmp_obj);

                match property.as_ref() {
                    MemberProperty::Identifier { name, .. } => {
                        let key_idx = self.constants.intern(Constant::String(name.clone()));
                        encode_op(&mut self.bytecode, Op::LoadLocal);
                        encode_u16(&mut self.bytecode, tmp_obj);
                        encode_op(&mut self.bytecode, Op::GetProp);
                        encode_u16(&mut self.bytecode, key_idx);
                        // [old]
                        let tmp_old = self.alloc_temp("<update.old>");
                        if !prefix {
                            encode_op(&mut self.bytecode, Op::Dup);
                            encode_op(&mut self.bytecode, Op::StoreLocal);
                            encode_u16(&mut self.bytecode, tmp_old);
                        }
                        encode_op(&mut self.bytecode, op); // [new]
                        let tmp_new = self.alloc_temp("<update.new>");
                        encode_op(&mut self.bytecode, Op::StoreLocal);
                        encode_u16(&mut self.bytecode, tmp_new);
                        // write through member
                        encode_op(&mut self.bytecode, Op::LoadLocal);
                        encode_u16(&mut self.bytecode, tmp_obj);
                        encode_op(&mut self.bytecode, Op::LoadLocal);
                        encode_u16(&mut self.bytecode, tmp_new);
                        encode_op(&mut self.bytecode, Op::SetProp);
                        encode_u16(&mut self.bytecode, key_idx);
                        // SetProp pushes new; drop it and load expression result.
                        encode_op(&mut self.bytecode, Op::Pop);
                        encode_op(&mut self.bytecode, Op::LoadLocal);
                        encode_u16(&mut self.bytecode, if prefix { tmp_new } else { tmp_old });
                    }
                    MemberProperty::Computed { expr, .. } => {
                        let tmp_key = self.alloc_temp("<update.key>");
                        self.compile_expr(expr)?;
                        encode_op(&mut self.bytecode, Op::StoreLocal);
                        encode_u16(&mut self.bytecode, tmp_key);

                        encode_op(&mut self.bytecode, Op::LoadLocal);
                        encode_u16(&mut self.bytecode, tmp_obj);
                        encode_op(&mut self.bytecode, Op::LoadLocal);
                        encode_u16(&mut self.bytecode, tmp_key);
                        encode_op(&mut self.bytecode, Op::GetIndex);
                        // [old]
                        let tmp_old = self.alloc_temp("<update.old>");
                        if !prefix {
                            encode_op(&mut self.bytecode, Op::Dup);
                            encode_op(&mut self.bytecode, Op::StoreLocal);
                            encode_u16(&mut self.bytecode, tmp_old);
                        }
                        encode_op(&mut self.bytecode, op);
                        let tmp_new = self.alloc_temp("<update.new>");
                        encode_op(&mut self.bytecode, Op::StoreLocal);
                        encode_u16(&mut self.bytecode, tmp_new);
                        encode_op(&mut self.bytecode, Op::LoadLocal);
                        encode_u16(&mut self.bytecode, tmp_obj);
                        encode_op(&mut self.bytecode, Op::LoadLocal);
                        encode_u16(&mut self.bytecode, tmp_key);
                        encode_op(&mut self.bytecode, Op::LoadLocal);
                        encode_u16(&mut self.bytecode, tmp_new);
                        encode_op(&mut self.bytecode, Op::SetIndex);
                        encode_op(&mut self.bytecode, Op::Pop);
                        encode_op(&mut self.bytecode, Op::LoadLocal);
                        encode_u16(&mut self.bytecode, if prefix { tmp_new } else { tmp_old });
                    }
                    MemberProperty::Private { name, .. } => {
                        let key_idx = self.constants.intern(Constant::String(format!("#{}", name)));
                        encode_op(&mut self.bytecode, Op::LoadLocal);
                        encode_u16(&mut self.bytecode, tmp_obj);
                        encode_op(&mut self.bytecode, Op::GetProp);
                        encode_u16(&mut self.bytecode, key_idx);
                        let tmp_old = self.alloc_temp("<update.old>");
                        if !prefix {
                            encode_op(&mut self.bytecode, Op::Dup);
                            encode_op(&mut self.bytecode, Op::StoreLocal);
                            encode_u16(&mut self.bytecode, tmp_old);
                        }
                        encode_op(&mut self.bytecode, op);
                        let tmp_new = self.alloc_temp("<update.new>");
                        encode_op(&mut self.bytecode, Op::StoreLocal);
                        encode_u16(&mut self.bytecode, tmp_new);
                        encode_op(&mut self.bytecode, Op::LoadLocal);
                        encode_u16(&mut self.bytecode, tmp_obj);
                        encode_op(&mut self.bytecode, Op::LoadLocal);
                        encode_u16(&mut self.bytecode, tmp_new);
                        encode_op(&mut self.bytecode, Op::SetProp);
                        encode_u16(&mut self.bytecode, key_idx);
                        encode_op(&mut self.bytecode, Op::Pop);
                        encode_op(&mut self.bytecode, Op::LoadLocal);
                        encode_u16(&mut self.bytecode, if prefix { tmp_new } else { tmp_old });
                    }
                }
            }
            _ => return Err(self.err(span, "update on non-identifier non-member target not yet supported")),
        }
        Ok(())
    }

    // ───────────────── Tier-Ω.5.f: class lowering ─────────────────

    /// Lower a class declaration / expression. Leaves the class's
    /// constructor function on the operand stack.
    ///
    /// Strategy: a class is sugar over function + prototype + property
    /// installation. The class body emits:
    ///   1. (if extends) evaluate super-class, stash in a hidden local
    ///      `<super.ctor>` and the super prototype in `<super.proto>`.
    ///   2. Allocate the prototype object; if extends, wire its
    ///      [[Prototype]] to the parent's prototype via SetPrototype.
    ///   3. Build the constructor closure (default no-op if absent).
    ///      Bind to a hidden local so methods that capture super can
    ///      land. Wire ctor.prototype = <proto>; proto.constructor = ctor.
    ///      If extends, wire ctor.[[Prototype]] = super-ctor for static
    ///      inheritance.
    ///   4. Install each instance / static method onto its target with
    ///      ClassFrame pushed so super-references resolve to the
    ///      synthesized outer-local names.
    ///
    /// Method-shorthand `super.method` / `super(...)` references in the
    /// method body resolve via the existing upvalue machinery — the
    /// synthesized outer-local names are real entries in `self.locals`,
    /// and the sub-compiler captures them as upvalues per Tier-Ω.5.c.
    fn compile_class(
        &mut self,
        span: Span,
        _name: Option<&BindingIdentifier>,
        super_class: Option<&Expr>,
        members: &[ClassMember],
    ) -> Result<(), CompileError> {
        let seq = self.class_seq;
        self.class_seq += 1;

        let super_ctor_name = format!("<class${}.super.ctor>", seq);
        let super_proto_name = format!("<class${}.super.proto>", seq);
        let proto_name = format!("<class${}.proto>", seq);
        let ctor_name = format!("<class${}.ctor>", seq);

        // ── 1. extends evaluation ──────────────────────────────────
        let super_ctor_slot = if let Some(sc) = super_class {
            let slot = self.alloc_temp(&super_ctor_name);
            self.compile_expr(sc)?;
            encode_op(&mut self.bytecode, Op::StoreLocal);
            encode_u16(&mut self.bytecode, slot);
            // <super.proto> = <super.ctor>.prototype
            let proto_slot = self.alloc_temp(&super_proto_name);
            encode_op(&mut self.bytecode, Op::LoadLocal);
            encode_u16(&mut self.bytecode, slot);
            let key_proto = self.constants.intern(Constant::String("prototype".into()));
            encode_op(&mut self.bytecode, Op::GetProp);
            encode_u16(&mut self.bytecode, key_proto);
            encode_op(&mut self.bytecode, Op::StoreLocal);
            encode_u16(&mut self.bytecode, proto_slot);
            Some((slot, proto_slot))
        } else {
            None
        };

        // ── 2. prototype object allocation + extends-wiring ────────
        let proto_slot = self.alloc_temp(&proto_name);
        encode_op(&mut self.bytecode, Op::NewObject);
        encode_op(&mut self.bytecode, Op::StoreLocal);
        encode_u16(&mut self.bytecode, proto_slot);
        if let Some((_sc, sp)) = super_ctor_slot {
            encode_op(&mut self.bytecode, Op::LoadLocal);
            encode_u16(&mut self.bytecode, proto_slot);
            encode_op(&mut self.bytecode, Op::LoadLocal);
            encode_u16(&mut self.bytecode, sp);
            encode_op(&mut self.bytecode, Op::SetPrototype);
        }

        // ── 3. constructor closure ─────────────────────────────────
        //
        // Find an explicit `constructor` member, else synthesize a no-op.
        let mut ctor_params: Vec<Parameter> = Vec::new();
        let mut ctor_body: Vec<Stmt> = Vec::new();
        let mut has_explicit_ctor = false;
        for m in members {
            if let ClassMember::Method { kind: MethodKind::Constructor, params, body, .. } = m {
                ctor_params = params.clone();
                ctor_body = body.clone();
                has_explicit_ctor = true;
                break;
            }
        }

        // Tier-Ω.5.o: synthesize `this.<name> = <init>` statements from
        // instance Field members. Insert at the START of the constructor
        // body. For derived classes without an explicit constructor, also
        // synthesize `super(...args)` ahead of field inits so the parent
        // constructor (and its own field inits) runs first.
        let mut field_init_stmts: Vec<Stmt> = Vec::new();
        for m in members {
            if let ClassMember::Field { name: f_name, is_static, init, span: f_span } = m {
                if *is_static { continue; }
                let key_expr_prop: MemberProperty = match f_name {
                    ClassMemberName::Identifier { name, span } =>
                        MemberProperty::Identifier { name: name.clone(), span: *span },
                    ClassMemberName::String { value, span } =>
                        MemberProperty::Computed {
                            expr: Expr::StringLiteral { value: value.clone(), span: *span },
                            span: *span,
                        },
                    ClassMemberName::Number { value, span } =>
                        MemberProperty::Computed {
                            expr: Expr::NumberLiteral { value: *value, span: *span },
                            span: *span,
                        },
                    ClassMemberName::Computed { expr, span } =>
                        MemberProperty::Computed { expr: expr.clone(), span: *span },
                    ClassMemberName::Private { span, .. } => {
                        return Err(self.err(*span,
                            "private class fields (#x) not yet supported (deferred from Tier-Ω.5.f scope ceiling; see private-fields v1 substrate task)"));
                    }
                };
                let target = Expr::Member {
                    object: Box::new(Expr::This { span: *f_span }),
                    property: Box::new(key_expr_prop),
                    optional: false,
                    span: *f_span,
                };
                let value = match init {
                    Some(e) => e.clone(),
                    None => Expr::Identifier { name: "undefined".to_string(), span: *f_span },
                };
                let assign = Expr::Assign {
                    operator: AssignOp::Assign,
                    target: Box::new(target),
                    value: Box::new(value),
                    span: *f_span,
                };
                field_init_stmts.push(Stmt::Expression { expr:assign, span: *f_span });
            }
        }
        if !has_explicit_ctor && super_class.is_some() {
            // Synthesize `constructor(...__args) { super(...__args); <fields>; }`.
            let s = span;
            let args_id = BindingIdentifier { name: "__args".to_string(), span: s };
            ctor_params = vec![Parameter {
                target: BindingPattern::Identifier(args_id.clone()),
                default: None,
                rest: true,
                span: s,
            }];
            let super_call = Expr::Call {
                callee: Box::new(Expr::Super { span: s }),
                arguments: vec![Argument::Spread {
                    expr: Expr::Identifier { name: "__args".to_string(), span: s },
                    span: s,
                }],
                optional: false,
                span: s,
            };
            let mut synth: Vec<Stmt> = Vec::new();
            synth.push(Stmt::Expression { expr:super_call, span: s });
            synth.extend(field_init_stmts.clone());
            ctor_body = synth;
        } else if !field_init_stmts.is_empty() {
            // Prepend field inits to existing (or empty) body.
            let mut new_body: Vec<Stmt> = field_init_stmts.clone();
            new_body.extend(ctor_body.into_iter());
            ctor_body = new_body;
        }

        // Push class context for the constructor body.
        self.class_stack.push(ClassFrame {
            super_ctor_name: super_ctor_slot.map(|_| super_ctor_name.clone()),
            super_proto_name: super_ctor_slot.map(|_| super_proto_name.clone()),
            in_constructor: true,
            is_static: false,
        });
        let ctor_proto = self.compile_function_proto(None, false, false, &ctor_params, &ctor_body)?;
        self.class_stack.pop();
        let ctor_captures = ctor_proto.upvalues.clone();
        let ctor_idx = self.constants.intern(Constant::Function(Box::new(ctor_proto)));
        encode_op(&mut self.bytecode, Op::MakeClosure);
        encode_u16(&mut self.bytecode, ctor_idx);
        emit_captures(&mut self.bytecode, &ctor_captures);
        let ctor_slot = self.alloc_temp(&ctor_name);
        encode_op(&mut self.bytecode, Op::StoreLocal);
        encode_u16(&mut self.bytecode, ctor_slot);

        // ctor.prototype = <proto>
        let key_proto = self.constants.intern(Constant::String("prototype".into()));
        encode_op(&mut self.bytecode, Op::LoadLocal);
        encode_u16(&mut self.bytecode, ctor_slot);
        encode_op(&mut self.bytecode, Op::LoadLocal);
        encode_u16(&mut self.bytecode, proto_slot);
        encode_op(&mut self.bytecode, Op::SetProp);
        encode_u16(&mut self.bytecode, key_proto);
        encode_op(&mut self.bytecode, Op::Pop);

        // <proto>.constructor = ctor
        let key_constructor = self.constants.intern(Constant::String("constructor".into()));
        encode_op(&mut self.bytecode, Op::LoadLocal);
        encode_u16(&mut self.bytecode, proto_slot);
        encode_op(&mut self.bytecode, Op::LoadLocal);
        encode_u16(&mut self.bytecode, ctor_slot);
        encode_op(&mut self.bytecode, Op::SetProp);
        encode_u16(&mut self.bytecode, key_constructor);
        encode_op(&mut self.bytecode, Op::Pop);

        // ctor.[[Prototype]] = <super.ctor> for static-method inheritance.
        if let Some((sc, _sp)) = super_ctor_slot {
            encode_op(&mut self.bytecode, Op::LoadLocal);
            encode_u16(&mut self.bytecode, ctor_slot);
            encode_op(&mut self.bytecode, Op::LoadLocal);
            encode_u16(&mut self.bytecode, sc);
            encode_op(&mut self.bytecode, Op::SetPrototype);
        }

        // ── 4. methods ─────────────────────────────────────────────
        for m in members {
            match m {
                ClassMember::Method { kind, params, body, name: m_name, is_static, is_async, is_generator, span: m_span } => {
                    if matches!(kind, MethodKind::Constructor) { continue; }
                    if !matches!(kind, MethodKind::Method) {
                        return Err(self.err(*m_span,
                            "getter / setter class members not yet supported"));
                    }
                    if *is_async || *is_generator {
                        return Err(self.err(*m_span,
                            "async / generator class methods not yet supported"));
                    }
                    let method_key = match m_name {
                        ClassMemberName::Identifier { name, .. } => name.clone(),
                        ClassMemberName::String { value, .. } => value.clone(),
                        ClassMemberName::Number { value, .. } => {
                            if value.fract() == 0.0 { format!("{}", *value as i64) }
                            else { format!("{}", value) }
                        }
                        ClassMemberName::Private { .. } | ClassMemberName::Computed { .. } => {
                            return Err(self.err(*m_span,
                                "private / computed class member names not yet supported"));
                        }
                    };

                    // Push class context: not the constructor, so super(...)
                    // is forbidden inside the method; super.x is allowed
                    // and resolves through the prototype.
                    self.class_stack.push(ClassFrame {
                        super_ctor_name: super_ctor_slot.map(|_| super_ctor_name.clone()),
                        super_proto_name: super_ctor_slot.map(|_| super_proto_name.clone()),
                        in_constructor: false,
                        is_static: *is_static,
                    });
                    let m_proto = self.compile_function_proto(None, false, false, params, body)?;
                    self.class_stack.pop();
                    let captures = m_proto.upvalues.clone();
                    let m_idx = self.constants.intern(Constant::Function(Box::new(m_proto)));

                    // Push the target object on the stack first, then the
                    // method closure, then SetProp.
                    let target_slot = if *is_static { ctor_slot } else { proto_slot };
                    encode_op(&mut self.bytecode, Op::LoadLocal);
                    encode_u16(&mut self.bytecode, target_slot);
                    encode_op(&mut self.bytecode, Op::MakeClosure);
                    encode_u16(&mut self.bytecode, m_idx);
                    emit_captures(&mut self.bytecode, &captures);
                    let key_idx = self.constants.intern(Constant::String(method_key));
                    encode_op(&mut self.bytecode, Op::SetProp);
                    encode_u16(&mut self.bytecode, key_idx);
                    encode_op(&mut self.bytecode, Op::Pop);
                }
                ClassMember::Field { name: f_name, is_static, init, span: f_span } => {
                    // Tier-Ω.5.o: instance fields were folded into the
                    // constructor body above. Static fields run once at
                    // class-definition time: lower as `ctor.<key> = init`.
                    if !*is_static { continue; }
                    // Reject private static fields with the same clearer
                    // error as instance private fields.
                    if let ClassMemberName::Private { span: p_span, .. } = f_name {
                        return Err(self.err(*p_span,
                            "private class fields (#x) not yet supported (deferred from Tier-Ω.5.f scope ceiling; see private-fields v1 substrate task)"));
                    }
                    encode_op(&mut self.bytecode, Op::LoadLocal);
                    encode_u16(&mut self.bytecode, ctor_slot);
                    match init {
                        Some(e) => self.compile_expr(e)?,
                        None => { encode_op(&mut self.bytecode, Op::PushUndef); }
                    }
                    match f_name {
                        ClassMemberName::Identifier { name, .. }
                        | ClassMemberName::String { value: name, .. } => {
                            let idx = self.constants.intern(Constant::String(name.clone()));
                            encode_op(&mut self.bytecode, Op::SetProp);
                            encode_u16(&mut self.bytecode, idx);
                        }
                        ClassMemberName::Number { value, .. } => {
                            let name = if value.fract() == 0.0 {
                                format!("{}", *value as i64)
                            } else { format!("{}", value) };
                            let idx = self.constants.intern(Constant::String(name));
                            encode_op(&mut self.bytecode, Op::SetProp);
                            encode_u16(&mut self.bytecode, idx);
                        }
                        ClassMemberName::Computed { span: c_span, .. } => {
                            return Err(self.err(*c_span,
                                "computed class field names not yet supported"));
                        }
                        ClassMemberName::Private { .. } => unreachable!(),
                    }
                    encode_op(&mut self.bytecode, Op::Pop);
                    let _ = f_span;
                }
                ClassMember::StaticBlock { span: b_span, .. } => {
                    return Err(self.err(*b_span,
                        "static initializer blocks not yet supported (Tier-Ω.5.f scope ceiling)"));
                }
            }
        }

        // ── result: leave the constructor on the stack ─────────────
        encode_op(&mut self.bytecode, Op::LoadLocal);
        encode_u16(&mut self.bytecode, ctor_slot);
        let _ = span;
        Ok(())
    }

    /// Lower `super(args...)` inside a derived-class constructor body.
    /// Emits a method-call on the parent constructor with the current
    /// `this` as receiver. The result is left on the stack (Pop'd by
    /// the surrounding ExpressionStatement).
    fn compile_super_call(
        &mut self,
        span: Span,
        arguments: &[Argument],
    ) -> Result<(), CompileError> {
        let frame = self.class_stack.last().cloned()
            .ok_or_else(|| self.err(span, "super(...) outside of a class"))?;
        if !frame.in_constructor {
            return Err(self.err(span,
                "super(...) is only valid inside a derived-class constructor"));
        }
        let super_ctor_name = frame.super_ctor_name.clone().ok_or_else(|| {
            self.err(span,
                "super(...) used in a class with no `extends` clause")
        })?;
        let n = arguments.len();
        if n > 255 {
            return Err(self.err(span, "too many super-call arguments (>255)"));
        }
        if Self::args_has_spread(arguments) {
            // Tier-Ω.5.k: spread super(...) → __apply(super_ctor, this, args).
            let apply_name = self.constants.intern(
                Constant::String("__apply".to_string()));
            encode_op(&mut self.bytecode, Op::LoadGlobal);
            encode_u16(&mut self.bytecode, apply_name);
            self.emit_load_ident(&super_ctor_name);
            encode_op(&mut self.bytecode, Op::PushThis);
            self.emit_args_array(arguments)?;
            encode_op(&mut self.bytecode, Op::Call);
            encode_u8(&mut self.bytecode, 3);
        } else {
            // Receiver = current `this`.
            encode_op(&mut self.bytecode, Op::PushThis);
            // Method = parent constructor.
            self.emit_load_ident(&super_ctor_name);
            for a in arguments {
                match a {
                    Argument::Expr(e) => self.compile_expr(e)?,
                    Argument::Spread { .. } => unreachable!(),
                }
            }
            encode_op(&mut self.bytecode, Op::CallMethod);
            encode_u8(&mut self.bytecode, n as u8);
        }
        Ok(())
    }

    /// Lower `super.x` (bare read) inside a class method body. Resolves
    /// against the parent prototype (instance methods) or the parent
    /// constructor (static methods).
    fn compile_super_member_load(
        &mut self,
        span: Span,
        property: &MemberProperty,
    ) -> Result<(), CompileError> {
        let frame = self.class_stack.last().cloned()
            .ok_or_else(|| self.err(span, "super reference outside of a class"))?;
        let target_name = if frame.is_static {
            frame.super_ctor_name.clone()
        } else {
            frame.super_proto_name.clone()
        }.ok_or_else(|| self.err(span,
            "super reference in a class with no `extends` clause"))?;
        self.emit_load_ident(&target_name);
        match property {
            MemberProperty::Identifier { name, .. } => {
                let idx = self.constants.intern(Constant::String(name.clone()));
                encode_op(&mut self.bytecode, Op::GetProp);
                encode_u16(&mut self.bytecode, idx);
            }
            MemberProperty::Computed { expr, .. } => {
                self.compile_expr(expr)?;
                encode_op(&mut self.bytecode, Op::GetIndex);
            }
            MemberProperty::Private { name, .. } => {
                let idx = self.constants.intern(Constant::String(format!("#{}", name)));
                encode_op(&mut self.bytecode, Op::GetProp);
                encode_u16(&mut self.bytecode, idx);
            }
        }
        Ok(())
    }

    /// Lower `super.method(args...)` — a super member-call with the
    /// current `this` as receiver. The method lookup goes through the
    /// parent prototype (instance) or constructor (static).
    fn compile_super_member_call(
        &mut self,
        span: Span,
        property: &MemberProperty,
        arguments: &[Argument],
    ) -> Result<(), CompileError> {
        let n = arguments.len();
        if n > 255 {
            return Err(self.err(span, "too many super-call arguments (>255)"));
        }
        if Self::args_has_spread(arguments) {
            // Tier-Ω.5.k: spread super.m(...) → __apply(method, this, args).
            let apply_name = self.constants.intern(
                Constant::String("__apply".to_string()));
            encode_op(&mut self.bytecode, Op::LoadGlobal);
            encode_u16(&mut self.bytecode, apply_name);
            self.compile_super_member_load(span, property)?;
            encode_op(&mut self.bytecode, Op::PushThis);
            self.emit_args_array(arguments)?;
            encode_op(&mut self.bytecode, Op::Call);
            encode_u8(&mut self.bytecode, 3);
        } else {
            // Receiver = current `this`.
            encode_op(&mut self.bytecode, Op::PushThis);
            // Method = (parent prototype | parent ctor) [.property].
            self.compile_super_member_load(span, property)?;
            for a in arguments {
                match a {
                    Argument::Expr(e) => self.compile_expr(e)?,
                    Argument::Spread { .. } => unreachable!(),
                }
            }
            encode_op(&mut self.bytecode, Op::CallMethod);
            encode_u8(&mut self.bytecode, n as u8);
        }
        Ok(())
    }
}
