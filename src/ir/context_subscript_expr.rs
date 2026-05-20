use super::{
    GlobalBinding, LoweredExpr, LoweredLValue, LoweringContext, lowered_expr_scalar_type,
    scalar_size,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Expr, ScalarType};

impl LoweringContext {
    pub(in crate::ir) fn lower_subscript(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<LoweredExpr> {
        if let Some(subscript) = self.lower_global_array_subscript_expr(array, index)? {
            return Ok(subscript);
        }
        if let Some(subscript) = self.lower_field_array_subscript_expr(array, index)? {
            return Ok(subscript);
        }
        if let Some(subscript) = self.lower_struct_array_subscript_expr(array, index)? {
            return Ok(subscript);
        }
        if let Some(subscript) = self.lower_local_array_subscript_expr(array, index)? {
            return Ok(subscript);
        }
        self.lower_pointer_subscript_expr(array, index)
    }

    pub(in crate::ir) fn lower_subscript_lvalue(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<LoweredLValue> {
        if let Some(subscript) = self.lower_global_array_subscript_lvalue(array, index)? {
            return Ok(subscript);
        }
        if let Some(subscript) = self.lower_field_array_subscript_lvalue(array, index)? {
            return Ok(subscript);
        }
        if let Some(subscript) = self.lower_local_array_subscript_lvalue(array, index)? {
            return Ok(subscript);
        }
        self.lower_pointer_subscript_lvalue(array, index)
    }

    pub(in crate::ir) fn lower_global_array_subscript_expr(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredExpr>> {
        if let Some(pointer) = self.resolve_global_int_matrix_row(array, index)? {
            return Ok(Some(pointer));
        }
        if let Some(pointer) = self.resolve_global_short_matrix_row(array, index)? {
            return Ok(Some(pointer));
        }
        if let Some(pointer) = self.resolve_global_byte_matrix_row(array, index)? {
            return Ok(Some(pointer));
        }
        if let Expr::Identifier(name) = array
            && let Some(GlobalBinding::UnsignedCharArray { is_unsigned }) =
                self.global_bindings.get(name)
        {
            return Ok(Some(LoweredExpr::GlobalByteSubscript {
                name: name.clone(),
                index: Box::new(self.lower_expr(index)?),
                is_unsigned: *is_unsigned,
            }));
        }
        if let Expr::Identifier(name) = array
            && self.global_bindings.get(name) == Some(&GlobalBinding::IntArray)
        {
            return Ok(Some(LoweredExpr::GlobalIntSubscript {
                name: name.clone(),
                index: Box::new(self.lower_expr(index)?),
            }));
        }
        if let Some((pointer, element_unsigned)) = self.resolve_global_short_array(array) {
            return Ok(Some(Self::pointer_subscript_expr(
                pointer,
                self.lower_expr(index)?,
                ScalarType::Int,
                2,
                element_unsigned,
            )));
        }
        if let Some(pointer) = self.resolve_global_pointer_matrix_row(array, index)? {
            return Ok(Some(pointer));
        }
        if let Expr::Identifier(name) = array
            && matches!(
                self.global_bindings.get(name),
                Some(GlobalBinding::PointerArray { .. })
            )
        {
            return Ok(Some(LoweredExpr::GlobalPointerSubscript {
                name: name.clone(),
                index: Box::new(self.lower_expr(index)?),
            }));
        }
        Ok(None)
    }

    pub(in crate::ir) fn lower_field_array_subscript_expr(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredExpr>> {
        if let Some((pointer, element_type, element_byte_size, element_unsigned)) =
            self.resolve_array_field_subscript(array)?
        {
            return Ok(Some(Self::pointer_subscript_expr(
                pointer,
                self.lower_expr(index)?,
                element_type,
                element_byte_size,
                element_unsigned,
            )));
        }
        if let Some((pointer, flat_index, element_type, element_byte_size, element_unsigned)) =
            self.resolve_nested_array_field_subscript(array, index)?
        {
            return Ok(Some(Self::pointer_subscript_expr(
                pointer,
                flat_index,
                element_type,
                element_byte_size,
                element_unsigned,
            )));
        }
        Ok(None)
    }

    pub(in crate::ir) fn lower_struct_array_subscript_expr(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredExpr>> {
        if let Some(pointer) = self.resolve_global_struct_matrix_row(array, index)? {
            return Ok(Some(pointer));
        }
        if let Some(address) = self.resolve_global_struct_subscript_address(array, index)? {
            return Ok(Some(address.pointer));
        }
        self.resolve_local_char_matrix_row(array, index)
    }

    pub(in crate::ir) fn lower_local_array_subscript_expr(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredExpr>> {
        if let Some((pointer, element_unsigned)) = self.resolve_local_char_array(array)? {
            return Ok(Some(Self::pointer_subscript_expr(
                pointer,
                self.lower_expr(index)?,
                ScalarType::Int,
                1,
                element_unsigned,
            )));
        }
        if let Some((pointer, element_unsigned)) = self.resolve_local_short_array(array)? {
            return Ok(Some(Self::pointer_subscript_expr(
                pointer,
                self.lower_expr(index)?,
                ScalarType::Int,
                2,
                element_unsigned,
            )));
        }
        if let Some(pointer) = self.resolve_local_pointer_array(array)? {
            return Ok(Some(Self::pointer_subscript_expr(
                pointer,
                self.lower_expr(index)?,
                ScalarType::Pointer,
                scalar_size(ScalarType::Pointer),
                false,
            )));
        }
        Ok(None)
    }

    pub(in crate::ir) fn lower_pointer_subscript_expr(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<LoweredExpr> {
        let pointer = self.lower_expr(array)?;
        if lowered_expr_scalar_type(&pointer) != Some(ScalarType::Pointer)
            && self.pointer_referent_for_expr(array).is_err()
        {
            return Err(CompileError::new(
                "only pointer and global byte-array subscripts are supported",
            ));
        }
        let (element_type, element_byte_size) = self.pointer_subscript_layout(array);
        Ok(Self::pointer_subscript_expr(
            pointer,
            self.lower_expr(index)?,
            element_type,
            element_byte_size,
            self.pointer_subscript_element_unsigned(array),
        ))
    }
}
