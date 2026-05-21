use super::aarch64_addressing::emit_aarch64_pointer_offset;
use super::aarch64_binary::{
    emit_aarch64_binary_expr, emit_aarch64_i32_to_register, emit_aarch64_i64_to_register,
    emit_aarch64_width_adjustment,
};
use super::aarch64_calls::emit_aarch64_call_expr;
use super::aarch64_conditionals::emit_aarch64_conditional;
use super::aarch64_loads::{emit_aarch64_load_double_literal, emit_aarch64_load_string_address};
use super::aarch64_memory_expr::emit_aarch64_memory_expr;
use super::aarch64_temporaries::{emit_aarch64_load_f32_local, emit_aarch64_load_temporary};
use super::aarch64_unary::emit_aarch64_unary_expr;
use super::aarch64_variadic::emit_aarch64_va_arg;
use super::data_literals::label_name;
use super::frames::LabelAllocator;
use super::widths::{
    BinaryExpr, ConditionalExpr, PointerOffsetExpr, ValueWidth, cast_width, expr_width,
    scalar_width,
};
use crate::diagnostics::CompileResult;
use crate::ir::LoweredExpr;
use crate::parser::ScalarType;

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
    if let (LoweredExpr::Integer(value) | LoweredExpr::LongInteger(value), ValueWidth::I64) =
        (expr, target_width)
    {
        return emit_aarch64_i64_to_register(*value, "x0", assembly);
    }
    if matches!(expr, LoweredExpr::IndirectCall { .. }) && target_width == ValueWidth::I64 {
        return emit_aarch64_expr_natural(expr, temporary_base, depth, labels, assembly);
    }
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
        LoweredExpr::LongInteger(value) => emit_aarch64_i64_to_register(*value, "x0", assembly),
        LoweredExpr::DoubleLiteral(value) => {
            emit_aarch64_load_double_literal(value, labels, assembly)
        }
        LoweredExpr::StringLiteral(value) => {
            emit_aarch64_load_string_address(value, labels, assembly)
        }
        LoweredExpr::VaArg { list, scalar_type } => {
            emit_aarch64_va_arg(list, *scalar_type, temporary_base, depth, labels, assembly)
        }
        LoweredExpr::LocalAddress { offset, .. } => {
            write_assembly!(assembly, "\tadd x0, sp, #{offset}\n")
        }
        LoweredExpr::GlobalAddress { name } => emit_aarch64_global_address(name, labels, assembly),
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
        LoweredExpr::PointerFieldAddress { pointer, offset } => emit_aarch64_pointer_field_address(
            pointer,
            *offset,
            temporary_base,
            depth,
            labels,
            assembly,
        ),
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
        } => emit_aarch64_local_expr(*offset, *scalar_type, assembly),
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
        LoweredExpr::Comma { left, right } => {
            emit_aarch64_expr(left, temporary_base, depth, labels, assembly)?;
            emit_aarch64_expr_natural(right, temporary_base, depth, labels, assembly)
        }
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

fn emit_aarch64_local_expr(
    offset: usize,
    scalar_type: ScalarType,
    assembly: &mut String,
) -> CompileResult<()> {
    if scalar_type == ScalarType::ComplexFloat {
        return emit_aarch64_load_f32_local(offset, assembly);
    }
    emit_aarch64_load_temporary(scalar_width(scalar_type), offset, assembly)
}

fn emit_aarch64_pointer_field_address(
    pointer: &LoweredExpr,
    offset: usize,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
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

fn emit_aarch64_global_address(
    name: &str,
    labels: &LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = label_name(name, labels.target);
    write_assembly!(assembly, "\tadrp x0, {label}@PAGE\n")?;
    write_assembly!(assembly, "\tadd x0, x0, {label}@PAGEOFF\n")
}
