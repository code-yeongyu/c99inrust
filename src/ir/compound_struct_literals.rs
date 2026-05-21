use super::{LoweredExpr, LoweringContext, const_eval, scalar_size, zero_expr_for};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Expr, FieldType, LocalStructInitializerValue, ScalarType};

impl LoweringContext {
    pub(in crate::ir) fn lower_struct_compound_member(
        &self,
        base: &Expr,
        field: &str,
        dereference: bool,
    ) -> CompileResult<Option<LoweredExpr>> {
        if dereference {
            return Ok(None);
        }
        let Some((struct_name, values)) = self.struct_compound_member_source(base)? else {
            return Ok(None);
        };
        self.lower_compound_struct_member_value(&struct_name, values, field)
            .map(Some)
    }

    pub(in crate::ir) fn lower_struct_compound_array_field_subscript(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredExpr>> {
        let Expr::Member {
            base,
            field,
            dereference: false,
        } = array
        else {
            return Ok(None);
        };
        let Some((struct_name, values)) = self.struct_compound_member_source(base)? else {
            return Ok(None);
        };
        let layout = self.struct_layout(&struct_name)?;
        let Some((field_index, field_type)) =
            layout
                .fields
                .iter()
                .enumerate()
                .find_map(|(index, candidate)| {
                    (candidate.name == *field).then_some((index, &candidate.field_type))
                })
        else {
            return Err(CompileError::new(format!(
                "unknown struct field: {struct_name}.{field}"
            )));
        };
        let FieldType::Array {
            element_type,
            element_size,
            element_unsigned,
            ..
        } = field_type
        else {
            return Ok(None);
        };
        let index = const_eval(index)?;
        if index < 0 {
            return Err(CompileError::new(
                "compound literal array field subscript is negative",
            ));
        }
        let index = usize::try_from(index).map_err(|_| {
            CompileError::new("compound literal array field subscript is too large")
        })?;
        let Some(LocalStructInitializerValue::Nested(array_values)) = values.get(field_index)
        else {
            return Ok(Some(zero_expr_for(*element_type)));
        };
        lower_compound_scalar_value(
            self,
            array_values.get(index),
            *element_type,
            *element_size,
            *element_unsigned,
        )
        .map(Some)
    }

    fn lower_compound_struct_member_value(
        &self,
        struct_name: &str,
        values: &[LocalStructInitializerValue],
        field: &str,
    ) -> CompileResult<LoweredExpr> {
        let layout = self.struct_layout(struct_name)?;
        let Some((index, field_type)) =
            layout
                .fields
                .iter()
                .enumerate()
                .find_map(|(index, candidate)| {
                    (candidate.name == field).then_some((index, &candidate.field_type))
                })
        else {
            return Err(CompileError::new(format!(
                "unknown struct field: {struct_name}.{field}"
            )));
        };
        lower_compound_field_value(self, values.get(index), field_type)
    }

    fn struct_compound_member_source<'a>(
        &self,
        base: &'a Expr,
    ) -> CompileResult<Option<(String, &'a [LocalStructInitializerValue])>> {
        match base {
            Expr::StructCompoundLiteral {
                struct_name,
                values,
            } => Ok(Some((struct_name.clone(), values))),
            Expr::Member {
                base,
                field,
                dereference: false,
            } => self.nested_struct_compound_member_source(base, field),
            _ => Ok(None),
        }
    }

    fn nested_struct_compound_member_source<'a>(
        &self,
        base: &'a Expr,
        field: &str,
    ) -> CompileResult<Option<(String, &'a [LocalStructInitializerValue])>> {
        let Some((struct_name, values)) = self.struct_compound_member_source(base)? else {
            return Ok(None);
        };
        let layout = self.struct_layout(&struct_name)?;
        let Some((index, field_type)) =
            layout
                .fields
                .iter()
                .enumerate()
                .find_map(|(index, candidate)| {
                    (candidate.name == field).then_some((index, &candidate.field_type))
                })
        else {
            return Err(CompileError::new(format!(
                "unknown struct field: {struct_name}.{field}"
            )));
        };
        let FieldType::Struct(nested_struct_name) = field_type else {
            return Ok(None);
        };
        match values.get(index) {
            Some(LocalStructInitializerValue::Nested(values)) => {
                Ok(Some((nested_struct_name.clone(), values)))
            }
            Some(LocalStructInitializerValue::Expr(_)) => Err(CompileError::new(
                "compound literal nested member requires nested values",
            )),
            None => Ok(Some((nested_struct_name.clone(), &[]))),
        }
    }
}

fn lower_compound_field_value(
    context: &LoweringContext,
    value: Option<&LocalStructInitializerValue>,
    field_type: &FieldType,
) -> CompileResult<LoweredExpr> {
    match field_type {
        FieldType::Scalar(field) => lower_compound_scalar_value(
            context,
            value,
            field.scalar_type,
            field.byte_size,
            field.is_unsigned,
        ),
        FieldType::Pointer { .. } => lower_compound_scalar_value(
            context,
            value,
            ScalarType::Pointer,
            scalar_size(ScalarType::Pointer),
            false,
        ),
        FieldType::Array { .. } | FieldType::Struct(_) | FieldType::StructArray { .. } => {
            Err(CompileError::new("compound literal member is not scalar"))
        }
    }
}

fn lower_compound_scalar_value(
    context: &LoweringContext,
    value: Option<&LocalStructInitializerValue>,
    scalar_type: ScalarType,
    byte_size: usize,
    is_unsigned: bool,
) -> CompileResult<LoweredExpr> {
    let Some(value) = value else {
        return Ok(zero_expr_for(scalar_type));
    };
    let expr = expr_from_initializer(value)?;
    if scalar_type == ScalarType::Int
        && let Ok(value) = const_eval(expr)
        && let Some(value) = narrow_integer(value, byte_size, is_unsigned)
    {
        return Ok(LoweredExpr::Integer(value));
    }
    if scalar_type == ScalarType::Int && is_unsigned && matches!(byte_size, 1 | 2) {
        return Ok(LoweredExpr::Binary {
            op: crate::parser::BinaryOp::BitAnd,
            left: Box::new(context.lower_expr(expr)?),
            right: Box::new(LoweredExpr::Integer(unsigned_mask(byte_size))),
        });
    }
    context.lower_expr(expr)
}

fn narrow_integer(value: i64, byte_size: usize, is_unsigned: bool) -> Option<i64> {
    let modulus = unsigned_modulus(byte_size)?;
    let narrowed = value.rem_euclid(modulus);
    if is_unsigned || narrowed < modulus / 2 {
        Some(narrowed)
    } else {
        Some(narrowed - modulus)
    }
}

fn unsigned_mask(byte_size: usize) -> i64 {
    unsigned_modulus(byte_size).map_or(0, |modulus| modulus - 1)
}

const fn unsigned_modulus(byte_size: usize) -> Option<i64> {
    match byte_size {
        1 => Some(256),
        2 => Some(65_536),
        _ => None,
    }
}

fn expr_from_initializer(value: &LocalStructInitializerValue) -> CompileResult<&Expr> {
    match value {
        LocalStructInitializerValue::Expr(expr) => Ok(expr),
        LocalStructInitializerValue::Nested(values) if values.len() == 1 => {
            expr_from_initializer(&values[0])
        }
        LocalStructInitializerValue::Nested(_) => Err(CompileError::new(
            "compound literal scalar requires one value",
        )),
    }
}
