use super::aarch64_addressing::emit_aarch64_load_pointer_field;
use super::aarch64_assign::emit_aarch64_assign;
use super::aarch64_loads::{
    emit_aarch64_load_global, emit_aarch64_load_global_byte_subscript,
    emit_aarch64_load_global_int_subscript, emit_aarch64_load_global_pointer_subscript,
    emit_aarch64_load_pointer_subscript,
};
use super::aarch64_post_increment::emit_aarch64_post_increment;
use super::frames::LabelAllocator;
use super::widths::{PointerFieldExpr, PointerSubscriptExpr, scalar_width};
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::LoweredExpr;

pub(in crate::codegen) fn emit_aarch64_memory_expr(
    expr: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    match expr {
        LoweredExpr::Global { name, scalar_type } => {
            emit_aarch64_load_global(name, scalar_width(*scalar_type), labels.target, assembly)
        }
        LoweredExpr::GlobalByteSubscript {
            name,
            index,
            is_unsigned,
        } => emit_aarch64_load_global_byte_subscript(
            name,
            index,
            *is_unsigned,
            temporary_base,
            depth,
            labels,
            assembly,
        ),
        LoweredExpr::GlobalIntSubscript { name, index } => emit_aarch64_load_global_int_subscript(
            name,
            index,
            temporary_base,
            depth,
            labels,
            assembly,
        ),
        LoweredExpr::GlobalPointerSubscript { name, index } => {
            emit_aarch64_load_global_pointer_subscript(
                name,
                index,
                temporary_base,
                depth,
                labels,
                assembly,
            )
        }
        LoweredExpr::PointerSubscript {
            pointer,
            index,
            element_type,
            element_byte_size,
            element_unsigned,
        } => emit_aarch64_load_pointer_subscript(
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
            assembly,
        ),
        LoweredExpr::PointerField {
            pointer,
            offset,
            scalar_type,
            byte_size,
            is_unsigned,
        } => emit_aarch64_load_pointer_field(
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
            assembly,
        ),
        LoweredExpr::Assign { target, value } => {
            emit_aarch64_assign(target, value, temporary_base, depth, labels, assembly)
        }
        LoweredExpr::PostIncrement { target, increment } => {
            emit_aarch64_post_increment(target, *increment, temporary_base, depth, labels, assembly)
        }
        _ => Err(CompileError::new(
            "internal error: expected aarch64 memory expression",
        )),
    }
}
