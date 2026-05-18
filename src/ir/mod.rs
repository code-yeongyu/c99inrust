use std::collections::{HashMap, HashSet};

use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{
    BinaryOp, Constant, Expr, FieldType, Function, Global, GlobalInitializer, LValue, Program,
    ReturnType, ScalarType, Statement, StructLayout, SwitchCase, UnaryOp,
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
    IntArray(Vec<i32>),
    PointerNull,
    PointerArray(usize),
    ZeroBytes(usize),
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
    pub byte_size: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    StoreLocal {
        slot: usize,
        offset: usize,
        scalar_type: ScalarType,
        value: LoweredExpr,
    },
    InitLocalBytes {
        offset: usize,
        values: Vec<u8>,
    },
    InitLocalInts {
        offset: usize,
        values: Vec<i32>,
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
    GlobalIntSubscript {
        name: String,
        index: Box<Self>,
    },
    GlobalPointerSubscript {
        name: String,
        index: Box<Self>,
    },
    GlobalAddress {
        name: String,
    },
    PointerSubscript {
        pointer: Box<Self>,
        index: Box<Self>,
        element_type: ScalarType,
    },
    PointerOffset {
        pointer: Box<Self>,
        index: Box<Self>,
        byte_size: usize,
    },
    PointerField {
        pointer: Box<Self>,
        offset: usize,
        scalar_type: ScalarType,
    },
    PointerFieldAddress {
        pointer: Box<Self>,
        offset: usize,
    },
    Assign {
        target: LoweredLValue,
        value: Box<Self>,
    },
    PostIncrement {
        target: LoweredLValue,
    },
    Local {
        offset: usize,
        scalar_type: ScalarType,
    },
    LocalAddress {
        offset: usize,
        byte_size: usize,
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
    GlobalByteSubscript {
        name: String,
        index: Box<LoweredExpr>,
    },
    GlobalIntSubscript {
        name: String,
        index: Box<LoweredExpr>,
    },
    GlobalPointerSubscript {
        name: String,
        index: Box<LoweredExpr>,
    },
    PointerSubscript {
        pointer: Box<LoweredExpr>,
        index: Box<LoweredExpr>,
        element_type: ScalarType,
    },
    PointerField {
        pointer: Box<LoweredExpr>,
        offset: usize,
        scalar_type: ScalarType,
    },
}

/// Lowers parsed functions into stack-slot IR.
///
/// # Errors
///
/// Returns an error when a function body uses unsupported semantics such as
/// undeclared locals or a missing `int` return.
pub fn lower(program: &Program) -> CompileResult<LoweredProgram> {
    let structs = program
        .structs
        .iter()
        .map(|layout| (layout.name.clone(), layout.clone()))
        .collect::<HashMap<_, _>>();
    let constants = lower_constants(&program.constants)?;
    let (globals, global_bindings) = lower_globals(&program.globals, &constants, &structs)?;
    let mut functions = Vec::with_capacity(program.functions.len());
    for function in &program.functions {
        functions.push(lower_function_with_globals(
            function,
            &structs,
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
    constants: &HashMap<String, i64>,
    structs: &HashMap<String, StructLayout>,
) -> CompileResult<(Vec<LoweredGlobal>, HashMap<String, GlobalBinding>)> {
    let mut lowered = Vec::with_capacity(globals.len());
    let mut bindings = HashMap::with_capacity(globals.len());
    let mut definitions = HashSet::with_capacity(globals.len());
    for global in globals {
        if let Some(binding) = lower_extern_global_binding(&global.initializer, structs)? {
            insert_global_binding(&mut bindings, &global.name, binding)?;
            continue;
        }
        let (initializer, binding) = lower_defined_global_initializer(global, constants, structs)?;
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
    insert_standard_stream_bindings(&mut bindings);
    Ok((lowered, bindings))
}

fn lower_extern_global_binding(
    initializer: &GlobalInitializer,
    structs: &HashMap<String, StructLayout>,
) -> CompileResult<Option<GlobalBinding>> {
    let binding = match initializer {
        GlobalInitializer::Extern(scalar_type) => GlobalBinding::from_scalar_type(*scalar_type)?,
        GlobalInitializer::ExternPointer { referent } => GlobalBinding::Pointer {
            referent: referent.clone(),
        },
        GlobalInitializer::ExternIntArray => GlobalBinding::IntArray,
        GlobalInitializer::ExternPointerArray => GlobalBinding::PointerArray,
        GlobalInitializer::ExternStructArray { struct_name } => {
            let layout = structs.get(struct_name).ok_or_else(|| {
                CompileError::new(format!("unknown struct-array type: {struct_name}"))
            })?;
            GlobalBinding::StructArray {
                struct_name: struct_name.clone(),
                byte_size: layout.size,
                length: None,
            }
        }
        GlobalInitializer::Int(_)
        | GlobalInitializer::IntArray(_)
        | GlobalInitializer::IntConstant(_)
        | GlobalInitializer::PointerNull { .. }
        | GlobalInitializer::PointerArray(_)
        | GlobalInitializer::StructObject { .. }
        | GlobalInitializer::StructArray { .. }
        | GlobalInitializer::UnsignedCharArray(_) => return Ok(None),
    };
    Ok(Some(binding))
}

fn lower_defined_global_initializer(
    global: &Global,
    constants: &HashMap<String, i64>,
    structs: &HashMap<String, StructLayout>,
) -> CompileResult<(LoweredGlobalInitializer, GlobalBinding)> {
    match &global.initializer {
        GlobalInitializer::Int(value) => Ok((
            LoweredGlobalInitializer::Int(i32::try_from(*value).map_err(|_| {
                CompileError::new(format!(
                    "global int initializer does not fit i32: {}",
                    global.name
                ))
            })?),
            GlobalBinding::Int,
        )),
        GlobalInitializer::IntArray(values) => Ok((
            LoweredGlobalInitializer::IntArray(values.clone()),
            GlobalBinding::IntArray,
        )),
        GlobalInitializer::IntConstant(name) => {
            lower_int_constant_global(name, &global.name, constants)
        }
        GlobalInitializer::PointerNull { referent } => Ok((
            LoweredGlobalInitializer::PointerNull,
            GlobalBinding::Pointer {
                referent: referent.clone(),
            },
        )),
        GlobalInitializer::PointerArray(length) => Ok((
            LoweredGlobalInitializer::PointerArray(*length),
            GlobalBinding::PointerArray,
        )),
        GlobalInitializer::StructObject { struct_name } => {
            lower_struct_object_global(struct_name, structs)
        }
        GlobalInitializer::StructArray {
            struct_name,
            length,
        } => lower_struct_array_global(struct_name, *length, structs),
        GlobalInitializer::UnsignedCharArray(values) => Ok((
            LoweredGlobalInitializer::UnsignedCharArray(values.clone()),
            GlobalBinding::UnsignedCharArray,
        )),
        GlobalInitializer::Extern(_)
        | GlobalInitializer::ExternPointer { .. }
        | GlobalInitializer::ExternIntArray
        | GlobalInitializer::ExternPointerArray
        | GlobalInitializer::ExternStructArray { .. } => Err(CompileError::new(
            "internal error: extern global reached definition lowering",
        )),
    }
}

fn lower_int_constant_global(
    name: &str,
    global_name: &str,
    constants: &HashMap<String, i64>,
) -> CompileResult<(LoweredGlobalInitializer, GlobalBinding)> {
    let Some(value) = constants.get(name) else {
        return Err(CompileError::new(format!(
            "unknown global initializer constant: {name}"
        )));
    };
    Ok((
        LoweredGlobalInitializer::Int(i32::try_from(*value).map_err(|_| {
            CompileError::new(format!(
                "global int initializer does not fit i32: {global_name}"
            ))
        })?),
        GlobalBinding::Int,
    ))
}

fn lower_struct_object_global(
    struct_name: &str,
    structs: &HashMap<String, StructLayout>,
) -> CompileResult<(LoweredGlobalInitializer, GlobalBinding)> {
    let layout = structs
        .get(struct_name)
        .ok_or_else(|| CompileError::new(format!("unknown struct object type: {struct_name}")))?;
    Ok((
        LoweredGlobalInitializer::ZeroBytes(layout.size),
        GlobalBinding::StructObject {
            struct_name: struct_name.to_owned(),
            byte_size: layout.size,
        },
    ))
}

fn lower_struct_array_global(
    struct_name: &str,
    length: usize,
    structs: &HashMap<String, StructLayout>,
) -> CompileResult<(LoweredGlobalInitializer, GlobalBinding)> {
    let layout = structs
        .get(struct_name)
        .ok_or_else(|| CompileError::new(format!("unknown struct-array type: {struct_name}")))?;
    let byte_len = length
        .checked_mul(layout.size)
        .ok_or_else(|| CompileError::new("global struct-array size overflow"))?;
    Ok((
        LoweredGlobalInitializer::ZeroBytes(byte_len),
        GlobalBinding::StructArray {
            struct_name: struct_name.to_owned(),
            byte_size: layout.size,
            length: Some(length),
        },
    ))
}

fn insert_standard_stream_bindings(bindings: &mut HashMap<String, GlobalBinding>) {
    for name in ["stdin", "stdout", "stderr"] {
        bindings
            .entry(name.to_owned())
            .or_insert(GlobalBinding::Pointer { referent: None });
    }
}

fn insert_global_binding(
    bindings: &mut HashMap<String, GlobalBinding>,
    name: &str,
    binding: GlobalBinding,
) -> CompileResult<()> {
    if let Some(existing) = bindings.get(name) {
        let Some(merged) = merge_global_binding(existing, &binding) else {
            return Err(CompileError::new(format!(
                "conflicting global declaration: {name}"
            )));
        };
        bindings.insert(name.to_owned(), merged);
        return Ok(());
    }
    bindings.insert(name.to_owned(), binding);
    Ok(())
}

fn merge_global_binding(
    existing: &GlobalBinding,
    incoming: &GlobalBinding,
) -> Option<GlobalBinding> {
    if existing == incoming {
        return Some(existing.clone());
    }
    match (existing, incoming) {
        (
            GlobalBinding::StructArray {
                struct_name,
                byte_size,
                length,
            },
            GlobalBinding::StructArray {
                struct_name: incoming_name,
                byte_size: incoming_byte_size,
                length: incoming_length,
            },
        ) if struct_name == incoming_name && byte_size == incoming_byte_size => {
            let merged_length = match (*length, *incoming_length) {
                (Some(existing_length), Some(new_length)) if existing_length == new_length => {
                    Some(existing_length)
                }
                (Some(existing_length), None) | (None, Some(existing_length)) => {
                    Some(existing_length)
                }
                (None, None) => None,
                (Some(_), Some(_)) => return None,
            };
            Some(GlobalBinding::StructArray {
                struct_name: struct_name.clone(),
                byte_size: *byte_size,
                length: merged_length,
            })
        }
        _ => None,
    }
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
        Expr::SizeOfExpr { .. } => Err(CompileError::new(
            "sizeof expression is not an integer constant expression",
        )),
        Expr::Subscript { .. } => Err(CompileError::new(
            "subscript expression is not an integer constant expression",
        )),
        Expr::Dereference { .. } => Err(CompileError::new(
            "dereference expression is not an integer constant expression",
        )),
        Expr::AddressOf { .. } => Err(CompileError::new(
            "address expression is not an integer constant expression",
        )),
        Expr::Member { .. } => Err(CompileError::new(
            "member expression is not an integer constant expression",
        )),
        Expr::Assignment { .. } => Err(CompileError::new(
            "assignment expression is not an integer constant expression",
        )),
        Expr::PostIncrement { .. } => Err(CompileError::new(
            "post-increment expression is not an integer constant expression",
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
        Expr::Cast { target, expr, .. } => {
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
    let structs = HashMap::new();
    let global_bindings = HashMap::new();
    let constants = HashMap::new();
    lower_function_with_globals(function, &structs, &global_bindings, &constants)
}

fn lower_function_with_globals(
    function: &Function,
    structs: &HashMap<String, StructLayout>,
    global_bindings: &HashMap<String, GlobalBinding>,
    constants: &HashMap<String, i64>,
) -> CompileResult<LoweredFunction> {
    let mut context =
        LoweringContext::new(function.return_type, structs, global_bindings, constants);
    for parameter in &function.parameters {
        context.declare_local(
            &parameter.name,
            parameter.scalar_type,
            parameter.referent.clone(),
        )?;
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum LocalBinding {
    Scalar {
        slot: usize,
        scalar_type: ScalarType,
        referent: Option<String>,
    },
    CharArray {
        slot: usize,
        length: usize,
    },
    IntArray {
        slot: usize,
        length: usize,
    },
    StructObject {
        slot: usize,
        struct_name: String,
        byte_size: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GlobalBinding {
    Int,
    IntArray,
    Pointer {
        referent: Option<String>,
    },
    PointerArray,
    StructObject {
        struct_name: String,
        byte_size: usize,
    },
    StructArray {
        struct_name: String,
        byte_size: usize,
        length: Option<usize>,
    },
    UnsignedCharArray,
}

impl GlobalBinding {
    fn from_scalar_type(scalar_type: ScalarType) -> CompileResult<Self> {
        match scalar_type {
            ScalarType::Int => Ok(Self::Int),
            ScalarType::Pointer => Ok(Self::Pointer { referent: None }),
            ScalarType::LongLong | ScalarType::Double => {
                Err(CompileError::new("unsupported extern global scalar type"))
            }
        }
    }

    const fn scalar_type(&self) -> Option<ScalarType> {
        match self {
            Self::Int => Some(ScalarType::Int),
            Self::Pointer { .. } => Some(ScalarType::Pointer),
            Self::IntArray
            | Self::PointerArray
            | Self::StructObject { .. }
            | Self::StructArray { .. }
            | Self::UnsignedCharArray => None,
        }
    }

    const fn is_addressable_array(&self) -> bool {
        matches!(
            self,
            Self::IntArray
                | Self::PointerArray
                | Self::StructArray { .. }
                | Self::UnsignedCharArray
        )
    }
}

struct ResolvedMember {
    pointer: LoweredExpr,
    offset: usize,
    field_type: FieldType,
}

struct StructAddress {
    pointer: LoweredExpr,
    offset: usize,
    struct_name: String,
}

struct LoweringContext {
    return_type: ReturnType,
    structs: HashMap<String, StructLayout>,
    global_bindings: HashMap<String, GlobalBinding>,
    constants: HashMap<String, i64>,
    scopes: Vec<HashMap<String, LocalBinding>>,
    local_slots: Vec<LocalSlot>,
    next_local_offset: usize,
    instructions: Vec<Instruction>,
    next_label: usize,
    break_labels: Vec<usize>,
    continue_labels: Vec<usize>,
    has_return: bool,
}

impl LoweringContext {
    fn new(
        return_type: ReturnType,
        structs: &HashMap<String, StructLayout>,
        global_bindings: &HashMap<String, GlobalBinding>,
        constants: &HashMap<String, i64>,
    ) -> Self {
        Self {
            return_type,
            structs: structs.clone(),
            global_bindings: global_bindings.clone(),
            constants: constants.clone(),
            scopes: vec![HashMap::new()],
            local_slots: Vec::new(),
            next_local_offset: 0,
            instructions: Vec::new(),
            next_label: 0,
            break_labels: Vec::new(),
            continue_labels: Vec::new(),
            has_return: false,
        }
    }

    fn lower_statement(&mut self, statement: &Statement) -> CompileResult<()> {
        match statement {
            Statement::Empty => Ok(()),
            Statement::Block(statements) => self.lower_block(statements),
            Statement::Declaration {
                scalar_type,
                name,
                referent,
                initializer,
            } => self.lower_declaration(*scalar_type, name, referent.clone(), initializer.as_ref()),
            Statement::LocalCharArray {
                name,
                length,
                initializer,
            } => self.lower_local_char_array(name, *length, initializer.as_deref()),
            Statement::LocalIntArray {
                name,
                length,
                initializer,
            } => self.lower_local_int_array(name, *length, initializer.as_deref()),
            Statement::LocalStruct { name, struct_name } => {
                self.lower_local_struct_object(name, struct_name)
            }
            Statement::LocalConstants(constants) => {
                for constant in constants {
                    self.constants.insert(constant.name.clone(), constant.value);
                }
                Ok(())
            }
            Statement::DeclarationList(declarations) => {
                for declaration in declarations {
                    self.lower_statement(declaration)?;
                }
                Ok(())
            }
            Statement::Assignment { target, value } => self.lower_assignment(target, value),
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => self.lower_if(condition, then_branch, else_branch.as_deref()),
            Statement::While { condition, body } => self.lower_while(condition, body),
            Statement::DoWhile { body, condition } => self.lower_do_while(body, condition),
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
            Statement::Switch {
                condition,
                cases,
                default,
            } => self.lower_switch(condition, cases, default),
            Statement::Expression(Expr::PostIncrement { target }) => {
                self.lower_post_increment_statement(target)
            }
            Statement::Expression(expr) => {
                let expr = self.lower_expr(expr)?;
                self.instructions.push(Instruction::Eval(expr));
                Ok(())
            }
            Statement::Break => {
                let Some(label) = self.break_labels.last() else {
                    return Err(CompileError::new("break statement outside loop"));
                };
                self.instructions.push(Instruction::Jump { label: *label });
                Ok(())
            }
            Statement::Continue => {
                let Some(label) = self.continue_labels.last() else {
                    return Err(CompileError::new("continue statement outside loop"));
                };
                self.instructions.push(Instruction::Jump { label: *label });
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

    fn lower_declaration(
        &mut self,
        scalar_type: ScalarType,
        name: &str,
        referent: Option<String>,
        initializer: Option<&Expr>,
    ) -> CompileResult<()> {
        let slot = self.declare_local(name, scalar_type, referent)?;
        let value = initializer.map_or_else(
            || Ok(zero_expr_for(scalar_type)),
            |expr| self.lower_expr(expr),
        )?;
        self.instructions.push(Instruction::StoreLocal {
            slot,
            offset: self.local_offset(slot)?,
            scalar_type,
            value,
        });
        Ok(())
    }

    fn lower_local_char_array(
        &mut self,
        name: &str,
        length: usize,
        initializer: Option<&str>,
    ) -> CompileResult<()> {
        let slot = self.declare_char_array(name, length)?;
        if let Some(value) = initializer {
            self.instructions.push(Instruction::InitLocalBytes {
                offset: self.local_offset(slot)?,
                values: local_char_array_initializer_values(value, length)?,
            });
        }
        Ok(())
    }

    fn lower_local_int_array(
        &mut self,
        name: &str,
        length: usize,
        initializer: Option<&[i32]>,
    ) -> CompileResult<()> {
        let slot = self.declare_int_array(name, length)?;
        if let Some(values) = initializer {
            self.instructions.push(Instruction::InitLocalInts {
                offset: self.local_offset(slot)?,
                values: values.to_vec(),
            });
        }
        Ok(())
    }

    fn lower_local_struct_object(&mut self, name: &str, struct_name: &str) -> CompileResult<()> {
        self.declare_struct_object(name, struct_name).map(|_| ())
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
        self.break_labels.push(end_label);
        self.continue_labels.push(start_label);
        let result = self.lower_branch(body);
        self.continue_labels.pop();
        self.break_labels.pop();
        result?;
        self.instructions
            .push(Instruction::Jump { label: start_label });
        self.instructions
            .push(Instruction::Label { label: end_label });
        Ok(())
    }

    fn lower_do_while(&mut self, body: &Statement, condition: &Expr) -> CompileResult<()> {
        let start_label = self.fresh_label();
        let continue_label = self.fresh_label();
        let end_label = self.fresh_label();
        self.instructions
            .push(Instruction::Label { label: start_label });
        self.break_labels.push(end_label);
        self.continue_labels.push(continue_label);
        let result = self.lower_branch(body);
        self.continue_labels.pop();
        self.break_labels.pop();
        result?;
        self.instructions.push(Instruction::Label {
            label: continue_label,
        });
        let condition = self.lower_expr(condition)?;
        self.instructions.push(Instruction::JumpIfZero {
            condition,
            label: end_label,
        });
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
        let continue_label = self.fresh_label();
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
        self.break_labels.push(end_label);
        self.continue_labels.push(continue_label);
        let result = self.lower_branch(body);
        self.continue_labels.pop();
        self.break_labels.pop();
        result?;
        self.instructions.push(Instruction::Label {
            label: continue_label,
        });
        if let Some(statement) = post {
            self.lower_statement(statement)?;
        }
        self.instructions
            .push(Instruction::Jump { label: start_label });
        self.instructions
            .push(Instruction::Label { label: end_label });
        self.pop_scope()
    }

    fn lower_switch(
        &mut self,
        condition: &Expr,
        cases: &[SwitchCase],
        default: &[Statement],
    ) -> CompileResult<()> {
        self.scopes.push(HashMap::new());
        let end_label = self.fresh_label();
        let default_label = (!default.is_empty()).then(|| self.fresh_label());
        let case_labels = (0..cases.len())
            .map(|_| self.fresh_label())
            .collect::<Vec<_>>();
        for (case, label) in cases.iter().zip(case_labels.iter().copied()) {
            let next_label = self.fresh_label();
            self.instructions.push(Instruction::JumpIfZero {
                condition: LoweredExpr::Binary {
                    op: BinaryOp::Equal,
                    left: Box::new(self.lower_expr(condition)?),
                    right: Box::new(self.lower_expr(&case.value)?),
                },
                label: next_label,
            });
            self.instructions.push(Instruction::Jump { label });
            self.instructions
                .push(Instruction::Label { label: next_label });
        }
        self.instructions.push(Instruction::Jump {
            label: default_label.unwrap_or(end_label),
        });
        self.break_labels.push(end_label);
        for (case, label) in cases.iter().zip(case_labels.iter().copied()) {
            self.instructions.push(Instruction::Label { label });
            for statement in &case.statements {
                self.lower_statement(statement)?;
            }
        }
        if let Some(label) = default_label {
            self.instructions.push(Instruction::Label { label });
            for statement in default {
                self.lower_statement(statement)?;
            }
        }
        self.break_labels.pop();
        self.instructions
            .push(Instruction::Label { label: end_label });
        self.pop_scope()
    }

    fn lower_branch(&mut self, statement: &Statement) -> CompileResult<()> {
        self.scopes.push(HashMap::new());
        self.lower_statement(statement)?;
        self.pop_scope()
    }

    fn declare_local(
        &mut self,
        name: &str,
        scalar_type: ScalarType,
        referent: Option<String>,
    ) -> CompileResult<usize> {
        self.declare_slot(
            name,
            scalar_type,
            scalar_size(scalar_type),
            scalar_size(scalar_type),
            LocalBinding::Scalar {
                slot: self.local_slots.len(),
                scalar_type,
                referent,
            },
        )
    }

    fn declare_char_array(&mut self, name: &str, length: usize) -> CompileResult<usize> {
        self.declare_slot(
            name,
            ScalarType::Int,
            length,
            1,
            LocalBinding::CharArray {
                slot: self.local_slots.len(),
                length,
            },
        )
    }

    fn declare_int_array(&mut self, name: &str, length: usize) -> CompileResult<usize> {
        let byte_size = local_int_array_byte_size(length)?;
        self.declare_slot(
            name,
            ScalarType::Int,
            byte_size,
            scalar_size(ScalarType::Int),
            LocalBinding::IntArray {
                slot: self.local_slots.len(),
                length,
            },
        )
    }

    fn declare_struct_object(&mut self, name: &str, struct_name: &str) -> CompileResult<usize> {
        let layout = self.struct_layout(struct_name)?.clone();
        self.declare_slot(
            name,
            ScalarType::Pointer,
            layout.size,
            struct_alignment(&layout),
            LocalBinding::StructObject {
                slot: self.local_slots.len(),
                struct_name: struct_name.to_owned(),
                byte_size: layout.size,
            },
        )
    }

    fn declare_slot(
        &mut self,
        name: &str,
        scalar_type: ScalarType,
        byte_size: usize,
        alignment: usize,
        binding: LocalBinding,
    ) -> CompileResult<usize> {
        let Some(scope) = self.scopes.last() else {
            return Err(CompileError::new("internal error: no local scope"));
        };
        if scope.contains_key(name) {
            return Err(CompileError::new(format!(
                "duplicate local declaration: {name}"
            )));
        }
        let slot = self.local_slots.len();
        let offset = align_to(self.next_local_offset, alignment);
        self.next_local_offset = offset + byte_size;
        self.local_slots.push(LocalSlot {
            offset,
            scalar_type,
            byte_size,
        });
        let Some(scope) = self.scopes.last_mut() else {
            return Err(CompileError::new("internal error: no local scope"));
        };
        scope.insert(name.to_string(), binding);
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
            .find_map(|scope| scope.get(name).cloned())
    }

    fn local_offset(&self, slot: usize) -> CompileResult<usize> {
        self.local_slots
            .get(slot)
            .map(|local_slot| local_slot.offset)
            .ok_or_else(|| CompileError::new("internal error: missing local slot"))
    }

    fn struct_layout(&self, struct_name: &str) -> CompileResult<&StructLayout> {
        self.structs
            .get(struct_name)
            .ok_or_else(|| CompileError::new(format!("unknown struct: {struct_name}")))
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
                    return match binding {
                        LocalBinding::Scalar {
                            slot, scalar_type, ..
                        } => Ok(LoweredExpr::Local {
                            offset: self.local_offset(slot)?,
                            scalar_type,
                        }),
                        LocalBinding::CharArray { slot, length } => Ok(LoweredExpr::LocalAddress {
                            offset: self.local_offset(slot)?,
                            byte_size: length,
                        }),
                        LocalBinding::IntArray { slot, length } => Ok(LoweredExpr::LocalAddress {
                            offset: self.local_offset(slot)?,
                            byte_size: local_int_array_byte_size(length)?,
                        }),
                        LocalBinding::StructObject {
                            slot, byte_size, ..
                        } => Ok(LoweredExpr::LocalAddress {
                            offset: self.local_offset(slot)?,
                            byte_size,
                        }),
                    };
                }
                if let Some(scalar_type) = self
                    .global_bindings
                    .get(name)
                    .and_then(GlobalBinding::scalar_type)
                {
                    return Ok(LoweredExpr::Global {
                        name: name.clone(),
                        scalar_type,
                    });
                }
                if self
                    .global_bindings
                    .get(name)
                    .is_some_and(GlobalBinding::is_addressable_array)
                {
                    return Ok(LoweredExpr::GlobalAddress { name: name.clone() });
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
            Expr::Member {
                base,
                field,
                dereference,
            } => self.lower_member_expr(base, field, *dereference),
            Expr::SizeOfExpr { expr } => self.lower_sizeof_expr(expr),
            Expr::Dereference { pointer } => self.lower_subscript(pointer, &Expr::Integer(0)),
            Expr::AddressOf { target } => self.lower_address_of(target),
            Expr::Subscript { array, index } => self.lower_subscript(array, index),
            Expr::Assignment { target, value } => {
                let target = self.lower_lvalue(target)?;
                Ok(LoweredExpr::Assign {
                    target,
                    value: Box::new(self.lower_expr(value)?),
                })
            }
            Expr::PostIncrement { target } => self.lower_post_increment_expr(target),
            Expr::Unary { op, expr } => Ok(LoweredExpr::Unary {
                op: *op,
                expr: Box::new(self.lower_expr(expr)?),
            }),
            Expr::Cast { target, expr, .. } => Ok(LoweredExpr::Cast {
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

    fn lower_assignment(&mut self, target: &LValue, value: &Expr) -> CompileResult<()> {
        if let Some(target) = self.resolve_struct_lvalue_address(target)? {
            let source = self.resolve_struct_address(value)?;
            if source.struct_name != target.struct_name {
                return Err(CompileError::new("incompatible struct assignment"));
            }
            self.push_struct_copy(&target, &source)?;
            return Ok(());
        }
        let target = self.lower_lvalue(target)?;
        let value = self.lower_expr(value)?;
        self.push_store(target, value);
        Ok(())
    }

    fn lower_lvalue(&self, target: &LValue) -> CompileResult<LoweredLValue> {
        match target {
            LValue::Identifier(name) => {
                if let Some(binding) = self.local_binding(name) {
                    return match binding {
                        LocalBinding::Scalar {
                            slot, scalar_type, ..
                        } => Ok(LoweredLValue::Local {
                            slot,
                            offset: self.local_offset(slot)?,
                            scalar_type,
                        }),
                        LocalBinding::CharArray { .. } | LocalBinding::IntArray { .. } => Err(
                            CompileError::new("assignment to local array is not supported"),
                        ),
                        LocalBinding::StructObject { .. } => Err(CompileError::new(
                            "direct assignment to local struct object is not supported",
                        )),
                    };
                }
                if let Some(scalar_type) = self
                    .global_bindings
                    .get(name)
                    .and_then(GlobalBinding::scalar_type)
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
            LValue::Member {
                base,
                field,
                dereference,
            } => self.lower_member_lvalue(base, field, *dereference),
        }
    }

    fn resolve_struct_lvalue_address(
        &self,
        target: &LValue,
    ) -> CompileResult<Option<StructAddress>> {
        match target {
            LValue::Identifier(name) => {
                let Some(LocalBinding::StructObject {
                    slot,
                    struct_name,
                    byte_size,
                }) = self.local_binding(name)
                else {
                    return Ok(None);
                };
                Ok(Some(StructAddress {
                    pointer: LoweredExpr::LocalAddress {
                        offset: self.local_offset(slot)?,
                        byte_size,
                    },
                    offset: 0,
                    struct_name,
                }))
            }
            LValue::Member {
                base,
                field,
                dereference,
            } => {
                let member = self.resolve_member_access(base, field, *dereference)?;
                let FieldType::Struct(struct_name) = member.field_type else {
                    return Ok(None);
                };
                Ok(Some(StructAddress {
                    pointer: member.pointer,
                    offset: member.offset,
                    struct_name,
                }))
            }
            LValue::Subscript { .. } => Ok(None),
        }
    }

    fn lower_sizeof_expr(&self, expr: &Expr) -> CompileResult<LoweredExpr> {
        if let Expr::Identifier(name) = expr
            && let Some(binding) = self.local_binding(name)
        {
            let size = match binding {
                LocalBinding::Scalar { scalar_type, .. } => scalar_size(scalar_type),
                LocalBinding::CharArray { length, .. } => length,
                LocalBinding::IntArray { length, .. } => local_int_array_byte_size(length)?,
                LocalBinding::StructObject { byte_size, .. } => byte_size,
            };
            return i64::try_from(size)
                .map(LoweredExpr::Integer)
                .map_err(|_| CompileError::new("sizeof result does not fit i64"));
        }
        if let Expr::Identifier(name) = expr
            && let Some(GlobalBinding::StructArray {
                byte_size,
                length: Some(length),
                ..
            }) = self.global_bindings.get(name)
        {
            let size = byte_size
                .checked_mul(*length)
                .ok_or_else(|| CompileError::new("sizeof global array overflow"))?;
            return i64::try_from(size)
                .map(LoweredExpr::Integer)
                .map_err(|_| CompileError::new("sizeof result does not fit i64"));
        }
        let expr = self.lower_expr(expr)?;
        let size = lowered_expr_scalar_type(&expr).map_or(4, scalar_size);
        i64::try_from(size)
            .map(LoweredExpr::Integer)
            .map_err(|_| CompileError::new("sizeof result does not fit i64"))
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
        if let Expr::Identifier(name) = array
            && self.global_bindings.get(name) == Some(&GlobalBinding::IntArray)
        {
            return Ok(LoweredExpr::GlobalIntSubscript {
                name: name.clone(),
                index: Box::new(self.lower_expr(index)?),
            });
        }
        if let Expr::Identifier(name) = array
            && self.global_bindings.get(name) == Some(&GlobalBinding::PointerArray)
        {
            return Ok(LoweredExpr::GlobalPointerSubscript {
                name: name.clone(),
                index: Box::new(self.lower_expr(index)?),
            });
        }
        if let Some((pointer, element_type)) = self.resolve_array_field_subscript(array)? {
            return Ok(LoweredExpr::PointerSubscript {
                pointer: Box::new(pointer),
                index: Box::new(self.lower_expr(index)?),
                element_type,
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
        if let Expr::Identifier(name) = array
            && self.global_bindings.get(name) == Some(&GlobalBinding::UnsignedCharArray)
        {
            return Ok(LoweredLValue::GlobalByteSubscript {
                name: name.clone(),
                index: Box::new(self.lower_expr(index)?),
            });
        }
        if let Expr::Identifier(name) = array
            && self.global_bindings.get(name) == Some(&GlobalBinding::IntArray)
        {
            return Ok(LoweredLValue::GlobalIntSubscript {
                name: name.clone(),
                index: Box::new(self.lower_expr(index)?),
            });
        }
        if let Expr::Identifier(name) = array
            && self.global_bindings.get(name) == Some(&GlobalBinding::PointerArray)
        {
            return Ok(LoweredLValue::GlobalPointerSubscript {
                name: name.clone(),
                index: Box::new(self.lower_expr(index)?),
            });
        }
        if let Some((pointer, element_type)) = self.resolve_array_field_subscript(array)? {
            return Ok(LoweredLValue::PointerSubscript {
                pointer: Box::new(pointer),
                index: Box::new(self.lower_expr(index)?),
                element_type,
            });
        }

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

    fn lower_address_of(&self, target: &LValue) -> CompileResult<LoweredExpr> {
        match target {
            LValue::Subscript { array, index } => {
                if let Some(address) = self.resolve_global_struct_subscript_address(array, index)? {
                    return Ok(address.pointer);
                }
                if let Ok(address) = self.resolve_pointer_struct_subscript_address(array, index) {
                    return Ok(address.pointer);
                }
                let pointer = self.lower_expr(array)?;
                if lowered_expr_scalar_type(&pointer) != Some(ScalarType::Pointer) {
                    return Err(CompileError::new(
                        "address of subscript requires a pointer base",
                    ));
                }
                Ok(LoweredExpr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(pointer),
                    right: Box::new(self.lower_expr(index)?),
                })
            }
            LValue::Identifier(name) => {
                let Some(binding) = self.local_binding(name) else {
                    if self.global_bindings.contains_key(name) {
                        return Ok(LoweredExpr::GlobalAddress { name: name.clone() });
                    }
                    return Err(CompileError::new("unsupported address-of target"));
                };
                match binding {
                    LocalBinding::Scalar {
                        slot, scalar_type, ..
                    } => Ok(LoweredExpr::LocalAddress {
                        offset: self.local_offset(slot)?,
                        byte_size: scalar_size(scalar_type),
                    }),
                    LocalBinding::CharArray { slot, length } => Ok(LoweredExpr::LocalAddress {
                        offset: self.local_offset(slot)?,
                        byte_size: length,
                    }),
                    LocalBinding::IntArray { slot, length } => Ok(LoweredExpr::LocalAddress {
                        offset: self.local_offset(slot)?,
                        byte_size: local_int_array_byte_size(length)?,
                    }),
                    LocalBinding::StructObject {
                        slot, byte_size, ..
                    } => Ok(LoweredExpr::LocalAddress {
                        offset: self.local_offset(slot)?,
                        byte_size,
                    }),
                }
            }
            LValue::Member {
                base,
                field,
                dereference,
            } => {
                let member = self.resolve_member_access(base, field, *dereference)?;
                Ok(pointer_field_address(member.pointer, member.offset))
            }
        }
    }

    fn lower_member_expr(
        &self,
        base: &Expr,
        field: &str,
        dereference: bool,
    ) -> CompileResult<LoweredExpr> {
        let member = self.resolve_member_access(base, field, dereference)?;
        let scalar_type = match member.field_type {
            FieldType::Scalar(scalar_type) => scalar_type,
            FieldType::Pointer { .. } => ScalarType::Pointer,
            FieldType::Array { .. } => {
                return Ok(pointer_field_address(member.pointer, member.offset));
            }
            FieldType::Struct(_) => {
                return Err(CompileError::new("struct member value is not supported"));
            }
        };
        Ok(LoweredExpr::PointerField {
            pointer: Box::new(member.pointer),
            offset: member.offset,
            scalar_type,
        })
    }

    fn lower_member_lvalue(
        &self,
        base: &Expr,
        field: &str,
        dereference: bool,
    ) -> CompileResult<LoweredLValue> {
        let member = self.resolve_member_access(base, field, dereference)?;
        let scalar_type = match member.field_type {
            FieldType::Scalar(scalar_type) => scalar_type,
            FieldType::Pointer { .. } => ScalarType::Pointer,
            FieldType::Array { .. } => {
                return Err(CompileError::new(
                    "assignment to array member is not supported",
                ));
            }
            FieldType::Struct(_) => {
                return Err(CompileError::new(
                    "assignment to struct member is not supported",
                ));
            }
        };
        Ok(LoweredLValue::PointerField {
            pointer: Box::new(member.pointer),
            offset: member.offset,
            scalar_type,
        })
    }

    fn resolve_member_access(
        &self,
        base: &Expr,
        field: &str,
        dereference: bool,
    ) -> CompileResult<ResolvedMember> {
        let access = if dereference {
            let pointer = self.lower_expr(base)?;
            if lowered_expr_scalar_type(&pointer) != Some(ScalarType::Pointer) {
                return Err(CompileError::new("member dereference requires a pointer"));
            }
            StructAddress {
                pointer,
                offset: 0,
                struct_name: self.pointer_referent_for_expr(base)?,
            }
        } else {
            self.resolve_struct_address(base)?
        };
        let layout = self
            .structs
            .get(&access.struct_name)
            .ok_or_else(|| CompileError::new(format!("unknown struct: {}", access.struct_name)))?;
        let field = layout
            .fields
            .iter()
            .find(|candidate| candidate.name == field)
            .ok_or_else(|| {
                CompileError::new(format!(
                    "unknown struct field: {}.{field}",
                    access.struct_name
                ))
            })?;
        Ok(ResolvedMember {
            pointer: access.pointer,
            offset: access
                .offset
                .checked_add(field.offset)
                .ok_or_else(|| CompileError::new("struct member offset overflow"))?,
            field_type: field.field_type.clone(),
        })
    }

    fn resolve_array_field_subscript(
        &self,
        array: &Expr,
    ) -> CompileResult<Option<(LoweredExpr, ScalarType)>> {
        let Expr::Member {
            base,
            field,
            dereference,
        } = array
        else {
            return Ok(None);
        };
        let member = self.resolve_member_access(base, field, *dereference)?;
        let FieldType::Array { element_type, .. } = member.field_type else {
            return Ok(None);
        };
        Ok(Some((
            pointer_field_address(member.pointer, member.offset),
            element_type,
        )))
    }

    fn resolve_struct_address(&self, expr: &Expr) -> CompileResult<StructAddress> {
        if let Expr::Identifier(name) = expr
            && let Some(LocalBinding::StructObject {
                slot,
                struct_name,
                byte_size,
            }) = self.local_binding(name)
        {
            return Ok(StructAddress {
                pointer: LoweredExpr::LocalAddress {
                    offset: self.local_offset(slot)?,
                    byte_size,
                },
                offset: 0,
                struct_name,
            });
        }
        if let Expr::Identifier(name) = expr
            && let Some(GlobalBinding::StructObject {
                struct_name,
                byte_size: _,
            }) = self.global_bindings.get(name)
        {
            return Ok(StructAddress {
                pointer: LoweredExpr::GlobalAddress { name: name.clone() },
                offset: 0,
                struct_name: struct_name.clone(),
            });
        }
        if let Expr::Member {
            base,
            field,
            dereference,
        } = expr
        {
            let member = self.resolve_member_access(base, field, *dereference)?;
            let FieldType::Struct(struct_name) = member.field_type else {
                return Err(CompileError::new("member base is not a struct"));
            };
            return Ok(StructAddress {
                pointer: member.pointer,
                offset: member.offset,
                struct_name,
            });
        }
        if let Expr::Subscript { array, index } = expr {
            if let Some(address) = self.resolve_global_struct_subscript_address(array, index)? {
                return Ok(address);
            }
            return self.resolve_pointer_struct_subscript_address(array, index);
        }
        Err(CompileError::new("member access requires a struct base"))
    }

    fn resolve_global_struct_subscript_address(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<StructAddress>> {
        let Expr::Identifier(name) = array else {
            return Ok(None);
        };
        let Some(GlobalBinding::StructArray {
            struct_name,
            byte_size,
            ..
        }) = self.global_bindings.get(name)
        else {
            return Ok(None);
        };
        Ok(Some(StructAddress {
            pointer: LoweredExpr::PointerOffset {
                pointer: Box::new(LoweredExpr::GlobalAddress { name: name.clone() }),
                index: Box::new(self.lower_expr(index)?),
                byte_size: *byte_size,
            },
            offset: 0,
            struct_name: struct_name.clone(),
        }))
    }

    fn resolve_pointer_struct_subscript_address(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<StructAddress> {
        let struct_name = self.pointer_referent_for_expr(array)?;
        let byte_size = self.struct_layout(&struct_name)?.size;
        Ok(StructAddress {
            pointer: LoweredExpr::PointerOffset {
                pointer: Box::new(self.lower_expr(array)?),
                index: Box::new(self.lower_expr(index)?),
                byte_size,
            },
            offset: 0,
            struct_name,
        })
    }

    fn pointer_referent_for_expr(&self, expr: &Expr) -> CompileResult<String> {
        if let Expr::Identifier(name) = expr
            && let Some(binding) = self.local_binding(name)
            && let LocalBinding::Scalar {
                referent: Some(referent),
                ..
            } = binding
        {
            return Ok(referent);
        }
        if let Expr::Identifier(name) = expr
            && let Some(GlobalBinding::Pointer {
                referent: Some(referent),
            }) = self.global_bindings.get(name)
        {
            return Ok(referent.clone());
        }
        if let Expr::Member {
            base,
            field,
            dereference,
        } = expr
        {
            let member = self.resolve_member_access(base, field, *dereference)?;
            if let FieldType::Pointer {
                referent: Some(referent),
            } = member.field_type
            {
                return Ok(referent);
            }
        }
        if let Expr::Cast {
            target: ScalarType::Pointer,
            referent: Some(referent),
            ..
        } = expr
        {
            return Ok(referent.clone());
        }
        Err(CompileError::new(
            "pointer member access requires a typed pointer",
        ))
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
            target @ (LoweredLValue::GlobalByteSubscript { .. }
            | LoweredLValue::GlobalIntSubscript { .. }
            | LoweredLValue::GlobalPointerSubscript { .. }
            | LoweredLValue::PointerSubscript { .. }
            | LoweredLValue::PointerField { .. }) => {
                self.instructions
                    .push(Instruction::Eval(LoweredExpr::Assign {
                        target,
                        value: Box::new(value),
                    }));
            }
        }
    }

    fn push_struct_copy(
        &mut self,
        target: &StructAddress,
        source: &StructAddress,
    ) -> CompileResult<()> {
        let layout = self.struct_layout(&target.struct_name)?.clone();
        for field in layout.fields {
            let target_offset = target
                .offset
                .checked_add(field.offset)
                .ok_or_else(|| CompileError::new("struct member offset overflow"))?;
            let source_offset = source
                .offset
                .checked_add(field.offset)
                .ok_or_else(|| CompileError::new("struct member offset overflow"))?;
            match field.field_type {
                FieldType::Scalar(scalar_type) => self.push_struct_scalar_copy(
                    target,
                    source,
                    target_offset,
                    source_offset,
                    scalar_type,
                ),
                FieldType::Pointer { .. } => self.push_struct_scalar_copy(
                    target,
                    source,
                    target_offset,
                    source_offset,
                    ScalarType::Pointer,
                ),
                FieldType::Array {
                    element_type,
                    length,
                } => {
                    let element_size = scalar_size(element_type);
                    for index in 0..length {
                        let element_offset = index
                            .checked_mul(element_size)
                            .and_then(|offset| target_offset.checked_add(offset))
                            .ok_or_else(|| CompileError::new("struct array offset overflow"))?;
                        let source_element_offset = index
                            .checked_mul(element_size)
                            .and_then(|offset| source_offset.checked_add(offset))
                            .ok_or_else(|| CompileError::new("struct array offset overflow"))?;
                        self.push_struct_scalar_copy(
                            target,
                            source,
                            element_offset,
                            source_element_offset,
                            element_type,
                        );
                    }
                }
                FieldType::Struct(struct_name) => {
                    self.push_struct_copy(
                        &StructAddress {
                            pointer: target.pointer.clone(),
                            offset: target_offset,
                            struct_name: struct_name.clone(),
                        },
                        &StructAddress {
                            pointer: source.pointer.clone(),
                            offset: source_offset,
                            struct_name,
                        },
                    )?;
                }
            }
        }
        Ok(())
    }

    fn push_struct_scalar_copy(
        &mut self,
        target: &StructAddress,
        source: &StructAddress,
        target_offset: usize,
        source_offset: usize,
        scalar_type: ScalarType,
    ) {
        self.push_store(
            LoweredLValue::PointerField {
                pointer: Box::new(target.pointer.clone()),
                offset: target_offset,
                scalar_type,
            },
            LoweredExpr::PointerField {
                pointer: Box::new(source.pointer.clone()),
                offset: source_offset,
                scalar_type,
            },
        );
    }

    fn lower_post_increment_statement(&mut self, target: &LValue) -> CompileResult<()> {
        let target = self.lower_lvalue(target)?;
        ensure_post_increment_scalar(&target)?;
        let current = lowered_lvalue_to_expr(&target);
        self.push_store(
            target,
            LoweredExpr::Binary {
                op: BinaryOp::Add,
                left: Box::new(current),
                right: Box::new(LoweredExpr::Integer(1)),
            },
        );
        Ok(())
    }

    fn lower_post_increment_expr(&self, target: &LValue) -> CompileResult<LoweredExpr> {
        let target = self.lower_lvalue(target)?;
        ensure_post_increment_scalar(&target)?;
        Ok(LoweredExpr::PostIncrement { target })
    }
}

fn ensure_post_increment_scalar(target: &LoweredLValue) -> CompileResult<()> {
    if !matches!(
        lowered_lvalue_scalar_type(target),
        ScalarType::Int | ScalarType::Pointer
    ) {
        return Err(CompileError::new(
            "post-increment currently supports int and pointer lvalues only",
        ));
    }
    Ok(())
}

const fn lowered_expr_scalar_type(expr: &LoweredExpr) -> Option<ScalarType> {
    match expr {
        LoweredExpr::Global { scalar_type, .. }
        | LoweredExpr::Local { scalar_type, .. }
        | LoweredExpr::Cast {
            target: scalar_type,
            ..
        }
        | LoweredExpr::PointerField { scalar_type, .. } => Some(*scalar_type),
        LoweredExpr::GlobalIntSubscript { .. } => Some(ScalarType::Int),
        LoweredExpr::LocalAddress { .. }
        | LoweredExpr::GlobalPointerSubscript { .. }
        | LoweredExpr::GlobalAddress { .. }
        | LoweredExpr::PointerOffset { .. }
        | LoweredExpr::PointerFieldAddress { .. } => Some(ScalarType::Pointer),
        LoweredExpr::PointerSubscript { element_type, .. } => Some(*element_type),
        LoweredExpr::Assign { target, .. } | LoweredExpr::PostIncrement { target } => {
            Some(lowered_lvalue_scalar_type(target))
        }
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
        }
        | LoweredLValue::PointerField { scalar_type, .. } => *scalar_type,
        LoweredLValue::GlobalByteSubscript { .. } | LoweredLValue::GlobalIntSubscript { .. } => {
            ScalarType::Int
        }
        LoweredLValue::GlobalPointerSubscript { .. } => ScalarType::Pointer,
    }
}

fn lowered_lvalue_to_expr(target: &LoweredLValue) -> LoweredExpr {
    match target {
        LoweredLValue::Local {
            offset,
            scalar_type,
            ..
        } => LoweredExpr::Local {
            offset: *offset,
            scalar_type: *scalar_type,
        },
        LoweredLValue::Global { name, scalar_type } => LoweredExpr::Global {
            name: name.clone(),
            scalar_type: *scalar_type,
        },
        LoweredLValue::GlobalByteSubscript { name, index } => LoweredExpr::GlobalByteSubscript {
            name: name.clone(),
            index: index.clone(),
        },
        LoweredLValue::GlobalIntSubscript { name, index } => LoweredExpr::GlobalIntSubscript {
            name: name.clone(),
            index: index.clone(),
        },
        LoweredLValue::GlobalPointerSubscript { name, index } => {
            LoweredExpr::GlobalPointerSubscript {
                name: name.clone(),
                index: index.clone(),
            }
        }
        LoweredLValue::PointerSubscript {
            pointer,
            index,
            element_type,
        } => LoweredExpr::PointerSubscript {
            pointer: pointer.clone(),
            index: index.clone(),
            element_type: *element_type,
        },
        LoweredLValue::PointerField {
            pointer,
            offset,
            scalar_type,
        } => LoweredExpr::PointerField {
            pointer: pointer.clone(),
            offset: *offset,
            scalar_type: *scalar_type,
        },
    }
}

fn pointer_field_address(pointer: LoweredExpr, offset: usize) -> LoweredExpr {
    if offset == 0 {
        pointer
    } else {
        LoweredExpr::PointerFieldAddress {
            pointer: Box::new(pointer),
            offset,
        }
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

fn local_char_array_initializer_values(value: &str, length: usize) -> CompileResult<Vec<u8>> {
    if value.len() > length {
        return Err(CompileError::new(
            "local char array initializer is too large",
        ));
    }
    let mut values = Vec::with_capacity(length);
    values.extend_from_slice(value.as_bytes());
    if values.len() < length {
        values.push(0);
    }
    values.resize(length, 0);
    Ok(values)
}

fn local_int_array_byte_size(length: usize) -> CompileResult<usize> {
    length
        .checked_mul(scalar_size(ScalarType::Int))
        .ok_or_else(|| CompileError::new("local int array size overflow"))
}

fn struct_alignment(layout: &StructLayout) -> usize {
    layout.size.clamp(1, 8)
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
        Instruction::Return(None)
        | Instruction::Jump { .. }
        | Instruction::Label { .. }
        | Instruction::InitLocalBytes { .. }
        | Instruction::InitLocalInts { .. } => {}
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
        LoweredExpr::GlobalByteSubscript { index, .. }
        | LoweredExpr::GlobalIntSubscript { index, .. }
        | LoweredExpr::GlobalPointerSubscript { index, .. } => {
            inline_constant_calls_in_expr(index, constants);
        }
        LoweredExpr::PointerSubscript { pointer, index, .. }
        | LoweredExpr::PointerOffset { pointer, index, .. } => {
            inline_constant_calls_in_expr(pointer, constants);
            inline_constant_calls_in_expr(index, constants);
        }
        LoweredExpr::PointerField { pointer, .. }
        | LoweredExpr::PointerFieldAddress { pointer, .. } => {
            inline_constant_calls_in_expr(pointer, constants);
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
        LoweredExpr::PostIncrement { .. }
        | LoweredExpr::Integer(_)
        | LoweredExpr::DoubleLiteral(_)
        | LoweredExpr::StringLiteral(_)
        | LoweredExpr::Global { .. }
        | LoweredExpr::GlobalAddress { .. }
        | LoweredExpr::Local { .. }
        | LoweredExpr::LocalAddress { .. } => {}
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
