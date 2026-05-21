use super::{LoweredExpr, is_complex_scalar, lowered_expr_scalar_type};
use crate::parser::{BinaryOp, ScalarType, UnaryOp};

pub(in crate::ir) fn real_scalar_expr_type(value: &LoweredExpr) -> Option<ScalarType> {
    if let Some(scalar_type) = lowered_expr_scalar_type(value) {
        return real_arithmetic_scalar_type(scalar_type);
    }
    match value {
        LoweredExpr::Integer(_) => Some(ScalarType::Int),
        LoweredExpr::LongInteger(_) => Some(ScalarType::LongLong),
        LoweredExpr::DoubleLiteral(_) => Some(ScalarType::Double),
        LoweredExpr::Unary { op, expr } => real_unary_scalar_type(*op, expr),
        LoweredExpr::Binary { op, left, right } => real_binary_scalar_type(*op, left, right),
        _ => None,
    }
}

pub(in crate::ir) fn real_scalar_lane_expr(value: &LoweredExpr, index: i64) -> LoweredExpr {
    if index == 0 {
        value.clone()
    } else {
        LoweredExpr::DoubleLiteral("0.0".to_owned())
    }
}

fn real_unary_scalar_type(op: UnaryOp, expr: &LoweredExpr) -> Option<ScalarType> {
    if op == UnaryOp::LogicalNot {
        return Some(ScalarType::Int);
    }
    real_scalar_expr_type(expr)
}

fn real_binary_scalar_type(
    op: BinaryOp,
    left: &LoweredExpr,
    right: &LoweredExpr,
) -> Option<ScalarType> {
    let left_type = real_scalar_expr_type(left)?;
    let right_type = real_scalar_expr_type(right)?;
    if binary_op_returns_int(op) {
        Some(ScalarType::Int)
    } else {
        Some(promoted_real_scalar_type(left_type, right_type))
    }
}

const fn real_arithmetic_scalar_type(scalar_type: ScalarType) -> Option<ScalarType> {
    if is_complex_scalar(scalar_type) {
        return None;
    }
    match scalar_type {
        ScalarType::Bool
        | ScalarType::Int
        | ScalarType::LongLong
        | ScalarType::Double
        | ScalarType::LongDouble => Some(scalar_type),
        ScalarType::ComplexFloat
        | ScalarType::ComplexDouble
        | ScalarType::ComplexLongDouble
        | ScalarType::Pointer
        | ScalarType::VaList => None,
    }
}

const fn binary_op_returns_int(op: BinaryOp) -> bool {
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

const fn promoted_real_scalar_type(left: ScalarType, right: ScalarType) -> ScalarType {
    if matches!(left, ScalarType::LongDouble) || matches!(right, ScalarType::LongDouble) {
        ScalarType::LongDouble
    } else if matches!(left, ScalarType::Double) || matches!(right, ScalarType::Double) {
        ScalarType::Double
    } else if matches!(left, ScalarType::LongLong) || matches!(right, ScalarType::LongLong) {
        ScalarType::LongLong
    } else {
        ScalarType::Int
    }
}
