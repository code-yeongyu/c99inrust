use super::aarch64_expr::emit_aarch64_expr_with_width;
use super::aarch64_temporaries::{
    emit_aarch64_load_temporary, emit_aarch64_load_temporary_to_register,
    emit_aarch64_store_temporary,
};
use super::data_literals::label_name;
use super::frames::LabelAllocator;
use super::sized_fields::emit_aarch64_store as emit_aarch64_store_sized_field;
use super::widths::{
    GlobalByteSubscriptExpr, PointerFieldExpr, TEMPORARY_BYTES, ValueWidth, scalar_width,
};
use crate::diagnostics::CompileResult;
use crate::ir::LoweredExpr;

pub(in crate::codegen) fn emit_aarch64_store_global_byte_subscript(
    subscript: GlobalByteSubscriptExpr<'_>,
    value: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
    let label = label_name(subscript.name, labels.target);
    emit_aarch64_expr_with_width(
        value,
        ValueWidth::I32,
        temporary_base,
        depth,
        labels,
        assembly,
    )?;
    emit_aarch64_store_temporary(ValueWidth::I32, value_offset, assembly)?;
    emit_aarch64_expr_with_width(
        subscript.index,
        ValueWidth::I32,
        temporary_base,
        depth + 1,
        labels,
        assembly,
    )?;
    assembly.push_str("\tmov w17, w0\n");
    write_assembly!(assembly, "\tadrp x16, {label}@PAGE\n")?;
    write_assembly!(assembly, "\tadd x16, x16, {label}@PAGEOFF\n")?;
    emit_aarch64_load_temporary(ValueWidth::I32, value_offset, assembly)?;
    assembly.push_str("\tstrb w0, [x16, w17, sxtw]\n");
    Ok(())
}

pub(in crate::codegen) fn emit_aarch64_store_global_int_subscript(
    subscript: GlobalByteSubscriptExpr<'_>,
    value: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
    let label = label_name(subscript.name, labels.target);
    emit_aarch64_expr_with_width(
        value,
        ValueWidth::I32,
        temporary_base,
        depth,
        labels,
        assembly,
    )?;
    emit_aarch64_store_temporary(ValueWidth::I32, value_offset, assembly)?;
    emit_aarch64_expr_with_width(
        subscript.index,
        ValueWidth::I32,
        temporary_base,
        depth + 1,
        labels,
        assembly,
    )?;
    assembly.push_str("\tmov w17, w0\n");
    write_assembly!(assembly, "\tadrp x16, {label}@PAGE\n")?;
    write_assembly!(assembly, "\tadd x16, x16, {label}@PAGEOFF\n")?;
    emit_aarch64_load_temporary(ValueWidth::I32, value_offset, assembly)?;
    assembly.push_str("\tstr w0, [x16, w17, sxtw #2]\n");
    Ok(())
}

pub(in crate::codegen) fn emit_aarch64_store_global_pointer_subscript(
    subscript: GlobalByteSubscriptExpr<'_>,
    value: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
    let label = label_name(subscript.name, labels.target);
    emit_aarch64_expr_with_width(
        value,
        ValueWidth::I64,
        temporary_base,
        depth,
        labels,
        assembly,
    )?;
    emit_aarch64_store_temporary(ValueWidth::I64, value_offset, assembly)?;
    emit_aarch64_expr_with_width(
        subscript.index,
        ValueWidth::I32,
        temporary_base,
        depth + 1,
        labels,
        assembly,
    )?;
    assembly.push_str("\tmov w17, w0\n");
    write_assembly!(assembly, "\tadrp x16, {label}@PAGE\n")?;
    write_assembly!(assembly, "\tadd x16, x16, {label}@PAGEOFF\n")?;
    emit_aarch64_load_temporary_to_register(ValueWidth::I64, value_offset, "0", assembly)?;
    assembly.push_str("\tstr x0, [x16, w17, sxtw #3]\n");
    Ok(())
}
pub(in crate::codegen) fn emit_aarch64_store_pointer_field(
    field: PointerFieldExpr<'_>,
    value: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let width = scalar_width(field.scalar_type);
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
    emit_aarch64_expr_with_width(value, width, temporary_base, depth, labels, assembly)?;
    emit_aarch64_store_temporary(width, value_offset, assembly)?;
    emit_aarch64_expr_with_width(
        field.pointer,
        ValueWidth::I64,
        temporary_base,
        depth + 1,
        labels,
        assembly,
    )?;
    assembly.push_str("\tmov x16, x0\n");
    emit_aarch64_load_temporary(width, value_offset, assembly)?;
    emit_aarch64_store_sized_field(field.byte_size, width, "x16", field.offset, assembly)
}
