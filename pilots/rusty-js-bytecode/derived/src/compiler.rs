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
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            bytecode: Vec::new(),
            constants: ConstantsPool::new(),
            locals: Vec::new(),
            source_map: Vec::new(),
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
                // v1: treat all variable declarations as global stores. Real
                // scope resolution lands in round 3.c.c.
                for d in &v.declarators {
                    if d.names.len() != 1 {
                        return Err(self.err(d.span, "destructure declarators not yet supported in compiler v1"));
                    }
                    let name = &d.names[0];
                    if let Some(init) = &d.init {
                        self.compile_expr(init)?;
                    } else {
                        encode_op(&mut self.bytecode, Op::PushUndef);
                    }
                    let idx = self.constants.intern(Constant::String(name.name.clone()));
                    encode_op(&mut self.bytecode, Op::StoreGlobal);
                    encode_u16(&mut self.bytecode, idx);
                }
            }
            Stmt::Throw { argument, .. } => {
                self.compile_expr(argument)?;
                encode_op(&mut self.bytecode, Op::Throw);
            }
            Stmt::Debugger { .. } => {
                encode_op(&mut self.bytecode, Op::Debugger);
            }
            _ => {
                // Not yet supported in v1 (round 3.c.b). Round 3.c.c lands
                // If/For/While/Switch/Try/etc.
                return Err(self.err(span, "statement form not yet supported in compiler v1"));
            }
        }
        Ok(())
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
                // v1: every identifier load is global; round 3.c.c resolves
                // locals + upvalues.
                let name_idx = self.constants.intern(Constant::String(name.clone()));
                encode_op(&mut self.bytecode, Op::LoadGlobal);
                encode_u16(&mut self.bytecode, name_idx);
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
                self.compile_expr(left)?;
                self.compile_expr(right)?;
                let op = match operator {
                    BinaryOp::Add => Op::Add, BinaryOp::Sub => Op::Sub,
                    BinaryOp::Mul => Op::Mul, BinaryOp::Div => Op::Div,
                    BinaryOp::Mod => Op::Mod, BinaryOp::Pow => Op::Pow,
                    BinaryOp::Shl => Op::Shl, BinaryOp::Shr => Op::Shr, BinaryOp::UShr => Op::UShr,
                    BinaryOp::Lt => Op::Lt, BinaryOp::Gt => Op::Gt,
                    BinaryOp::Le => Op::Le, BinaryOp::Ge => Op::Ge,
                    BinaryOp::Eq => Op::Eq, BinaryOp::Ne => Op::Ne,
                    BinaryOp::StrictEq => Op::StrictEq, BinaryOp::StrictNe => Op::StrictNe,
                    BinaryOp::Instanceof => Op::Instanceof, BinaryOp::In => Op::In,
                    BinaryOp::BitAnd => Op::BitAnd, BinaryOp::BitOr => Op::BitOr,
                    BinaryOp::BitXor => Op::BitXor,
                    BinaryOp::LogicalAnd | BinaryOp::LogicalOr | BinaryOp::NullishCoalesce => {
                        // Short-circuit operators compile to conditional
                        // jumps. They're treated as binary ops in AST but
                        // need control-flow encoding — round 3.c.c work.
                        return Err(self.err(e.span(), "short-circuit logical operators not yet supported"));
                    }
                };
                encode_op(&mut self.bytecode, op);
            }
            Expr::Parenthesized { expr, .. } => self.compile_expr(expr)?,
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
