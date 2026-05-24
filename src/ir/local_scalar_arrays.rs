use super::{
    LocalBinding, LoweredExpr, LoweringContext, local_scalar_array_byte_size, scalar_size,
};
use crate::diagnostics::CompileResult;
use crate::parser::{Expr, ScalarType};

impl LoweringContext {
    pub(in crate::ir) fn lower_local_scalar_array(
        &mut self,
        name: &str,
        scalar_type: ScalarType,
        length: usize,
    ) -> CompileResult<()> {
        self.declare_scalar_array(name, scalar_type, length)?;
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
            scalar_size(scalar_type),
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
}
