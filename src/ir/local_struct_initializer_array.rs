use super::{LoweredExpr, LoweredLValue, LoweringContext, StructAddress};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Expr, LocalStructInitializerValue, ScalarType};

#[derive(Clone, Copy)]
pub(in crate::ir) struct LocalStructArrayField<'a> {
    pub(in crate::ir) target: &'a StructAddress,
    pub(in crate::ir) offset: usize,
    pub(in crate::ir) element_type: ScalarType,
    pub(in crate::ir) element_size: usize,
    pub(in crate::ir) element_unsigned: bool,
    pub(in crate::ir) length: usize,
}

impl LoweringContext {
    pub(in crate::ir) fn push_local_struct_array_field_initializer(
        &mut self,
        field: LocalStructArrayField<'_>,
        values: &[LocalStructInitializerValue],
        value_index: &mut usize,
    ) -> CompileResult<()> {
        let Some(value) = values.get(*value_index) else {
            return Ok(());
        };
        if let Some(bytes) = string_array_initializer(value, field.element_size, field.length)? {
            *value_index += 1;
            return self.push_array_field_bytes(field, &bytes);
        }
        if let LocalStructInitializerValue::Nested(nested_values) = value {
            *value_index += 1;
            let mut nested_exprs = Vec::new();
            collect_array_exprs(nested_values, &mut nested_exprs);
            return self.push_array_field_exprs(field, &nested_exprs);
        }
        let mut exprs = Vec::new();
        while exprs.len() < field.length {
            let Some(value) = values.get(*value_index) else {
                break;
            };
            let LocalStructInitializerValue::Expr(expr) = value else {
                break;
            };
            exprs.push(expr);
            *value_index += 1;
        }
        self.push_array_field_exprs(field, &exprs)
    }

    fn push_array_field_exprs(
        &mut self,
        field: LocalStructArrayField<'_>,
        exprs: &[&Expr],
    ) -> CompileResult<()> {
        if exprs.len() > field.length {
            return Err(CompileError::new(
                "too many local array field initializer values",
            ));
        }
        for (index, expr) in exprs.iter().enumerate() {
            let element_offset = element_field_offset(field.offset, index, field.element_size)?;
            self.push_store(
                LoweredLValue::PointerField {
                    pointer: Box::new(field.target.pointer.clone()),
                    offset: element_offset,
                    scalar_type: field.element_type,
                    byte_size: field.element_size,
                    is_unsigned: field.element_unsigned,
                },
                self.lower_expr(expr)?,
            );
        }
        Ok(())
    }

    fn push_array_field_bytes(
        &mut self,
        field: LocalStructArrayField<'_>,
        bytes: &[u8],
    ) -> CompileResult<()> {
        for (index, byte) in bytes.iter().enumerate() {
            let element_offset = element_field_offset(field.offset, index, 1)?;
            self.push_store(
                LoweredLValue::PointerField {
                    pointer: Box::new(field.target.pointer.clone()),
                    offset: element_offset,
                    scalar_type: ScalarType::Int,
                    byte_size: 1,
                    is_unsigned: field.element_unsigned,
                },
                LoweredExpr::Integer(i64::from(*byte)),
            );
        }
        Ok(())
    }
}

fn collect_array_exprs<'a>(values: &'a [LocalStructInitializerValue], exprs: &mut Vec<&'a Expr>) {
    for value in values {
        match value {
            LocalStructInitializerValue::Expr(expr) => exprs.push(expr),
            LocalStructInitializerValue::Nested(nested) => collect_array_exprs(nested, exprs),
        }
    }
}

fn string_array_initializer(
    value: &LocalStructInitializerValue,
    element_size: usize,
    length: usize,
) -> CompileResult<Option<Vec<u8>>> {
    let LocalStructInitializerValue::Expr(Expr::StringLiteral(value)) = value else {
        return Ok(None);
    };
    if element_size != 1 {
        return Ok(None);
    }
    if value.len() > length {
        return Err(CompileError::new(
            "local string initializer exceeds array field size",
        ));
    }
    Ok(Some(value.as_bytes().to_vec()))
}

fn element_field_offset(
    base_offset: usize,
    index: usize,
    element_size: usize,
) -> CompileResult<usize> {
    index
        .checked_mul(element_size)
        .and_then(|offset| base_offset.checked_add(offset))
        .ok_or_else(|| CompileError::new("local array field initializer offset overflow"))
}
