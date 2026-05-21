use super::pointer_cast;
use crate::ir::{LoweredExpr, LoweredLValue};
use crate::parser::{BinaryOp, ScalarType, UnaryOp};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::codegen) enum ValueWidth {
    I32,
    I64,
    F64,
}

pub(in crate::codegen) const TEMPORARY_BYTES: usize = 8;
pub(in crate::codegen) const X86_64_VARIADIC_GP_SAVE_BYTES: usize = 48;
pub(in crate::codegen) const X86_64_VARIADIC_FP_OFFSET: usize = 48;
pub(in crate::codegen) const X86_64_VARIADIC_GP_REGISTERS: [&str; 6] =
    ["%rdi", "%rsi", "%rdx", "%rcx", "%r8", "%r9"];

#[derive(Clone, Copy)]
pub(in crate::codegen) struct BinaryExpr<'a> {
    pub(in crate::codegen) op: BinaryOp,
    pub(in crate::codegen) left: &'a LoweredExpr,
    pub(in crate::codegen) right: &'a LoweredExpr,
}

#[derive(Clone, Copy)]
pub(in crate::codegen) struct ConditionalExpr<'a> {
    pub(in crate::codegen) condition: &'a LoweredExpr,
    pub(in crate::codegen) then_expr: &'a LoweredExpr,
    pub(in crate::codegen) else_expr: &'a LoweredExpr,
}

#[derive(Clone, Copy)]
pub(in crate::codegen) struct PointerSubscriptExpr<'a> {
    pub(in crate::codegen) pointer: &'a LoweredExpr,
    pub(in crate::codegen) index: &'a LoweredExpr,
    pub(in crate::codegen) element_type: ScalarType,
    pub(in crate::codegen) element_byte_size: usize,
    pub(in crate::codegen) element_unsigned: bool,
}

#[derive(Clone, Copy)]
pub(in crate::codegen) struct PointerOffsetExpr<'a> {
    pub(in crate::codegen) pointer: &'a LoweredExpr,
    pub(in crate::codegen) index: &'a LoweredExpr,
    pub(in crate::codegen) byte_size: usize,
}

#[derive(Clone, Copy)]
pub(in crate::codegen) struct GlobalByteSubscriptExpr<'a> {
    pub(in crate::codegen) name: &'a str,
    pub(in crate::codegen) index: &'a LoweredExpr,
    pub(in crate::codegen) is_unsigned: bool,
}

#[derive(Clone, Copy)]
pub(in crate::codegen) struct PointerFieldExpr<'a> {
    pub(in crate::codegen) pointer: &'a LoweredExpr,
    pub(in crate::codegen) offset: usize,
    pub(in crate::codegen) scalar_type: ScalarType,
    pub(in crate::codegen) byte_size: usize,
    pub(in crate::codegen) is_unsigned: bool,
}

pub(in crate::codegen) const fn scalar_width(scalar_type: ScalarType) -> ValueWidth {
    match scalar_type {
        ScalarType::Bool | ScalarType::Int => ValueWidth::I32,
        ScalarType::LongLong
        | ScalarType::ComplexFloat
        | ScalarType::ComplexDouble
        | ScalarType::ComplexLongDouble
        | ScalarType::Pointer
        | ScalarType::VaList => ValueWidth::I64,
        ScalarType::Double | ScalarType::LongDouble => ValueWidth::F64,
    }
}

