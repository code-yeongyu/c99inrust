use super::{
    GlobalBinding, LocalBinding, LoweredExpr, LoweringContext, local_array,
    local_char_matrix_byte_size, local_pointer_array_byte_size, local_short_array_byte_size,
    scalar_size,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Expr, ScalarType};

impl LoweringContext {
    pub(in crate::ir) fn resolve_local_pointer_array(
        &self,
        array: &Expr,
    ) -> CompileResult<Option<LoweredExpr>> {
        let Expr::Identifier(name) = array else {
            return Ok(None);
        };
        let Some(LocalBinding::PointerArray { slot, length }) = self.local_binding(name) else {
            return Ok(None);
        };
        Ok(Some(LoweredExpr::LocalAddress {
            offset: self.local_offset(slot)?,
            byte_size: local_pointer_array_byte_size(length)?,
        }))
    }

    pub(in crate::ir) fn resolve_local_short_array(
        &self,
        array: &Expr,
    ) -> CompileResult<Option<(LoweredExpr, bool)>> {
        let Expr::Identifier(name) = array else {
            return Ok(None);
        };
        let Some(LocalBinding::ShortArray {
            slot,
            length,
            is_unsigned,
        }) = self.local_binding(name)
        else {
            return Ok(None);
        };
        Ok(Some((
            LoweredExpr::LocalAddress {
                offset: self.local_offset(slot)?,
                byte_size: local_short_array_byte_size(length)?,
            },
            is_unsigned,
        )))
    }

    pub(in crate::ir) fn resolve_local_char_array(
        &self,
        array: &Expr,
    ) -> CompileResult<Option<(LoweredExpr, bool)>> {
        let binding = if let Expr::Identifier(name) = array {
            self.local_binding(name)
        } else {
            None
        };
        local_array::char_array_pointer(array, binding.as_ref(), |slot| self.local_offset(slot))
    }

    pub(in crate::ir) fn resolve_global_short_array(
        &self,
        array: &Expr,
    ) -> Option<(LoweredExpr, bool)> {
        let Expr::Identifier(name) = array else {
            return None;
        };
        let Some(GlobalBinding::ShortArray {
            is_unsigned,
            columns: None,
        }) = self.global_bindings.get(name)
        else {
            return None;
        };
        Some((
            LoweredExpr::GlobalAddress { name: name.clone() },
            *is_unsigned,
        ))
    }

    pub(in crate::ir) fn resolve_local_char_matrix_row(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredExpr>> {
        let Expr::Identifier(name) = array else {
            return Ok(None);
        };
        let Some(LocalBinding::CharMatrix {
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
                byte_size: local_char_matrix_byte_size(rows, columns)?,
            }),
            index: Box::new(self.lower_expr(index)?),
            byte_size: columns,
        }))
    }

    pub(in crate::ir) fn resolve_global_byte_matrix_row(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredExpr>> {
        let Expr::Identifier(name) = array else {
            return Ok(None);
        };
        let Some(GlobalBinding::UnsignedCharMatrix { columns, .. }) =
            self.global_bindings.get(name)
        else {
            return Ok(None);
        };
        Ok(Some(LoweredExpr::PointerOffset {
            pointer: Box::new(LoweredExpr::GlobalAddress { name: name.clone() }),
            index: Box::new(self.lower_expr(index)?),
            byte_size: *columns,
        }))
    }

    pub(in crate::ir) fn resolve_global_int_matrix_row(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredExpr>> {
        let Expr::Identifier(name) = array else {
            return Ok(None);
        };
        let Some(GlobalBinding::IntMatrix { columns }) = self.global_bindings.get(name) else {
            return Ok(None);
        };
        let byte_size = columns
            .checked_mul(scalar_size(ScalarType::Int))
            .ok_or_else(|| CompileError::new("global int matrix row size overflow"))?;
        Ok(Some(LoweredExpr::PointerOffset {
            pointer: Box::new(LoweredExpr::GlobalAddress { name: name.clone() }),
            index: Box::new(self.lower_expr(index)?),
            byte_size,
        }))
    }

    pub(in crate::ir) fn resolve_global_short_matrix_row(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredExpr>> {
        let Expr::Identifier(name) = array else {
            return Ok(None);
        };
        let Some(GlobalBinding::ShortArray {
            columns: Some(columns),
            ..
        }) = self.global_bindings.get(name)
        else {
            return Ok(None);
        };
        let byte_size = columns
            .checked_mul(2)
            .ok_or_else(|| CompileError::new("global short matrix row size overflow"))?;
        Ok(Some(LoweredExpr::PointerOffset {
            pointer: Box::new(LoweredExpr::GlobalAddress { name: name.clone() }),
            index: Box::new(self.lower_expr(index)?),
            byte_size,
        }))
    }

    pub(in crate::ir) fn resolve_global_pointer_matrix_row(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredExpr>> {
        let Expr::Identifier(name) = array else {
            return Ok(None);
        };
        let Some(GlobalBinding::PointerArray {
            columns: Some(columns),
            ..
        }) = self.global_bindings.get(name)
        else {
            return Ok(None);
        };
        let byte_size = columns
            .checked_mul(scalar_size(ScalarType::Pointer))
            .ok_or_else(|| CompileError::new("global pointer matrix row size overflow"))?;
        Ok(Some(LoweredExpr::PointerOffset {
            pointer: Box::new(LoweredExpr::GlobalAddress { name: name.clone() }),
            index: Box::new(self.lower_expr(index)?),
            byte_size,
        }))
    }

    pub(in crate::ir) fn resolve_global_struct_matrix_row(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredExpr>> {
        let Expr::Identifier(name) = array else {
            return Ok(None);
        };
        let Some(GlobalBinding::StructArray {
            byte_size,
            columns: Some(columns),
            ..
        }) = self.global_bindings.get(name)
        else {
            return Ok(None);
        };
        let row_size = columns
            .checked_mul(*byte_size)
            .ok_or_else(|| CompileError::new("global struct matrix row size overflow"))?;
        Ok(Some(LoweredExpr::PointerOffset {
            pointer: Box::new(LoweredExpr::GlobalAddress { name: name.clone() }),
            index: Box::new(self.lower_expr(index)?),
            byte_size: row_size,
        }))
    }
}
