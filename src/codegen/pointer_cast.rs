use super::widths::ValueWidth;
use crate::ir::{LoweredExpr, LoweredLValue};
use crate::parser::ScalarType;

pub(in crate::codegen) fn width(target: ScalarType, expr: &LoweredExpr) -> Option<ValueWidth> {
    (target == ScalarType::Int && expr_is_pointer(expr)).then_some(ValueWidth::I64)
}

fn expr_is_pointer(expr: &LoweredExpr) -> bool {
    match expr {
        LoweredExpr::Call { return_type, .. }
        | LoweredExpr::Global {
            scalar_type: return_type,
            ..
        }
        | LoweredExpr::Local {
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
        } => *return_type == ScalarType::Pointer,
        LoweredExpr::Assign { target, .. } | LoweredExpr::PostIncrement { target, .. } => {
            lvalue_is_pointer(target)
        }
        LoweredExpr::StringLiteral(_)
        | LoweredExpr::GlobalPointerSubscript { .. }
        | LoweredExpr::GlobalAddress { .. }
        | LoweredExpr::PointerOffset { .. }
        | LoweredExpr::PointerFieldAddress { .. }
        | LoweredExpr::LocalAddress { .. } => true,
        LoweredExpr::Cast { target, .. } => *target == ScalarType::Pointer,
        LoweredExpr::Conditional {
            then_expr,
            else_expr,
            ..
        } => expr_is_pointer(then_expr) && expr_is_pointer(else_expr),
        LoweredExpr::Comma { right, .. } => expr_is_pointer(right),
        LoweredExpr::IndirectCall { .. }
        | LoweredExpr::Integer(_)
        | LoweredExpr::DoubleLiteral(_)
        | LoweredExpr::GlobalByteSubscript { .. }
        | LoweredExpr::GlobalIntSubscript { .. }
        | LoweredExpr::Unary { .. }
        | LoweredExpr::Binary { .. } => false,
    }
}

fn lvalue_is_pointer(target: &LoweredLValue) -> bool {
    match target {
        LoweredLValue::Local { scalar_type, .. }
        | LoweredLValue::Global { scalar_type, .. }
        | LoweredLValue::PointerSubscript {
            element_type: scalar_type,
            ..
        }
        | LoweredLValue::PointerField { scalar_type, .. } => *scalar_type == ScalarType::Pointer,
        LoweredLValue::GlobalPointerSubscript { .. } => true,
        LoweredLValue::GlobalByteSubscript { .. } | LoweredLValue::GlobalIntSubscript { .. } => {
            false
        }
    }
}
