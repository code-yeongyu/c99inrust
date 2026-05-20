use super::data_literals::label_name;
use super::frames::LabelAllocator;
use super::stack_helpers::x86_stack_object_offset;
use super::target::Target;
use super::widths::{
    BinaryExpr, ConditionalExpr, ValueWidth, cast_width, expr_width, scalar_width,
};
use super::x86_64_addressing::emit_x86_64_address_expr;
use super::x86_64_binary::{emit_x86_64_binary_expr, emit_x86_64_width_adjustment};
use super::x86_64_calls::emit_x86_64_call_expr;
use super::x86_64_conditionals::emit_x86_64_conditional;
use super::x86_64_expr_special::emit_x86_64_global_or_assignment_expr;
use super::x86_64_loads::{emit_x86_64_load_double_literal, emit_x86_64_load_string_address};
use super::x86_64_temporaries::emit_x86_64_load_temporary;
use super::x86_64_unary::{emit_x86_64_i64_integer, emit_x86_64_integer, emit_x86_64_unary_expr};
use crate::diagnostics::CompileResult;
use crate::ir::LoweredExpr;

pub(in crate::codegen) fn emit_x86_64_expr(
    expr: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    emit_x86_64_expr_with_width(
        expr,
        expr_width(expr),
        temporary_base,
        depth,
        target,
        labels,
        assembly,
    )
}

pub(in crate::codegen) fn emit_x86_64_expr_with_width(
    expr: &LoweredExpr,
    target_width: ValueWidth,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    if let (LoweredExpr::Integer(value) | LoweredExpr::LongInteger(value), ValueWidth::I64) =
        (expr, target_width)
    {
        return emit_x86_64_i64_integer(*value, assembly);
    }
    if matches!(expr, LoweredExpr::IndirectCall { .. }) && target_width == ValueWidth::I64 {
        return emit_x86_64_expr_natural(expr, temporary_base, depth, target, labels, assembly);
    }
    let natural_width = expr_width(expr);
    emit_x86_64_expr_natural(expr, temporary_base, depth, target, labels, assembly)?;
    emit_x86_64_width_adjustment(natural_width, target_width, assembly);
    Ok(())
}

pub(in crate::codegen) fn emit_x86_64_expr_natural(
    expr: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    match expr {
        expr @ (LoweredExpr::Call { .. } | LoweredExpr::IndirectCall { .. }) => {
            emit_x86_64_call_expr(expr, temporary_base, depth, target, labels, assembly)
        }
        LoweredExpr::Integer(value) => emit_x86_64_integer(*value, assembly),
        LoweredExpr::LongInteger(value) => emit_x86_64_i64_integer(*value, assembly),
        LoweredExpr::DoubleLiteral(value) => {
            emit_x86_64_load_double_literal(value, target, labels, assembly)
        }
        LoweredExpr::StringLiteral(value) => {
            emit_x86_64_load_string_address(value, target, labels, assembly)
        }
        LoweredExpr::LocalAddress { offset, byte_size } => write_assembly!(
            assembly,
            "\tleaq {}(%rbp), %rax\n",
            x86_stack_object_offset(*offset, *byte_size)
        ),
        LoweredExpr::GlobalAddress { name } => {
            let label = label_name(name, target);
            write_assembly!(assembly, "\tleaq {label}(%rip), %rax\n")
        }
        expr @ (LoweredExpr::PointerOffset { .. } | LoweredExpr::PointerFieldAddress { .. }) => {
            emit_x86_64_address_expr(expr, temporary_base, depth, target, labels, assembly)
        }
        expr @ (LoweredExpr::Global { .. }
        | LoweredExpr::GlobalByteSubscript { .. }
        | LoweredExpr::GlobalIntSubscript { .. }
        | LoweredExpr::GlobalPointerSubscript { .. }
        | LoweredExpr::PointerSubscript { .. }
        | LoweredExpr::PointerField { .. }
        | LoweredExpr::Assign { .. }
        | LoweredExpr::PostIncrement { .. }) => emit_x86_64_global_or_assignment_expr(
            expr,
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        ),
        LoweredExpr::Local {
            offset,
            scalar_type,
        } => emit_x86_64_load_temporary(scalar_width(*scalar_type), *offset, assembly),
        LoweredExpr::Unary { op, expr } => {
            emit_x86_64_unary_expr(*op, expr, temporary_base, depth, target, labels, assembly)
        }
        LoweredExpr::Cast {
            target: scalar_type,
            expr,
        } => emit_x86_64_expr_with_width(
            expr,
            cast_width(*scalar_type, expr),
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        ),
        LoweredExpr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => emit_x86_64_conditional(
            ConditionalExpr {
                condition,
                then_expr,
                else_expr,
            },
            expr_width(expr),
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        ),
        LoweredExpr::Comma { left, right } => {
            emit_x86_64_expr(left, temporary_base, depth, target, labels, assembly)?;
            emit_x86_64_expr_natural(right, temporary_base, depth, target, labels, assembly)
        }
        LoweredExpr::Binary { op, left, right } => emit_x86_64_binary_expr(
            BinaryExpr {
                op: *op,
                left,
                right,
            },
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        ),
    }
}
