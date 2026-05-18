use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{BinaryOp, Expr, Function, Program, UnaryOp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredProgram {
    pub functions: Vec<LoweredFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredFunction {
    pub name: String,
    pub return_value: i64,
}

pub fn lower(program: &Program) -> CompileResult<LoweredProgram> {
    let mut functions = Vec::with_capacity(program.functions.len());
    for function in &program.functions {
        functions.push(lower_function(function)?);
    }
    Ok(LoweredProgram { functions })
}

pub fn const_eval(expr: &Expr) -> CompileResult<i64> {
    match expr {
        Expr::Integer(value) => Ok(*value),
        Expr::Unary { op, expr } => {
            let value = const_eval(expr)?;
            match op {
                UnaryOp::Plus => Ok(value),
                UnaryOp::Minus => value
                    .checked_neg()
                    .ok_or_else(|| CompileError::new("integer overflow in unary minus")),
                UnaryOp::BitNot => Ok(!value),
                UnaryOp::LogicalNot => Ok(i64::from(value == 0)),
            }
        }
        Expr::Binary { op, left, right } => {
            let left = const_eval(left)?;
            let right = const_eval(right)?;
            eval_binary(*op, left, right)
        }
    }
}

pub fn lower_function(function: &Function) -> CompileResult<LoweredFunction> {
    Ok(LoweredFunction {
        name: function.name.clone(),
        return_value: const_eval(&function.return_expr)?,
    })
}

fn eval_binary(op: BinaryOp, left: i64, right: i64) -> CompileResult<i64> {
    match op {
        BinaryOp::Mul => left
            .checked_mul(right)
            .ok_or_else(|| CompileError::new("integer overflow in multiplication")),
        BinaryOp::Div => {
            if right == 0 {
                return Err(CompileError::new("division by zero"));
            }
            left.checked_div(right)
                .ok_or_else(|| CompileError::new("integer overflow in division"))
        }
        BinaryOp::Mod => {
            if right == 0 {
                return Err(CompileError::new("modulo by zero"));
            }
            left.checked_rem(right)
                .ok_or_else(|| CompileError::new("integer overflow in modulo"))
        }
        BinaryOp::Add => left
            .checked_add(right)
            .ok_or_else(|| CompileError::new("integer overflow in addition")),
        BinaryOp::Sub => left
            .checked_sub(right)
            .ok_or_else(|| CompileError::new("integer overflow in subtraction")),
        BinaryOp::ShiftLeft => shift_count(right).and_then(|count| {
            left.checked_shl(count)
                .ok_or_else(|| CompileError::new("integer overflow in left shift"))
        }),
        BinaryOp::ShiftRight => shift_count(right).and_then(|count| {
            left.checked_shr(count)
                .ok_or_else(|| CompileError::new("integer overflow in right shift"))
        }),
        BinaryOp::BitAnd => Ok(left & right),
        BinaryOp::BitXor => Ok(left ^ right),
        BinaryOp::BitOr => Ok(left | right),
    }
}

fn shift_count(value: i64) -> CompileResult<u32> {
    let count =
        u32::try_from(value).map_err(|_| CompileError::new("shift count must be non-negative"))?;
    if count >= i64::BITS {
        return Err(CompileError::new("shift count is too large"));
    }
    Ok(count)
}
