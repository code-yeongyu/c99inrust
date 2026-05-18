use std::collections::{HashMap, HashSet};

use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{
    BinaryOp, Constant, Expr, Function, Global, GlobalInitializer, LValue, Program, ReturnType,
    ScalarType, Statement, UnaryOp,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredProgram {
    pub globals: Vec<LoweredGlobal>,
    pub functions: Vec<LoweredFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredGlobal {
    pub name: String,
    pub initializer: LoweredGlobalInitializer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoweredGlobalInitializer {
    Int(i32),
    PointerNull,
    UnsignedCharArray(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredFunction {
    pub name: String,
    pub return_type: ReturnType,
    pub parameter_count: usize,
    pub local_slots: Vec<LocalSlot>,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalSlot {
    pub offset: usize,
    pub scalar_type: ScalarType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    StoreLocal {
        slot: usize,
        offset: usize,
        scalar_type: ScalarType,
        value: LoweredExpr,
    },
    StoreGlobal {
        name: String,
        scalar_type: ScalarType,
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
    Eval(LoweredExpr),
    Return(Option<LoweredExpr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoweredExpr {
    Call {
        callee: String,
        args: Vec<Self>,
    },
    Integer(i64),
    DoubleLiteral(String),
    StringLiteral(String),
    Global {
        name: String,
        scalar_type: ScalarType,
    },
    GlobalByteSubscript {
        name: String,
        index: Box<Self>,
    },
    PointerSubscript {
        pointer: Box<Self>,
        index: Box<Self>,
        element_type: ScalarType,
    },
    Assign {
        target: LoweredLValue,
        value: Box<Self>,
    },
    Local {
        offset: usize,
        scalar_type: ScalarType,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Self>,
    },
    Cast {
        target: ScalarType,
        expr: Box<Self>,
    },
    Conditional {
        condition: Box<Self>,
        then_expr: Box<Self>,
        else_expr: Box<Self>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Self>,
        right: Box<Self>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoweredLValue {
    Local {
        slot: usize,
        offset: usize,
        scalar_type: ScalarType,
    },
    Global {
        name: String,
        scalar_type: ScalarType,
    },
    PointerSubscript {
        pointer: Box<LoweredExpr>,
        index: Box<LoweredExpr>,
        element_type: ScalarType,
    },
}

/// Lowers parsed functions into stack-slot IR.
///
/// # Errors
///
/// Returns an error when a function body uses unsupported semantics such as
/// undeclared locals or a missing `int` return.
pub fn lower(program: &Program) -> CompileResult<LoweredProgram> {
    let (globals, global_bindings) = lower_globals(&program.globals)?;
    let constants = lower_constants(&program.constants)?;
    let mut functions = Vec::with_capacity(program.functions.len());
    for function in &program.functions {
        functions.push(lower_function_with_globals(
            function,
            &global_bindings,
            &constants,
        )?);
    }
    let constant_returns = constant_return_functions(&functions);
    inline_constant_calls(&mut functions, &constant_returns);
    Ok(LoweredProgram { globals, functions })
}

fn lower_globals(
    globals: &[Global],
) -> CompileResult<(Vec<LoweredGlobal>, HashMap<String, GlobalBinding>)> {
    let mut lowered = Vec::with_capacity(globals.len());
    let mut bindings = HashMap::with_capacity(globals.len());
    let mut definitions = HashSet::with_capacity(globals.len());
    for global in globals {
        let (initializer, binding) = match &global.initializer {
            GlobalInitializer::Extern(scalar_type) => {
                insert_global_binding(
                    &mut bindings,
                    &global.name,
                    GlobalBinding::from_scalar_type(*scalar_type)?,
                )?;
                continue;
            }
            GlobalInitializer::Int(value) => (
                LoweredGlobalInitializer::Int(i32::try_from(*value).map_err(|_| {
                    CompileError::new(format!(
                        "global int initializer does not fit i32: {}",
                        global.name
                    ))
                })?),
                GlobalBinding::Int,
            ),
            GlobalInitializer::PointerNull => (
                LoweredGlobalInitializer::PointerNull,
                GlobalBinding::Pointer,
            ),
            GlobalInitializer::UnsignedCharArray(values) => (
                LoweredGlobalInitializer::UnsignedCharArray(values.clone()),
                GlobalBinding::UnsignedCharArray,
            ),
        };
        if !definitions.insert(global.name.clone()) {
            return Err(CompileError::new(format!(
                "duplicate global declaration: {}",
                global.name
            )));
        }
        insert_global_binding(&mut bindings, &global.name, binding)?;
        lowered.push(LoweredGlobal {
            name: global.name.clone(),
            initializer,
        });
    }
    Ok((lowered, bindings))
}

fn insert_global_binding(
    bindings: &mut HashMap<String, GlobalBinding>,
    name: &str,
    binding: GlobalBinding,
) -> CompileResult<()> {
    if let Some(existing) = bindings.get(name)
        && *existing != binding
    {
        return Err(CompileError::new(format!(
            "conflicting global declaration: {name}"
        )));
    }
    bindings.insert(name.to_owned(), binding);
    Ok(())
}

fn lower_constants(constants: &[Constant]) -> CompileResult<HashMap<String, i64>> {
    let mut bindings = HashMap::with_capacity(constants.len());
    for constant in constants {
        if bindings
            .insert(constant.name.clone(), constant.value)
            .is_some()
        {
            return Err(CompileError::new(format!(
                "duplicate constant declaration: {}",
                constant.name
            )));
        }
    }
    Ok(bindings)
}

/// Evaluates a constant integer expression.
///
/// # Errors
///
/// Returns an error when the expression is not constant or overflows the current
/// checked integer model.
pub fn const_eval(expr: &Expr) -> CompileResult<i64> {
    match expr {
        Expr::Call { callee, .. } => Err(CompileError::new(format!(
            "call to {callee} is not a constant expression"
        ))),
        Expr::Identifier(name) => Err(CompileError::new(format!(
            "identifier {name} is not a constant expression"
        ))),
        Expr::Integer(value) => Ok(*value),
        Expr::DoubleLiteral(_) => Err(CompileError::new(
            "double literal is not an integer constant expression",
        )),
        Expr::StringLiteral(_) => Err(CompileError::new(
            "string literal is not an integer constant expression",
        )),
        Expr::Subscript { .. } => Err(CompileError::new(
            "subscript expression is not an integer constant expression",
        )),
        Expr::Assignment { .. } => Err(CompileError::new(
            "assignment expression is not an integer constant expression",
        )),
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
        Expr::Cast { target, expr } => {
            let value = const_eval(expr)?;
            cast_const_value(*target, value)
        }
        Expr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => {
            if const_eval(condition)? == 0 {
                const_eval(else_expr)
            } else {
                const_eval(then_expr)
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
/// undeclared locals, duplicate locals in a scope, or a missing `int` return.
pub fn lower_function(function: &Function) -> CompileResult<LoweredFunction> {
    let global_bindings = HashMap::new();
    let constants = HashMap::new();
    lower_function_with_globals(function, &global_bindings, &constants)
}

fn lower_function_with_globals(
    function: &Function,
    global_bindings: &HashMap<String, GlobalBinding>,
    constants: &HashMap<String, i64>,
) -> CompileResult<LoweredFunction> {
    let mut context = LoweringContext::new(function.return_type, global_bindings, constants);
    for parameter in &function.parameters {
        context.declare_local(&parameter.name, parameter.scalar_type)?;
    }
    for statement in &function.statements {
        context.lower_statement(statement)?;
    }
    if function.return_type == ReturnType::Int && !context.has_return {
        return Err(CompileError::new(format!(
            "function {} has no return statement",
            function.name
        )));
    }
    if function.return_type == ReturnType::Void && !ends_with_return(&context.instructions) {
        context.instructions.push(Instruction::Return(None));
    }
    Ok(LoweredFunction {
        name: function.name.clone(),
        return_type: function.return_type,
        parameter_count: function.parameters.len(),
        local_slots: context.local_slots,
        instructions: context.instructions,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LocalBinding {
    slot: usize,
    scalar_type: ScalarType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GlobalBinding {
    Int,
    Pointer,
    UnsignedCharArray,
}

impl GlobalBinding {
    fn from_scalar_type(scalar_type: ScalarType) -> CompileResult<Self> {
        match scalar_type {
            ScalarType::Int => Ok(Self::Int),
            ScalarType::Pointer => Ok(Self::Pointer),
            ScalarType::LongLong | ScalarType::Double => {
                Err(CompileError::new("unsupported extern global scalar type"))
            }
        }
    }

    const fn scalar_type(self) -> Option<ScalarType> {
        match self {
            Self::Int => Some(ScalarType::Int),
            Self::Pointer => Some(ScalarType::Pointer),
            Self::UnsignedCharArray => None,
        }
    }
}

struct LoweringContext {
    return_type: ReturnType,
    global_bindings: HashMap<String, GlobalBinding>,
    constants: HashMap<String, i64>,
    scopes: Vec<HashMap<String, LocalBinding>>,
    local_slots: Vec<LocalSlot>,
    next_local_offset: usize,
    instructions: Vec<Instruction>,
    next_label: usize,
    has_return: bool,
}

impl LoweringContext {
    fn new(
        return_type: ReturnType,
        global_bindings: &HashMap<String, GlobalBinding>,
        constants: &HashMap<String, i64>,
    ) -> Self {
        Self {
            return_type,
            global_bindings: global_bindings.clone(),
            constants: constants.clone(),
            scopes: vec![HashMap::new()],
            local_slots: Vec::new(),
            next_local_offset: 0,
            instructions: Vec::new(),
            next_label: 0,
            has_return: false,
        }
    }

    fn lower_statement(&mut self, statement: &Statement) -> CompileResult<()> {
        match statement {
            Statement::Block(statements) => self.lower_block(statements),
            Statement::Declaration {
                scalar_type,
                name,
                initializer,
            } => {
                let slot = self.declare_local(name, *scalar_type)?;
                let value = initializer.as_ref().map_or_else(
                    || Ok(zero_expr_for(*scalar_type)),
                    |expr| self.lower_expr(expr),
                )?;
                self.instructions.push(Instruction::StoreLocal {
                    slot,
                    offset: self.local_offset(slot)?,
                    scalar_type: *scalar_type,
                    value,
                });
                Ok(())
            }
            Statement::Assignment { target, value } => {
                let target = self.lower_lvalue(target)?;
                let value = self.lower_expr(value)?;
                self.push_store(target, value);
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
            Statement::Expression(expr) => {
                let expr = self.lower_expr(expr)?;
                self.instructions.push(Instruction::Eval(expr));
                Ok(())
            }
            Statement::Return(expr) => {
                match (self.return_type, expr) {
                    (ReturnType::Int, Some(expr)) => {
                        let value = self.lower_expr(expr)?;
                        self.instructions.push(Instruction::Return(Some(value)));
                    }
                    (ReturnType::Int, None) => {
                        return Err(CompileError::new("int function must return a value"));
                    }
                    (ReturnType::Void, Some(_)) => {
                        return Err(CompileError::new("void function cannot return a value"));
                    }
                    (ReturnType::Void, None) => {
                        self.instructions.push(Instruction::Return(None));
                    }
                }
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

    fn declare_local(&mut self, name: &str, scalar_type: ScalarType) -> CompileResult<usize> {
        let Some(scope) = self.scopes.last_mut() else {
            return Err(CompileError::new("internal error: no local scope"));
        };
        if scope.contains_key(name) {
            return Err(CompileError::new(format!(
                "duplicate local declaration: {name}"
            )));
        }
        let slot = self.local_slots.len();
        let offset = align_to(self.next_local_offset, scalar_size(scalar_type));
        self.next_local_offset = offset + scalar_size(scalar_type);
        self.local_slots.push(LocalSlot {
            offset,
            scalar_type,
        });
        scope.insert(name.to_string(), LocalBinding { slot, scalar_type });
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

    fn local_binding(&self, name: &str) -> Option<LocalBinding> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    fn local_offset(&self, slot: usize) -> CompileResult<usize> {
        self.local_slots
            .get(slot)
            .map(|local_slot| local_slot.offset)
            .ok_or_else(|| CompileError::new("internal error: missing local slot"))
    }

    fn lower_expr(&self, expr: &Expr) -> CompileResult<LoweredExpr> {
        match expr {
            Expr::Call { callee, args } => Ok(LoweredExpr::Call {
                callee: callee.clone(),
                args: args
                    .iter()
                    .map(|arg| self.lower_expr(arg))
                    .collect::<CompileResult<Vec<_>>>()?,
            }),
            Expr::Identifier(name) => {
                if let Some(binding) = self.local_binding(name) {
                    return Ok(LoweredExpr::Local {
                        offset: self.local_offset(binding.slot)?,
                        scalar_type: binding.scalar_type,
                    });
                }
                if let Some(scalar_type) = self
                    .global_bindings
                    .get(name)
                    .and_then(|binding| binding.scalar_type())
                {
                    return Ok(LoweredExpr::Global {
                        name: name.clone(),
                        scalar_type,
                    });
                }
                if self.global_bindings.get(name) == Some(&GlobalBinding::UnsignedCharArray) {
                    return Err(CompileError::new(format!(
                        "global array requires a subscript: {name}"
                    )));
                }
                if let Some(value) = self.constants.get(name) {
                    return Ok(LoweredExpr::Integer(*value));
                }
                Err(CompileError::new(format!(
                    "unknown local or global: {name}"
                )))
            }
            Expr::Integer(value) => Ok(LoweredExpr::Integer(*value)),
            Expr::DoubleLiteral(value) => Ok(LoweredExpr::DoubleLiteral(value.clone())),
            Expr::StringLiteral(value) => Ok(LoweredExpr::StringLiteral(value.clone())),
            Expr::Subscript { array, index } => self.lower_subscript(array, index),
            Expr::Assignment { target, value } => {
                let target = self.lower_lvalue(target)?;
                Ok(LoweredExpr::Assign {
                    target,
                    value: Box::new(self.lower_expr(value)?),
                })
            }
            Expr::Unary { op, expr } => Ok(LoweredExpr::Unary {
                op: *op,
                expr: Box::new(self.lower_expr(expr)?),
            }),
            Expr::Cast { target, expr } => Ok(LoweredExpr::Cast {
                target: *target,
                expr: Box::new(self.lower_expr(expr)?),
            }),
            Expr::Conditional {
                condition,
                then_expr,
                else_expr,
            } => Ok(LoweredExpr::Conditional {
                condition: Box::new(self.lower_expr(condition)?),
                then_expr: Box::new(self.lower_expr(then_expr)?),
                else_expr: Box::new(self.lower_expr(else_expr)?),
            }),
            Expr::Binary { op, left, right } => Ok(LoweredExpr::Binary {
                op: *op,
                left: Box::new(self.lower_expr(left)?),
                right: Box::new(self.lower_expr(right)?),
            }),
        }
    }

    fn lower_lvalue(&self, target: &LValue) -> CompileResult<LoweredLValue> {
        match target {
            LValue::Identifier(name) => {
                if let Some(binding) = self.local_binding(name) {
                    return Ok(LoweredLValue::Local {
                        slot: binding.slot,
                        offset: self.local_offset(binding.slot)?,
                        scalar_type: binding.scalar_type,
                    });
                }
                if let Some(scalar_type) = self
                    .global_bindings
                    .get(name)
                    .and_then(|binding| binding.scalar_type())
                {
                    return Ok(LoweredLValue::Global {
                        name: name.clone(),
                        scalar_type,
                    });
                }
                Err(CompileError::new(format!(
                    "assignment to undeclared local or global: {name}"
                )))
            }
            LValue::Subscript { array, index } => self.lower_subscript_lvalue(array, index),
        }
    }

    fn lower_subscript(&self, array: &Expr, index: &Expr) -> CompileResult<LoweredExpr> {
        if let Expr::Identifier(name) = array
            && self.global_bindings.get(name) == Some(&GlobalBinding::UnsignedCharArray)
        {
            return Ok(LoweredExpr::GlobalByteSubscript {
                name: name.clone(),
                index: Box::new(self.lower_expr(index)?),
            });
        }

        let pointer = self.lower_expr(array)?;
        if lowered_expr_scalar_type(&pointer) != Some(ScalarType::Pointer) {
            return Err(CompileError::new(
                "only pointer and global byte-array subscripts are supported",
            ));
        }
        Ok(LoweredExpr::PointerSubscript {
            pointer: Box::new(pointer),
            index: Box::new(self.lower_expr(index)?),
            element_type: ScalarType::Int,
        })
    }

    fn lower_subscript_lvalue(&self, array: &Expr, index: &Expr) -> CompileResult<LoweredLValue> {
        let pointer = self.lower_expr(array)?;
        if lowered_expr_scalar_type(&pointer) != Some(ScalarType::Pointer) {
            return Err(CompileError::new(
                "assignment to non-pointer subscript targets is not supported",
            ));
        }
        Ok(LoweredLValue::PointerSubscript {
            pointer: Box::new(pointer),
            index: Box::new(self.lower_expr(index)?),
            element_type: ScalarType::Int,
        })
    }

    fn push_store(&mut self, target: LoweredLValue, value: LoweredExpr) {
        match target {
            LoweredLValue::Local {
                slot,
                offset,
                scalar_type,
            } => self.instructions.push(Instruction::StoreLocal {
                slot,
                offset,
                scalar_type,
                value,
            }),
            LoweredLValue::Global { name, scalar_type } => {
                self.instructions.push(Instruction::StoreGlobal {
                    name,
                    scalar_type,
                    value,
                });
            }
            target @ LoweredLValue::PointerSubscript { .. } => {
                self.instructions
                    .push(Instruction::Eval(LoweredExpr::Assign {
                        target,
                        value: Box::new(value),
                    }));
            }
        }
    }
}

const fn lowered_expr_scalar_type(expr: &LoweredExpr) -> Option<ScalarType> {
    match expr {
        LoweredExpr::Global { scalar_type, .. }
        | LoweredExpr::Local { scalar_type, .. }
        | LoweredExpr::Cast {
            target: scalar_type,
            ..
        } => Some(*scalar_type),
        LoweredExpr::PointerSubscript { element_type, .. } => Some(*element_type),
        LoweredExpr::Assign { target, .. } => Some(lowered_lvalue_scalar_type(target)),
        LoweredExpr::Integer(_)
        | LoweredExpr::DoubleLiteral(_)
        | LoweredExpr::StringLiteral(_)
        | LoweredExpr::Call { .. }
        | LoweredExpr::GlobalByteSubscript { .. }
        | LoweredExpr::Unary { .. }
        | LoweredExpr::Conditional { .. }
        | LoweredExpr::Binary { .. } => None,
    }
}

const fn lowered_lvalue_scalar_type(target: &LoweredLValue) -> ScalarType {
    match target {
        LoweredLValue::Local { scalar_type, .. }
        | LoweredLValue::Global { scalar_type, .. }
        | LoweredLValue::PointerSubscript {
            element_type: scalar_type,
            ..
        } => *scalar_type,
    }
}

fn cast_const_value(target: ScalarType, value: i64) -> CompileResult<i64> {
    match target {
        ScalarType::Int => i32::try_from(value)
            .map(i64::from)
            .map_err(|_| CompileError::new("integer cast result does not fit i32")),
        ScalarType::LongLong => Ok(value),
        ScalarType::Double | ScalarType::Pointer => Err(CompileError::new(
            "non-integer cast is not an integer constant expression",
        )),
    }
}

fn zero_expr_for(scalar_type: ScalarType) -> LoweredExpr {
    match scalar_type {
        ScalarType::Double => LoweredExpr::DoubleLiteral("0.0".to_string()),
        ScalarType::Int | ScalarType::LongLong | ScalarType::Pointer => LoweredExpr::Integer(0),
    }
}

const fn scalar_size(scalar_type: ScalarType) -> usize {
    match scalar_type {
        ScalarType::Int => 4,
        ScalarType::LongLong | ScalarType::Double | ScalarType::Pointer => 8,
    }
}

const fn align_to(value: usize, alignment: usize) -> usize {
    let remainder = value % alignment;
    if remainder == 0 {
        value
    } else {
        value + (alignment - remainder)
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

fn constant_return_functions(functions: &[LoweredFunction]) -> HashMap<String, i64> {
    let mut constants = HashMap::new();
    for function in functions {
        if function.local_slots.is_empty()
            && function.return_type == ReturnType::Int
            && let [Instruction::Return(Some(LoweredExpr::Integer(value)))] =
                function.instructions.as_slice()
        {
            constants.insert(function.name.clone(), *value);
        }
    }
    constants
}

fn inline_constant_calls(functions: &mut [LoweredFunction], constants: &HashMap<String, i64>) {
    for function in functions {
        for instruction in &mut function.instructions {
            inline_constant_calls_in_instruction(instruction, constants);
        }
    }
}

fn inline_constant_calls_in_instruction(
    instruction: &mut Instruction,
    constants: &HashMap<String, i64>,
) {
    match instruction {
        Instruction::StoreLocal { value, .. }
        | Instruction::StoreGlobal { value, .. }
        | Instruction::JumpIfZero {
            condition: value, ..
        }
        | Instruction::Eval(value)
        | Instruction::Return(Some(value)) => inline_constant_calls_in_expr(value, constants),
        Instruction::Return(None) | Instruction::Jump { .. } | Instruction::Label { .. } => {}
    }
}

fn inline_constant_calls_in_expr(expr: &mut LoweredExpr, constants: &HashMap<String, i64>) {
    match expr {
        LoweredExpr::Call { callee, args } => {
            if args.is_empty()
                && let Some(value) = constants.get(callee)
            {
                *expr = LoweredExpr::Integer(*value);
            } else {
                for arg in args {
                    inline_constant_calls_in_expr(arg, constants);
                }
            }
        }
        LoweredExpr::Unary { expr, .. } | LoweredExpr::Cast { expr, .. } => {
            inline_constant_calls_in_expr(expr, constants);
        }
        LoweredExpr::GlobalByteSubscript { index, .. } => {
            inline_constant_calls_in_expr(index, constants);
        }
        LoweredExpr::PointerSubscript { pointer, index, .. } => {
            inline_constant_calls_in_expr(pointer, constants);
            inline_constant_calls_in_expr(index, constants);
        }
        LoweredExpr::Assign { value, .. } => {
            inline_constant_calls_in_expr(value, constants);
        }
        LoweredExpr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => {
            inline_constant_calls_in_expr(condition, constants);
            inline_constant_calls_in_expr(then_expr, constants);
            inline_constant_calls_in_expr(else_expr, constants);
        }
        LoweredExpr::Binary { left, right, .. } => {
            inline_constant_calls_in_expr(left, constants);
            inline_constant_calls_in_expr(right, constants);
        }
        LoweredExpr::Integer(_)
        | LoweredExpr::DoubleLiteral(_)
        | LoweredExpr::StringLiteral(_)
        | LoweredExpr::Global { .. }
        | LoweredExpr::Local { .. } => {}
    }
}

const fn ends_with_return(instructions: &[Instruction]) -> bool {
    matches!(instructions.last(), Some(Instruction::Return(_)))
}

fn shift_count(value: i64) -> CompileResult<u32> {
    let count =
        u32::try_from(value).map_err(|_| CompileError::new("shift count must be non-negative"))?;
    if count >= i64::BITS {
        return Err(CompileError::new("shift count is too large"));
    }
    Ok(count)
}
