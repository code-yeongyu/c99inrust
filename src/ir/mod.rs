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
    StoreLocal {
        slot: usize,
        value: LoweredExpr,
    },
    JumpIfZero {
        condition: LoweredExpr,
        label: usize,
    },
    Jump {
        label: usize,
    },
    Label {
        label: usize,
    },
    Return(LoweredExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoweredExpr {
    Call {
        callee: String,
    },
    Integer(i64),
    Local(usize),
    Unary {
        op: UnaryOp,
        expr: Box<Self>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Self>,
        right: Box<Self>,
    },
}

/// Lowers parsed functions into stack-slot IR.
///
/// # Errors
///
/// Returns an error when a function body uses unsupported semantics such as
/// undeclared locals or a missing return.
pub fn lower(program: &Program) -> CompileResult<LoweredProgram> {
    let mut functions = Vec::with_capacity(program.functions.len());
    for function in &program.functions {
        functions.push(lower_function(function)?);
    }
    Ok(LoweredProgram { functions })
}

/// Evaluates a constant integer expression.
///
/// # Errors
///
/// Returns an error when the expression is not constant or overflows the current
/// checked integer model.
pub fn const_eval(expr: &Expr) -> CompileResult<i64> {
    match expr {
        Expr::Call { callee } => Err(CompileError::new(format!(
            "call to {callee} is not a constant expression"
        ))),
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
            if *op == BinaryOp::LogicalAnd && left == 0 {
                return Ok(0);
            }
            if *op == BinaryOp::LogicalOr && left != 0 {
                return Ok(1);
            }
            let right = const_eval(right)?;
            eval_binary(*op, left, right)
        }
    }
}

/// Lowers one parsed function into stack-slot IR.
///
/// # Errors
///
/// Returns an error when lowering detects unsupported semantics such as
/// undeclared locals, duplicate locals in a scope, or a missing return.
pub fn lower_function(function: &Function) -> CompileResult<LoweredFunction> {
    let mut context = LoweringContext::new();
    for statement in &function.statements {
        context.lower_statement(statement)?;
    }
    if !context.has_return {
        return Err(CompileError::new(format!(
            "function {} has no return statement",
            function.name
        )));
    }
    Ok(LoweredFunction {
        name: function.name.clone(),
        local_count: context.local_count,
        instructions: context.instructions,
    })
}

struct LoweringContext {
    scopes: Vec<HashMap<String, usize>>,
    local_count: usize,
    instructions: Vec<Instruction>,
    next_label: usize,
    has_return: bool,
}

