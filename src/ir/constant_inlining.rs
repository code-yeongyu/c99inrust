use super::{Instruction, LoweredExpr, LoweredFunction};
use crate::parser::ReturnType;
use std::collections::HashMap;

pub(in crate::ir) fn constant_return_functions(
    functions: &[LoweredFunction],
) -> HashMap<String, i64> {
    let mut constants = HashMap::new();
    for function in functions {
        if function.local_slots.is_empty()
            && function.return_type == ReturnType::Int
            && let [Instruction::Return(Some(LoweredExpr::Integer(value)))] =
                function.instructions.as_slice()
        {
            constants.insert(function.name.clone(), *value);
        }
    }
    constants
}

pub(in crate::ir) fn inline_constant_calls(
    functions: &mut [LoweredFunction],
    constants: &HashMap<String, i64>,
) {
    for function in functions {
        for instruction in &mut function.instructions {
            inline_constant_calls_in_instruction(instruction, constants);
        }
    }
}

pub(in crate::ir) fn inline_constant_calls_in_instruction(
    instruction: &mut Instruction,
    constants: &HashMap<String, i64>,
) {
    match instruction {
        Instruction::StoreLocal { value, .. }
        | Instruction::StoreGlobal { value, .. }
        | Instruction::JumpIfZero {
            condition: value, ..
        }
        | Instruction::Eval(value)
        | Instruction::Return(Some(value)) => inline_constant_calls_in_expr(value, constants),
        Instruction::Return(None)
        | Instruction::Jump { .. }
        | Instruction::Label { .. }
        | Instruction::InitLocalBytes { .. }
        | Instruction::InitLocalInts { .. } => {}
    }
}

pub(in crate::ir) fn inline_constant_calls_in_expr(
    expr: &mut LoweredExpr,
    constants: &HashMap<String, i64>,
) {
    match expr {
        LoweredExpr::Call { callee, args, .. } => {
            if args.is_empty()
                && let Some(value) = constants.get(callee)
            {
                *expr = LoweredExpr::Integer(*value);
            } else {
                for arg in args {
                    inline_constant_calls_in_expr(arg, constants);
                }
            }
        }
        LoweredExpr::IndirectCall { callee, args, .. } => {
            inline_constant_calls_in_expr(callee, constants);
            for arg in args {
                inline_constant_calls_in_expr(arg, constants);
            }
        }
        LoweredExpr::Unary { expr, .. } | LoweredExpr::Cast { expr, .. } => {
            inline_constant_calls_in_expr(expr, constants);
        }
        LoweredExpr::VaArg { list, .. } => {
            inline_constant_calls_in_expr(list, constants);
        }
        LoweredExpr::GlobalByteSubscript { index, .. }
        | LoweredExpr::GlobalIntSubscript { index, .. }
        | LoweredExpr::GlobalPointerSubscript { index, .. } => {
            inline_constant_calls_in_expr(index, constants);
        }
        LoweredExpr::PointerSubscript { pointer, index, .. }
        | LoweredExpr::PointerOffset { pointer, index, .. } => {
            inline_constant_calls_in_expr(pointer, constants);
            inline_constant_calls_in_expr(index, constants);
        }
        LoweredExpr::PointerField { pointer, .. }
        | LoweredExpr::PointerFieldAddress { pointer, .. } => {
            inline_constant_calls_in_expr(pointer, constants);
        }
        LoweredExpr::Assign { value, .. } => {
            inline_constant_calls_in_expr(value, constants);
        }
        LoweredExpr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => {
            inline_constant_calls_in_expr(condition, constants);
            inline_constant_calls_in_expr(then_expr, constants);
            inline_constant_calls_in_expr(else_expr, constants);
        }
        LoweredExpr::Comma { left, right } | LoweredExpr::Binary { left, right, .. } => {
            inline_constant_calls_in_expr(left, constants);
            inline_constant_calls_in_expr(right, constants);
        }
        LoweredExpr::PostIncrement { .. }
        | LoweredExpr::Integer(_)
        | LoweredExpr::LongInteger(_)
        | LoweredExpr::DoubleLiteral(_)
        | LoweredExpr::StringLiteral(_)
        | LoweredExpr::Global { .. }
        | LoweredExpr::GlobalAddress { .. }
        | LoweredExpr::Local { .. }
        | LoweredExpr::LocalAddress { .. } => {}
    }
}
