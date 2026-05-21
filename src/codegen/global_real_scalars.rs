use super::data_literals::double_literal_bits;
use crate::diagnostics::CompileResult;

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
    emit_double_global(label, real, assembly)?;
    if byte_len > 8 {
        let tail = byte_len - 8;
        write_assembly!(assembly, "\t.zero {tail}\n")?;
    }
    Ok(())
}
