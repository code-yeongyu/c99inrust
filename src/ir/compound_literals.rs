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
        let Expr::StructCompoundLiteral {
            struct_name,
            values,
        } = base
        else {
            return Ok(None);
        };
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
        lower_compound_field_value(self, values.get(index), field_type).map(Some)
    }

    pub(in crate::ir) fn lower_array_compound_subscript(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredExpr>> {
        let Expr::ArrayCompoundLiteral {
            element_type,
            values,
            ..
        } = array
        else {
            return Ok(None);
        };
        let index = const_eval(index)?;
        if index < 0 {
            return Err(CompileError::new("compound literal subscript is negative"));
        }
        let index = usize::try_from(index)
            .map_err(|_| CompileError::new("compound literal subscript is too large"))?;
        if let Some(value) = values.get(index) {
            return self.lower_expr(value).map(Some);
        }
        Ok(Some(zero_expr_for(*element_type)))
    }

    pub(in crate::ir) fn compound_literal_size(&self, expr: &Expr) -> CompileResult<usize> {
        match expr {
            Expr::StructCompoundLiteral { struct_name, .. } => {
                self.struct_layout(struct_name).map(|layout| layout.size)
            }
            Expr::ArrayCompoundLiteral {
                element_byte_size,
                length,
                ..
            } => length
                .checked_mul(*element_byte_size)
                .ok_or_else(|| CompileError::new("compound literal array size overflow")),
            _ => Err(CompileError::new("expected compound literal")),
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
