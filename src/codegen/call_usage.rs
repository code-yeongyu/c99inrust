use crate::ir::{Instruction, LoweredExpr, LoweredFunction, LoweredLValue};
use crate::parser::BinaryOp;

pub(in crate::codegen) fn function_uses_call(function: &LoweredFunction) -> bool {
    function.instructions.iter().any(instruction_uses_call)
}

pub(in crate::codegen) fn function_uses_aarch64_preserved_temp(function: &LoweredFunction) -> bool {
    function
        .instructions
        .iter()
        .any(instruction_needs_preserved_temp)
}

pub(in crate::codegen) fn instruction_needs_preserved_temp(instruction: &Instruction) -> bool {
    match instruction {
        Instruction::StoreLocal { value, .. }
        | Instruction::StoreGlobal { value, .. }
        | Instruction::JumpIfZero {
            condition: value, ..
        }
        | Instruction::Eval(value)
        | Instruction::Return(Some(value)) => expr_needs_preserved_temp(value),
        Instruction::StoreComplexReturn { pointer, .. } => expr_needs_preserved_temp(pointer),
        Instruction::Return(None)
        | Instruction::Jump { .. }
        | Instruction::Label { .. }
        | Instruction::InitLocalBytes { .. }
        | Instruction::InitLocalInts { .. } => false,
    }
}

pub(in crate::codegen) fn expr_needs_preserved_temp(expr: &LoweredExpr) -> bool {
    match expr {
        LoweredExpr::Binary {
            op: BinaryOp::LogicalAnd | BinaryOp::LogicalOr,
            left,
            right,
        } => expr_needs_preserved_temp(left) || expr_needs_preserved_temp(right),
        LoweredExpr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => {
            expr_needs_preserved_temp(condition)
                || expr_needs_preserved_temp(then_expr)
                || expr_needs_preserved_temp(else_expr)
        }
        LoweredExpr::IndexSelect {
            index,
            cases,
            default,
        } => {
            expr_needs_preserved_temp(index)
                || cases.iter().any(expr_needs_preserved_temp)
                || expr_needs_preserved_temp(default)
        }
        LoweredExpr::Binary { left, right, .. } => {
            expr_is_direct_call(right)
                || expr_needs_preserved_temp(left)
                || expr_needs_preserved_temp(right)
        }
        LoweredExpr::Cast { expr, .. } | LoweredExpr::Unary { expr, .. } => {
            expr_needs_preserved_temp(expr)
        }
        LoweredExpr::VaArg { list, .. } => expr_needs_preserved_temp(list),
        LoweredExpr::Comma { left, right } => {
            expr_needs_preserved_temp(left) || expr_needs_preserved_temp(right)
        }
        LoweredExpr::GlobalByteSubscript { index, .. }
        | LoweredExpr::GlobalIntSubscript { index, .. }
        | LoweredExpr::GlobalPointerSubscript { index, .. } => expr_needs_preserved_temp(index),
        LoweredExpr::PointerSubscript { pointer, index, .. }
        | LoweredExpr::PointerOffset { pointer, index, .. } => {
            expr_needs_preserved_temp(pointer) || expr_needs_preserved_temp(index)
        }
        LoweredExpr::PointerFieldAddress { pointer, .. }
        | LoweredExpr::PointerField { pointer, .. } => expr_needs_preserved_temp(pointer),
        LoweredExpr::Assign { target, value } => {
            lvalue_needs_preserved_temp(target) || expr_needs_preserved_temp(value)
        }
        LoweredExpr::PostIncrement { target, .. } => lvalue_needs_preserved_temp(target),
        LoweredExpr::Call { args, .. } => args.iter().any(expr_needs_preserved_temp),
        LoweredExpr::IndirectCall { callee, args, .. } => {
            expr_needs_preserved_temp(callee) || args.iter().any(expr_needs_preserved_temp)
        }
        LoweredExpr::Integer(_)
        | LoweredExpr::LongInteger(_)
        | LoweredExpr::DoubleLiteral(_)
        | LoweredExpr::StringLiteral(_)
        | LoweredExpr::Global { .. }
        | LoweredExpr::GlobalAddress { .. }
        | LoweredExpr::Local { .. }
        | LoweredExpr::LocalAddress { .. } => false,
    }
}

