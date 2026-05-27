use super::frames::LabelAllocator;
use super::target::Target;
use super::widths::{ValueWidth, binary_operand_width, expr_width};
use super::x86_64_conditionals::emit_x86_64_compare_result_to_zero;
use super::x86_64_expr::{emit_x86_64_expr, emit_x86_64_expr_with_width};
use super::x86_64_temporaries::{
    emit_x86_64_load_temporary, emit_x86_64_move_result_to_rhs, emit_x86_64_store_temporary,
};
use crate::diagnostics::CompileResult;
use crate::ir::LoweredExpr;
use crate::parser::{BinaryOp, UnaryOp};

pub(in crate::codegen) fn emit_x86_64_jump_if_zero(
    condition: &LoweredExpr,
    label: &str,
    temporary_base: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    if emit_comparison_false_jump(condition, label, temporary_base, target, labels, assembly)? {
        return Ok(());
    }
    if emit_logical_not_false_jump(condition, label, temporary_base, target, labels, assembly)? {
        return Ok(());
    }
    emit_x86_64_expr(condition, temporary_base, 0, target, labels, assembly)?;
    emit_x86_64_compare_result_to_zero(expr_width(condition), assembly);
    write_assembly!(assembly, "\tje {label}\n")
}

fn emit_comparison_false_jump(
    condition: &LoweredExpr,
    label: &str,
    temporary_base: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<bool> {
    let LoweredExpr::Binary { op, left, right } = condition else {
        return Ok(false);
    };
    let width = binary_operand_width(*op, left, right);
    let Some(jump) = false_jump_for_comparison(*op, width) else {
        return Ok(false);
    };
    let temporary_offset = temporary_base;
    emit_x86_64_expr_with_width(left, width, temporary_base, 1, target, labels, assembly)?;
    emit_x86_64_store_temporary(width, temporary_offset, assembly)?;
    emit_x86_64_expr_with_width(right, width, temporary_base, 1, target, labels, assembly)?;
    emit_x86_64_move_result_to_rhs(width, assembly);
    emit_x86_64_load_temporary(width, temporary_offset, assembly)?;
    emit_compare(width, assembly);
    write_assembly!(assembly, "\t{jump} {label}\n")?;
    Ok(true)
}

fn emit_logical_not_false_jump(
    condition: &LoweredExpr,
    label: &str,
    temporary_base: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<bool> {
    let LoweredExpr::Unary {
        op: UnaryOp::LogicalNot,
        expr,
    } = condition
    else {
        return Ok(false);
    };
    emit_x86_64_expr(expr, temporary_base, 0, target, labels, assembly)?;
    emit_x86_64_compare_result_to_zero(expr_width(expr), assembly);
    write_assembly!(assembly, "\tjne {label}\n")?;
    Ok(true)
}

fn emit_compare(width: ValueWidth, assembly: &mut String) {
    match width {
        ValueWidth::I32 => assembly.push_str("\tcmpl %ecx, %eax\n"),
        ValueWidth::I64 => assembly.push_str("\tcmpq %rcx, %rax\n"),
        ValueWidth::F64 => assembly.push_str("\tucomisd %xmm1, %xmm0\n"),
    }
}

const fn false_jump_for_comparison(op: BinaryOp, width: ValueWidth) -> Option<&'static str> {
    match (op, width) {
        (BinaryOp::Less, ValueWidth::F64) => Some("jae"),
        (BinaryOp::LessEqual, ValueWidth::F64) => Some("ja"),
        (BinaryOp::Greater, ValueWidth::F64) => Some("jbe"),
        (BinaryOp::GreaterEqual, ValueWidth::F64) => Some("jb"),
        (BinaryOp::Less, _) => Some("jge"),
        (BinaryOp::LessEqual, _) => Some("jg"),
        (BinaryOp::Greater, _) => Some("jle"),
        (BinaryOp::GreaterEqual, _) => Some("jl"),
        (BinaryOp::Equal, _) => Some("jne"),
        (BinaryOp::NotEqual, _) => Some("je"),
        (
            BinaryOp::Mul
            | BinaryOp::Div
            | BinaryOp::Mod
            | BinaryOp::Add
            | BinaryOp::Sub
            | BinaryOp::ShiftLeft
            | BinaryOp::ShiftRight
            | BinaryOp::BitAnd
            | BinaryOp::BitXor
            | BinaryOp::BitOr
            | BinaryOp::LogicalAnd
            | BinaryOp::LogicalOr,
            _,
        ) => None,
    }
}
