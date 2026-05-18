use std::collections::HashMap;

use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{BinaryOp, Expr, Function, Program, Statement, UnaryOp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredProgram {
    pub functions: Vec<LoweredFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredFunction {
    pub name: String,
    pub local_count: usize,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    StoreLocal { slot: usize, value: LoweredExpr },
    Return(LoweredExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoweredExpr {
    Integer(i64),
    Local(usize),
    Unary {
        op: UnaryOp,
        expr: Box<LoweredExpr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<LoweredExpr>,
        right: Box<LoweredExpr>,
    },
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
        Expr::Identifier(name) => Err(CompileError::new(format!(
            "identifier {name} is not a constant expression"
        ))),
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
    let mut locals = HashMap::new();
    let mut instructions = Vec::new();
    let mut has_return = false;
    for statement in &function.statements {
        match statement {
            Statement::Declaration { name, initializer } => {
                if locals.contains_key(name) {
                    return Err(CompileError::new(format!(
                        "duplicate local declaration: {name}"
                    )));
                }
                let slot = locals.len();
                locals.insert(name.clone(), slot);
                let value = initializer
                    .as_ref()
                    .map_or(Ok(LoweredExpr::Integer(0)), |expr| {
                        lower_expr(expr, &locals)
                    })?;
                instructions.push(Instruction::StoreLocal { slot, value });
            }
            Statement::Assignment { name, value } => {
                let Some(slot) = locals.get(name).copied() else {
                    return Err(CompileError::new(format!(
                        "assignment to undeclared local: {name}"
                    )));
                };
                instructions.push(Instruction::StoreLocal {
                    slot,
                    value: lower_expr(value, &locals)?,
                });
            }
            Statement::Return(expr) => {
                instructions.push(Instruction::Return(lower_expr(expr, &locals)?));
                has_return = true;
            }
        }
    }
    if !has_return {
        return Err(CompileError::new(format!(
            "function {} has no return statement",
            function.name
        )));
    }
    Ok(LoweredFunction {
        name: function.name.clone(),
        local_count: locals.len(),
        instructions,
    })
}

fn lower_expr(expr: &Expr, locals: &HashMap<String, usize>) -> CompileResult<LoweredExpr> {
    match expr {
        Expr::Identifier(name) => {
            let Some(slot) = locals.get(name).copied() else {
                return Err(CompileError::new(format!("unknown local: {name}")));
            };
            Ok(LoweredExpr::Local(slot))
        }
        Expr::Integer(value) => Ok(LoweredExpr::Integer(*value)),
        Expr::Unary { op, expr } => Ok(LoweredExpr::Unary {
            op: *op,
            expr: Box::new(lower_expr(expr, locals)?),
        }),
        Expr::Binary { op, left, right } => Ok(LoweredExpr::Binary {
            op: *op,
            left: Box::new(lower_expr(left, locals)?),
            right: Box::new(lower_expr(right, locals)?),
        }),
    }
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
