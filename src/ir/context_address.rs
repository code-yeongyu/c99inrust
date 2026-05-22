use super::{
    LocalBinding, LoweredExpr, LoweringContext, local_char_matrix_byte_size,
    local_int_array_byte_size, local_int_matrix_byte_size, local_pointer_array_byte_size,
    local_short_array_byte_size, lowered_expr_scalar_type, pointer_field_address, scalar_size,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Expr, LValue, ScalarType};

impl LoweringContext {
    pub(in crate::ir) fn lower_address_of(&self, target: &LValue) -> CompileResult<LoweredExpr> {
        match target {
            LValue::Subscript { array, index } => self.lower_address_of_subscript(array, index),
            LValue::Identifier(name) => self.lower_address_of_identifier(name),
            LValue::Member {
                base,
                field,
                dereference,
            } => self.lower_address_of_member(base, field, *dereference),
            LValue::ScalarCompoundLiteral { .. } => Err(CompileError::new(
                "scalar compound literal address requires storage lowering",
            )),
        }
    }

    pub(in crate::ir) fn lower_address_of_subscript(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<LoweredExpr> {
        if let Some(pointer) = self.resolve_global_int_matrix_row(array, index)? {
            return Ok(pointer);
        }
        if let Some(pointer) = self.resolve_global_short_matrix_row(array, index)? {
            return Ok(pointer);
        }
        if let Some(pointer) = self.resolve_global_byte_matrix_row(array, index)? {
            return Ok(pointer);
        }
        if let Some((pointer, _element_unsigned)) = self.resolve_global_short_array(array) {
            return Ok(LoweredExpr::PointerOffset {
                pointer: Box::new(pointer),
                index: Box::new(self.lower_expr(index)?),
                byte_size: 2,
            });
        }
        if let Some((pointer, _element_type, element_byte_size, _element_unsigned)) =
            self.resolve_array_field_subscript(array)?
        {
            return Ok(LoweredExpr::PointerOffset {
                pointer: Box::new(pointer),
                index: Box::new(self.lower_expr(index)?),
                byte_size: element_byte_size,
            });
        }
        if let Some((pointer, flat_index, _element_type, element_byte_size, _element_unsigned)) =
            self.resolve_nested_array_field_subscript(array, index)?
        {
            return Ok(LoweredExpr::PointerOffset {
                pointer: Box::new(pointer),
                index: Box::new(flat_index),
                byte_size: element_byte_size,
            });
        }
        if let Some((pointer, _element_unsigned)) = self.resolve_local_char_array(array)? {
            return Ok(LoweredExpr::PointerOffset {
                pointer: Box::new(pointer),
                index: Box::new(self.lower_expr(index)?),
                byte_size: 1,
            });
        }
        if let Some((pointer, _element_unsigned)) = self.resolve_local_short_array(array)? {
            return Ok(LoweredExpr::PointerOffset {
                pointer: Box::new(pointer),
                index: Box::new(self.lower_expr(index)?),
                byte_size: 2,
            });
        }
        self.lower_address_of_struct_subscript(array, index)?
            .map_or_else(|| self.lower_address_of_pointer_subscript(array, index), Ok)
    }

    pub(in crate::ir) fn lower_address_of_struct_subscript(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredExpr>> {
        if let Some(address) = self.resolve_global_struct_subscript_address(array, index)? {
            return Ok(Some(address.pointer));
        }
        if let Some(address) = self.resolve_struct_array_field_subscript_address(array, index)? {
            return Ok(Some(address.pointer));
        }
        Ok(self
            .resolve_pointer_struct_subscript_address(array, index)
            .ok()
            .map(|address| address.pointer))
    }

    pub(in crate::ir) fn lower_address_of_pointer_subscript(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<LoweredExpr> {
        let pointer = self.lower_expr(array)?;
        if lowered_expr_scalar_type(&pointer) != Some(ScalarType::Pointer) {
            return Err(CompileError::new(
                "address of subscript requires a pointer base",
            ));
        }
        let (_element_type, element_byte_size) = self.pointer_subscript_layout(array);
        Ok(LoweredExpr::PointerOffset {
            pointer: Box::new(pointer),
            index: Box::new(self.lower_expr(index)?),
            byte_size: element_byte_size,
        })
    }

    pub(in crate::ir) fn lower_address_of_identifier(
        &self,
        name: &str,
    ) -> CompileResult<LoweredExpr> {
        let Some(binding) = self.local_binding(name) else {
            if self.global_bindings.contains_key(name) {
                return Ok(LoweredExpr::GlobalAddress {
                    name: name.to_owned(),
                });
            }
            return Err(CompileError::new("unsupported address-of target"));
        };
        self.lower_address_of_local_binding(&binding)
    }

    pub(in crate::ir) fn lower_address_of_local_binding(
        &self,
        binding: &LocalBinding,
    ) -> CompileResult<LoweredExpr> {
        let (slot, byte_size) = match binding {
            LocalBinding::StaticScalar { global_name, .. } => {
                return Ok(LoweredExpr::GlobalAddress {
                    name: global_name.clone(),
                });
            }
            LocalBinding::Scalar {
                slot, scalar_type, ..
            } => (*slot, scalar_size(*scalar_type)),
            LocalBinding::CharArray { slot, length, .. } => (*slot, *length),
            LocalBinding::IntArray { slot, length } => (*slot, local_int_array_byte_size(*length)?),
            LocalBinding::IntMatrix {
                slot,
                rows,
                columns,
            } => (*slot, local_int_matrix_byte_size(*rows, *columns)?),
            LocalBinding::ShortArray { slot, length, .. } => {
                (*slot, local_short_array_byte_size(*length)?)
            }
            LocalBinding::CharMatrix {
                slot,
                rows,
                columns,
            } => (*slot, local_char_matrix_byte_size(*rows, *columns)?),
            LocalBinding::PointerArray { slot, length, .. } => {
                (*slot, local_pointer_array_byte_size(*length)?)
            }
            LocalBinding::StructObject {
                slot, byte_size, ..
            } => (*slot, *byte_size),
            LocalBinding::StructArray {
                slot,
                byte_size,
                length,
                ..
            } => (
                *slot,
                byte_size
                    .checked_mul(*length)
                    .ok_or_else(|| CompileError::new("local struct array size overflow"))?,
            ),
            LocalBinding::VaList { slot } => (*slot, scalar_size(ScalarType::VaList)),
        };
        Ok(LoweredExpr::LocalAddress {
            offset: self.local_offset(slot)?,
            byte_size,
        })
    }

    pub(in crate::ir) fn lower_address_of_member(
        &self,
        base: &Expr,
        field: &str,
        dereference: bool,
    ) -> CompileResult<LoweredExpr> {
        let member = self.resolve_member_access(base, field, dereference)?;
        Ok(pointer_field_address(member.pointer, member.offset))
    }
}
