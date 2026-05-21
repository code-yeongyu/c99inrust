use super::stack_helpers::call_arg_depth;
use crate::ir::{Instruction, LoweredExpr, LoweredFunction, LoweredLValue};
use crate::parser::BinaryOp;

pub(in crate::codegen) fn instruction_depth(instruction: &Instruction) -> usize {
    match instruction {
        Instruction::StoreLocal { value, .. }
        | Instruction::StoreGlobal { value, .. }
        | Instruction::Eval(value)
        | Instruction::Return(Some(value)) => expr_depth(value),
        Instruction::JumpIfZero { condition, .. } => expr_depth(condition),
        Instruction::Return(None)
        | Instruction::Jump { .. }
        | Instruction::Label { .. }
        | Instruction::InitLocalBytes { .. }
        | Instruction::InitLocalInts { .. } => 0,
    }
}

pub(in crate::codegen) fn next_available_label(function: &LoweredFunction) -> usize {
    function
        .instructions
        .iter()
        .filter_map(instruction_label)
        .max()
        .map_or(0, |label| label + 1)
}

pub(in crate::codegen) fn should_share_aarch64_epilogue(
    function: &LoweredFunction,
    stack_bytes: usize,
) -> bool {
    stack_bytes > 0
        && function
            .instructions
            .iter()
            .filter(|instruction| matches!(instruction, Instruction::Return(_)))
            .take(2)
            .count()
            > 1
}

pub(in crate::codegen) const fn instruction_label(instruction: &Instruction) -> Option<usize> {
    match instruction {
        Instruction::StoreLocal { .. }
        | Instruction::StoreGlobal { .. }
        | Instruction::Eval(_)
        | Instruction::Return(_)
        | Instruction::InitLocalBytes { .. }
        | Instruction::InitLocalInts { .. } => None,
        Instruction::JumpIfZero { label, .. }
        | Instruction::Jump { label }
        | Instruction::Label { label } => Some(*label),
    }
}

pub(in crate::codegen) fn expr_depth(expr: &LoweredExpr) -> usize {
    match expr {
        LoweredExpr::Integer(_)
        | LoweredExpr::LongInteger(_)
        | LoweredExpr::DoubleLiteral(_)
        | LoweredExpr::StringLiteral(_)
        | LoweredExpr::Global { .. }
        | LoweredExpr::GlobalAddress { .. }
        | LoweredExpr::Local { .. }
        | LoweredExpr::LocalAddress { .. } => 0,
        LoweredExpr::VaArg { list, .. } => expr_depth(list),
        LoweredExpr::Call { args, .. } => call_arg_depth(args),
        LoweredExpr::IndirectCall { callee, args } => {
            call_arg_depth(args).max(args.len() + 1 + expr_depth(callee))
        }
        LoweredExpr::Cast { expr, .. } | LoweredExpr::Unary { expr, .. } => expr_depth(expr),
        LoweredExpr::GlobalByteSubscript { index, .. }
        | LoweredExpr::GlobalIntSubscript { index, .. }
        | LoweredExpr::GlobalPointerSubscript { index, .. } => expr_depth(index),
        LoweredExpr::PointerSubscript { pointer, index, .. }
        | LoweredExpr::PointerOffset { pointer, index, .. } => {
            pointer_lvalue_address_depth(pointer, index)
        }
        LoweredExpr::PointerFieldAddress { pointer, .. }
        | LoweredExpr::PointerField { pointer, .. } => 1 + expr_depth(pointer),
        LoweredExpr::Assign { target, value } => assign_expr_depth(target, value),
        LoweredExpr::PostIncrement { target, .. } => 1 + lvalue_address_depth(target),
        LoweredExpr::Binary {
            op: BinaryOp::LogicalAnd | BinaryOp::LogicalOr,
            left,
            right,
        }
        | LoweredExpr::Comma { left, right } => expr_depth(left).max(expr_depth(right)),
        LoweredExpr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => expr_depth(condition)
            .max(expr_depth(then_expr))
            .max(expr_depth(else_expr)),
        LoweredExpr::Binary { left, right, .. } => 1 + expr_depth(left).max(expr_depth(right)),
    }
}

pub(in crate::codegen) fn assign_expr_depth(target: &LoweredLValue, value: &LoweredExpr) -> usize {
    expr_depth(value)
        .max(1)
        .max(1 + lvalue_address_depth(target))
}

pub(in crate::codegen) fn lvalue_address_depth(target: &LoweredLValue) -> usize {
    match target {
        LoweredLValue::Local { .. } | LoweredLValue::Global { .. } => 0,
        LoweredLValue::GlobalByteSubscript { index, .. }
        | LoweredLValue::GlobalIntSubscript { index, .. }
        | LoweredLValue::GlobalPointerSubscript { index, .. } => expr_depth(index),
        LoweredLValue::PointerSubscript { pointer, index, .. } => {
            pointer_lvalue_address_depth(pointer, index)
        }
        LoweredLValue::PointerField { pointer, .. } => 1 + expr_depth(pointer),
    }
}

pub(in crate::codegen) fn pointer_lvalue_address_depth(
    pointer: &LoweredExpr,
    index: &LoweredExpr,
) -> usize {
    1 + expr_depth(pointer).max(expr_depth(index))
}
