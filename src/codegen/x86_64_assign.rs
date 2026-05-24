use super::frames::LabelAllocator;
use super::target::Target;
use super::widths::{
    GlobalByteSubscriptExpr, PointerFieldExpr, PointerSubscriptExpr, lowered_lvalue_width,
};
use super::x86_64_expr::emit_x86_64_expr_with_width;
use super::x86_64_global_pointer_stores::{
    emit_x86_64_store_global_pointer_subscript, emit_x86_64_store_pointer_field,
};
use super::x86_64_loads::{emit_x86_64_store_global, emit_x86_64_store_global_bool};
use super::x86_64_pointer_stores::{
    emit_x86_64_store_global_byte_subscript, emit_x86_64_store_global_int_subscript,
    emit_x86_64_store_pointer_subscript,
};
use super::x86_64_temporaries::emit_x86_64_store_result;
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::{LoweredExpr, LoweredLValue};
use crate::parser::ScalarType;

pub(in crate::codegen) fn emit_x86_64_assign(
    target: &LoweredLValue,
    value: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    codegen_target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let width = lowered_lvalue_width(target);
    match target {
        LoweredLValue::Local { offset, .. } => {
            emit_x86_64_expr_with_width(
                value,
                width,
                temporary_base,
                depth,
                codegen_target,
                labels,
                assembly,
            )?;
            emit_x86_64_store_result(width, *offset, assembly)
        }
        LoweredLValue::Global { name, scalar_type } => {
            emit_x86_64_expr_with_width(
                value,
                width,
                temporary_base,
                depth,
                codegen_target,
                labels,
                assembly,
            )?;
            if *scalar_type == ScalarType::Bool {
                emit_x86_64_store_global_bool(name, codegen_target, assembly)
            } else {
                emit_x86_64_store_global(name, width, codegen_target, assembly)
            }
        }
        _ => emit_x86_64_assign_indirect(
            target,
            value,
            temporary_base,
            depth,
            codegen_target,
            labels,
            assembly,
        ),
    }
}

pub(in crate::codegen) fn emit_x86_64_assign_indirect(
    target: &LoweredLValue,
    value: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    codegen_target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    match target {
        LoweredLValue::GlobalByteSubscript {
            name,
            index,
            is_unsigned,
        } => emit_x86_64_store_global_byte_subscript(
            GlobalByteSubscriptExpr {
                name,
                index,
                is_unsigned: *is_unsigned,
            },
            value,
            temporary_base,
            depth,
            codegen_target,
            labels,
            assembly,
        ),
        LoweredLValue::GlobalIntSubscript { name, index } => {
            emit_x86_64_store_global_int_subscript(
                GlobalByteSubscriptExpr {
                    name,
                    index,
                    is_unsigned: false,
                },
                value,
                temporary_base,
                depth,
                codegen_target,
                labels,
                assembly,
            )
        }
        LoweredLValue::GlobalPointerSubscript { name, index } => {
            emit_x86_64_store_global_pointer_subscript(
                GlobalByteSubscriptExpr {
                    name,
                    index,
                    is_unsigned: false,
                },
                value,
                temporary_base,
                depth,
                codegen_target,
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
        } => emit_x86_64_store_pointer_subscript(
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
            codegen_target,
            labels,
            assembly,
        ),
        LoweredLValue::PointerField {
            pointer,
            offset,
            scalar_type,
            byte_size,
            is_unsigned,
        } => emit_x86_64_store_pointer_field(
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
            codegen_target,
            labels,
            assembly,
        ),
        LoweredLValue::Local { .. } | LoweredLValue::Global { .. } => Err(CompileError::new(
            "internal error: expected indirect assignment target",
        )),
    }
}
