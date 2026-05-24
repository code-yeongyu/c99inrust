use super::{LoweredExpr, LoweringContext, const_eval, zero_expr_for};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{BinaryOp, Expr};

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
        let lowered_index = self.lower_expr(index)?;
        let mut selected = zero_expr_for(*element_type);
        for (index, value) in values.iter().enumerate().rev() {
            let index = i64::try_from(index)
                .map_err(|_| CompileError::new("compound literal subscript is too large"))?;
            selected = LoweredExpr::Conditional {
                condition: Box::new(LoweredExpr::Binary {
                    op: BinaryOp::Equal,
                    left: Box::new(lowered_index.clone()),
                    right: Box::new(LoweredExpr::Integer(index)),
                }),
                then_expr: Box::new(self.lower_expr(value)?),
                else_expr: Box::new(selected),
            };
        }
        Ok(Some(selected))
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
