use super::{
    LoweredExpr, LoweredLValue, lowered_expr_scalar_type, pointer_field_address, scalar_size,
};
use crate::parser::{BinaryOp, ScalarType, UnaryOp};

pub(in crate::ir) fn complex_indirect_target(
    target: &LoweredLValue,
) -> Option<(LoweredExpr, ScalarType)> {
    match target {
        LoweredLValue::PointerSubscript {
            pointer,
            index,
            element_type,
            element_byte_size,
            ..
        } if is_complex_scalar(*element_type) => Some((
            LoweredExpr::PointerOffset {
                pointer: pointer.clone(),
                index: index.clone(),
                byte_size: *element_byte_size,
            },
            *element_type,
        )),
        LoweredLValue::PointerField {
            pointer,
            offset,
            scalar_type,
            ..
        } if is_complex_scalar(*scalar_type) => Some((
            pointer_field_address((**pointer).clone(), *offset),
            *scalar_type,
        )),
        _ => None,
    }
}

pub(in crate::ir) fn complex_object_pointer(
    value: &LoweredExpr,
    scalar_type: ScalarType,
) -> Option<LoweredExpr> {
    match value {
        LoweredExpr::Local {
            offset,
            scalar_type: source_type,
        } if *source_type == scalar_type => Some(LoweredExpr::LocalAddress {
            offset: *offset,
            byte_size: scalar_size(*source_type),
        }),
        LoweredExpr::Global {
            name,
            scalar_type: source_type,
        } if *source_type == scalar_type => Some(LoweredExpr::GlobalAddress { name: name.clone() }),
        LoweredExpr::PointerSubscript {
            pointer,
            index,
            element_type,
            element_byte_size,
            ..
        } if *element_type == scalar_type => Some(LoweredExpr::PointerOffset {
            pointer: pointer.clone(),
            index: index.clone(),
            byte_size: *element_byte_size,
        }),
        LoweredExpr::PointerField {
            pointer,
            offset,
            scalar_type: source_type,
            ..
        } if *source_type == scalar_type => {
            Some(pointer_field_address((**pointer).clone(), *offset))
        }
        _ => None,
    }
}

pub(in crate::ir) fn complex_lane_expr(
    source_pointer: &LoweredExpr,
    index: i64,
    element_byte_size: usize,
) -> LoweredExpr {
    LoweredExpr::PointerSubscript {
        pointer: Box::new(source_pointer.clone()),
        index: Box::new(LoweredExpr::Integer(index)),
        element_type: ScalarType::Double,
        element_byte_size,
        element_unsigned: false,
    }
}

pub(in crate::ir) fn complex_binary_operands(
    value: &LoweredExpr,
    scalar_type: ScalarType,
) -> Option<(BinaryOp, LoweredExpr, LoweredExpr)> {
    let LoweredExpr::Binary { op, left, right } = value else {
        return None;
    };
    if !matches!(
        op,
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div
    ) {
        return None;
    }
    Some((
        *op,
        complex_object_pointer(left, scalar_type)?,
        complex_object_pointer(right, scalar_type)?,
    ))
}

pub(in crate::ir) fn complex_unary_operand(
    value: &LoweredExpr,
    scalar_type: ScalarType,
) -> Option<(UnaryOp, LoweredExpr)> {
    let LoweredExpr::Unary { op, expr } = value else {
        return None;
    };
    if !matches!(op, UnaryOp::Plus | UnaryOp::Minus) {
        return None;
    }
    Some((*op, complex_object_pointer(expr, scalar_type)?))
}

pub(in crate::ir) fn complex_truth_expr(
    value: &LoweredExpr,
    scalar_type: ScalarType,
) -> Option<LoweredExpr> {
    let pointer = complex_object_pointer(value, scalar_type)?;
    let element_byte_size = complex_lane_byte_size(scalar_type);
    let lane_count = scalar_size(scalar_type) / element_byte_size;
    let imaginary_index = i64::try_from(lane_count / 2).ok()?;
    Some(LoweredExpr::Binary {
        op: BinaryOp::LogicalOr,
        left: Box::new(lane_not_zero(&pointer, 0, element_byte_size)),
        right: Box::new(lane_not_zero(&pointer, imaginary_index, element_byte_size)),
    })
}

pub(in crate::ir) fn complex_equality_expr(
    op: BinaryOp,
    left: &LoweredExpr,
    right: &LoweredExpr,
) -> Option<LoweredExpr> {
    if !matches!(op, BinaryOp::Equal | BinaryOp::NotEqual) {
        return None;
    }
    let scalar_type = lowered_expr_scalar_type(left)?;
    if scalar_type != lowered_expr_scalar_type(right)? || !is_complex_scalar(scalar_type) {
        return None;
    }
    let left_pointer = complex_object_pointer(left, scalar_type)?;
    let right_pointer = complex_object_pointer(right, scalar_type)?;
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
        complex_lane_comparison(&left_pointer, &right_pointer, lane_op, 0, element_byte_size)?;
    for index in 1..lane_count {
        expr = LoweredExpr::Binary {
            op: join_op,
            left: Box::new(expr),
            right: Box::new(complex_lane_comparison(
                &left_pointer,
                &right_pointer,
                lane_op,
                i64::try_from(index).ok()?,
                element_byte_size,
            )?),
        };
    }
    Some(expr)
}

fn complex_lane_comparison(
    left_pointer: &LoweredExpr,
    right_pointer: &LoweredExpr,
    op: BinaryOp,
    index: i64,
    element_byte_size: usize,
) -> Option<LoweredExpr> {
    match op {
        BinaryOp::Equal | BinaryOp::NotEqual => {}
        _ => return None,
    }
    Some(LoweredExpr::Binary {
        op,
        left: Box::new(complex_lane_expr(left_pointer, index, element_byte_size)),
        right: Box::new(complex_lane_expr(right_pointer, index, element_byte_size)),
    })
}

fn lane_not_zero(pointer: &LoweredExpr, index: i64, element_byte_size: usize) -> LoweredExpr {
    LoweredExpr::Binary {
        op: BinaryOp::NotEqual,
        left: Box::new(complex_lane_expr(pointer, index, element_byte_size)),
        right: Box::new(LoweredExpr::DoubleLiteral("0.0".to_owned())),
    }
}

pub(in crate::ir) const fn complex_lane_byte_size(scalar_type: ScalarType) -> usize {
    match scalar_type {
        ScalarType::ComplexFloat => 4,
        _ => scalar_size(ScalarType::Double),
    }
}

pub(in crate::ir) const fn is_complex_scalar(scalar_type: ScalarType) -> bool {
    matches!(
        scalar_type,
        ScalarType::ComplexFloat | ScalarType::ComplexDouble | ScalarType::ComplexLongDouble
    )
}
