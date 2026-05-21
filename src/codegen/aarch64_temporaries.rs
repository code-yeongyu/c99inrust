use super::aarch64_addressing::{aarch64_register_prefix, aarch64_result_register};
use super::aarch64_binary::emit_aarch64_i32_to_register;
use super::widths::ValueWidth;
use crate::diagnostics::{CompileError, CompileResult};

pub(in crate::codegen) fn emit_aarch64_store_temporary(
    width: ValueWidth,
    offset: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    let register = aarch64_result_register(width);
    write_assembly!(assembly, "\tstr {register}, [sp, #{offset}]\n")
}

pub(in crate::codegen) fn emit_aarch64_store_result(
    width: ValueWidth,
    offset: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    let register = aarch64_result_register(width);
    write_assembly!(assembly, "\tstr {register}, [sp, #{offset}]\n")
}

pub(in crate::codegen) fn emit_aarch64_init_local_bytes(
    offset: usize,
    values: &[u8],
    assembly: &mut String,
) -> CompileResult<()> {
    for (index, value) in values.iter().enumerate() {
        let byte_offset = offset
            .checked_add(index)
            .ok_or_else(|| CompileError::new("local byte initializer offset overflow"))?;
        write_assembly!(assembly, "\tmov w16, #{value}\n")?;
        write_assembly!(assembly, "\tstrb w16, [sp, #{byte_offset}]\n")?;
    }
    Ok(())
}

pub(in crate::codegen) fn emit_aarch64_init_local_ints(
    offset: usize,
    values: &[i32],
    assembly: &mut String,
) -> CompileResult<()> {
    for (index, value) in values.iter().enumerate() {
        let byte_offset = offset
            .checked_add(
                index
                    .checked_mul(4)
                    .ok_or_else(|| CompileError::new("local int initializer offset overflow"))?,
            )
            .ok_or_else(|| CompileError::new("local int initializer offset overflow"))?;
        emit_aarch64_i32_to_register(i64::from(*value), "w16", assembly)?;
        write_assembly!(assembly, "\tstr w16, [sp, #{byte_offset}]\n")?;
    }
    Ok(())
}

pub(in crate::codegen) fn emit_aarch64_load_temporary(
    width: ValueWidth,
    offset: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    let register = aarch64_result_register(width);
    write_assembly!(assembly, "\tldr {register}, [sp, #{offset}]\n")
}

pub(in crate::codegen) fn emit_aarch64_load_f32_local(
    offset: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    write_assembly!(assembly, "\tldr s0, [sp, #{offset}]\n")?;
    assembly.push_str("\tfcvt d0, s0\n");
    Ok(())
}

pub(in crate::codegen) fn emit_aarch64_load_temporary_to_register(
    width: ValueWidth,
    offset: usize,
    register: &str,
    assembly: &mut String,
) -> CompileResult<()> {
    let prefix = aarch64_register_prefix(width);
    write_assembly!(assembly, "\tldr {prefix}{register}, [sp, #{offset}]\n")
}
