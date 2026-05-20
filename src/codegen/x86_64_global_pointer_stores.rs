use super::data_literals::label_name;
use super::frames::LabelAllocator;
use super::sized_fields::emit_x86_64_store as emit_x86_64_store_sized_field;
use super::target::Target;
use super::widths::{
    GlobalByteSubscriptExpr, PointerFieldExpr, TEMPORARY_BYTES, ValueWidth, scalar_width,
};
use super::x86_64_expr::emit_x86_64_expr_with_width;
use super::x86_64_temporaries::{emit_x86_64_load_temporary, emit_x86_64_store_temporary};
use crate::diagnostics::CompileResult;
use crate::ir::LoweredExpr;

pub(in crate::codegen) fn emit_x86_64_store_global_pointer_subscript(
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
        ValueWidth::I64,
        temporary_base,
        depth,
        target,
        labels,
        assembly,
    )?;
    emit_x86_64_store_temporary(ValueWidth::I64, value_offset, assembly)?;
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
    emit_x86_64_load_temporary(ValueWidth::I64, value_offset, assembly)?;
    assembly.push_str("\tmovq %rax, (%rcx,%rdx,8)\n");
    Ok(())
}

pub(in crate::codegen) fn emit_x86_64_store_pointer_field(
    field: PointerFieldExpr<'_>,
    value: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let width = scalar_width(field.scalar_type);
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
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
        field.pointer,
        ValueWidth::I64,
        temporary_base,
        depth + 1,
        target,
        labels,
        assembly,
    )?;
    assembly.push_str("\tmovq %rax, %rcx\n");
    emit_x86_64_load_temporary(width, value_offset, assembly)?;
    emit_x86_64_store_sized_field(field.byte_size, width, "%rcx", field.offset, assembly)
}