impl LoweringContext {
    fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            local_count: 0,
            instructions: Vec::new(),
            next_label: 0,
            has_return: false,
        }
    }

    fn lower_statement(&mut self, statement: &Statement) -> CompileResult<()> {
        match statement {
            Statement::Block(statements) => self.lower_block(statements),
            Statement::Declaration { name, initializer } => {
                let slot = self.declare_local(name)?;
                let value = initializer
                    .as_ref()
                    .map_or(Ok(LoweredExpr::Integer(0)), |expr| self.lower_expr(expr))?;
                self.instructions
                    .push(Instruction::StoreLocal { slot, value });
                Ok(())
            }
            Statement::Assignment { name, value } => {
                let Some(slot) = self.local_slot(name) else {
                    return Err(CompileError::new(format!(
                        "assignment to undeclared local: {name}"
                    )));
                };
                let value = self.lower_expr(value)?;
                self.instructions
                    .push(Instruction::StoreLocal { slot, value });
                Ok(())
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => self.lower_if(condition, then_branch, else_branch.as_deref()),
            Statement::While { condition, body } => self.lower_while(condition, body),
            Statement::For {
                initializer,
                condition,
                post,
                body,
            } => self.lower_for(
                initializer.as_deref(),
                condition.as_ref(),
                post.as_deref(),
                body,
            ),
            Statement::Return(expr) => {
                let value = self.lower_expr(expr)?;
                self.instructions.push(Instruction::Return(value));
                self.has_return = true;
                Ok(())
            }
        }
    }

    fn lower_block(&mut self, statements: &[Statement]) -> CompileResult<()> {
        self.scopes.push(HashMap::new());
        for statement in statements {
            self.lower_statement(statement)?;
        }
        self.pop_scope()
    }

    fn lower_if(
        &mut self,
        condition: &Expr,
        then_branch: &Statement,
        else_branch: Option<&Statement>,
    ) -> CompileResult<()> {
        let else_label = self.fresh_label();
        let end_label = self.fresh_label();
        let condition = self.lower_expr(condition)?;
        self.instructions.push(Instruction::JumpIfZero {
            condition,
            label: else_label,
        });
        self.lower_branch(then_branch)?;
        if else_branch.is_some() {
            self.instructions
                .push(Instruction::Jump { label: end_label });
        }
        self.instructions
            .push(Instruction::Label { label: else_label });
        if let Some(statement) = else_branch {
            self.lower_branch(statement)?;
            self.instructions
                .push(Instruction::Label { label: end_label });
        }
        Ok(())
    }

    fn lower_while(&mut self, condition: &Expr, body: &Statement) -> CompileResult<()> {
        let start_label = self.fresh_label();
        let end_label = self.fresh_label();
        self.instructions
            .push(Instruction::Label { label: start_label });
        let condition = self.lower_expr(condition)?;
        self.instructions.push(Instruction::JumpIfZero {
            condition,
            label: end_label,
        });
        self.lower_branch(body)?;
        self.instructions
            .push(Instruction::Jump { label: start_label });
        self.instructions
            .push(Instruction::Label { label: end_label });
        Ok(())
    }

    fn lower_for(
        &mut self,
        initializer: Option<&Statement>,
        condition: Option<&Expr>,
        post: Option<&Statement>,
        body: &Statement,
    ) -> CompileResult<()> {
        self.scopes.push(HashMap::new());
        if let Some(statement) = initializer {
            self.lower_statement(statement)?;
        }
        let start_label = self.fresh_label();
        let end_label = self.fresh_label();
        self.instructions
            .push(Instruction::Label { label: start_label });
        if let Some(expr) = condition {
            let condition = self.lower_expr(expr)?;
            self.instructions.push(Instruction::JumpIfZero {
                condition,
                label: end_label,
            });
        }
        self.lower_branch(body)?;
        if let Some(statement) = post {
            self.lower_statement(statement)?;
        }
        self.instructions
            .push(Instruction::Jump { label: start_label });
        self.instructions
            .push(Instruction::Label { label: end_label });
        self.pop_scope()
    }

    fn lower_branch(&mut self, statement: &Statement) -> CompileResult<()> {
        self.scopes.push(HashMap::new());
        self.lower_statement(statement)?;
        self.pop_scope()
    }

    fn declare_local(&mut self, name: &str) -> CompileResult<usize> {
        let Some(scope) = self.scopes.last_mut() else {
            return Err(CompileError::new("internal error: no local scope"));
        };
        if scope.contains_key(name) {
            return Err(CompileError::new(format!(
                "duplicate local declaration: {name}"
            )));
        }
        let slot = self.local_count;
        self.local_count += 1;
        scope.insert(name.to_string(), slot);
        Ok(slot)
    }

    fn pop_scope(&mut self) -> CompileResult<()> {
        if self.scopes.pop().is_none() {
            return Err(CompileError::new("internal error: no local scope to pop"));
        }
        Ok(())
    }

    const fn fresh_label(&mut self) -> usize {
        let label = self.next_label;
        self.next_label += 1;
        label
    }

    fn local_slot(&self, name: &str) -> Option<usize> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    fn lower_expr(&self, expr: &Expr) -> CompileResult<LoweredExpr> {
        match expr {
            Expr::Call { callee } => Ok(LoweredExpr::Call {
                callee: callee.clone(),
            }),
            Expr::Identifier(name) => {
                let Some(slot) = self.local_slot(name) else {
                    return Err(CompileError::new(format!("unknown local: {name}")));
                };
                Ok(LoweredExpr::Local(slot))
            }
            Expr::Integer(value) => Ok(LoweredExpr::Integer(*value)),
            Expr::Unary { op, expr } => Ok(LoweredExpr::Unary {
                op: *op,
                expr: Box::new(self.lower_expr(expr)?),
            }),
            Expr::Binary { op, left, right } => Ok(LoweredExpr::Binary {
                op: *op,
                left: Box::new(self.lower_expr(left)?),
                right: Box::new(self.lower_expr(right)?),
            }),
        }
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
        BinaryOp::Less => Ok(i64::from(left < right)),
        BinaryOp::LessEqual => Ok(i64::from(left <= right)),
        BinaryOp::Greater => Ok(i64::from(left > right)),
        BinaryOp::GreaterEqual => Ok(i64::from(left >= right)),
        BinaryOp::Equal => Ok(i64::from(left == right)),
        BinaryOp::NotEqual => Ok(i64::from(left != right)),
        BinaryOp::LogicalAnd => Ok(i64::from(left != 0 && right != 0)),
        BinaryOp::LogicalOr => Ok(i64::from(left != 0 || right != 0)),
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
