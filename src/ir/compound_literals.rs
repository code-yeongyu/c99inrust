use super::{LoweredExpr, LoweringContext, const_eval, zero_expr_for};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::Expr;

impl LoweringContext {
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
        if let Ok(index) = const_eval(index) {
            if index < 0 {
                return Err(CompileError::new("compound literal subscript is negative"));
            }
            let index = usize::try_from(index)
                .map_err(|_| CompileError::new("compound literal subscript is too large"))?;
            if let Some(value) = values.get(index) {
                return self.lower_expr(value).map(Some);
            }
            return Ok(Some(zero_expr_for(*element_type)));
        }
        Ok(Some(LoweredExpr::IndexSelect {
            index: Box::new(self.lower_expr(index)?),
            cases: values
                .iter()
                .map(|value| self.lower_expr(value))
                .collect::<CompileResult<Vec<_>>>()?,
            default: Box::new(zero_expr_for(*element_type)),
        }))
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
