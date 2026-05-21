use super::{
    ComplexBinaryLanes, LoweredExpr, complex_arithmetic_lane_expr, complex_lane_byte_size,
    complex_lane_expr, complex_object_pointer, is_complex_scalar, lowered_expr_scalar_type,
    real_scalar_expr_type, real_scalar_lane_expr, scalar_size,
};
use crate::parser::{BinaryOp, ScalarType, UnaryOp};

pub(in crate::ir) fn complex_truth_expr(value: &LoweredExpr) -> Option<LoweredExpr> {
    let scalar_type = complex_expr_scalar_type(value)?;
    let element_byte_size = complex_lane_byte_size(scalar_type);
    let imaginary_index = complex_imaginary_lane_index(scalar_type, element_byte_size)?;
    Some(binary(
        BinaryOp::LogicalOr,
        lane_not_zero(value, scalar_type, 0, element_byte_size)?,
        lane_not_zero(value, scalar_type, imaginary_index, element_byte_size)?,
    ))
}

pub(in crate::ir) fn complex_equality_expr(
    op: BinaryOp,
    left: &LoweredExpr,
    right: &LoweredExpr,
) -> Option<LoweredExpr> {
    if !matches!(op, BinaryOp::Equal | BinaryOp::NotEqual) {
        return None;
    }
    let scalar_type = complex_binary_result_type(left, right)?;
    let lane_op = if op == BinaryOp::Equal {
        BinaryOp::Equal
    } else {
        BinaryOp::NotEqual
    };
    let join_op = if op == BinaryOp::Equal {
        BinaryOp::LogicalAnd
    } else {
        BinaryOp::LogicalOr
    };
    let element_byte_size = complex_lane_byte_size(scalar_type);
    let lane_count = scalar_size(scalar_type) / element_byte_size;
    let mut expr =
        complex_lane_comparison(left, right, scalar_type, lane_op, 0, element_byte_size)?;
    for index in 1..lane_count {
        expr = binary(
            join_op,
            expr,
            complex_lane_comparison(
                left,
                right,
                scalar_type,
                lane_op,
                i64::try_from(index).ok()?,
                element_byte_size,
            )?,
        );
    }
    Some(expr)
}

pub(in crate::ir) fn complex_expr_scalar_type(value: &LoweredExpr) -> Option<ScalarType> {
    let scalar_type = lowered_expr_scalar_type(value);
    if scalar_type.is_some_and(is_complex_scalar) {
        return scalar_type;
    }
    match value {
        LoweredExpr::Unary {
            op: UnaryOp::Plus | UnaryOp::Minus,
            expr,
        } => complex_expr_scalar_type(expr),
        LoweredExpr::Cast { target, .. } if is_complex_scalar(*target) => Some(*target),
        LoweredExpr::Binary { op, left, right } if is_complex_arithmetic_op(*op) => {
            complex_binary_result_type(left, right)
        }
        _ => None,
    }
}

pub(in crate::ir) fn complex_lane_value_expr(
    value: &LoweredExpr,
    scalar_type: ScalarType,
    index: i64,
    element_byte_size: usize,
) -> Option<LoweredExpr> {
    if let Some(pointer) = complex_object_pointer(value, scalar_type) {
        return Some(complex_lane_expr(&pointer, index, element_byte_size));
    }
    match value {
        LoweredExpr::Unary { op, expr } if matches!(op, UnaryOp::Plus | UnaryOp::Minus) => {
            complex_unary_lane_expr(*op, expr, scalar_type, index, element_byte_size)
        }
        LoweredExpr::Cast { target, expr } if *target == scalar_type => {
            Some(complex_cast_lane_expr(expr, index))
        }
        LoweredExpr::Binary { op, left, right } if is_complex_arithmetic_op(*op) => {
            complex_binary_lane_expr(*op, left, right, scalar_type, index, element_byte_size)
        }
        _ if real_scalar_expr_type(value).is_some() => Some(real_scalar_lane_expr(value, index)),
        _ => None,
    }
}

