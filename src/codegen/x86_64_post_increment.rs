use super::frames::LabelAllocator;
use super::sized_fields::{
    emit_x86_64_load as emit_x86_64_load_sized_field,
    emit_x86_64_store as emit_x86_64_store_sized_field,
};
use super::target::Target;
use super::widths::{
    PointerFieldExpr, PointerSubscriptExpr, TEMPORARY_BYTES, ValueWidth, lowered_lvalue_width,
    scalar_width,
};
use super::x86_64_expr::emit_x86_64_expr_with_width;
use super::x86_64_loads::{emit_x86_64_load_global, emit_x86_64_store_global};
use super::x86_64_pointer_stores::{
    emit_x86_64_load_pointer_subscript_result, emit_x86_64_store_pointer_subscript_result,
};
use super::x86_64_temporaries::{
    emit_x86_64_load_temporary, emit_x86_64_load_temporary_to_register, emit_x86_64_store_result,
    emit_x86_64_store_temporary,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::LoweredLValue;

pub(in crate::codegen) fn emit_x86_64_post_increment(
    target: &LoweredLValue,
    increment: i64,
    temporary_base: usize,
    depth: usize,
    codegen_target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let width = lowered_lvalue_width(target);
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
    match target {
        LoweredLValue::Local { offset, .. } => {
            emit_x86_64_load_temporary(width, *offset, assembly)?;
            emit_x86_64_store_temporary(width, value_offset, assembly)?;
            emit_x86_64_increment_result(width, increment, assembly)?;
            emit_x86_64_store_result(width, *offset, assembly)?;
            emit_x86_64_load_temporary(width, value_offset, assembly)
        }
        LoweredLValue::Global { name, .. } => {
            emit_x86_64_load_global(name, width, codegen_target, assembly)?;
            emit_x86_64_store_temporary(width, value_offset, assembly)?;
            emit_x86_64_increment_result(width, increment, assembly)?;
            emit_x86_64_store_global(name, width, codegen_target, assembly)?;
            emit_x86_64_load_temporary(width, value_offset, assembly)
        }
        LoweredLValue::PointerField {
            pointer,
            offset,
            scalar_type,
            byte_size,
            is_unsigned,
        } => emit_x86_64_post_increment_pointer_field(
            PointerFieldExpr {
                pointer,
                offset: *offset,
                scalar_type: *scalar_type,
                byte_size: *byte_size,
                is_unsigned: *is_unsigned,
            },
            temporary_base,
            depth,
            codegen_target,
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
        } => emit_x86_64_post_increment_pointer_subscript(
            PointerSubscriptExpr {
                pointer,
                index,
                element_type: *element_type,
                element_byte_size: *element_byte_size,
                element_unsigned: *element_unsigned,
            },
            temporary_base,
            depth,
            codegen_target,
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

pub(in crate::codegen) fn emit_x86_64_post_increment_pointer_subscript(
    subscript: PointerSubscriptExpr<'_>,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    increment: i64,
    assembly: &mut String,
) -> CompileResult<()> {
    let width = scalar_width(subscript.element_type);
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
    let base_offset = temporary_base + ((depth + 1) * TEMPORARY_BYTES);
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
    emit_x86_64_load_pointer_subscript_result(
        subscript.element_byte_size,
        width,
        subscript.element_unsigned,
        assembly,
    )?;
    emit_x86_64_store_temporary(width, value_offset, assembly)?;
    emit_x86_64_increment_result(width, increment, assembly)?;
    emit_x86_64_store_pointer_subscript_result(subscript.element_byte_size, width, assembly)?;
    emit_x86_64_load_temporary(width, value_offset, assembly)
}

pub(in crate::codegen) fn emit_x86_64_post_increment_pointer_field(
    field: PointerFieldExpr<'_>,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    increment: i64,
    assembly: &mut String,
) -> CompileResult<()> {
    let width = scalar_width(field.scalar_type);
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
    let base_offset = temporary_base + ((depth + 1) * TEMPORARY_BYTES);
    emit_x86_64_expr_with_width(
        field.pointer,
        ValueWidth::I64,
        temporary_base,
        depth + 2,
        target,
        labels,
        assembly,
    )?;
    emit_x86_64_store_temporary(ValueWidth::I64, base_offset, assembly)?;
    emit_x86_64_load_temporary_to_register(ValueWidth::I64, base_offset, "%rcx", assembly)?;
    emit_x86_64_load_sized_field(
        field.byte_size,
        width,
        field.is_unsigned,
        "%rcx",
        field.offset,
        assembly,
    )?;
    emit_x86_64_store_temporary(width, value_offset, assembly)?;
    emit_x86_64_increment_result(width, increment, assembly)?;
    emit_x86_64_load_temporary_to_register(ValueWidth::I64, base_offset, "%rcx", assembly)?;
    emit_x86_64_store_sized_field(field.byte_size, width, "%rcx", field.offset, assembly)?;
    emit_x86_64_load_temporary(width, value_offset, assembly)
}

pub(in crate::codegen) fn emit_x86_64_increment_result(
    width: ValueWidth,
    increment: i64,
    assembly: &mut String,
) -> CompileResult<()> {
    let immediate = i32::try_from(increment)
        .map_err(|_| CompileError::new("post-increment value does not fit x86-64 immediate"))?;
    match width {
        ValueWidth::I32 => write_assembly!(assembly, "\taddl ${immediate}, %eax\n"),
        ValueWidth::I64 => write_assembly!(assembly, "\taddq ${immediate}, %rax\n"),
        ValueWidth::F64 => Err(CompileError::new("unsupported double post-increment")),
    }
}
