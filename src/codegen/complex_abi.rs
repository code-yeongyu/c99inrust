use crate::ir::LoweredExpr;
use crate::parser::{BinaryOp, ReturnType, ScalarType, UnaryOp};

pub(in crate::codegen) const fn is_complex_scalar(scalar_type: ScalarType) -> bool {
    matches!(
        scalar_type,
        ScalarType::ComplexFloat | ScalarType::ComplexDouble | ScalarType::ComplexLongDouble
    )
}

pub(in crate::codegen) fn expr_complex_scalar_type(expr: &LoweredExpr) -> Option<ScalarType> {
    match expr {
        LoweredExpr::Call { return_type, .. }
        | LoweredExpr::IndirectCall { return_type, .. }
        | LoweredExpr::Local {
            scalar_type: return_type,
            ..
        }
        | LoweredExpr::Global {
            scalar_type: return_type,
            ..
        }
        | LoweredExpr::PointerSubscript {
            element_type: return_type,
            ..
        }
        | LoweredExpr::PointerField {
            scalar_type: return_type,
            ..
        } if is_complex_scalar(*return_type) => Some(*return_type),
        LoweredExpr::Unary {
            op: UnaryOp::Plus | UnaryOp::Minus,
            expr,
        } => expr_complex_scalar_type(expr),
        LoweredExpr::Cast { target, .. } if is_complex_scalar(*target) => Some(*target),
        LoweredExpr::Binary { op, left, right } if is_complex_arithmetic_op(*op) => {
            complex_binary_result_type(left, right)
        }
        _ => None,
    }
}

fn complex_binary_result_type(left: &LoweredExpr, right: &LoweredExpr) -> Option<ScalarType> {
    match (
        expr_complex_scalar_type(left),
        expr_complex_scalar_type(right),
    ) {
        (Some(left_type), Some(right_type)) if left_type == right_type => Some(left_type),
        (Some(left_type), None) => Some(left_type),
        (None, Some(right_type)) => Some(right_type),
        _ => None,
    }
}

const fn is_complex_arithmetic_op(op: BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div
    )
}

pub(in crate::codegen) const fn return_complex_scalar_type(
    return_type: ReturnType,
) -> Option<ScalarType> {
    match return_type {
        ReturnType::ComplexFloat => Some(ScalarType::ComplexFloat),
        ReturnType::ComplexDouble => Some(ScalarType::ComplexDouble),
        ReturnType::ComplexLongDouble => Some(ScalarType::ComplexLongDouble),
        ReturnType::Int
        | ReturnType::Pointer
        | ReturnType::Double
        | ReturnType::LongDouble
        | ReturnType::Void => None,
    }
}
