use super::{LoweredExpr, LoweredLValue};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::ScalarType;

pub(in crate::ir) fn ensure_post_increment_scalar(target: &LoweredLValue) -> CompileResult<()> {
    if !matches!(
        lowered_lvalue_scalar_type(target),
        ScalarType::Int | ScalarType::Pointer
    ) {
        return Err(CompileError::new(
            "post-increment currently supports int and pointer lvalues only",
        ));
    }
    Ok(())
}

pub(in crate::ir) const fn lowered_expr_scalar_type(expr: &LoweredExpr) -> Option<ScalarType> {
    match expr {
        LoweredExpr::Global { scalar_type, .. }
        | LoweredExpr::Local { scalar_type, .. }
        | LoweredExpr::Call {
            return_type: scalar_type,
            ..
        }
        | LoweredExpr::VaArg { scalar_type, .. }
        | LoweredExpr::Cast {
            target: scalar_type,
            ..
        }
        | LoweredExpr::PointerField { scalar_type, .. } => Some(*scalar_type),
        LoweredExpr::GlobalIntSubscript { .. } => Some(ScalarType::Int),
        LoweredExpr::StringLiteral(_)
        | LoweredExpr::LocalAddress { .. }
        | LoweredExpr::GlobalPointerSubscript { .. }
        | LoweredExpr::GlobalAddress { .. }
        | LoweredExpr::PointerOffset { .. }
        | LoweredExpr::PointerFieldAddress { .. } => Some(ScalarType::Pointer),
        LoweredExpr::PointerSubscript { element_type, .. } => Some(*element_type),
        LoweredExpr::Assign { target, .. } | LoweredExpr::PostIncrement { target, .. } => {
            Some(lowered_lvalue_scalar_type(target))
        }
        LoweredExpr::LongInteger(_) => Some(ScalarType::LongLong),
        LoweredExpr::Integer(_)
        | LoweredExpr::DoubleLiteral(_)
        | LoweredExpr::IndirectCall { .. }
        | LoweredExpr::GlobalByteSubscript { .. }
        | LoweredExpr::Unary { .. }
        | LoweredExpr::Conditional { .. }
        | LoweredExpr::Comma { .. }
        | LoweredExpr::Binary { .. } => None,
    }
}

pub(in crate::ir) const fn lowered_lvalue_scalar_type(target: &LoweredLValue) -> ScalarType {
    match target {
        LoweredLValue::Local { scalar_type, .. }
        | LoweredLValue::Global { scalar_type, .. }
        | LoweredLValue::PointerSubscript {
            element_type: scalar_type,
            ..
        }
        | LoweredLValue::PointerField { scalar_type, .. } => *scalar_type,
        LoweredLValue::GlobalByteSubscript { .. } | LoweredLValue::GlobalIntSubscript { .. } => {
            ScalarType::Int
        }
        LoweredLValue::GlobalPointerSubscript { .. } => ScalarType::Pointer,
    }
}

pub(in crate::ir) fn lowered_lvalue_to_expr(target: &LoweredLValue) -> LoweredExpr {
    match target {
        LoweredLValue::Local {
            offset,
            scalar_type,
            ..
        } => LoweredExpr::Local {
            offset: *offset,
            scalar_type: *scalar_type,
        },
        LoweredLValue::Global { name, scalar_type } => LoweredExpr::Global {
            name: name.clone(),
            scalar_type: *scalar_type,
        },
        LoweredLValue::GlobalByteSubscript {
            name,
            index,
            is_unsigned,
        } => LoweredExpr::GlobalByteSubscript {
            name: name.clone(),
            index: index.clone(),
            is_unsigned: *is_unsigned,
        },
        LoweredLValue::GlobalIntSubscript { name, index } => LoweredExpr::GlobalIntSubscript {
            name: name.clone(),
            index: index.clone(),
        },
        LoweredLValue::GlobalPointerSubscript { name, index } => {
            LoweredExpr::GlobalPointerSubscript {
                name: name.clone(),
                index: index.clone(),
            }
        }
        LoweredLValue::PointerSubscript {
            pointer,
            index,
            element_type,
            element_byte_size,
            element_unsigned,
        } => LoweredExpr::PointerSubscript {
            pointer: pointer.clone(),
            index: index.clone(),
            element_type: *element_type,
            element_byte_size: *element_byte_size,
            element_unsigned: *element_unsigned,
        },
        LoweredLValue::PointerField {
            pointer,
            offset,
            scalar_type,
            byte_size,
            is_unsigned,
        } => LoweredExpr::PointerField {
            pointer: pointer.clone(),
            offset: *offset,
            scalar_type: *scalar_type,
            byte_size: *byte_size,
            is_unsigned: *is_unsigned,
        },
    }
}

pub(in crate::ir) fn pointer_field_address(pointer: LoweredExpr, offset: usize) -> LoweredExpr {
    if offset == 0 {
        pointer
    } else {
        LoweredExpr::PointerFieldAddress {
            pointer: Box::new(pointer),
            offset,
        }
    }
}
