use super::widths::TEMPORARY_BYTES;
use crate::diagnostics::CompileResult;

pub(in crate::codegen) fn save_aarch64_fp_registers(
    count: usize,
    base_offset: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    for index in 0..count {
        write_assembly!(
            assembly,
            "\tstr d{index}, [sp, #{}]\n",
            base_offset + (index * TEMPORARY_BYTES)
        )?;
    }
    Ok(())
}

pub(in crate::codegen) fn restore_aarch64_fp_registers(
    count: usize,
    base_offset: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    for index in 0..count {
        write_assembly!(
            assembly,
            "\tldr d{index}, [sp, #{}]\n",
            base_offset + (index * TEMPORARY_BYTES)
        )?;
    }
    Ok(())
}
