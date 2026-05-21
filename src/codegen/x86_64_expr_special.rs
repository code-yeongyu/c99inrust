use super::frames::LabelAllocator;
use super::target::Target;
use super::widths::{
    GlobalByteSubscriptExpr, PointerFieldExpr, PointerSubscriptExpr, scalar_width,
};
use super::x86_64_addressing::emit_x86_64_load_pointer_field;
use super::x86_64_assign::emit_x86_64_assign;
use super::x86_64_loads::{
    emit_x86_64_load_global, emit_x86_64_load_global_byte_subscript,
    emit_x86_64_load_global_f32_as_f64, emit_x86_64_load_global_int_subscript,
    emit_x86_64_load_global_pointer_subscript, emit_x86_64_load_pointer_subscript,
};
use super::x86_64_post_increment::emit_x86_64_post_increment;
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::LoweredExpr;
use crate::parser::ScalarType;

pub(in crate::codegen) fn emit_x86_64_global_or_assignment_expr(
    expr: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    match expr {
        LoweredExpr::Global { name, scalar_type } => {
            if *scalar_type == ScalarType::ComplexFloat {
                return emit_x86_64_load_global_f32_as_f64(name, target, assembly);
            }
            emit_x86_64_load_global(name, scalar_width(*scalar_type), target, assembly)
        }
        LoweredExpr::GlobalByteSubscript {
            name,
            index,
            is_unsigned,
        } => emit_x86_64_load_global_byte_subscript(
            GlobalByteSubscriptExpr {
                name,
                index,
                is_unsigned: *is_unsigned,
            },
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        ),
        LoweredExpr::GlobalIntSubscript { name, index } => emit_x86_64_load_global_int_subscript(
            name,
            index,
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        ),
        LoweredExpr::GlobalPointerSubscript { name, index } => {
            emit_x86_64_load_global_pointer_subscript(
                name,
                index,
                temporary_base,
                depth,
                target,
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
        } => emit_x86_64_load_pointer_subscript(
            PointerSubscriptExpr {
                pointer,
                index,
                element_type: *element_type,
                element_byte_size: *element_byte_size,
                element_unsigned: *element_unsigned,
            },
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        ),
        _ => emit_x86_64_pointer_field_or_assignment_expr(
            expr,
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        ),
    }
}

pub(in crate::codegen) fn emit_x86_64_pointer_field_or_assignment_expr(
    expr: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    match expr {
        LoweredExpr::PointerField {
            pointer,
            offset,
            scalar_type,
            byte_size,
            is_unsigned,
        } => emit_x86_64_load_pointer_field(
            PointerFieldExpr {
                pointer,
                offset: *offset,
                scalar_type: *scalar_type,
                byte_size: *byte_size,
                is_unsigned: *is_unsigned,
            },
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        ),
        LoweredExpr::Assign {
            target: lvalue,
            value,
        } => emit_x86_64_assign(
            lvalue,
            value,
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        ),
        LoweredExpr::PostIncrement {
            target: lvalue,
            increment,
        } => emit_x86_64_post_increment(
            lvalue,
            *increment,
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        ),
        _ => Err(CompileError::new(
            "internal error: expected x86-64 global expression",
        )),
    }
}
