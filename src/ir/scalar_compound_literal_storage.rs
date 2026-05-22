use super::{LoweredExpr, LoweredLValue, LoweringContext, pointer_arithmetic, scalar_size};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Expr, LValue, ScalarType};

impl LoweringContext {
    pub(in crate::ir) fn is_scalar_compound_address(initializer: &Expr) -> bool {
        scalar_compound_address(initializer).is_ok()
    }

    pub(in crate::ir) fn lower_scalar_compound_pointer_initializer(
        &mut self,
        pointer_slot: usize,
        initializer: &Expr,
    ) -> CompileResult<()> {
        let target = LoweredLValue::Local {
            slot: pointer_slot,
            offset: self.local_offset(pointer_slot)?,
            scalar_type: ScalarType::Pointer,
            referent: None,
        };
        self.lower_scalar_compound_pointer_assignment(target, initializer)
    }

    pub(in crate::ir) fn lower_scalar_compound_pointer_assignment(
        &mut self,
        target: LoweredLValue,
        initializer: &Expr,
    ) -> CompileResult<()> {
        let pointer = self.lower_scalar_compound_pointer(initializer)?;
        self.push_store(target, pointer)
    }

    fn lower_scalar_compound_pointer(&mut self, initializer: &Expr) -> CompileResult<LoweredExpr> {
        let (scalar_type, referent, value) = scalar_compound_address(initializer)?;
        let byte_size = scalar_compound_byte_size(scalar_type, referent);
        let slot = self.declare_anonymous_slot(scalar_type, byte_size, byte_size)?;
        let pointer = LoweredExpr::LocalAddress {
            offset: self.local_offset(slot)?,
            byte_size,
        };
        let target = LoweredLValue::PointerSubscript {
            pointer: Box::new(pointer.clone()),
            index: Box::new(LoweredExpr::Integer(0)),
            element_type: scalar_type,
            element_byte_size: byte_size,
            element_unsigned: referent.is_some_and(pointer_arithmetic::is_unsigned_integer),
        };
        let value = self.lower_expr(value)?;
        self.push_store(target, value)?;
        Ok(pointer)
    }
}

fn scalar_compound_address(initializer: &Expr) -> CompileResult<(ScalarType, Option<&str>, &Expr)> {
    let Expr::AddressOf {
        target:
            LValue::ScalarCompoundLiteral {
                scalar_type,
                referent,
                value,
            },
    } = initializer
    else {
        return Err(CompileError::new(
            "expected address of scalar compound literal",
        ));
    };
    Ok((*scalar_type, referent.as_deref(), value))
}

fn scalar_compound_byte_size(scalar_type: ScalarType, referent: Option<&str>) -> usize {
    if let Some(referent) = referent
        && let Some(byte_size) = pointer_arithmetic::byte_size(referent)
    {
        return byte_size;
    }
    scalar_size(scalar_type)
}
