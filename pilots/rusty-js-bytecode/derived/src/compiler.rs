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
}

#[derive(Debug)]
struct LoopFrame {
    /// Bytecode offset where `continue` should jump to (loop test or update).
    continue_target: usize,
    /// Operand-byte offsets of unresolved `break` jumps. Patched on loop exit.
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
                self.loop_stack.push(LoopFrame { continue_target: loop_start, break_patches: Vec::new() });
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
                // Continue target is the test position; we'll record it after body.
                self.loop_stack.push(LoopFrame { continue_target: 0, break_patches: Vec::new() });
                self.compile_stmt(body)?;
                let test_pos = self.bytecode.len();
                // Now set the continue target retroactively on the current frame.
                self.loop_stack.last_mut().unwrap().continue_target = test_pos;
                self.compile_expr(test)?;
                let jump_back = self.emit_jump(Op::JumpIfTrue);
                self.patch_jump_to(jump_back, loop_start);
                let frame = self.loop_stack.pop().unwrap();
                for site in frame.break_patches { self.patch_jump_at(site); }
            }
            Stmt::For { init, test, update, body, .. } => {
                // Init runs once, in the surrounding scope.
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
                // continue jumps to update; if no update, continue jumps to test.
                let cont_target = test_pos;
                self.loop_stack.push(LoopFrame { continue_target: cont_target, break_patches: Vec::new() });
                let jump_if_false = if let Some(t) = test {
                    self.compile_expr(t)?;
                    Some(self.emit_jump(Op::JumpIfFalse))
                } else { None };
                self.compile_stmt(body)?;
                let update_pos = self.bytecode.len();
                self.loop_stack.last_mut().unwrap().continue_target = update_pos;
                if let Some(u) = update {
                    self.compile_expr(u)?;
                    encode_op(&mut self.bytecode, Op::Pop);
                }
                self.emit_back_jump(test_pos);
                if let Some(j) = jump_if_false { self.patch_jump(j); }
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
            Stmt::Continue { label, .. } => {
                if label.is_some() {
                    return Err(self.err(span, "labelled continue not yet supported"));
                }
                if let Some(frame) = self.loop_stack.last() {
                    let target = frame.continue_target;
                    self.emit_back_jump(target);
                } else {
                    return Err(self.err(span, "continue outside of loop"));
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
                self.compile_expr(value)?;
                // The value remains on the stack as the assignment's result.
                encode_op(&mut self.bytecode, Op::Dup);
                match target.as_ref() {
                    Expr::Identifier { name, .. } => {
                        if let Some(slot) = self.resolve_local(name) {
                            encode_op(&mut self.bytecode, Op::StoreLocal);
                            encode_u16(&mut self.bytecode, slot);
                        } else {
                            let idx = self.constants.intern(Constant::String(name.clone()));
                            encode_op(&mut self.bytecode, Op::StoreGlobal);
                            encode_u16(&mut self.bytecode, idx);
                        }
                    }
                    _ => return Err(self.err(e.span(), "complex assignment target not yet supported")),
                }
            }
            Expr::This { .. } => {
                // v1: emit a global "this" reference. Round 3.c.d (functions)
                // wires real this-binding.
                let idx = self.constants.intern(Constant::String("this".into()));
                encode_op(&mut self.bytecode, Op::LoadGlobal);
                encode_u16(&mut self.bytecode, idx);
            }
            _ => {
                return Err(self.err(e.span(), "expression form not yet supported in compiler v1"));
            }
        }
        Ok(())
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
