use super::data_literals::{
    double_literal_bits, emit_double_literal_data, emit_string_literal_data, label_name,
};
use super::frames::LabelAllocator;
use super::stack_helpers::memory_scale_bytes_for_byte_size;
use super::target::Target;
use super::widths::{PointerSubscriptExpr, TEMPORARY_BYTES, ValueWidth, scalar_width};
use super::x86_64_addressing::{x86_64_instruction_suffix, x86_64_result_register};
use super::x86_64_expr::emit_x86_64_expr_with_width;
use super::x86_64_temporaries::{
    emit_x86_64_load_temporary_to_register, emit_x86_64_store_temporary,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::LoweredExpr;

pub(in crate::codegen) fn emit_x86_64_load_double_literal(
    value: &str,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = labels.fresh();
    write_assembly!(assembly, "\tmovsd {label}(%rip), %xmm0\n")?;
    emit_double_literal_data(&label, double_literal_bits(value)?, target, assembly)
}

pub(in crate::codegen) fn emit_x86_64_load_string_address(
    value: &str,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = labels.fresh();
    write_assembly!(assembly, "\tleaq {label}(%rip), %rax\n")?;
    emit_string_literal_data(&label, value, target, assembly)
}

pub(in crate::codegen) fn emit_x86_64_load_global(
    name: &str,
    width: ValueWidth,
    target: Target,
    assembly: &mut String,
) -> CompileResult<()> {
    if target == Target::X86_64UnknownLinuxGnu && name == "errno" {
        assembly.push_str("\tcall __errno_location\n");
        let suffix = x86_64_instruction_suffix(width);
        let register = x86_64_result_register(width);
        return write_assembly!(assembly, "\tmov{suffix} (%rax), {register}\n");
    }
    let label = label_name(name, target);
    let suffix = x86_64_instruction_suffix(width);
    let register = x86_64_result_register(width);
    write_assembly!(assembly, "\tmov{suffix} {label}(%rip), {register}\n")
}

pub(in crate::codegen) fn emit_x86_64_store_global(
    name: &str,
    width: ValueWidth,
    target: Target,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = label_name(name, target);
    let suffix = x86_64_instruction_suffix(width);
    let register = x86_64_result_register(width);
    write_assembly!(assembly, "\tmov{suffix} {register}, {label}(%rip)\n")
}

pub(in crate::codegen) fn emit_x86_64_load_global_byte_subscript(
    name: &str,
    index: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = label_name(name, target);
    emit_x86_64_expr_with_width(
        index,
        ValueWidth::I32,
        temporary_base,
        depth,
        target,
        labels,
        assembly,
    )?;
    assembly.push_str("\tcltq\n");
    write_assembly!(assembly, "\tleaq {label}(%rip), %rcx\n")?;
    assembly.push_str("\tmovzbl (%rcx,%rax), %eax\n");
    Ok(())
}

pub(in crate::codegen) fn emit_x86_64_load_global_int_subscript(
    name: &str,
    index: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = label_name(name, target);
    emit_x86_64_expr_with_width(
        index,
        ValueWidth::I32,
        temporary_base,
        depth,
        target,
        labels,
        assembly,
    )?;
    assembly.push_str("\tcltq\n");
    write_assembly!(assembly, "\tleaq {label}(%rip), %rcx\n")?;
    assembly.push_str("\tmovl (%rcx,%rax,4), %eax\n");
    Ok(())
}

pub(in crate::codegen) fn emit_x86_64_load_global_pointer_subscript(
    name: &str,
    index: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = label_name(name, target);
    emit_x86_64_expr_with_width(
        index,
        ValueWidth::I32,
        temporary_base,
        depth,
        target,
        labels,
        assembly,
    )?;
    assembly.push_str("\tcltq\n");
    write_assembly!(assembly, "\tleaq {label}(%rip), %rcx\n")?;
    assembly.push_str("\tmovq (%rcx,%rax,8), %rax\n");
    Ok(())
}

pub(in crate::codegen) fn emit_x86_64_load_pointer_subscript(
    subscript: PointerSubscriptExpr<'_>,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let base_offset = temporary_base + (depth * TEMPORARY_BYTES);
    let width = scalar_width(subscript.element_type);
    emit_x86_64_expr_with_width(
        subscript.pointer,
        ValueWidth::I64,
        temporary_base,
        depth + 1,
        target,
        labels,
        assembly,
    )?;
    emit_x86_64_store_temporary(ValueWidth::I64, base_offset, assembly)?;
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
    emit_x86_64_load_temporary_to_register(ValueWidth::I64, base_offset, "%rcx", assembly)?;
    if subscript.element_byte_size == 1 && width == ValueWidth::I32 {
        return write_assembly!(assembly, "\tmovzbl (%rcx,%rax,1), %eax\n");
    }
    if subscript.element_byte_size == 2 && width == ValueWidth::I32 && subscript.element_unsigned {
        return write_assembly!(assembly, "\tmovzwl (%rcx,%rax,2), %eax\n");
    }
    if subscript.element_byte_size == 2 && width == ValueWidth::I32 {
        return write_assembly!(assembly, "\tmovswl (%rcx,%rax,2), %eax\n");
    }
    let Some(scale) = memory_scale_bytes_for_byte_size(subscript.element_byte_size) else {
        return Err(CompileError::new(
            "unsupported pointer subscript element size",
        ));
    };
    write_assembly!(
        assembly,
        "\tmov{} (%rcx,%rax,{}), {}\n",
        x86_64_instruction_suffix(width),
        scale,
        x86_64_result_register(width)
    )
}
