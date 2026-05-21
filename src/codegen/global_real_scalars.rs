use super::data_literals::double_literal_bits;
use crate::diagnostics::{CompileError, CompileResult};

pub(in crate::codegen) fn emit_double_global(
    label: &str,
    value: &str,
    assembly: &mut String,
) -> CompileResult<()> {
    assembly.push_str(".p2align 3\n");
    write_assembly!(assembly, "{label}:\n")?;
    let bits = double_literal_bits(value)?;
    write_assembly!(assembly, "\t.quad 0x{bits:016x}\n")
}

pub(in crate::codegen) fn emit_real_then_zero_global(
    label: &str,
    real: &str,
    byte_len: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    let real_len = real_part_byte_len(byte_len);
    if real_len == 4 {
        emit_float_global(label, real, assembly)?;
    } else {
        emit_double_global(label, real, assembly)?;
    }
    if byte_len > real_len {
        write_assembly!(assembly, "\t.zero {}\n", byte_len - real_len)?;
    }
    Ok(())
}

fn emit_float_global(label: &str, value: &str, assembly: &mut String) -> CompileResult<()> {
    assembly.push_str(".p2align 2\n");
    write_assembly!(assembly, "{label}:\n")?;
    let bits = float_literal_bits(value)?;
    write_assembly!(assembly, "\t.long 0x{bits:08x}\n")
}

fn float_literal_bits(value: &str) -> CompileResult<u32> {
    value
        .parse::<f32>()
        .map(f32::to_bits)
        .map_err(|_| CompileError::new(format!("invalid float literal: {value}")))
}

const fn real_part_byte_len(byte_len: usize) -> usize {
    if byte_len == 8 { 4 } else { 8 }
}
