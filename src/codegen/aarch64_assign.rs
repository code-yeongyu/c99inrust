use super::aarch64_expr::emit_aarch64_expr_with_width;
use super::aarch64_global_stores::{
    emit_aarch64_store_global_byte_subscript, emit_aarch64_store_global_int_subscript,
    emit_aarch64_store_global_pointer_subscript, emit_aarch64_store_pointer_field,
};
use super::aarch64_loads::emit_aarch64_store_global;
use super::aarch64_pointer_subscript::emit_aarch64_store_pointer_subscript;
use super::aarch64_temporaries::emit_aarch64_store_result;
use super::frames::LabelAllocator;
use super::widths::{
    GlobalByteSubscriptExpr, PointerFieldExpr, PointerSubscriptExpr, lowered_lvalue_width,
};
use crate::diagnostics::CompileResult;
use crate::ir::{LoweredExpr, LoweredLValue};

pub(in crate::codegen) fn emit_aarch64_assign(
    target: &LoweredLValue,
    value: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let width = lowered_lvalue_width(target);
    match target {
        LoweredLValue::Local { offset, .. } => {
            emit_aarch64_expr_with_width(value, width, temporary_base, depth, labels, assembly)?;
            emit_aarch64_store_result(width, *offset, assembly)
        }
        LoweredLValue::Global { name, .. } => {
            emit_aarch64_expr_with_width(value, width, temporary_base, depth, labels, assembly)?;
            emit_aarch64_store_global(name, width, labels.target, assembly)
        }
        LoweredLValue::GlobalByteSubscript {
            name,
            index,
            is_unsigned,
        } => emit_aarch64_store_global_byte_subscript(
            GlobalByteSubscriptExpr {
                name,
                index,
                is_unsigned: *is_unsigned,
            },
            value,
            temporary_base,
            depth,
            labels,
            assembly,
        ),
        LoweredLValue::GlobalIntSubscript { name, index } => {
            emit_aarch64_store_global_int_subscript(
                GlobalByteSubscriptExpr {
                    name,
                    index,
                    is_unsigned: false,
                },
                value,
                temporary_base,
                depth,
                labels,
                assembly,
            )
        }
        LoweredLValue::GlobalPointerSubscript { name, index } => {
            emit_aarch64_store_global_pointer_subscript(
                GlobalByteSubscriptExpr {
                    name,
                    index,
                    is_unsigned: false,
                },
                value,
                temporary_base,
                depth,
                labels,
                assembly,
            )
        }
        LoweredLValue::PointerSubscript {
            pointer,
            index,
            element_type,
            element_byte_size,
            element_unsigned,
        } => emit_aarch64_store_pointer_subscript(
            PointerSubscriptExpr {
                pointer,
                index,
                element_type: *element_type,
                element_byte_size: *element_byte_size,
                element_unsigned: *element_unsigned,
            },
            value,
            temporary_base,
            depth,
            labels,
            assembly,
        ),
        LoweredLValue::PointerField {
            pointer,
            offset,
            scalar_type,
            byte_size,
            is_unsigned,
        } => emit_aarch64_store_pointer_field(
            PointerFieldExpr {
                pointer,
                offset: *offset,
                scalar_type: *scalar_type,
                byte_size: *byte_size,
                is_unsigned: *is_unsigned,
            },
            value,
            temporary_base,
            depth,
            labels,
            assembly,
        ),
    }
}
