use super::{
    GlobalBinding, LoweredExpr, LoweredLValue, LoweringContext, lowered_expr_scalar_type,
    pointer_arithmetic, scalar_size,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Expr, ScalarType};

impl LoweringContext {
    pub(in crate::ir) fn lower_global_array_subscript_lvalue(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredLValue>> {
        if let Some(pointer) = self.resolve_global_byte_matrix_row(array, index)? {
            return Ok(Some(Self::pointer_subscript_lvalue(
                pointer,
                LoweredExpr::Integer(0),
                ScalarType::Int,
                1,
                true,
            )));
        }
        if let Some(pointer) = self.resolve_global_short_matrix_row(array, index)? {
            return Ok(Some(Self::pointer_subscript_lvalue(
                pointer,
                LoweredExpr::Integer(0),
                ScalarType::Int,
                2,
                false,
            )));
        }
        if let Expr::Identifier(name) = array
            && let Some(GlobalBinding::UnsignedCharArray { is_unsigned }) =
                self.global_bindings.get(name)
        {
            return Ok(Some(LoweredLValue::GlobalByteSubscript {
                name: name.clone(),
                index: Box::new(self.lower_expr(index)?),
                is_unsigned: *is_unsigned,
            }));
        }
        if let Expr::Identifier(name) = array
            && self.global_bindings.get(name) == Some(&GlobalBinding::IntArray)
        {
            return Ok(Some(LoweredLValue::GlobalIntSubscript {
                name: name.clone(),
                index: Box::new(self.lower_expr(index)?),
            }));
        }
        if let Some((pointer, element_unsigned)) = self.resolve_global_short_array(array) {
            return Ok(Some(Self::pointer_subscript_lvalue(
                pointer,
                self.lower_expr(index)?,
                ScalarType::Int,
                2,
                element_unsigned,
            )));
        }
        if let Expr::Identifier(name) = array
            && let Some(GlobalBinding::ScalarArray { scalar_type, .. }) =
                self.global_bindings.get(name)
        {
            return Ok(Some(Self::pointer_subscript_lvalue(
                LoweredExpr::GlobalAddress { name: name.clone() },
                self.lower_expr(index)?,
                *scalar_type,
                scalar_size(*scalar_type),
                false,
            )));
        }
        if let Some(pointer) = self.resolve_global_pointer_matrix_row(array, index)? {
            return Ok(Some(Self::pointer_subscript_lvalue(
                pointer,
                LoweredExpr::Integer(0),
                ScalarType::Pointer,
                scalar_size(ScalarType::Pointer),
                false,
            )));
        }
        if let Expr::Identifier(name) = array
            && matches!(
                self.global_bindings.get(name),
                Some(GlobalBinding::PointerArray { .. })
            )
        {
            return Ok(Some(LoweredLValue::GlobalPointerSubscript {
                name: name.clone(),
                index: Box::new(self.lower_expr(index)?),
            }));
        }
        Ok(None)
    }

    pub(in crate::ir) fn lower_field_array_subscript_lvalue(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredLValue>> {
        if let Some((pointer, element_type, element_byte_size, element_unsigned)) =
            self.resolve_array_field_subscript(array)?
        {
            return Ok(Some(Self::pointer_subscript_lvalue(
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
            return Ok(Some(Self::pointer_subscript_lvalue(
                pointer,
                flat_index,
                element_type,
                element_byte_size,
                element_unsigned,
            )));
        }
        Ok(None)
    }

    pub(in crate::ir) fn lower_local_array_subscript_lvalue(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredLValue>> {
        if let Some((pointer, element_unsigned)) = self.resolve_local_char_array(array)? {
            return Ok(Some(Self::pointer_subscript_lvalue(
                pointer,
                self.lower_expr(index)?,
                ScalarType::Int,
                1,
                element_unsigned,
            )));
        }
        if let Some((pointer, element_unsigned)) = self.resolve_local_short_array(array)? {
            return Ok(Some(Self::pointer_subscript_lvalue(
                pointer,
                self.lower_expr(index)?,
                ScalarType::Int,
                2,
                element_unsigned,
            )));
        }
        if let Some((pointer, scalar_type)) = self.resolve_local_scalar_array(array)? {
            return Ok(Some(Self::pointer_subscript_lvalue(
                pointer,
                self.lower_expr(index)?,
                scalar_type,
                scalar_size(scalar_type),
                false,
            )));
        }
        if let Some(pointer) = self.resolve_local_pointer_array(array)? {
            return Ok(Some(Self::pointer_subscript_lvalue(
                pointer,
                self.lower_expr(index)?,
                ScalarType::Pointer,
                scalar_size(ScalarType::Pointer),
                false,
            )));
        }
        Ok(None)
    }

    pub(in crate::ir) fn lower_pointer_subscript_lvalue(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<LoweredLValue> {
        let pointer = self.lower_expr(array)?;
        if lowered_expr_scalar_type(&pointer) != Some(ScalarType::Pointer)
            && self.pointer_referent_for_expr(array).is_err()
        {
            return Err(CompileError::new(
                "assignment to non-pointer subscript targets is not supported",
            ));
        }
        let (element_type, element_byte_size) = self.pointer_subscript_layout(array);
        Ok(Self::pointer_subscript_lvalue(
            pointer,
            self.lower_expr(index)?,
            element_type,
            element_byte_size,
            self.pointer_subscript_element_unsigned(array),
        ))
    }

    pub(in crate::ir) fn pointer_subscript_expr(
        pointer: LoweredExpr,
        index: LoweredExpr,
        element_type: ScalarType,
        element_byte_size: usize,
        element_unsigned: bool,
    ) -> LoweredExpr {
        LoweredExpr::PointerSubscript {
            pointer: Box::new(pointer),
            index: Box::new(index),
            element_type,
            element_byte_size,
            element_unsigned,
        }
    }

    pub(in crate::ir) fn pointer_subscript_lvalue(
        pointer: LoweredExpr,
        index: LoweredExpr,
        element_type: ScalarType,
        element_byte_size: usize,
        element_unsigned: bool,
    ) -> LoweredLValue {
        LoweredLValue::PointerSubscript {
            pointer: Box::new(pointer),
            index: Box::new(index),
            element_type,
            element_byte_size,
            element_unsigned,
        }
    }

    pub(in crate::ir) fn pointer_subscript_layout(&self, array: &Expr) -> (ScalarType, usize) {
        let element_type = self.pointer_subscript_element_type(array);
        let element_byte_size = self
            .pointer_referent_for_expr(array)
            .ok()
            .and_then(|referent| pointer_arithmetic::byte_size(&referent))
            .unwrap_or_else(|| scalar_size(element_type));
        (element_type, element_byte_size)
    }

    pub(in crate::ir) fn pointer_subscript_element_type(&self, array: &Expr) -> ScalarType {
        match self.pointer_referent_for_expr(array).ok().as_deref() {
            Some(referent)
                if pointer_arithmetic::is_pointer(referent)
                    || pointer_arithmetic::is_function_pointer(referent) =>
            {
                ScalarType::Pointer
            }
            Some("float" | "double") => ScalarType::Double,
            Some("long double") => ScalarType::LongDouble,
            Some("float _Complex") => ScalarType::ComplexFloat,
            Some("double _Complex") => ScalarType::ComplexDouble,
            Some("long double _Complex") => ScalarType::ComplexLongDouble,
            _ => ScalarType::Int,
        }
    }

    pub(in crate::ir) fn pointer_subscript_element_unsigned(&self, array: &Expr) -> bool {
        self.pointer_referent_for_expr(array)
            .is_ok_and(|referent| pointer_arithmetic::is_unsigned_integer(&referent))
    }
}
