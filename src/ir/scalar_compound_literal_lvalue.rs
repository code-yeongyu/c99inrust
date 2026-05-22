use super::{LoweredExpr, LoweringContext};
use crate::diagnostics::CompileResult;
use crate::parser::{BinaryOp, Expr, LValue, ScalarType};

impl LoweringContext {
    pub(in crate::ir) fn lower_scalar_compound_assignment_expr(
        &self,
        target: &LValue,
        value: &Expr,
    ) -> CompileResult<Option<LoweredExpr>> {
        let LValue::ScalarCompoundLiteral {
            scalar_type,
            referent,
            value: initializer,
        } = target
        else {
            return Ok(None);
        };
        Ok(Some(LoweredExpr::Comma {
            left: Box::new(self.lower_scalar_compound_value(
                *scalar_type,
                referent.as_deref(),
                initializer,
            )?),
            right: Box::new(self.lower_scalar_compound_value(
                *scalar_type,
                referent.as_deref(),
                value,
            )?),
        }))
    }

    pub(in crate::ir) fn lower_scalar_compound_post_increment_expr(
        &self,
        target: &LValue,
    ) -> CompileResult<Option<LoweredExpr>> {
        let LValue::ScalarCompoundLiteral {
            scalar_type,
            referent,
            value,
        } = target
        else {
            return Ok(None);
        };
        Ok(Some(self.lower_scalar_compound_value(
            *scalar_type,
            referent.as_deref(),
            value,
        )?))
    }

    fn lower_scalar_compound_value(
        &self,
        scalar_type: ScalarType,
        referent: Option<&str>,
        value: &Expr,
    ) -> CompileResult<LoweredExpr> {
        if scalar_type == ScalarType::Int
            && matches!(referent, Some("byte" | "char" | "unsigned short" | "short"))
        {
            return Ok(scalar_compound_lowered_value(
                scalar_type,
                referent,
                self.lower_expr(value)?,
            ));
        }
        self.lower_cast_expr(scalar_type, value)
    }

    pub(in crate::ir) fn lower_scalar_compound_prefix_increment_expr(
        &self,
        target: &LValue,
        decrement: bool,
    ) -> CompileResult<Option<LoweredExpr>> {
        let LValue::ScalarCompoundLiteral {
            scalar_type,
            referent,
            value,
        } = target
        else {
            return Ok(None);
        };
        let current = self.lower_scalar_compound_value(*scalar_type, referent.as_deref(), value)?;
        let updated = LoweredExpr::Binary {
            op: if decrement {
                BinaryOp::Sub
            } else {
                BinaryOp::Add
            },
            left: Box::new(current),
            right: Box::new(LoweredExpr::Integer(1)),
        };
        Ok(Some(scalar_compound_lowered_value(
            *scalar_type,
            referent.as_deref(),
            updated,
        )))
    }
}

fn scalar_compound_lowered_value(
    scalar_type: ScalarType,
    referent: Option<&str>,
    value: LoweredExpr,
) -> LoweredExpr {
    if scalar_type == ScalarType::Int {
        match referent {
            Some("byte") => return masked_integer(value, 255),
            Some("char") => return signed_narrow_integer(value, 255, 128, 256),
            Some("unsigned short") => return masked_integer(value, 65_535),
            Some("short") => return signed_narrow_integer(value, 65_535, 32_768, 65_536),
            _ => {}
        }
    }
    LoweredExpr::Cast {
        target: scalar_type,
        expr: Box::new(value),
    }
}

fn signed_narrow_integer(expr: LoweredExpr, mask: i64, sign_bit: i64, range: i64) -> LoweredExpr {
    let masked = masked_integer(expr, mask);
    LoweredExpr::Conditional {
        condition: Box::new(LoweredExpr::Binary {
            op: BinaryOp::GreaterEqual,
            left: Box::new(masked.clone()),
            right: Box::new(LoweredExpr::Integer(sign_bit)),
        }),
        then_expr: Box::new(LoweredExpr::Binary {
            op: BinaryOp::Sub,
            left: Box::new(masked.clone()),
            right: Box::new(LoweredExpr::Integer(range)),
        }),
        else_expr: Box::new(masked),
    }
}

fn masked_integer(expr: LoweredExpr, mask: i64) -> LoweredExpr {
    LoweredExpr::Binary {
        op: BinaryOp::BitAnd,
        left: Box::new(expr),
        right: Box::new(LoweredExpr::Integer(mask)),
    }
}
