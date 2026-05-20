use super::aarch64_addressing::{emit_aarch64_load_pointer_field, emit_aarch64_pointer_offset};
use super::aarch64_assign::emit_aarch64_assign;
use super::aarch64_binary::{
    emit_aarch64_binary_expr, emit_aarch64_i32_to_register, emit_aarch64_width_adjustment,
};
use super::aarch64_calls::emit_aarch64_call_expr;
use super::aarch64_conditionals::emit_aarch64_conditional;
use super::aarch64_loads::{
    emit_aarch64_load_double_literal, emit_aarch64_load_global,
    emit_aarch64_load_global_byte_subscript, emit_aarch64_load_global_int_subscript,
    emit_aarch64_load_global_pointer_subscript, emit_aarch64_load_pointer_subscript,
    emit_aarch64_load_string_address,
};
use super::aarch64_post_increment::emit_aarch64_post_increment;
use super::aarch64_temporaries::emit_aarch64_load_temporary;
use super::aarch64_unary::emit_aarch64_unary_expr;
use super::data_literals::label_name;
use super::frames::LabelAllocator;
use super::widths::{
    BinaryExpr, ConditionalExpr, PointerFieldExpr, PointerOffsetExpr, PointerSubscriptExpr,
    ValueWidth, cast_width, expr_width, scalar_width,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::LoweredExpr;

pub(in crate::codegen) fn emit_aarch64_expr(
    expr: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    emit_aarch64_expr_with_width(
        expr,
        expr_width(expr),
        temporary_base,
        depth,
        labels,
        assembly,
    )
}

pub(in crate::codegen) fn emit_aarch64_expr_with_width(
    expr: &LoweredExpr,
    target_width: ValueWidth,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let natural_width = expr_width(expr);
    emit_aarch64_expr_natural(expr, temporary_base, depth, labels, assembly)?;
    emit_aarch64_width_adjustment(natural_width, target_width, assembly);
    Ok(())
}

pub(in crate::codegen) fn emit_aarch64_expr_natural(
    expr: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    match expr {
        expr @ (LoweredExpr::Call { .. } | LoweredExpr::IndirectCall { .. }) => {
            emit_aarch64_call_expr(expr, temporary_base, depth, labels, assembly)
        }
        LoweredExpr::Integer(value) => emit_aarch64_i32_to_register(*value, "w0", assembly),
        LoweredExpr::DoubleLiteral(value) => {
            emit_aarch64_load_double_literal(value, labels, assembly)
        }
        LoweredExpr::StringLiteral(value) => {
            emit_aarch64_load_string_address(value, labels, assembly)
        }
        LoweredExpr::LocalAddress { offset, .. } => {
            write_assembly!(assembly, "\tadd x0, sp, #{offset}\n")
        }
        LoweredExpr::GlobalAddress { name } => {
            let label = label_name(name, labels.target);
            write_assembly!(assembly, "\tadrp x0, {label}@PAGE\n")?;
            write_assembly!(assembly, "\tadd x0, x0, {label}@PAGEOFF\n")
        }
        LoweredExpr::PointerOffset {
            pointer,
            index,
            byte_size,
        } => emit_aarch64_pointer_offset(
            PointerOffsetExpr {
                pointer,
                index,
                byte_size: *byte_size,
            },
            temporary_base,
            depth,
            labels,
            assembly,
        ),
        LoweredExpr::PointerFieldAddress { pointer, offset } => {
            emit_aarch64_expr_with_width(
                pointer,
                ValueWidth::I64,
                temporary_base,
                depth + 1,
                labels,
                assembly,
            )?;
            write_assembly!(assembly, "\tadd x0, x0, #{offset}\n")
        }
        expr @ (LoweredExpr::Global { .. }
        | LoweredExpr::GlobalByteSubscript { .. }
        | LoweredExpr::GlobalIntSubscript { .. }
        | LoweredExpr::GlobalPointerSubscript { .. }
        | LoweredExpr::PointerSubscript { .. }
        | LoweredExpr::PointerField { .. }
        | LoweredExpr::Assign { .. }
        | LoweredExpr::PostIncrement { .. }) => {
            emit_aarch64_memory_expr(expr, temporary_base, depth, labels, assembly)
        }
        LoweredExpr::Local {
            offset,
            scalar_type,
        } => emit_aarch64_load_temporary(scalar_width(*scalar_type), *offset, assembly),
        LoweredExpr::Unary { op, expr } => {
            emit_aarch64_unary_expr(*op, expr, temporary_base, depth, labels, assembly)
        }
        LoweredExpr::Cast { target, expr } => emit_aarch64_expr_with_width(
            expr,
            cast_width(*target, expr),
            temporary_base,
            depth,
            labels,
            assembly,
        ),
        LoweredExpr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => emit_aarch64_conditional(
            ConditionalExpr {
                condition,
                then_expr,
                else_expr,
            },
            expr_width(expr),
            temporary_base,
            depth,
            labels,
            assembly,
        ),
        LoweredExpr::Binary { op, left, right } => emit_aarch64_binary_expr(
            BinaryExpr {
                op: *op,
                left,
                right,
            },
            temporary_base,
            depth,
            labels,
            assembly,
        ),
    }
}

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
        LoweredExpr::GlobalByteSubscript { name, index } => {
            emit_aarch64_load_global_byte_subscript(
                name,
                index,
                temporary_base,
                depth,
                labels,
                assembly,
            )
        }
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
