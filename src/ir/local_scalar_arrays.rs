use super::{
    LocalBinding, LoweredExpr, LoweredLValue, LoweringContext, local_scalar_array_byte_size,
    local_scalar_array_element_size, zero_expr_for,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Expr, ScalarType};

impl LoweringContext {
    pub(in crate::ir) fn lower_local_scalar_array(
        &mut self,
        name: &str,
        scalar_type: ScalarType,
        length: usize,
        initializer: Option<&[Expr]>,
    ) -> CompileResult<()> {
        let slot = self.declare_scalar_array(name, scalar_type, length)?;
        if let Some(initializer) = initializer {
            self.initialize_local_scalar_array(slot, scalar_type, length, initializer)?;
        }
        Ok(())
    }

    pub(in crate::ir) fn declare_scalar_array(
        &mut self,
        name: &str,
        scalar_type: ScalarType,
        length: usize,
    ) -> CompileResult<usize> {
        let byte_size = local_scalar_array_byte_size(scalar_type, length)?;
        self.declare_slot(
            name,
            scalar_type,
            byte_size,
            local_scalar_array_element_size(scalar_type),
            LocalBinding::ScalarArray {
                slot: self.local_slots.len(),
                scalar_type,
                length,
            },
        )
    }

    pub(in crate::ir) fn resolve_local_scalar_array(
        &self,
        array: &Expr,
    ) -> CompileResult<Option<(LoweredExpr, ScalarType)>> {
        let Expr::Identifier(name) = array else {
            return Ok(None);
        };
        let Some(LocalBinding::ScalarArray {
            slot,
            scalar_type,
            length,
        }) = self.local_binding(name)
        else {
            return Ok(None);
        };
        Ok(Some((
            LoweredExpr::LocalAddress {
                offset: self.local_offset(slot)?,
                byte_size: local_scalar_array_byte_size(scalar_type, length)?,
            },
            scalar_type,
        )))
    }

    fn initialize_local_scalar_array(
        &mut self,
        slot: usize,
        scalar_type: ScalarType,
        length: usize,
        initializer: &[Expr],
    ) -> CompileResult<()> {
        if initializer.len() > length {
            return Err(CompileError::new(
                "local scalar array initializer is too large",
            ));
        }
        let offset = self.local_offset(slot)?;
        let byte_size = local_scalar_array_byte_size(scalar_type, length)?;
        for index in 0..length {
            let value = initializer.get(index).map_or_else(
                || Ok(zero_expr_for(scalar_type)),
                |expr| self.lower_expr(expr),
            )?;
            self.push_store(
                LoweredLValue::PointerSubscript {
                    pointer: Box::new(LoweredExpr::LocalAddress { offset, byte_size }),
                    index: Box::new(LoweredExpr::Integer(
                        i64::try_from(index)
                            .map_err(|_| CompileError::new("local scalar array index overflow"))?,
                    )),
                    element_type: scalar_type,
                    element_byte_size: local_scalar_array_element_size(scalar_type),
                    element_unsigned: false,
                },
                value,
            )?;
        }
        Ok(())
    }
}
