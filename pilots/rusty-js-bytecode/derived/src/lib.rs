//! rusty-js-bytecode — stack-based bytecode + single-pass compiler from
//! rusty_js_ast AST. Per specs/rusty-js-bytecode-design.md.
//!
//! v1 scope (round 3.c.b): literals + arithmetic + comparison + simple
//! variable access via global slot. Control flow + scope resolution +
//! function bodies + try/catch follow in 3.c.c and 3.c.d.

pub mod op;
pub mod constants;
pub mod compiler;
pub mod disasm;

pub use op::{Op, encode_op};
pub use constants::{Constant, ConstantsPool};
pub use compiler::{CompiledModule, Compiler, CompileError, LocalDescriptor, UpvalueDescriptor, UpvalueSource};
pub use disasm::disassemble;

/// Convenience: parse + compile a module source string.
pub fn compile_module(src: &str) -> Result<CompiledModule, CompileError> {
    let ast = rusty_js_parser::parse_module(src)
        .map_err(|e| CompileError { span: e.span, message: format!("parse: {}", e.message) })?;
    let mut c = Compiler::new();
    c.compile_module(&ast)
}
