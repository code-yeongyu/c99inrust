use super::aarch64_assign::{
    emit_aarch64_load_pointer_subscript_result, emit_aarch64_store_pointer_subscript_result,
};
use super::aarch64_expr::emit_aarch64_expr_with_width;
use super::aarch64_loads::{emit_aarch64_load_global, emit_aarch64_store_global};
use super::aarch64_temporaries::{
    emit_aarch64_load_temporary, emit_aarch64_load_temporary_to_register,
    emit_aarch64_store_result, emit_aarch64_store_temporary,
};
use super::frames::LabelAllocator;
use super::sized_fields::{
    emit_aarch64_load as emit_aarch64_load_sized_field,
    emit_aarch64_store as emit_aarch64_store_sized_field,
};
use super::widths::{
    PointerFieldExpr, PointerSubscriptExpr, TEMPORARY_BYTES, ValueWidth, lowered_lvalue_width,
    scalar_width,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::LoweredLValue;

pub(in crate::codegen) fn emit_aarch64_post_increment(
    target: &LoweredLValue,
    increment: i64,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let width = lowered_lvalue_width(target);
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
    match target {
        LoweredLValue::Local { offset, .. } => {
            emit_aarch64_load_temporary(width, *offset, assembly)?;
            emit_aarch64_store_temporary(width, value_offset, assembly)?;
            emit_aarch64_increment_result(width, increment, assembly)?;
            emit_aarch64_store_result(width, *offset, assembly)?;
            emit_aarch64_load_temporary(width, value_offset, assembly)
        }
        LoweredLValue::Global { name, .. } => {
            emit_aarch64_load_global(name, width, labels.target, assembly)?;
            emit_aarch64_store_temporary(width, value_offset, assembly)?;
            emit_aarch64_increment_result(width, increment, assembly)?;
            emit_aarch64_store_global(name, width, labels.target, assembly)?;
            emit_aarch64_load_temporary(width, value_offset, assembly)
        }
        LoweredLValue::PointerField {
            pointer,
            offset,
            scalar_type,
            byte_size,
            is_unsigned,
        } => emit_aarch64_post_increment_pointer_field(
            PointerFieldExpr {
                pointer,
                offset: *offset,
                scalar_type: *scalar_type,
                byte_size: *byte_size,
                is_unsigned: *is_unsigned,
            },
            temporary_base,
            depth,
            labels,
            increment,
            assembly,
        ),
        LoweredLValue::PointerSubscript {
            pointer,
            index,
            element_type,
            element_byte_size,
            element_unsigned,
        } => emit_aarch64_post_increment_pointer_subscript(
            PointerSubscriptExpr {
                pointer,
                index,
                element_type: *element_type,
                element_byte_size: *element_byte_size,
                element_unsigned: *element_unsigned,
            },
            temporary_base,
            depth,
            labels,
            increment,
            assembly,
        ),
        LoweredLValue::GlobalByteSubscript { .. }
        | LoweredLValue::GlobalIntSubscript { .. }
        | LoweredLValue::GlobalPointerSubscript { .. } => Err(CompileError::new(
            "post-increment expression supports direct lvalues only",
        )),
    }
}

pub(in crate::codegen) fn emit_aarch64_post_increment_pointer_subscript(
    subscript: PointerSubscriptExpr<'_>,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    increment: i64,
    assembly: &mut String,
) -> CompileResult<()> {
    let width = scalar_width(subscript.element_type);
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
    let base_offset = temporary_base + ((depth + 1) * TEMPORARY_BYTES);
    emit_aarch64_expr_with_width(
        subscript.pointer,
        ValueWidth::I64,
        temporary_base,
        depth + 2,
        labels,
        assembly,
    )?;
    emit_aarch64_store_temporary(ValueWidth::I64, base_offset, assembly)?;
    emit_aarch64_expr_with_width(
        subscript.index,
        ValueWidth::I32,
        temporary_base,
        depth + 2,
        labels,
        assembly,
    )?;
    assembly.push_str("\tmov w17, w0\n");
    emit_aarch64_load_temporary_to_register(ValueWidth::I64, base_offset, "16", assembly)?;
    emit_aarch64_load_pointer_subscript_result(
        subscript.element_byte_size,
        width,
        subscript.element_unsigned,
        assembly,
    )?;
    emit_aarch64_store_temporary(width, value_offset, assembly)?;
    emit_aarch64_increment_result(width, increment, assembly)?;
    emit_aarch64_store_pointer_subscript_result(subscript.element_byte_size, width, assembly)?;
    emit_aarch64_load_temporary(width, value_offset, assembly)
}

pub(in crate::codegen) fn emit_aarch64_post_increment_pointer_field(
    field: PointerFieldExpr<'_>,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    increment: i64,
    assembly: &mut String,
) -> CompileResult<()> {
    let width = scalar_width(field.scalar_type);
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
    let base_offset = temporary_base + ((depth + 1) * TEMPORARY_BYTES);
    emit_aarch64_expr_with_width(
        field.pointer,
        ValueWidth::I64,
        temporary_base,
        depth + 2,
        labels,
        assembly,
    )?;
    emit_aarch64_store_temporary(ValueWidth::I64, base_offset, assembly)?;
    emit_aarch64_load_temporary_to_register(ValueWidth::I64, base_offset, "16", assembly)?;
    emit_aarch64_load_sized_field(
        field.byte_size,
        width,
        field.is_unsigned,
        "x16",
        field.offset,
        assembly,
    )?;
    emit_aarch64_store_temporary(width, value_offset, assembly)?;
    emit_aarch64_increment_result(width, increment, assembly)?;
    emit_aarch64_load_temporary_to_register(ValueWidth::I64, base_offset, "16", assembly)?;
    emit_aarch64_store_sized_field(field.byte_size, width, "x16", field.offset, assembly)?;
    emit_aarch64_load_temporary(width, value_offset, assembly)
}

pub(in crate::codegen) fn emit_aarch64_increment_result(
    width: ValueWidth,
    increment: i64,
    assembly: &mut String,
) -> CompileResult<()> {
    let immediate = increment.unsigned_abs();
    if immediate > 4095 {
        return Err(CompileError::new(
            "post-increment value does not fit aarch64 immediate",
        ));
    }
    let op = if increment.is_negative() {
        "sub"
    } else {
        "add"
    };
    match width {
        ValueWidth::I32 => {
            write_assembly!(assembly, "\t{op} w0, w0, #{immediate}\n")
        }
        ValueWidth::I64 => {
            write_assembly!(assembly, "\t{op} x0, x0, #{immediate}\n")
        }
        ValueWidth::F64 => Err(CompileError::new("unsupported double post-increment")),
    }
}
