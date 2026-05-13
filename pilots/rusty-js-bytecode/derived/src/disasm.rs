//! Disassembler — reverses bytecode into a human-readable form per
//! design spec §IV. Used by the test harness for bytecode-shape
//! assertions; also useful for debugging.

use crate::compiler::CompiledModule;
use crate::op::*;

pub fn disassemble(m: &CompiledModule) -> String {
    let mut out = String::new();
    let mut off = 0;
    while off < m.bytecode.len() {
        let op_byte = m.bytecode[off];
        let op = match op_from_byte(op_byte) {
            Some(op) => op,
            None => {
                out.push_str(&format!("{:5}  <invalid 0x{:02X}>\n", off, op_byte));
                off += 1;
                continue;
            }
        };
        let opname = format!("{:?}", op);
        let osize = op.operand_size();
        let operand_str = match osize {
            0 => String::new(),
            1 => format!(" {}", m.bytecode[off + 1]),
            2 => format!(" {}", decode_u16(&m.bytecode, off + 1)),
            4 => match op {
                Op::PushI32 => format!(" {}", decode_i32(&m.bytecode, off + 1)),
                _ => format!(" {}", decode_i32(&m.bytecode, off + 1)),
            },
            _ => String::new(),
        };
        // Constant-resolving operand: render the constant inline for the
        // const-pool-indexed opcodes.
        let const_resolved = match op {
            Op::PushConst | Op::LoadGlobal | Op::StoreGlobal | Op::GetProp | Op::SetProp
            | Op::InitProp | Op::LoadLocal | Op::StoreLocal | Op::LoadArg | Op::StoreArg
            | Op::LoadUpvalue | Op::StoreUpvalue | Op::DefineLocal => {
                let idx = decode_u16(&m.bytecode, off + 1) as usize;
                m.constants.entries().get(idx).map(|c| format!("  ; {}", render_constant(c)))
            }
            _ => None,
        };
        out.push_str(&format!("{:5}  {}{}{}\n", off, opname, operand_str,
            const_resolved.unwrap_or_default()));
        off += 1 + osize;
    }
    out
}

fn render_constant(c: &crate::constants::Constant) -> String {
    use crate::constants::Constant::*;
    match c {
        Number(v) => format!("{}", v),
        BigInt(s) => format!("{}n", s),
        String(s) => format!("{:?}", s),
        Regex { body, flags } => format!("/{}/{}", body, flags),
        Function(_) => "<function>".to_string(),
    }
}
