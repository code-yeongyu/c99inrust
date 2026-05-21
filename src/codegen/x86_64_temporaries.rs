use super::stack_helpers::{x86_stack_byte_offset, x86_stack_object_offset, x86_stack_offset};
use super::widths::ValueWidth;
use super::x86_64_addressing::{x86_64_instruction_suffix, x86_64_result_register};
use crate::diagnostics::{CompileError, CompileResult};

pub(in crate::codegen) fn emit_x86_64_store_temporary(
    width: ValueWidth,
    offset: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    let suffix = x86_64_instruction_suffix(width);
    let register = x86_64_result_register(width);
    write_assembly!(
        assembly,
        "\tmov{suffix} {register}, {}(%rbp)\n",
        x86_stack_offset(offset, width)
    )
}

pub(in crate::codegen) fn emit_x86_64_store_result(
    width: ValueWidth,
    offset: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    let suffix = x86_64_instruction_suffix(width);
    let register = x86_64_result_register(width);
    write_assembly!(
        assembly,
        "\tmov{suffix} {register}, {}(%rbp)\n",
        x86_stack_offset(offset, width)
    )
}

pub(in crate::codegen) fn emit_x86_64_init_local_bytes(
    offset: usize,
    values: &[u8],
    assembly: &mut String,
) -> CompileResult<()> {
    for (index, value) in values.iter().enumerate() {
        let byte_offset = offset
            .checked_add(index)
            .ok_or_else(|| CompileError::new("local byte initializer offset overflow"))?;
        write_assembly!(
            assembly,
            "\tmovb ${value}, {}(%rbp)\n",
            x86_stack_byte_offset(offset, values.len(), byte_offset)
        )?;
    }
    Ok(())
}

pub(in crate::codegen) fn emit_x86_64_init_local_ints(
    offset: usize,
    values: &[i32],
    assembly: &mut String,
) -> CompileResult<()> {
    let byte_size = values
        .len()
        .checked_mul(4)
        .ok_or_else(|| CompileError::new("local int initializer size overflow"))?;
    for (index, value) in values.iter().enumerate() {
        let byte_offset = offset
            .checked_add(
                index
                    .checked_mul(4)
                    .ok_or_else(|| CompileError::new("local int initializer offset overflow"))?,
            )
            .ok_or_else(|| CompileError::new("local int initializer offset overflow"))?;
        write_assembly!(
            assembly,
            "\tmovl ${value}, {}(%rbp)\n",
            x86_stack_byte_offset(offset, byte_size, byte_offset)
        )?;
    }
    Ok(())
}

pub(in crate::codegen) fn emit_x86_64_load_temporary(
    width: ValueWidth,
    offset: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    let suffix = x86_64_instruction_suffix(width);
    let register = x86_64_result_register(width);
    write_assembly!(
        assembly,
        "\tmov{suffix} {}(%rbp), {register}\n",
        x86_stack_offset(offset, width)
    )
}

pub(in crate::codegen) fn emit_x86_64_load_object_start(
    width: ValueWidth,
    offset: usize,
    byte_size: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    let suffix = x86_64_instruction_suffix(width);
    let register = x86_64_result_register(width);
    write_assembly!(
        assembly,
        "\tmov{suffix} {}(%rbp), {register}\n",
        x86_stack_object_offset(offset, byte_size)
    )
}

pub(in crate::codegen) fn emit_x86_64_load_temporary_to_register(
    width: ValueWidth,
    offset: usize,
    register: &str,
    assembly: &mut String,
) -> CompileResult<()> {
    let suffix = x86_64_instruction_suffix(width);
    write_assembly!(
        assembly,
        "\tmov{suffix} {}(%rbp), {register}\n",
        x86_stack_offset(offset, width)
    )
}

pub(in crate::codegen) fn emit_x86_64_move_result_to_rhs(width: ValueWidth, assembly: &mut String) {
    match width {
        ValueWidth::I32 => assembly.push_str("\tmovl %eax, %ecx\n"),
        ValueWidth::I64 => assembly.push_str("\tmovq %rax, %rcx\n"),
        ValueWidth::F64 => assembly.push_str("\tmovsd %xmm0, %xmm1\n"),
    }
}
