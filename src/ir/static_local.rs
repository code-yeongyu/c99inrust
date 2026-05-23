use std::collections::HashMap;

use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{BinaryOp, Expr, ScalarType, StructLayout, UnaryOp};

use super::{
    GlobalBinding, LoweredGlobalInitializer, cast_const_value, const_eval, eval_binary, scalar_size,
};

pub(in crate::ir) fn scalar_initializer(
    scalar_type: ScalarType,
    referent: Option<&str>,
    initializer: Option<&Expr>,
    constants: &HashMap<String, i64>,
    structs: &HashMap<String, StructLayout>,
    global_bindings: &HashMap<String, GlobalBinding>,
) -> CompileResult<LoweredGlobalInitializer> {
    match scalar_type {
        ScalarType::Bool | ScalarType::Int | ScalarType::LongLong => {
            integer_scalar_initializer(scalar_type, initializer, constants)
        }
        ScalarType::Pointer => super::static_local_pointer::initializer(
            initializer,
            constants,
            referent,
            structs,
            global_bindings,
        ),
        ScalarType::Double | ScalarType::LongDouble => {
            real_initializer(initializer, constants).map(LoweredGlobalInitializer::Double)
        }
        ScalarType::ComplexFloat | ScalarType::ComplexDouble | ScalarType::ComplexLongDouble => {
            complex_initializer(scalar_type, initializer, constants)
        }
        ScalarType::VaList => Err(CompileError::new("static local does not support va_list")),
    }
}

fn integer_scalar_initializer(
    scalar_type: ScalarType,
    initializer: Option<&Expr>,
    constants: &HashMap<String, i64>,
) -> CompileResult<LoweredGlobalInitializer> {
    let value = initializer.map_or(Ok(0), |expr| eval_with_constants(expr, constants))?;
    match scalar_type {
        ScalarType::Bool => Ok(LoweredGlobalInitializer::Int(i32::from(value != 0))),
        ScalarType::Int => Ok(LoweredGlobalInitializer::Int(
            i32::try_from(value)
                .map_err(|_| CompileError::new("static local int initializer does not fit i32"))?,
        )),
        ScalarType::LongLong => Ok(LoweredGlobalInitializer::LongLong(value)),
        _ => Err(CompileError::new("expected static integer scalar")),
    }
}

fn complex_initializer(
    scalar_type: ScalarType,
    initializer: Option<&Expr>,
    constants: &HashMap<String, i64>,
) -> CompileResult<LoweredGlobalInitializer> {
    initializer.map_or_else(
        || {
            Ok(LoweredGlobalInitializer::ZeroBytes(scalar_size(
                scalar_type,
            )))
        },
        |expr| {
            real_expr(expr, constants).map(|real| LoweredGlobalInitializer::RealThenZero {
                real,
                byte_len: scalar_size(scalar_type),
            })
        },
    )
}

fn real_initializer(
    initializer: Option<&Expr>,
    constants: &HashMap<String, i64>,
) -> CompileResult<String> {
    initializer.map_or_else(|| Ok("0".to_owned()), |expr| real_expr(expr, constants))
}

fn real_expr(expr: &Expr, constants: &HashMap<String, i64>) -> CompileResult<String> {
    match expr {
        Expr::DoubleLiteral(value) => Ok(value.clone()),
        Expr::Integer(value) | Expr::LongInteger(value) => Ok(value.to_string()),
        Expr::Identifier(name) => constants
            .get(name)
            .map(ToString::to_string)
            .ok_or_else(|| CompileError::new(format!("identifier {name} is not a constant"))),
        Expr::Unary {
            op: UnaryOp::Plus,
            expr,
        }
        | Expr::Cast { expr, .. } => real_expr(expr, constants),
        Expr::Unary {
            op: UnaryOp::Minus,
            expr,
        } => real_expr(expr, constants).map(|value| negated_real(&value)),
        _ => eval_with_constants(expr, constants).map(|value| value.to_string()),
    }
}

fn negated_real(value: &str) -> String {
    value
        .strip_prefix('-')
        .map_or_else(|| format!("-{value}"), ToOwned::to_owned)
}

pub(in crate::ir) fn eval_with_constants(
    expr: &Expr,
    constants: &HashMap<String, i64>,
) -> CompileResult<i64> {
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
