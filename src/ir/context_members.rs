use super::{
    ArrayFieldSubscript, LoweredExpr, LoweredLValue, LoweringContext, NestedArrayFieldSubscript,
    ResolvedMember, StructAddress, lowered_expr_scalar_type, pointer_field_address, scalar_size,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{BinaryOp, Expr, FieldType, ScalarType};

impl LoweringContext {
    pub(in crate::ir) fn lower_member_expr(
        &self,
        base: &Expr,
        field: &str,
        dereference: bool,
    ) -> CompileResult<LoweredExpr> {
        let member = self.resolve_member_access(base, field, dereference)?;
        let (scalar_type, byte_size, is_unsigned) = match member.field_type {
            FieldType::Scalar(field) => (field.scalar_type, field.byte_size, field.is_unsigned),
            FieldType::Pointer { .. } => {
                (ScalarType::Pointer, scalar_size(ScalarType::Pointer), false)
            }
            FieldType::Array { .. } | FieldType::StructArray { .. } => {
                return Ok(pointer_field_address(member.pointer, member.offset));
            }
            FieldType::Struct(_) => {
                return Err(CompileError::new("struct member value is not supported"));
            }
        };
        Ok(LoweredExpr::PointerField {
            pointer: Box::new(member.pointer),
            offset: member.offset,
            scalar_type,
            byte_size,
            is_unsigned,
        })
    }

    pub(in crate::ir) fn lower_member_lvalue(
        &self,
        base: &Expr,
        field: &str,
        dereference: bool,
    ) -> CompileResult<LoweredLValue> {
        let member = self.resolve_member_access(base, field, dereference)?;
        let (scalar_type, byte_size, is_unsigned) = match member.field_type {
            FieldType::Scalar(field) => (field.scalar_type, field.byte_size, field.is_unsigned),
            FieldType::Pointer { .. } => {
                (ScalarType::Pointer, scalar_size(ScalarType::Pointer), false)
            }
            FieldType::Array { .. } | FieldType::StructArray { .. } => {
                return Err(CompileError::new(
                    "assignment to array member is not supported",
                ));
            }
            FieldType::Struct(_) => {
                return Err(CompileError::new(
                    "assignment to struct member is not supported",
                ));
            }
        };
        Ok(LoweredLValue::PointerField {
            pointer: Box::new(member.pointer),
            offset: member.offset,
            scalar_type,
            byte_size,
            is_unsigned,
        })
    }

    pub(in crate::ir) fn resolve_member_access(
        &self,
        base: &Expr,
        field: &str,
        dereference: bool,
    ) -> CompileResult<ResolvedMember> {
        let access = if dereference {
            let pointer = self.lower_expr(base)?;
            if lowered_expr_scalar_type(&pointer) != Some(ScalarType::Pointer)
                && !self.expr_is_pointer_return_call(base)
            {
                return Err(CompileError::new("member dereference requires a pointer"));
            }
            StructAddress {
                pointer,
                offset: 0,
                struct_name: self.pointer_referent_for_expr(base)?,
            }
        } else {
            self.resolve_struct_address(base)?
        };
        let layout = self
            .structs
            .get(&access.struct_name)
            .ok_or_else(|| CompileError::new(format!("unknown struct: {}", access.struct_name)))?;
        let field = layout
            .fields
            .iter()
            .find(|candidate| candidate.name == field)
            .ok_or_else(|| {
                CompileError::new(format!(
                    "unknown struct field: {}.{field}",
                    access.struct_name
                ))
            })?;
        Ok(ResolvedMember {
            pointer: access.pointer,
            offset: access
                .offset
                .checked_add(field.offset)
                .ok_or_else(|| CompileError::new("struct member offset overflow"))?,
            field_type: field.field_type.clone(),
        })
    }

    pub(in crate::ir) fn resolve_array_field_subscript(
        &self,
        array: &Expr,
    ) -> CompileResult<Option<ArrayFieldSubscript>> {
        let Expr::Member {
            base,
            field,
            dereference,
        } = array
        else {
            return Ok(None);
        };
        let member = self.resolve_member_access(base, field, *dereference)?;
        let FieldType::Array {
            element_type,
            element_size,
            element_unsigned,
            ..
        } = member.field_type
        else {
            return Ok(None);
        };
        Ok(Some((
            pointer_field_address(member.pointer, member.offset),
            element_type,
            element_size,
            element_unsigned,
        )))
    }

    pub(in crate::ir) fn resolve_nested_array_field_subscript(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<NestedArrayFieldSubscript>> {
        let Expr::Subscript {
            array: nested_array,
            index: row_index,
        } = array
        else {
            return Ok(None);
        };
        let Expr::Member {
            base,
            field,
            dereference,
        } = nested_array.as_ref()
        else {
            return Ok(None);
        };
        let member = self.resolve_member_access(base, field, *dereference)?;
        let FieldType::Array {
            element_type,
            element_size,
            element_unsigned,
            columns: Some(columns),
            ..
        } = member.field_type
        else {
            return Ok(None);
        };
        let columns = i64::try_from(columns)
            .map_err(|_| CompileError::new("struct array column count does not fit i64"))?;
        let row_offset = LoweredExpr::Binary {
            op: BinaryOp::Mul,
            left: Box::new(self.lower_expr(row_index)?),
            right: Box::new(LoweredExpr::Integer(columns)),
        };
        let flat_index = LoweredExpr::Binary {
            op: BinaryOp::Add,
            left: Box::new(row_offset),
            right: Box::new(self.lower_expr(index)?),
        };
        Ok(Some((
            pointer_field_address(member.pointer, member.offset),
            flat_index,
            element_type,
            element_size,
            element_unsigned,
        )))
    }

    pub(in crate::ir) fn resolve_struct_array_field_subscript_address(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<StructAddress>> {
        let Expr::Member {
            base,
            field,
            dereference,
        } = array
        else {
            return Ok(None);
        };
        let member = self.resolve_member_access(base, field, *dereference)?;
        let FieldType::StructArray { struct_name, .. } = member.field_type else {
            return Ok(None);
        };
        let byte_size = self.struct_layout(&struct_name)?.size;
        Ok(Some(StructAddress {
            pointer: LoweredExpr::PointerOffset {
                pointer: Box::new(pointer_field_address(member.pointer, member.offset)),
                index: Box::new(self.lower_expr(index)?),
                byte_size,
            },
            offset: 0,
            struct_name,
        }))
    }
}
