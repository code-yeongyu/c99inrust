use super::{LocalBinding, LoweredExpr, LoweringContext, local_int_matrix_byte_size, scalar_size};
use crate::diagnostics::CompileResult;
use crate::parser::{Expr, ScalarType};

impl LoweringContext {
    pub(in crate::ir) fn declare_int_matrix(
        &mut self,
        name: &str,
        rows: usize,
        columns: usize,
    ) -> CompileResult<usize> {
        self.declare_slot(
            name,
            ScalarType::Int,
            local_int_matrix_byte_size(rows, columns)?,
            scalar_size(ScalarType::Int),
            LocalBinding::IntMatrix {
                slot: self.local_slots.len(),
                rows,
                columns,
            },
        )
    }

    pub(in crate::ir) fn resolve_local_int_matrix_row(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredExpr>> {
        let Expr::Identifier(name) = array else {
            return Ok(None);
        };
        let Some(LocalBinding::IntMatrix {
            slot,
            rows,
            columns,
        }) = self.local_binding(name)
        else {
            return Ok(None);
        };
        Ok(Some(LoweredExpr::PointerOffset {
            pointer: Box::new(LoweredExpr::LocalAddress {
                offset: self.local_offset(slot)?,
                byte_size: local_int_matrix_byte_size(rows, columns)?,
            }),
            index: Box::new(self.lower_expr(index)?),
            byte_size: columns * scalar_size(ScalarType::Int),
        }))
    }
}
