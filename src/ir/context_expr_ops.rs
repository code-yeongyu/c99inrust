use super::{LoweredExpr, LoweringContext, complex_equality_expr};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{BinaryOp, Expr, LValue, UnaryOp};

impl LoweringContext {
    pub(in crate::ir) fn lower_assignment_expr(
        &self,
        target: &LValue,
        value: &Expr,
    ) -> CompileResult<LoweredExpr> {
        let target = self.lower_lvalue(target)?;
        Ok(LoweredExpr::Assign {
            target,
            value: Box::new(self.lower_expr(value)?),
        })
    }

    pub(in crate::ir) fn lower_conditional_expr(
        &self,
        condition: &Expr,
        then_expr: &Expr,
        else_expr: &Expr,
    ) -> CompileResult<LoweredExpr> {
        Ok(LoweredExpr::Conditional {
            condition: Box::new(self.lower_expr(condition)?),
            then_expr: Box::new(self.lower_expr(then_expr)?),
            else_expr: Box::new(self.lower_expr(else_expr)?),
        })
    }

    pub(in crate::ir) fn lower_binary_expr(
        &self,
        op: BinaryOp,
        left: &Expr,
        right: &Expr,
    ) -> CompileResult<LoweredExpr> {
        let left_referent = self.pointer_referent_for_expr(left).ok();
        let right_referent = self.pointer_referent_for_expr(right).ok();
        if op == BinaryOp::Add {
            if let Some(referent) = left_referent.as_deref()
                && right_referent.is_none()
            {
                return self.lower_pointer_offset_expr(left, right, referent, false);
            }
            if let Some(referent) = right_referent.as_deref()
                && left_referent.is_none()
            {
                return self.lower_pointer_offset_expr(right, left, referent, false);
            }
        }
        if op == BinaryOp::Sub
            && let Some(referent) = left_referent.as_deref()
            && right_referent.is_none()
        {
            return self.lower_pointer_offset_expr(left, right, referent, true);
        }
        if op == BinaryOp::Sub
            && let (Some(left_referent), Some(right_referent)) =
                (left_referent.as_deref(), right_referent.as_deref())
        {
            let byte_size = self.pointer_difference_stride(left_referent, right_referent)?;
            return self.lower_pointer_difference_expr(left, right, byte_size);
        }
        let left = self.lower_expr(left)?;
        let right = self.lower_expr(right)?;
        if let Some(expr) = complex_equality_expr(op, &left, &right) {
            return Ok(expr);
        }
        Ok(LoweredExpr::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    pub(in crate::ir) fn lower_pointer_offset_expr(
        &self,
        pointer: &Expr,
        index: &Expr,
        referent: &str,
        subtract: bool,
    ) -> CompileResult<LoweredExpr> {
        let byte_size = self.pointer_referent_stride(referent)?;
        let index = self.lower_expr(index)?;
        let index = if subtract {
            LoweredExpr::Unary {
                op: UnaryOp::Minus,
                expr: Box::new(index),
            }
        } else {
            index
        };
        Ok(LoweredExpr::PointerOffset {
            pointer: Box::new(self.lower_expr(pointer)?),
            index: Box::new(index),
            byte_size,
        })
    }

    pub(in crate::ir) fn lower_pointer_difference_expr(
        &self,
        left: &Expr,
        right: &Expr,
        byte_size: usize,
    ) -> CompileResult<LoweredExpr> {
        let divisor = i64::try_from(byte_size)
            .map_err(|_| CompileError::new("pointer difference stride does not fit i64"))?;
        Ok(LoweredExpr::Binary {
            op: BinaryOp::Div,
            left: Box::new(LoweredExpr::Binary {
                op: BinaryOp::Sub,
                left: Box::new(self.lower_expr(left)?),
                right: Box::new(self.lower_expr(right)?),
            }),
            right: Box::new(LoweredExpr::Integer(divisor)),
        })
    }

    pub(in crate::ir) fn lower_unary_expr(
        &self,
        op: UnaryOp,
        expr: &Expr,
    ) -> CompileResult<LoweredExpr> {
        if op == UnaryOp::LogicalNot {
            return Ok(LoweredExpr::Binary {
                op: BinaryOp::Equal,
                left: Box::new(self.lower_condition_expr(expr)?),
                right: Box::new(LoweredExpr::Integer(0)),
            });
        }
        Ok(LoweredExpr::Unary {
            op,
            expr: Box::new(self.lower_expr(expr)?),
        })
    }
}