pub(in crate::codegen) fn expr_width(expr: &LoweredExpr) -> ValueWidth {
    match expr {
        LoweredExpr::Cast { target, expr } => cast_width(*target, expr),
        LoweredExpr::DoubleLiteral(_) => ValueWidth::F64,
        LoweredExpr::StringLiteral(_)
        | LoweredExpr::LongInteger(_)
        | LoweredExpr::LocalAddress { .. }
        | LoweredExpr::GlobalAddress { .. }
        | LoweredExpr::GlobalPointerSubscript { .. }
        | LoweredExpr::PointerOffset { .. }
        | LoweredExpr::PointerFieldAddress { .. } => ValueWidth::I64,
        LoweredExpr::Global { scalar_type, .. }
        | LoweredExpr::Local { scalar_type, .. }
        | LoweredExpr::VaArg { scalar_type, .. }
        | LoweredExpr::Call {
            return_type: scalar_type,
            ..
        }
        | LoweredExpr::PointerField { scalar_type, .. } => scalar_width(*scalar_type),
        LoweredExpr::GlobalByteSubscript { .. }
        | LoweredExpr::GlobalIntSubscript { .. }
        | LoweredExpr::PointerSubscript {
            element_type: ScalarType::Int,
            ..
        }
        | LoweredExpr::IndirectCall { .. }
        | LoweredExpr::Integer(_) => ValueWidth::I32,
        LoweredExpr::PointerSubscript { element_type, .. } => scalar_width(*element_type),
        LoweredExpr::Assign { target, .. } | LoweredExpr::PostIncrement { target, .. } => {
            lowered_lvalue_width(target)
        }
        LoweredExpr::Unary { op, expr } => match op {
            UnaryOp::LogicalNot => ValueWidth::I32,
            UnaryOp::Plus | UnaryOp::Minus | UnaryOp::BitNot => expr_width(expr),
        },
        LoweredExpr::Conditional {
            then_expr,
            else_expr,
            ..
        } => expr_width(then_expr).max(expr_width(else_expr)),
        LoweredExpr::Comma { right, .. } => expr_width(right),
        LoweredExpr::Binary { op, left, right } => binary_result_width(*op, left, right),
    }
}

pub(in crate::codegen) fn cast_width(target: ScalarType, expr: &LoweredExpr) -> ValueWidth {
    pointer_cast::width(target, expr).unwrap_or_else(|| scalar_width(target))
}

pub(in crate::codegen) const fn lowered_lvalue_width(target: &LoweredLValue) -> ValueWidth {
    match target {
        LoweredLValue::Local { scalar_type, .. } | LoweredLValue::Global { scalar_type, .. } => {
            scalar_width(*scalar_type)
        }
        LoweredLValue::GlobalByteSubscript { .. } | LoweredLValue::GlobalIntSubscript { .. } => {
            ValueWidth::I32
        }
        LoweredLValue::GlobalPointerSubscript { .. } => ValueWidth::I64,
        LoweredLValue::PointerSubscript { element_type, .. } => scalar_width(*element_type),
        LoweredLValue::PointerField { scalar_type, .. } => scalar_width(*scalar_type),
    }
}

pub(in crate::codegen) fn binary_result_width(
    op: BinaryOp,
    left: &LoweredExpr,
    right: &LoweredExpr,
) -> ValueWidth {
    if binary_returns_i32(op) {
        ValueWidth::I32
    } else {
        binary_operand_width(op, left, right)
    }
}

pub(in crate::codegen) fn binary_operand_width(
    op: BinaryOp,
    left: &LoweredExpr,
    right: &LoweredExpr,
) -> ValueWidth {
    match op {
        BinaryOp::LogicalAnd | BinaryOp::LogicalOr => ValueWidth::I32,
        BinaryOp::ShiftLeft | BinaryOp::ShiftRight => expr_width(left),
        BinaryOp::Mul
        | BinaryOp::Div
        | BinaryOp::Mod
        | BinaryOp::Add
        | BinaryOp::Sub
        | BinaryOp::Less
        | BinaryOp::LessEqual
        | BinaryOp::Greater
        | BinaryOp::GreaterEqual
        | BinaryOp::Equal
        | BinaryOp::NotEqual
        | BinaryOp::BitAnd
        | BinaryOp::BitXor
        | BinaryOp::BitOr => expr_width(left).max(expr_width(right)),
    }
}

pub(in crate::codegen) const fn binary_returns_i32(op: BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::Less
            | BinaryOp::LessEqual
            | BinaryOp::Greater
            | BinaryOp::GreaterEqual
            | BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::LogicalAnd
            | BinaryOp::LogicalOr
    )
}
pub(in crate::codegen) const fn width_bytes(width: ValueWidth) -> usize {
    match width {
        ValueWidth::I32 => 4,
        ValueWidth::I64 | ValueWidth::F64 => 8,
    }
}
