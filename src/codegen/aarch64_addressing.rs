use super::aarch64_binary::{aarch64_update_immediate, emit_aarch64_i32_to_register};
use super::aarch64_expr::{emit_aarch64_expr, emit_aarch64_expr_with_width};
use super::aarch64_temporaries::{
    emit_aarch64_load_temporary_to_register, emit_aarch64_store_temporary,
};
use super::frames::LabelAllocator;
use super::sized_fields::emit_aarch64_load as emit_aarch64_load_sized_field;
use super::stack_helpers::memory_scale_shift_for_byte_size;
use super::widths::{
    PointerFieldExpr, PointerOffsetExpr, TEMPORARY_BYTES, ValueWidth, scalar_width,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::LoweredExpr;
use crate::parser::ScalarType;

pub(in crate::codegen) fn emit_aarch64_pointer_offset(
    offset: PointerOffsetExpr<'_>,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let base_offset = temporary_base + (depth * TEMPORARY_BYTES);
    emit_aarch64_expr_with_width(
        offset.pointer,
        ValueWidth::I64,
        temporary_base,
        depth + 1,
        labels,
        assembly,
    )?;
    emit_aarch64_store_temporary(ValueWidth::I64, base_offset, assembly)?;
    emit_aarch64_expr_with_width(
        offset.index,
        ValueWidth::I32,
        temporary_base,
        depth + 1,
        labels,
        assembly,
    )?;
    emit_aarch64_load_temporary_to_register(ValueWidth::I64, base_offset, "16", assembly)?;
    if let Some(shift) = memory_scale_shift_for_byte_size(offset.byte_size) {
        if shift == 0 {
            assembly.push_str("\tadd x0, x16, w0, sxtw\n");
        } else {
            write_assembly!(assembly, "\tadd x0, x16, w0, sxtw #{shift}\n")?;
        }
        return Ok(());
    }
    assembly.push_str("\tsxtw x0, w0\n");
    let byte_size = i64::try_from(offset.byte_size)
        .map_err(|_| CompileError::new("pointer offset size does not fit i64"))?;
    emit_aarch64_i32_to_register(byte_size, "x17", assembly)?;
    assembly.push_str("\tmul x0, x0, x17\n");
    assembly.push_str("\tadd x0, x16, x0\n");
    Ok(())
}
pub(in crate::codegen) fn emit_aarch64_load_pointer_field(
    field: PointerFieldExpr<'_>,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let width = scalar_width(field.scalar_type);
    emit_aarch64_expr_with_width(
        field.pointer,
        ValueWidth::I64,
        temporary_base,
        depth + 1,
        labels,
        assembly,
    )?;
    emit_aarch64_load_sized_field(
        field.byte_size,
        width,
        field.is_unsigned,
        "x0",
        field.offset,
        assembly,
    )
}
pub(in crate::codegen) const fn aarch64_register_prefix(width: ValueWidth) -> &'static str {
    match width {
        ValueWidth::I32 => "w",
        ValueWidth::I64 => "x",
        ValueWidth::F64 => "d",
    }
}

pub(in crate::codegen) fn aarch64_parameter_register(index: usize, width: ValueWidth) -> String {
    let prefix = aarch64_register_prefix(width);
    format!("{prefix}{index}")
}

pub(in crate::codegen) const fn aarch64_result_register(width: ValueWidth) -> &'static str {
    match width {
        ValueWidth::I32 => "w0",
        ValueWidth::I64 => "x0",
        ValueWidth::F64 => "d0",
    }
}

pub(in crate::codegen) fn emit_aarch64_store_local(
    _slot: usize,
    offset: usize,
    scalar_type: ScalarType,
    value: &LoweredExpr,
    temporary_base: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    if scalar_type != ScalarType::Int {
        return emit_aarch64_expr_with_width(
            value,
            scalar_width(scalar_type),
            temporary_base,
            0,
            labels,
            assembly,
        );
    }
    if let LoweredExpr::Binary { op, left, right } = value
        && let (
            LoweredExpr::Local {
                offset: local_offset,
                ..
            },
            LoweredExpr::Integer(value),
        ) = (left.as_ref(), right.as_ref())
        && *local_offset == offset
        && let Some((instruction, immediate)) = aarch64_update_immediate(*op, *value)
    {
        write_assembly!(assembly, "\tldr w0, [sp, #{offset}]\n")?;
        write_assembly!(assembly, "\t{instruction} w0, w0, #{immediate}\n")?;
        return Ok(());
    }
    emit_aarch64_expr(value, temporary_base, 0, labels, assembly)
}