fn complex_binary_lane_expr(
    op: BinaryOp,
    left: &LoweredExpr,
    right: &LoweredExpr,
    scalar_type: ScalarType,
    index: i64,
    element_byte_size: usize,
) -> Option<LoweredExpr> {
    if complex_binary_result_type(left, right)? != scalar_type {
        return None;
    }
    match op {
        BinaryOp::Add | BinaryOp::Sub => Some(binary(
            op,
            complex_lane_value_expr(left, scalar_type, index, element_byte_size)?,
            complex_lane_value_expr(right, scalar_type, index, element_byte_size)?,
        )),
        BinaryOp::Mul | BinaryOp::Div => complex_arithmetic_lane_expr(
            op,
            complex_binary_components(left, right, scalar_type, element_byte_size)?,
            index,
            complex_imaginary_lane_index(scalar_type, element_byte_size)?,
        ),
        _ => None,
    }
}

fn complex_binary_components(
    left: &LoweredExpr,
    right: &LoweredExpr,
    scalar_type: ScalarType,
    element_byte_size: usize,
) -> Option<ComplexBinaryLanes> {
    let imaginary_index = complex_imaginary_lane_index(scalar_type, element_byte_size)?;
    Some(ComplexBinaryLanes {
        a: complex_lane_value_expr(left, scalar_type, 0, element_byte_size)?,
        b: complex_lane_value_expr(left, scalar_type, imaginary_index, element_byte_size)?,
        c: complex_lane_value_expr(right, scalar_type, 0, element_byte_size)?,
        d: complex_lane_value_expr(right, scalar_type, imaginary_index, element_byte_size)?,
    })
}

fn complex_unary_lane_expr(
    op: UnaryOp,
    expr: &LoweredExpr,
    scalar_type: ScalarType,
    index: i64,
    element_byte_size: usize,
) -> Option<LoweredExpr> {
    let lane = complex_lane_value_expr(expr, scalar_type, index, element_byte_size)?;
    if op == UnaryOp::Plus {
        Some(lane)
    } else {
        Some(LoweredExpr::Unary {
            op,
            expr: Box::new(lane),
        })
    }
}

fn complex_cast_lane_expr(expr: &LoweredExpr, index: i64) -> LoweredExpr {
    if index == 0 {
        expr.clone()
    } else {
        zero_lane_expr()
    }
}

fn complex_lane_comparison(
    left: &LoweredExpr,
    right: &LoweredExpr,
    scalar_type: ScalarType,
    op: BinaryOp,
    index: i64,
    element_byte_size: usize,
) -> Option<LoweredExpr> {
    Some(binary(
        op,
        complex_lane_value_expr(left, scalar_type, index, element_byte_size)?,
        complex_lane_value_expr(right, scalar_type, index, element_byte_size)?,
    ))
}

fn lane_not_zero(
    value: &LoweredExpr,
    scalar_type: ScalarType,
    index: i64,
    element_byte_size: usize,
) -> Option<LoweredExpr> {
    Some(binary(
        BinaryOp::NotEqual,
        complex_lane_value_expr(value, scalar_type, index, element_byte_size)?,
        zero_lane_expr(),
    ))
}

fn complex_binary_result_type(left: &LoweredExpr, right: &LoweredExpr) -> Option<ScalarType> {
    match (
        complex_expr_scalar_type(left),
        complex_expr_scalar_type(right),
        real_scalar_expr_type(left),
        real_scalar_expr_type(right),
    ) {
        (Some(left_type), Some(right_type), _, _) if left_type == right_type => Some(left_type),
        (Some(left_type), None, _, Some(_)) => Some(left_type),
        (None, Some(right_type), Some(_), _) => Some(right_type),
        _ => None,
    }
}

fn complex_imaginary_lane_index(scalar_type: ScalarType, element_byte_size: usize) -> Option<i64> {
    i64::try_from((scalar_size(scalar_type) / element_byte_size) / 2).ok()
}

const fn is_complex_arithmetic_op(op: BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div
    )
}

fn binary(op: BinaryOp, left: LoweredExpr, right: LoweredExpr) -> LoweredExpr {
    LoweredExpr::Binary {
        op,
        left: Box::new(left),
        right: Box::new(right),
    }
}

fn zero_lane_expr() -> LoweredExpr {
    LoweredExpr::DoubleLiteral("0.0".to_owned())
}
