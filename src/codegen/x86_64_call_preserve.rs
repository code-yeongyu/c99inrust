use super::stack_helpers::x86_stack_object_offset;
use super::widths::TEMPORARY_BYTES;
use crate::diagnostics::CompileResult;

pub(in crate::codegen) fn save_x86_64_fp_registers(
    count: usize,
    base_offset: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    for index in 0..count {
        write_assembly!(
            assembly,
            "\tmovsd %xmm{index}, {}(%rbp)\n",
            x86_stack_object_offset(base_offset + (index * TEMPORARY_BYTES), TEMPORARY_BYTES)
        )?;
    }
    Ok(())
}

pub(in crate::codegen) fn restore_x86_64_fp_registers(
    count: usize,
    base_offset: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    for index in 0..count {
        write_assembly!(
            assembly,
            "\tmovsd {}(%rbp), %xmm{index}\n",
            x86_stack_object_offset(base_offset + (index * TEMPORARY_BYTES), TEMPORARY_BYTES)
        )?;
    }
    Ok(())
}
