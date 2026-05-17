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
pub use compiler::{CompiledModule, Compiler, CompileError, LocalDescriptor, UpvalueDescriptor, UpvalueSource, ImportBinding, ImportBindingKind, ExportBinding};
pub use disasm::disassemble;

/// Convenience: parse + compile a module source string.
pub fn compile_module(src: &str) -> Result<CompiledModule, CompileError> {
    let ast = rusty_js_parser::parse_module(src)
        .map_err(|e| CompileError { span: e.span, message: format!("parse: {} @byte{}", e.message, e.span.start) })?;
    let mut c = Compiler::new();
    // Ω.5.P51.E1: precompute the source-line index. Stored on the resulting
    // CompiledModule and propagated to all FunctionProtos (they share the
    // same source). Runtime errors then convert pc → span → line:col without
    // re-scanning the source string at fault time.
    c.set_source_line_starts(compute_line_starts(src));
    c.compile_module(&ast)
}

/// Byte offsets of the start of each line in `src`. Index 0 is offset 0.
/// Line i starts at line_starts[i] (inclusive); line i ends at
/// line_starts[i+1] (exclusive, accounting for the newline byte itself).
pub fn compute_line_starts(src: &str) -> Vec<u32> {
    let mut v: Vec<u32> = Vec::with_capacity(src.len() / 32 + 1);
    v.push(0);
    for (i, b) in src.bytes().enumerate() {
        if b == b'\n' {
            v.push((i + 1) as u32);
        }
    }
    v
}

/// Convert a byte offset to (line, column), both 1-indexed for editor
/// conventions. Returns (1, 1) on empty input.
pub fn byte_offset_to_line_col(line_starts: &[u32], offset: u32) -> (u32, u32) {
    if line_starts.is_empty() { return (1, 1); }
    let idx = line_starts.partition_point(|&start| start <= offset);
    let line = idx as u32; // 1-indexed because partition_point returns count <= offset
    let col = offset + 1 - line_starts[idx - 1];
    (line, col)
}
