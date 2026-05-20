use std::collections::HashMap;

use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{BinaryOp, Expr, ScalarType, UnaryOp};

use super::{LoweredGlobalInitializer, cast_const_value, const_eval, eval_binary};

pub(in crate::ir) fn scalar_initializer(
    scalar_type: ScalarType,
    initializer: Option<&Expr>,
    constants: &HashMap<String, i64>,
) -> CompileResult<LoweredGlobalInitializer> {
    let value = initializer.map_or(Ok(0), |expr| eval_with_constants(expr, constants))?;
    match scalar_type {
        ScalarType::Int => Ok(LoweredGlobalInitializer::Int(
            i32::try_from(value)
                .map_err(|_| CompileError::new("static local int initializer does not fit i32"))?,
        )),
        ScalarType::Pointer if value == 0 => Ok(LoweredGlobalInitializer::PointerNull),
        ScalarType::Pointer => Err(CompileError::new(
            "static local pointer initializer must be null",
        )),
        ScalarType::LongLong | ScalarType::Double | ScalarType::VaList => Err(CompileError::new(
            "static local currently supports int and pointer scalars only",
        )),
    }
}

fn eval_with_constants(expr: &Expr, constants: &HashMap<String, i64>) -> CompileResult<i64> {
    match expr {
        Expr::Identifier(name) => constants
            .get(name)
            .copied()
            .ok_or_else(|| CompileError::new(format!("identifier {name} is not a constant"))),
        Expr::Unary { op, expr } => eval_unary(*op, expr, constants),
        Expr::Cast { target, expr, .. } => {
            let value = eval_with_constants(expr, constants)?;
            cast_const_value(*target, value)
        }
        Expr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => eval_conditional(condition, then_expr, else_expr, constants),
        Expr::Binary { op, left, right } => eval_binary_expr(*op, left, right, constants),
        _ => const_eval(expr),
    }
}

fn eval_unary(op: UnaryOp, expr: &Expr, constants: &HashMap<String, i64>) -> CompileResult<i64> {
    let value = eval_with_constants(expr, constants)?;
    match op {
        UnaryOp::Plus => Ok(value),
        UnaryOp::Minus => value
            .checked_neg()
            .ok_or_else(|| CompileError::new("integer overflow in unary minus")),
        UnaryOp::BitNot => Ok(!value),
        UnaryOp::LogicalNot => Ok(i64::from(value == 0)),
    }
}

fn eval_conditional(
    condition: &Expr,
    then_expr: &Expr,
    else_expr: &Expr,
    constants: &HashMap<String, i64>,
) -> CompileResult<i64> {
    if eval_with_constants(condition, constants)? == 0 {
        eval_with_constants(else_expr, constants)
    } else {
        eval_with_constants(then_expr, constants)
    }
}

fn eval_binary_expr(
    op: BinaryOp,
    left: &Expr,
    right: &Expr,
    constants: &HashMap<String, i64>,
) -> CompileResult<i64> {
    let left = eval_with_constants(left, constants)?;
    if op == BinaryOp::LogicalAnd && left == 0 {
        return Ok(0);
    }
    if op == BinaryOp::LogicalOr && left != 0 {
        return Ok(1);
    }
    let right = eval_with_constants(right, constants)?;
    eval_binary(op, left, right)
}
