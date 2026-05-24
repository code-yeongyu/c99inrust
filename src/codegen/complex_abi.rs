use crate::ir::LoweredExpr;
use crate::parser::{ReturnType, ScalarType};

pub(in crate::codegen) const fn is_complex_scalar(scalar_type: ScalarType) -> bool {
    matches!(
        scalar_type,
        ScalarType::ComplexFloat | ScalarType::ComplexDouble | ScalarType::ComplexLongDouble
    )
}

pub(in crate::codegen) const fn expr_complex_scalar_type(expr: &LoweredExpr) -> Option<ScalarType> {
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
        _ => None,
    }
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
