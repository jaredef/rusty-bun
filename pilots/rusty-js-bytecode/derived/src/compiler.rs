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
}

#[derive(Debug, Clone)]
pub struct CompiledModule {
    pub bytecode: Vec<u8>,
    pub constants: ConstantsPool,
    pub locals: Vec<LocalDescriptor>,
    pub source_map: Vec<(usize, Span)>,
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
    /// Tier-Ω.5.c: each enclosing-function level's locals + accumulated
    /// upvalues, walked when resolving identifiers inside nested functions.
    /// Innermost outer is at the back. Empty at the top-level module.
    enclosing: Vec<EnclosingFrame>,
    /// This proto's own upvalue descriptors (only meaningful when this
    /// Compiler is compiling a nested function, i.e. enclosing.is_empty()
    /// is false).
    upvalues: Vec<UpvalueDescriptor>,
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
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            bytecode: Vec::new(),
            constants: ConstantsPool::new(),
            locals: Vec::new(),
            source_map: Vec::new(),
            loop_stack: Vec::new(),
            enclosing: Vec::new(),
            upvalues: Vec::new(),
        }
    }

    pub fn compile_module(&mut self, m: &Module) -> Result<CompiledModule, CompileError> {
        for item in &m.body {
            match item {
                ModuleItem::Import(_) | ModuleItem::Export(_) => {
                    // Import/export entries are recorded at link time; the
                    // bytecode unit doesn't emit linkage opcodes in v1.
                }
                ModuleItem::Statement(s) => self.compile_stmt(s)?,
            }
        }
        encode_op(&mut self.bytecode, Op::ReturnUndef);
        Ok(CompiledModule {
            bytecode: std::mem::take(&mut self.bytecode),
            constants: std::mem::take(&mut self.constants),
            locals: std::mem::take(&mut self.locals),
            source_map: std::mem::take(&mut self.source_map),
        })
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
                    if d.names.len() != 1 {
                        return Err(self.err(d.span, "destructure declarators not yet supported"));
                    }
                    let name = &d.names[0];
                    // Allocate a local slot for the binding.
                    let slot = self.alloc_local(LocalDescriptor {
                        name: name.name.clone(),
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
                    continue_patches: Vec::new(), break_patches: Vec::new(),
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
                    continue_patches: Vec::new(), break_patches: Vec::new(),
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
                    continue_patches: Vec::new(), break_patches: Vec::new(),
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
                let (bind_slot, _bind_name) = match left {
                    rusty_js_ast::ForBinding::Decl { kind, name, .. } => {
                        let s = self.alloc_local(LocalDescriptor {
                            name: name.name.clone(), kind: *kind, depth: 0,
                        });
                        (s, name.name.clone())
                    }
                    rusty_js_ast::ForBinding::Identifier(id) => {
                        if let Some(s) = self.resolve_local(&id.name) { (s, id.name.clone()) }
                        else {
                            let s = self.alloc_local(LocalDescriptor {
                                name: id.name.clone(), kind: VariableKind::Let, depth: 0,
                            });
                            (s, id.name.clone())
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
                    continue_patches: Vec::new(), break_patches: Vec::new(),
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
                encode_op(&mut self.bytecode, Op::StoreLocal);
                encode_u16(&mut self.bytecode, bind_slot);
                self.compile_stmt(body)?;
                self.emit_back_jump(loop_start);
                self.patch_jump(j_done);
                // At the exit, the result object is on the stack — pop it.
                encode_op(&mut self.bytecode, Op::Pop);
                let frame = self.loop_stack.pop().unwrap();
                for site in frame.break_patches { self.patch_jump_at(site); }
            }
            Stmt::Break { label, .. } => {
                if label.is_some() {
                    return Err(self.err(span, "labelled break not yet supported"));
                }
                if let Some(frame) = self.loop_stack.last_mut() {
                    let patch_site = encode_op(&mut self.bytecode, Op::Jump);
                    encode_i32(&mut self.bytecode, 0);
                    frame.break_patches.push(patch_site);
                } else {
                    return Err(self.err(span, "break outside of loop"));
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
                if label.is_some() {
                    return Err(self.err(span, "labelled continue not yet supported"));
                }
                if self.loop_stack.is_empty() {
                    return Err(self.err(span, "continue outside of loop"));
                }
                let pending = self.loop_stack.last().unwrap().continue_pending;
                if pending {
                    // Record patch site; will be filled when continue_target
                    // is finalized.
                    let patch_site = encode_op(&mut self.bytecode, Op::Jump);
                    encode_i32(&mut self.bytecode, 0);
                    self.loop_stack.last_mut().unwrap().continue_patches.push(patch_site);
                } else {
                    let target = self.loop_stack.last().unwrap().continue_target;
                    self.emit_back_jump(target);
                }
            }
            _ => {
                return Err(self.err(span, "statement form not yet supported in compiler v1"));
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
            Expr::Assign { operator, target, value, .. } => {
                if !matches!(operator, AssignOp::Assign) {
                    return Err(self.err(e.span(), "compound assignment not yet supported"));
                }
                match target.as_ref() {
                    Expr::Identifier { name, .. } => {
                        self.compile_expr(value)?;
                        encode_op(&mut self.bytecode, Op::Dup);
                        if let Some(slot) = self.resolve_local(name) {
                            encode_op(&mut self.bytecode, Op::StoreLocal);
                            encode_u16(&mut self.bytecode, slot);
                        } else if let Some(up) = self.resolve_upvalue(name) {
                            encode_op(&mut self.bytecode, Op::StoreUpvalue);
                            encode_u16(&mut self.bytecode, up);
                        } else {
                            let idx = self.constants.intern(Constant::String(name.clone()));
                            encode_op(&mut self.bytecode, Op::StoreGlobal);
                            encode_u16(&mut self.bytecode, idx);
                        }
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
                    _ => return Err(self.err(e.span(), "complex assignment target not yet supported")),
                }
            }
            Expr::This { .. } => {
                // Tier-Ω.5.a: this now threads through the frame.
                encode_op(&mut self.bytecode, Op::PushThis);
            }
            Expr::Member { object, property, optional: _, .. } => {
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
                // Tier-Ω.5.a: when callee is a MemberExpression, emit a
                // method-call form so `this` threads as the receiver.
                let is_method = matches!(callee.as_ref(), Expr::Member { .. });
                if is_method {
                    if let Expr::Member { object, property, optional: _, .. } = callee.as_ref() {
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
                                Argument::Spread { .. } => {
                                    return Err(self.err(e.span(), "spread arguments not yet supported"));
                                }
                            }
                        }
                        encode_op(&mut self.bytecode, Op::CallMethod);
                        encode_u8(&mut self.bytecode, n as u8);
                    }
                } else {
                    self.compile_expr(callee)?;
                    for a in arguments {
                        match a {
                            Argument::Expr(e) => self.compile_expr(e)?,
                            Argument::Spread { .. } => {
                                return Err(self.err(e.span(), "spread arguments not yet supported"));
                            }
                        }
                    }
                    encode_op(&mut self.bytecode, Op::Call);
                    encode_u8(&mut self.bytecode, n as u8);
                }
            }
            Expr::New { callee, arguments, .. } => {
                self.compile_expr(callee)?;
                let n = arguments.len();
                if n > 255 {
                    return Err(self.err(e.span(), "too many new arguments (>255)"));
                }
                for a in arguments {
                    match a {
                        Argument::Expr(e) => self.compile_expr(e)?,
                        Argument::Spread { .. } => {
                            return Err(self.err(e.span(), "spread arguments not yet supported"));
                        }
                    }
                }
                encode_op(&mut self.bytecode, Op::New);
                encode_u8(&mut self.bytecode, n as u8);
            }
            Expr::Array { elements, .. } => {
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
                        ArrayElement::Spread { .. } => {
                            return Err(self.err(e.span(), "spread in array literal not yet supported"));
                        }
                    }
                }
            }
            Expr::Object { properties, .. } => {
                encode_op(&mut self.bytecode, Op::NewObject);
                for p in properties {
                    match p {
                        ObjectProperty::Property { key, value, .. } => {
                            self.compile_expr(value)?;
                            match key {
                                ObjectKey::Identifier { name, .. } | ObjectKey::String { value: name, .. } => {
                                    let idx = self.constants.intern(Constant::String(name.clone()));
                                    encode_op(&mut self.bytecode, Op::InitProp);
                                    encode_u16(&mut self.bytecode, idx);
                                }
                                ObjectKey::Number { value, .. } => {
                                    let name = if value.fract() == 0.0 {
                                        format!("{}", *value as i64)
                                    } else { format!("{}", value) };
                                    let idx = self.constants.intern(Constant::String(name));
                                    encode_op(&mut self.bytecode, Op::InitProp);
                                    encode_u16(&mut self.bytecode, idx);
                                }
                                ObjectKey::Computed { .. } => {
                                    return Err(self.err(e.span(), "computed object key not yet supported"));
                                }
                            }
                        }
                        ObjectProperty::Spread { .. } => {
                            return Err(self.err(e.span(), "spread in object literal not yet supported"));
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
                // v1: support identifier-target updates only.
                let name = match argument.as_ref() {
                    Expr::Identifier { name, .. } => name.clone(),
                    _ => return Err(self.err(e.span(), "update on non-identifier not yet supported")),
                };
                enum Tgt { Local(u16), Up(u16), Global(u16) }
                let slot = self.resolve_local(&name);
                let target = if let Some(s) = slot { Tgt::Local(s) }
                    else if let Some(u) = self.resolve_upvalue(&name) { Tgt::Up(u) }
                    else { Tgt::Global(self.constants.intern(Constant::String(name.clone()))) };
                // Load current value
                match &target {
                    Tgt::Local(s)  => { encode_op(&mut self.bytecode, Op::LoadLocal);  encode_u16(&mut self.bytecode, *s); }
                    Tgt::Up(u)     => { encode_op(&mut self.bytecode, Op::LoadUpvalue); encode_u16(&mut self.bytecode, *u); }
                    Tgt::Global(i) => { encode_op(&mut self.bytecode, Op::LoadGlobal); encode_u16(&mut self.bytecode, *i); }
                }
                if !prefix {
                    encode_op(&mut self.bytecode, Op::Dup);
                }
                encode_op(&mut self.bytecode, match operator {
                    UpdateOp::Inc => Op::Inc,
                    UpdateOp::Dec => Op::Dec,
                });
                // Store back
                if *prefix { encode_op(&mut self.bytecode, Op::Dup); }
                match &target {
                    Tgt::Local(s)  => { encode_op(&mut self.bytecode, Op::StoreLocal);  encode_u16(&mut self.bytecode, *s); }
                    Tgt::Up(u)     => { encode_op(&mut self.bytecode, Op::StoreUpvalue); encode_u16(&mut self.bytecode, *u); }
                    Tgt::Global(i) => { encode_op(&mut self.bytecode, Op::StoreGlobal); encode_u16(&mut self.bytecode, *i); }
                }
                if !prefix {
                    // Result is the prior value already on the stack (from
                    // the Dup before Inc/Dec); the Store consumed the new
                    // value, leaving the old.
                }
            }
            _ => {
                return Err(self.err(e.span(), "expression form not yet supported in compiler v1"));
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
            loop_stack: Vec::new(),
            enclosing: sub_enclosing,
            upvalues: Vec::new(),
        };
        let param_count = params.len() as u16;
        for p in params {
            for n in &p.names {
                sub.alloc_local(LocalDescriptor {
                    name: n.name.clone(),
                    kind: VariableKind::Let,
                    depth: 0,
                });
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
}
