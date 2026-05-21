use super::{LoweredExpr, LoweringContext};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::Expr;

impl LoweringContext {
    pub(in crate::ir) fn lower_expr(&self, expr: &Expr) -> CompileResult<LoweredExpr> {
        match expr {
            Expr::Call { callee, args } => self.lower_call_expr(callee, args),
            Expr::IndirectCall { callee, args } => self.lower_indirect_call_expr(callee, args),
            Expr::Identifier(name) => self.lower_identifier_expr(name),
            Expr::Integer(value) => Ok(LoweredExpr::Integer(*value)),
            Expr::LongInteger(value) => Ok(LoweredExpr::LongInteger(*value)),
            Expr::DoubleLiteral(value) => Ok(LoweredExpr::DoubleLiteral(value.clone())),
            Expr::StringLiteral(value) => Ok(LoweredExpr::StringLiteral(value.clone())),
            Expr::StructCompoundLiteral { .. } | Expr::ArrayCompoundLiteral { .. } => Err(
                CompileError::new("compound literal object cannot be used as a scalar value"),
            ),
            Expr::ScalarCompoundLiteral {
                scalar_type, value, ..
            } => self.lower_cast_expr(*scalar_type, value),
            Expr::VaArg {
                list, scalar_type, ..
            } => Ok(LoweredExpr::VaArg {
                list: Box::new(self.lower_expr(list)?),
                scalar_type: *scalar_type,
            }),
            Expr::Member {
                base,
                field,
                dereference,
            } => self.lower_member_expr(base, field, *dereference),
            Expr::SizeOfExpr { expr } => self.lower_sizeof_expr(expr),
            Expr::Dereference { pointer } => self.lower_subscript(pointer, &Expr::Integer(0)),
            Expr::AddressOf { target } => self.lower_address_of(target),
            Expr::Subscript { array, index } => self.lower_subscript(array, index),
            Expr::Assignment { target, value } => self.lower_assignment_expr(target, value),
            Expr::PostIncrement { target, decrement } => {
                self.lower_post_increment_expr(target, *decrement)
            }
            Expr::Unary { op, expr } => self.lower_unary_expr(*op, expr),
            Expr::Cast { target, expr, .. } => self.lower_cast_expr(*target, expr),
            Expr::Conditional {
                condition,
                then_expr,
                else_expr,
            } => self.lower_conditional_expr(condition, then_expr, else_expr),
            Expr::Comma { left, right } => Ok(LoweredExpr::Comma {
                left: Box::new(self.lower_expr(left)?),
                right: Box::new(self.lower_expr(right)?),
            }),
            Expr::Binary { op, left, right } => self.lower_binary_expr(*op, left, right),
        }
    }
}