pub(in crate::codegen) fn lvalue_needs_preserved_temp(target: &LoweredLValue) -> bool {
    match target {
        LoweredLValue::Local { .. } | LoweredLValue::Global { .. } => false,
        LoweredLValue::GlobalByteSubscript { index, .. }
        | LoweredLValue::GlobalIntSubscript { index, .. }
        | LoweredLValue::GlobalPointerSubscript { index, .. } => expr_needs_preserved_temp(index),
        LoweredLValue::PointerSubscript { pointer, index, .. } => {
            expr_needs_preserved_temp(pointer) || expr_needs_preserved_temp(index)
        }
        LoweredLValue::PointerField { pointer, .. } => expr_needs_preserved_temp(pointer),
    }
}

pub(in crate::codegen) const fn expr_is_direct_call(expr: &LoweredExpr) -> bool {
    matches!(
        expr,
        LoweredExpr::Call { .. } | LoweredExpr::IndirectCall { .. }
    )
}

pub(in crate::codegen) fn instruction_uses_call(instruction: &Instruction) -> bool {
    match instruction {
        Instruction::StoreLocal { value, .. }
        | Instruction::StoreGlobal { value, .. }
        | Instruction::JumpIfZero {
            condition: value, ..
        }
        | Instruction::Eval(value)
        | Instruction::Return(Some(value)) => expr_uses_call(value),
        Instruction::StoreComplexReturn { pointer, .. } => expr_uses_call(pointer),
        Instruction::Return(None)
        | Instruction::Jump { .. }
        | Instruction::Label { .. }
        | Instruction::InitLocalBytes { .. }
        | Instruction::InitLocalInts { .. } => false,
    }
}

pub(in crate::codegen) fn expr_uses_call(expr: &LoweredExpr) -> bool {
    match expr {
        LoweredExpr::Call { .. } | LoweredExpr::IndirectCall { .. } => true,
        LoweredExpr::Integer(_)
        | LoweredExpr::LongInteger(_)
        | LoweredExpr::DoubleLiteral(_)
        | LoweredExpr::StringLiteral(_)
        | LoweredExpr::Global { .. }
        | LoweredExpr::GlobalAddress { .. }
        | LoweredExpr::Local { .. }
        | LoweredExpr::LocalAddress { .. } => false,
        LoweredExpr::Cast { expr, .. } | LoweredExpr::Unary { expr, .. } => expr_uses_call(expr),
        LoweredExpr::VaArg { list, .. } => expr_uses_call(list),
        LoweredExpr::GlobalByteSubscript { index, .. }
        | LoweredExpr::GlobalIntSubscript { index, .. }
        | LoweredExpr::GlobalPointerSubscript { index, .. } => expr_uses_call(index),
        LoweredExpr::PointerSubscript { pointer, index, .. }
        | LoweredExpr::PointerOffset { pointer, index, .. } => {
            expr_uses_call(pointer) || expr_uses_call(index)
        }
        LoweredExpr::PointerFieldAddress { pointer, .. }
        | LoweredExpr::PointerField { pointer, .. } => expr_uses_call(pointer),
        LoweredExpr::Assign { target, value } => lvalue_uses_call(target) || expr_uses_call(value),
        LoweredExpr::PostIncrement { target, .. } => lvalue_uses_call(target),
        LoweredExpr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => expr_uses_call(condition) || expr_uses_call(then_expr) || expr_uses_call(else_expr),
        LoweredExpr::IndexSelect {
            index,
            cases,
            default,
        } => expr_uses_call(index) || cases.iter().any(expr_uses_call) || expr_uses_call(default),
        LoweredExpr::Comma { left, right } | LoweredExpr::Binary { left, right, .. } => {
            expr_uses_call(left) || expr_uses_call(right)
        }
    }
}

pub(in crate::codegen) fn lvalue_uses_call(target: &LoweredLValue) -> bool {
    match target {
        LoweredLValue::Local { .. } | LoweredLValue::Global { .. } => false,
        LoweredLValue::GlobalByteSubscript { index, .. }
        | LoweredLValue::GlobalIntSubscript { index, .. }
        | LoweredLValue::GlobalPointerSubscript { index, .. } => expr_uses_call(index),
        LoweredLValue::PointerSubscript { pointer, index, .. } => {
            expr_uses_call(pointer) || expr_uses_call(index)
        }
        LoweredLValue::PointerField { pointer, .. } => expr_uses_call(pointer),
    }
}
