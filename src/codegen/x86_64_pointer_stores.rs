use super::data_literals::label_name;
use super::frames::LabelAllocator;
use super::stack_helpers::memory_scale_bytes_for_byte_size;
use super::target::Target;
use super::widths::{
    GlobalByteSubscriptExpr, PointerSubscriptExpr, TEMPORARY_BYTES, ValueWidth, scalar_width,
};
use super::x86_64_addressing::{x86_64_instruction_suffix, x86_64_result_register};
use super::x86_64_expr::emit_x86_64_expr_with_width;
use super::x86_64_temporaries::{
    emit_x86_64_load_temporary, emit_x86_64_load_temporary_to_register, emit_x86_64_store_temporary,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::LoweredExpr;

pub(in crate::codegen) fn emit_x86_64_store_pointer_subscript(
    subscript: PointerSubscriptExpr<'_>,
    value: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let width = scalar_width(subscript.element_type);
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
    let base_offset = temporary_base + ((depth + 1) * TEMPORARY_BYTES);
    emit_x86_64_expr_with_width(
        value,
        width,
        temporary_base,
        depth,
        target,
        labels,
        assembly,
    )?;
    emit_x86_64_store_temporary(width, value_offset, assembly)?;
    emit_x86_64_expr_with_width(
        subscript.pointer,
        ValueWidth::I64,
        temporary_base,
        depth + 2,
        target,
        labels,
        assembly,
    )?;
    emit_x86_64_store_temporary(ValueWidth::I64, base_offset, assembly)?;
    emit_x86_64_expr_with_width(
        subscript.index,
        ValueWidth::I32,
        temporary_base,
        depth + 2,
        target,
        labels,
        assembly,
    )?;
    assembly.push_str("\tcltq\n");
    assembly.push_str("\tmovq %rax, %rdx\n");
    emit_x86_64_load_temporary_to_register(ValueWidth::I64, base_offset, "%rcx", assembly)?;
    emit_x86_64_load_temporary(width, value_offset, assembly)?;
    if subscript.element_byte_size == 1 && width == ValueWidth::I32 {
        return write_assembly!(assembly, "\tmovb %al, (%rcx,%rdx,1)\n");
    }
    if subscript.element_byte_size == 2 && width == ValueWidth::I32 {
        return write_assembly!(assembly, "\tmovw %ax, (%rcx,%rdx,2)\n");
    }
    let Some(scale) = memory_scale_bytes_for_byte_size(subscript.element_byte_size) else {
        return Err(CompileError::new(
            "unsupported pointer subscript element size",
        ));
    };
    write_assembly!(
        assembly,
        "\tmov{} {}, (%rcx,%rdx,{})\n",
        x86_64_instruction_suffix(width),
        x86_64_result_register(width),
        scale
    )
}

pub(in crate::codegen) fn emit_x86_64_load_pointer_subscript_result(
    element_byte_size: usize,
    width: ValueWidth,
    element_unsigned: bool,
    assembly: &mut String,
) -> CompileResult<()> {
    if element_byte_size == 1 && width == ValueWidth::I32 {
        return write_assembly!(assembly, "\tmovzbl (%rcx,%rdx,1), %eax\n");
    }
    if element_byte_size == 2 && width == ValueWidth::I32 && element_unsigned {
        return write_assembly!(assembly, "\tmovzwl (%rcx,%rdx,2), %eax\n");
    }
    if element_byte_size == 2 && width == ValueWidth::I32 {
        return write_assembly!(assembly, "\tmovswl (%rcx,%rdx,2), %eax\n");
    }
    let Some(scale) = memory_scale_bytes_for_byte_size(element_byte_size) else {
        return Err(CompileError::new(
            "unsupported pointer subscript element size",
        ));
    };
    write_assembly!(
        assembly,
        "\tmov{} (%rcx,%rdx,{}), {}\n",
        x86_64_instruction_suffix(width),
        scale,
        x86_64_result_register(width)
    )
}

pub(in crate::codegen) fn emit_x86_64_store_pointer_subscript_result(
    element_byte_size: usize,
    width: ValueWidth,
    assembly: &mut String,
) -> CompileResult<()> {
    if element_byte_size == 1 && width == ValueWidth::I32 {
        return write_assembly!(assembly, "\tmovb %al, (%rcx,%rdx,1)\n");
    }
    if element_byte_size == 2 && width == ValueWidth::I32 {
        return write_assembly!(assembly, "\tmovw %ax, (%rcx,%rdx,2)\n");
    }
    let Some(scale) = memory_scale_bytes_for_byte_size(element_byte_size) else {
        return Err(CompileError::new(
            "unsupported pointer subscript element size",
        ));
    };
    write_assembly!(
        assembly,
        "\tmov{} {}, (%rcx,%rdx,{})\n",
        x86_64_instruction_suffix(width),
        x86_64_result_register(width),
        scale
    )
}

pub(in crate::codegen) fn emit_x86_64_store_global_byte_subscript(
    subscript: GlobalByteSubscriptExpr<'_>,
    value: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
    let label = label_name(subscript.name, target);
    emit_x86_64_expr_with_width(
        value,
        ValueWidth::I32,
        temporary_base,
        depth,
        target,
        labels,
        assembly,
    )?;
    emit_x86_64_store_temporary(ValueWidth::I32, value_offset, assembly)?;
    emit_x86_64_expr_with_width(
        subscript.index,
        ValueWidth::I32,
        temporary_base,
        depth + 1,
        target,
        labels,
        assembly,
    )?;
    assembly.push_str("\tcltq\n");
    assembly.push_str("\tmovq %rax, %rdx\n");
    write_assembly!(assembly, "\tleaq {label}(%rip), %rcx\n")?;
    emit_x86_64_load_temporary(ValueWidth::I32, value_offset, assembly)?;
    assembly.push_str("\tmovb %al, (%rcx,%rdx)\n");
    Ok(())
}

pub(in crate::codegen) fn emit_x86_64_store_global_int_subscript(
    subscript: GlobalByteSubscriptExpr<'_>,
    value: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
    let label = label_name(subscript.name, target);
    emit_x86_64_expr_with_width(
        value,
        ValueWidth::I32,
        temporary_base,
        depth,
        target,
        labels,
        assembly,
    )?;
    emit_x86_64_store_temporary(ValueWidth::I32, value_offset, assembly)?;
    emit_x86_64_expr_with_width(
        subscript.index,
        ValueWidth::I32,
        temporary_base,
        depth + 1,
        target,
        labels,
        assembly,
    )?;
    assembly.push_str("\tcltq\n");
    assembly.push_str("\tmovq %rax, %rdx\n");
    write_assembly!(assembly, "\tleaq {label}(%rip), %rcx\n")?;
    emit_x86_64_load_temporary(ValueWidth::I32, value_offset, assembly)?;
    assembly.push_str("\tmovl %eax, (%rcx,%rdx,4)\n");
    Ok(())
}
