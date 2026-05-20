use std::collections::{HashMap, HashSet};

use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{
    BinaryOp, Constant, Expr, FieldType, Function, Global, GlobalInitializer, LValue,
    LocalCharArrayInitializer, PointerReturnFunction, Program, ReturnType, ScalarType, Statement,
    StructLayout, SwitchCase, UnaryOp,
};

mod builtin_calls;
mod call_args;
mod doom_alloc;
mod local_array;
mod pointer_arithmetic;
mod pointer_referent;
mod sizeof_expr;
mod static_local;
mod struct_initializer;

const POINTER_REFERENT: &str = "*";
const X86_64_VARIADIC_GP_SAVE_BYTES: usize = 48;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredProgram {
    pub globals: Vec<LoweredGlobal>,
    pub functions: Vec<LoweredFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredGlobal {
    pub name: String,
    pub is_static: bool,
    pub initializer: LoweredGlobalInitializer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoweredGlobalInitializer {
    Int(i32),
    IntArray(Vec<i32>),
    ShortArray(Vec<i32>),
    PointerNull,
    PointerString(String),
    PointerGlobalOffset {
        base: String,
        byte_offset: usize,
    },
    PointerArray(usize),
    PointerStringArray(Vec<String>),
    PointerNameArray {
        values: Vec<String>,
        length: usize,
    },
    StructArray {
        byte_len: usize,
        values: Vec<LoweredStructInitializerValue>,
    },
    ZeroBytes(usize),
    UnsignedCharArray(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredStructInitializerValue {
    pub byte_offset: usize,
    pub value: LoweredStructInitializerScalar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoweredStructInitializerScalar {
    Int { value: i32, byte_size: usize },
    IntString { value: String, byte_size: usize },
    LongLong(i64),
    Bytes { values: Vec<u8>, byte_len: usize },
    PointerNull,
    PointerInteger(i64),
    PointerString(String),
    PointerGlobalOffset { base: String, byte_offset: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredFunction {
    pub name: String,
    pub return_type: ReturnType,
    pub parameter_count: usize,
    pub is_variadic: bool,
    pub variadic_save_slot: Option<usize>,
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
        return_type: ScalarType,
    },
    IndirectCall {
        callee: Box<Self>,
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
        element_byte_size: usize,
        element_unsigned: bool,
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
        byte_size: usize,
        is_unsigned: bool,
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
        increment: i64,
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
        element_byte_size: usize,
        element_unsigned: bool,
    },
    PointerField {
        pointer: Box<LoweredExpr>,
        offset: usize,
        scalar_type: ScalarType,
        byte_size: usize,
        is_unsigned: bool,
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
    let (mut globals, global_bindings) = lower_globals(&program.globals, &constants, &structs)?;
    let pointer_return_functions =
        lower_pointer_return_functions(&program.pointer_return_functions);
    let function_names = lower_function_names(&program.functions, &program.function_prototypes);
    let mut functions = Vec::with_capacity(program.functions.len());
    for function in &program.functions {
        let lowered = lower_function_with_globals(
            function,
            &structs,
            &global_bindings,
            &constants,
            &pointer_return_functions,
            &function_names,
        )?;
        globals.extend(lowered.static_globals);
        functions.push(lowered.function);
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
        let (initializer, binding) =
            lower_defined_global_initializer(global, constants, structs, &bindings)?;
        if !definitions.insert(global.name.clone()) {
            return Err(CompileError::new(format!(
                "duplicate global declaration: {}",
                global.name
            )));
        }
        insert_global_binding(&mut bindings, &global.name, binding)?;
        lowered.push(LoweredGlobal {
            name: global.name.clone(),
            is_static: global.is_static,
            initializer,
        });
    }
    insert_builtin_libc_bindings(&mut bindings);
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
        GlobalInitializer::ExternShortArray {
            is_unsigned,
            columns,
        } => GlobalBinding::ShortArray {
            is_unsigned: *is_unsigned,
            columns: *columns,
        },
        GlobalInitializer::ExternPointerArray { referent, columns } => {
            GlobalBinding::PointerArray {
                referent: referent.clone(),
                columns: *columns,
            }
        }
        GlobalInitializer::ExternUnsignedCharArray => GlobalBinding::UnsignedCharArray,
        GlobalInitializer::ExternUnsignedCharMatrix { columns } => {
            GlobalBinding::UnsignedCharMatrix { columns: *columns }
        }
        GlobalInitializer::ExternStructArray { struct_name } => {
            let layout = structs.get(struct_name).ok_or_else(|| {
                CompileError::new(format!("unknown struct-array type: {struct_name}"))
            })?;
            GlobalBinding::StructArray {
                struct_name: struct_name.clone(),
                byte_size: layout.size,
                length: None,
                columns: None,
            }
        }
        GlobalInitializer::ExternStructObject { struct_name } => {
            let layout = structs.get(struct_name).ok_or_else(|| {
                CompileError::new(format!("unknown struct object type: {struct_name}"))
            })?;
            GlobalBinding::StructObject {
                struct_name: struct_name.clone(),
                byte_size: layout.size,
            }
        }
        GlobalInitializer::Int(_)
        | GlobalInitializer::IntArray(_)
        | GlobalInitializer::ShortArray { .. }
        | GlobalInitializer::IntMatrix { .. }
        | GlobalInitializer::DoubleArray { .. }
        | GlobalInitializer::IntConstant(_)
        | GlobalInitializer::PointerNull { .. }
        | GlobalInitializer::PointerString { .. }
        | GlobalInitializer::PointerSubscriptAddress { .. }
        | GlobalInitializer::PointerArray { .. }
        | GlobalInitializer::PointerStringArray { .. }
        | GlobalInitializer::PointerNameArray { .. }
        | GlobalInitializer::StructObject { .. }
        | GlobalInitializer::StructArray { .. }
        | GlobalInitializer::UnsignedCharArray(_)
        | GlobalInitializer::UnsignedCharMatrix { .. } => return Ok(None),
    };
    Ok(Some(binding))
}

fn lower_defined_global_initializer(
    global: &Global,
    constants: &HashMap<String, i64>,
    structs: &HashMap<String, StructLayout>,
    global_bindings: &HashMap<String, GlobalBinding>,
) -> CompileResult<(LoweredGlobalInitializer, GlobalBinding)> {
    if let Some(lowered) = lower_pointer_array_initializer(&global.initializer) {
        return lowered;
    }
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
        GlobalInitializer::ShortArray { .. } => lower_short_array_global_initializer(global),
        GlobalInitializer::IntMatrix { values, columns } => Ok((
            LoweredGlobalInitializer::IntArray(values.clone()),
            GlobalBinding::IntMatrix { columns: *columns },
        )),
        GlobalInitializer::DoubleArray { length } => {
            let byte_len = length
                .checked_mul(scalar_size(ScalarType::Double))
                .ok_or_else(|| CompileError::new("global double-array size overflow"))?;
            Ok((
                LoweredGlobalInitializer::ZeroBytes(byte_len),
                GlobalBinding::DoubleArray,
            ))
        }
        GlobalInitializer::IntConstant(name) => {
            lower_int_constant_global(name, &global.name, constants)
        }
        GlobalInitializer::PointerNull { referent } => Ok((
            LoweredGlobalInitializer::PointerNull,
            GlobalBinding::Pointer {
                referent: referent.clone(),
            },
        )),
        GlobalInitializer::PointerString { referent, value } => Ok((
            LoweredGlobalInitializer::PointerString(value.clone()),
            GlobalBinding::Pointer {
                referent: referent.clone(),
            },
        )),
        GlobalInitializer::PointerSubscriptAddress {
            referent,
            base,
            index,
        } => lower_global_pointer_subscript_address(referent.as_deref(), base, *index),
        GlobalInitializer::PointerArray { .. }
        | GlobalInitializer::PointerStringArray { .. }
        | GlobalInitializer::PointerNameArray { .. } => Err(CompileError::new(
            "internal error: pointer array global reached fallback lowering",
        )),
        GlobalInitializer::StructObject { struct_name } => {
            lower_struct_object_global(struct_name, structs)
        }
        GlobalInitializer::StructArray {
            struct_name,
            length,
            columns,
            values,
        } => struct_initializer::lower_struct_array_global(
            struct_name,
            *length,
            *columns,
            values,
            structs,
            global_bindings,
        ),
        GlobalInitializer::UnsignedCharArray(values) => Ok((
            LoweredGlobalInitializer::UnsignedCharArray(values.clone()),
            GlobalBinding::UnsignedCharArray,
        )),
        GlobalInitializer::UnsignedCharMatrix { values, columns } => Ok((
            LoweredGlobalInitializer::UnsignedCharArray(values.clone()),
            GlobalBinding::UnsignedCharMatrix { columns: *columns },
        )),
        GlobalInitializer::Extern(_)
        | GlobalInitializer::ExternPointer { .. }
        | GlobalInitializer::ExternIntArray
        | GlobalInitializer::ExternShortArray { .. }
        | GlobalInitializer::ExternPointerArray { .. }
        | GlobalInitializer::ExternUnsignedCharArray
        | GlobalInitializer::ExternUnsignedCharMatrix { .. }
        | GlobalInitializer::ExternStructArray { .. }
        | GlobalInitializer::ExternStructObject { .. } => Err(CompileError::new(
            "internal error: extern global reached definition lowering",
        )),
    }
}

fn lower_short_array_global_initializer(
    global: &Global,
) -> CompileResult<(LoweredGlobalInitializer, GlobalBinding)> {
    let GlobalInitializer::ShortArray {
        values,
        is_unsigned,
        columns,
    } = &global.initializer
    else {
        return Err(CompileError::new("expected short-array global initializer"));
    };
    Ok((
        LoweredGlobalInitializer::ShortArray(values.clone()),
        GlobalBinding::ShortArray {
            is_unsigned: *is_unsigned,
            columns: *columns,
        },
    ))
}

fn lower_pointer_array_initializer(
    initializer: &GlobalInitializer,
) -> Option<CompileResult<(LoweredGlobalInitializer, GlobalBinding)>> {
    match initializer {
        GlobalInitializer::PointerArray {
            referent,
            length,
            columns,
        } => Some(Ok((
            LoweredGlobalInitializer::PointerArray(*length),
            GlobalBinding::PointerArray {
                referent: referent.clone(),
                columns: *columns,
            },
        ))),
        GlobalInitializer::PointerStringArray { referent, values } => Some(Ok((
            LoweredGlobalInitializer::PointerStringArray(values.clone()),
            GlobalBinding::PointerArray {
                referent: referent.clone(),
                columns: None,
            },
        ))),
        GlobalInitializer::PointerNameArray {
            referent,
            values,
            length,
        } => Some(Ok((
            LoweredGlobalInitializer::PointerNameArray {
                values: values.clone(),
                length: *length,
            },
            GlobalBinding::PointerArray {
                referent: referent.clone(),
                columns: None,
            },
        ))),
        _ => None,
    }
}

fn lower_global_pointer_subscript_address(
    referent: Option<&str>,
    base: &str,
    index: usize,
) -> CompileResult<(LoweredGlobalInitializer, GlobalBinding)> {
    let stride = referent
        .and_then(pointer_arithmetic::byte_size)
        .unwrap_or_else(|| scalar_size(ScalarType::Int));
    let byte_offset = index
        .checked_mul(stride)
        .ok_or_else(|| CompileError::new("global pointer offset overflow"))?;
    Ok((
        LoweredGlobalInitializer::PointerGlobalOffset {
            base: base.to_owned(),
            byte_offset,
        },
        GlobalBinding::Pointer {
            referent: referent.map(ToOwned::to_owned),
        },
    ))
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

fn insert_builtin_libc_bindings(bindings: &mut HashMap<String, GlobalBinding>) {
    bindings
        .entry("errno".to_owned())
        .or_insert(GlobalBinding::Int);
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
            GlobalBinding::PointerArray { referent, columns },
            GlobalBinding::PointerArray {
                referent: incoming_referent,
                columns: incoming_columns,
            },
        ) if referent == incoming_referent => {
            let OptionalUsizeMerge::Compatible(merged_columns) =
                merge_optional_usize(*columns, *incoming_columns)
            else {
                return None;
            };
            Some(GlobalBinding::PointerArray {
                referent: referent.clone(),
                columns: merged_columns,
            })
        }
        (
            GlobalBinding::StructArray {
                struct_name,
                byte_size,
                length,
                columns,
            },
            GlobalBinding::StructArray {
                struct_name: incoming_name,
                byte_size: incoming_byte_size,
                length: incoming_length,
                columns: incoming_columns,
            },
        ) if struct_name == incoming_name && byte_size == incoming_byte_size => {
            let OptionalUsizeMerge::Compatible(merged_length) =
                merge_optional_usize(*length, *incoming_length)
            else {
                return None;
            };
            let OptionalUsizeMerge::Compatible(merged_columns) =
                merge_optional_usize(*columns, *incoming_columns)
            else {
                return None;
            };
            Some(GlobalBinding::StructArray {
                struct_name: struct_name.clone(),
                byte_size: *byte_size,
                length: merged_length,
                columns: merged_columns,
            })
        }
        _ => None,
    }
}

enum OptionalUsizeMerge {
    Compatible(Option<usize>),
    Conflict,
}

const fn merge_optional_usize(
    existing: Option<usize>,
    incoming: Option<usize>,
) -> OptionalUsizeMerge {
    match (existing, incoming) {
        (Some(existing), Some(new)) => {
            if existing == new {
                OptionalUsizeMerge::Compatible(Some(existing))
            } else {
                OptionalUsizeMerge::Conflict
            }
        }
        (Some(existing), None) | (None, Some(existing)) => {
            OptionalUsizeMerge::Compatible(Some(existing))
        }
        (None, None) => OptionalUsizeMerge::Compatible(None),
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

fn lower_pointer_return_functions(
    functions: &[PointerReturnFunction],
) -> HashMap<String, Option<String>> {
    functions
        .iter()
        .map(|function| (function.name.clone(), function.referent.clone()))
        .collect()
}

fn lower_function_names(functions: &[Function], function_prototypes: &[String]) -> HashSet<String> {
    functions
        .iter()
        .map(|function| function.name.clone())
        .chain(function_prototypes.iter().cloned())
        .collect()
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
        Expr::IndirectCall { .. } => Err(CompileError::new(
            "indirect call is not a constant expression",
        )),
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
    let pointer_return_functions = HashMap::new();
    let function_names = HashSet::new();
    lower_function_with_globals(
        function,
        &structs,
        &global_bindings,
        &constants,
        &pointer_return_functions,
        &function_names,
    )
    .map(|lowered| lowered.function)
}

struct LoweredFunctionWithStatics {
    function: LoweredFunction,
    static_globals: Vec<LoweredGlobal>,
}

fn lower_function_with_globals(
    function: &Function,
    structs: &HashMap<String, StructLayout>,
    global_bindings: &HashMap<String, GlobalBinding>,
    constants: &HashMap<String, i64>,
    pointer_return_functions: &HashMap<String, Option<String>>,
    function_names: &HashSet<String>,
) -> CompileResult<LoweredFunctionWithStatics> {
    let mut context = LoweringContext::new(
        &function.name,
        function.return_type,
        structs,
        global_bindings,
        constants,
        pointer_return_functions,
        function_names,
    );
    for parameter in &function.parameters {
        context.declare_local(
            &parameter.name,
            parameter.scalar_type,
            parameter.referent.clone(),
        )?;
    }
    let variadic_save_slot = if function.is_variadic {
        Some(context.declare_anonymous_slot(
            ScalarType::Pointer,
            X86_64_VARIADIC_GP_SAVE_BYTES,
            scalar_size(ScalarType::Pointer),
        )?)
    } else {
        None
    };
    for statement in &function.statements {
        context.lower_statement(statement)?;
    }
    if matches!(function.return_type, ReturnType::Int | ReturnType::Pointer) && !context.has_return
    {
        return Err(CompileError::new(format!(
            "function {} has no return statement",
            function.name
        )));
    }
    if function.return_type == ReturnType::Void && !ends_with_return(&context.instructions) {
        context.instructions.push(Instruction::Return(None));
    }
    Ok(LoweredFunctionWithStatics {
        function: LoweredFunction {
            name: function.name.clone(),
            return_type: function.return_type,
            parameter_count: function.parameters.len(),
            is_variadic: function.is_variadic,
            variadic_save_slot,
            local_slots: context.local_slots,
            instructions: context.instructions,
        },
        static_globals: context.static_globals,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LocalBinding {
    Scalar {
        slot: usize,
        scalar_type: ScalarType,
        referent: Option<String>,
    },
    StaticScalar {
        global_name: String,
        scalar_type: ScalarType,
        referent: Option<String>,
    },
    CharArray {
        slot: usize,
        length: usize,
    },
    CharMatrix {
        slot: usize,
        rows: usize,
        columns: usize,
    },
    IntArray {
        slot: usize,
        length: usize,
    },
    ShortArray {
        slot: usize,
        length: usize,
        is_unsigned: bool,
    },
    PointerArray {
        slot: usize,
        length: usize,
    },
    StructObject {
        slot: usize,
        struct_name: String,
        byte_size: usize,
    },
    VaList {
        slot: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GlobalBinding {
    Int,
    IntArray,
    ShortArray {
        is_unsigned: bool,
        columns: Option<usize>,
    },
    IntMatrix {
        columns: usize,
    },
    DoubleArray,
    Pointer {
        referent: Option<String>,
    },
    PointerArray {
        referent: Option<String>,
        columns: Option<usize>,
    },
    StructObject {
        struct_name: String,
        byte_size: usize,
    },
    StructArray {
        struct_name: String,
        byte_size: usize,
        length: Option<usize>,
        columns: Option<usize>,
    },
    UnsignedCharArray,
    UnsignedCharMatrix {
        columns: usize,
    },
}

impl GlobalBinding {
    fn from_scalar_type(scalar_type: ScalarType) -> CompileResult<Self> {
        match scalar_type {
            ScalarType::Int => Ok(Self::Int),
            ScalarType::Pointer => Ok(Self::Pointer { referent: None }),
            ScalarType::LongLong | ScalarType::Double | ScalarType::VaList => {
                Err(CompileError::new("unsupported extern global scalar type"))
            }
        }
    }

    const fn scalar_type(&self) -> Option<ScalarType> {
        match self {
            Self::Int => Some(ScalarType::Int),
            Self::Pointer { .. } => Some(ScalarType::Pointer),
            Self::IntArray
            | Self::ShortArray { .. }
            | Self::IntMatrix { .. }
            | Self::DoubleArray
            | Self::PointerArray { .. }
            | Self::StructObject { .. }
            | Self::StructArray { .. }
            | Self::UnsignedCharArray
            | Self::UnsignedCharMatrix { .. } => None,
        }
    }

    const fn is_addressable_array(&self) -> bool {
        matches!(
            self,
            Self::IntArray
                | Self::ShortArray { .. }
                | Self::IntMatrix { .. }
                | Self::DoubleArray
                | Self::PointerArray { .. }
                | Self::StructArray { .. }
                | Self::UnsignedCharArray
                | Self::UnsignedCharMatrix { .. }
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

type ArrayFieldSubscript = (LoweredExpr, ScalarType, usize, bool);
type NestedArrayFieldSubscript = (LoweredExpr, LoweredExpr, ScalarType, usize, bool);

struct LoweringContext {
    function_name: String,
    return_type: ReturnType,
    structs: HashMap<String, StructLayout>,
    global_bindings: HashMap<String, GlobalBinding>,
    static_globals: Vec<LoweredGlobal>,
    constants: HashMap<String, i64>,
    pointer_return_functions: HashMap<String, Option<String>>,
    function_names: HashSet<String>,
    scopes: Vec<HashMap<String, LocalBinding>>,
    local_slots: Vec<LocalSlot>,
    next_local_offset: usize,
    instructions: Vec<Instruction>,
    next_label: usize,
    named_labels: HashMap<String, usize>,
    break_labels: Vec<usize>,
    continue_labels: Vec<usize>,
    has_return: bool,
}

impl LoweringContext {
    fn new(
        function_name: &str,
        return_type: ReturnType,
        structs: &HashMap<String, StructLayout>,
        global_bindings: &HashMap<String, GlobalBinding>,
        constants: &HashMap<String, i64>,
        pointer_return_functions: &HashMap<String, Option<String>>,
        function_names: &HashSet<String>,
    ) -> Self {
        Self {
            function_name: function_name.to_owned(),
            return_type,
            structs: structs.clone(),
            global_bindings: global_bindings.clone(),
            static_globals: Vec::new(),
            constants: constants.clone(),
            pointer_return_functions: pointer_return_functions.clone(),
            function_names: function_names.clone(),
            scopes: vec![HashMap::new()],
            local_slots: Vec::new(),
            next_local_offset: 0,
            instructions: Vec::new(),
            next_label: 0,
            named_labels: HashMap::new(),
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
                is_static,
                scalar_type,
                name,
                referent,
                initializer,
            } => self.lower_declaration(
                *is_static,
                *scalar_type,
                name,
                referent.clone(),
                initializer.as_ref(),
            ),
            Statement::LocalCharArray {
                name,
                length,
                initializer,
            } => self.lower_local_char_array(name, *length, initializer.as_ref()),
            Statement::LocalCharMatrix {
                name,
                rows,
                columns,
                initializer,
            } => self.lower_local_char_matrix(name, *rows, *columns, initializer.as_deref()),
            Statement::LocalIntArray {
                name,
                length,
                initializer,
            } => self.lower_local_int_array(name, *length, initializer.as_deref()),
            Statement::LocalShortArray {
                name,
                length,
                is_unsigned,
            } => self.lower_local_short_array(name, *length, *is_unsigned),
            Statement::LocalPointerArray {
                name,
                length,
                initializer,
            } => self.lower_local_pointer_array(name, *length, initializer.as_deref()),
            Statement::LocalStruct { name, struct_name } => {
                self.lower_local_struct_object(name, struct_name)
            }
            Statement::LocalConstants(constants) => {
                for constant in constants {
                    self.constants.insert(constant.name.clone(), constant.value);
                }
                Ok(())
            }
            Statement::DeclarationList(declarations) | Statement::ExpressionList(declarations) => {
                self.lower_statement_list(declarations)
            }
            Statement::ExternGlobal(global) => self.lower_extern_global(global),
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
            Statement::Expression(Expr::PostIncrement { target, decrement }) => {
                self.lower_post_increment_statement(target, *decrement)
            }
            Statement::Expression(expr) => self.lower_expression_statement(expr),
            Statement::Break => self.lower_break(),
            Statement::Continue => self.lower_continue(),
            Statement::Label(label) => {
                self.lower_label(label);
                Ok(())
            }
            Statement::Goto(label) => {
                self.lower_goto(label);
                Ok(())
            }
            Statement::Return(expr) => self.lower_return(expr.as_ref()),
        }
    }

    fn lower_statement_list(&mut self, statements: &[Statement]) -> CompileResult<()> {
        for statement in statements {
            self.lower_statement(statement)?;
        }
        Ok(())
    }

    fn lower_expression_statement(&mut self, expr: &Expr) -> CompileResult<()> {
        let expr = self.lower_expr(expr)?;
        self.instructions.push(Instruction::Eval(expr));
        Ok(())
    }

    fn lower_break(&mut self) -> CompileResult<()> {
        let Some(label) = self.break_labels.last() else {
            return Err(CompileError::new("break statement outside loop"));
        };
        self.instructions.push(Instruction::Jump { label: *label });
        Ok(())
    }

    fn lower_continue(&mut self) -> CompileResult<()> {
        let Some(label) = self.continue_labels.last() else {
            return Err(CompileError::new("continue statement outside loop"));
        };
        self.instructions.push(Instruction::Jump { label: *label });
        Ok(())
    }

    fn lower_label(&mut self, name: &str) {
        let label = self.named_label(name);
        self.instructions.push(Instruction::Label { label });
    }

    fn lower_goto(&mut self, name: &str) {
        let label = self.named_label(name);
        self.instructions.push(Instruction::Jump { label });
    }

    fn lower_return(&mut self, expr: Option<&Expr>) -> CompileResult<()> {
        match (self.return_type, expr) {
            (ReturnType::Int | ReturnType::Pointer, Some(expr)) => {
                let value = self.lower_expr(expr)?;
                self.instructions.push(Instruction::Return(Some(value)));
            }
            (ReturnType::Int, None) => {
                return Err(CompileError::new("int function must return a value"));
            }
            (ReturnType::Pointer, None) => {
                return Err(CompileError::new("pointer function must return a value"));
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

    fn lower_declaration(
        &mut self,
        is_static: bool,
        scalar_type: ScalarType,
        name: &str,
        referent: Option<String>,
        initializer: Option<&Expr>,
    ) -> CompileResult<()> {
        if is_static {
            return self.lower_static_declaration(scalar_type, name, referent, initializer);
        }
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

    fn lower_static_declaration(
        &mut self,
        scalar_type: ScalarType,
        name: &str,
        referent: Option<String>,
        initializer: Option<&Expr>,
    ) -> CompileResult<()> {
        if !matches!(scalar_type, ScalarType::Int | ScalarType::Pointer) {
            return Err(CompileError::new(
                "static local currently supports int and pointer scalars only",
            ));
        }
        let initializer =
            static_local::scalar_initializer(scalar_type, initializer, &self.constants)?;
        let global_name = self.declare_static_scalar(name, scalar_type, referent)?;
        self.static_globals.push(LoweredGlobal {
            name: global_name,
            is_static: true,
            initializer,
        });
        Ok(())
    }

    fn lower_local_char_array(
        &mut self,
        name: &str,
        length: usize,
        initializer: Option<&LocalCharArrayInitializer>,
    ) -> CompileResult<()> {
        let slot = self.declare_char_array(name, length)?;
        if let Some(initializer) = initializer {
            self.instructions.push(Instruction::InitLocalBytes {
                offset: self.local_offset(slot)?,
                values: local_char_array_initializer_values(initializer, length)?,
            });
        }
        Ok(())
    }

    fn lower_local_char_matrix(
        &mut self,
        name: &str,
        rows: usize,
        columns: usize,
        initializer: Option<&[String]>,
    ) -> CompileResult<()> {
        let slot = self.declare_char_matrix(name, rows, columns)?;
        if let Some(values) = initializer {
            self.instructions.push(Instruction::InitLocalBytes {
                offset: self.local_offset(slot)?,
                values: local_char_matrix_initializer_values(values, rows, columns)?,
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

    fn lower_local_short_array(
        &mut self,
        name: &str,
        length: usize,
        is_unsigned: bool,
    ) -> CompileResult<()> {
        self.declare_short_array(name, length, is_unsigned)?;
        Ok(())
    }

    fn lower_local_pointer_array(
        &mut self,
        name: &str,
        length: usize,
        initializer: Option<&[Expr]>,
    ) -> CompileResult<()> {
        let slot = self.declare_pointer_array(name, length)?;
        if let Some(values) = initializer {
            if values.len() > length {
                return Err(CompileError::new(
                    "local pointer array initializer is too large",
                ));
            }
            let offset = self.local_offset(slot)?;
            let byte_size = local_pointer_array_byte_size(length)?;
            for (index, value) in values.iter().enumerate() {
                let index = i64::try_from(index)
                    .map_err(|_| CompileError::new("local pointer array index overflow"))?;
                let target = LoweredLValue::PointerSubscript {
                    pointer: Box::new(LoweredExpr::LocalAddress { offset, byte_size }),
                    index: Box::new(LoweredExpr::Integer(index)),
                    element_type: ScalarType::Pointer,
                    element_byte_size: scalar_size(ScalarType::Pointer),
                    element_unsigned: false,
                };
                let value = self.lower_expr(value)?;
                self.push_store(target, value);
            }
        }
        Ok(())
    }

    fn lower_extern_global(&mut self, global: &Global) -> CompileResult<()> {
        let Some(binding) = lower_extern_global_binding(&global.initializer, &self.structs)? else {
            return Err(CompileError::new(
                "internal error: non-extern block global declaration",
            ));
        };
        insert_global_binding(&mut self.global_bindings, &global.name, binding)
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
        if scalar_type == ScalarType::VaList {
            return self.declare_slot(
                name,
                scalar_type,
                scalar_size(scalar_type),
                scalar_size(ScalarType::Pointer),
                LocalBinding::VaList {
                    slot: self.local_slots.len(),
                },
            );
        }
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

    fn declare_static_scalar(
        &mut self,
        name: &str,
        scalar_type: ScalarType,
        referent: Option<String>,
    ) -> CompileResult<String> {
        self.ensure_scope_name_available(name)?;
        let global_name = format!("{}__static__{name}", self.function_name);
        self.insert_scope_binding(
            name,
            LocalBinding::StaticScalar {
                global_name: global_name.clone(),
                scalar_type,
                referent,
            },
        )?;
        Ok(global_name)
    }

    fn declare_anonymous_slot(
        &mut self,
        scalar_type: ScalarType,
        byte_size: usize,
        alignment: usize,
    ) -> CompileResult<usize> {
        let slot = self.local_slots.len();
        let offset = align_to(self.next_local_offset, alignment);
        self.next_local_offset = offset
            .checked_add(byte_size)
            .ok_or_else(|| CompileError::new("local stack size overflow"))?;
        self.local_slots.push(LocalSlot {
            offset,
            scalar_type,
            byte_size,
        });
        Ok(slot)
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

    fn declare_char_matrix(
        &mut self,
        name: &str,
        rows: usize,
        columns: usize,
    ) -> CompileResult<usize> {
        let byte_size = local_char_matrix_byte_size(rows, columns)?;
        self.declare_slot(
            name,
            ScalarType::Int,
            byte_size,
            1,
            LocalBinding::CharMatrix {
                slot: self.local_slots.len(),
                rows,
                columns,
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

    fn declare_short_array(
        &mut self,
        name: &str,
        length: usize,
        is_unsigned: bool,
    ) -> CompileResult<usize> {
        let byte_size = local_short_array_byte_size(length)?;
        self.declare_slot(
            name,
            ScalarType::Int,
            byte_size,
            2,
            LocalBinding::ShortArray {
                slot: self.local_slots.len(),
                length,
                is_unsigned,
            },
        )
    }

    fn declare_pointer_array(&mut self, name: &str, length: usize) -> CompileResult<usize> {
        let byte_size = local_pointer_array_byte_size(length)?;
        self.declare_slot(
            name,
            ScalarType::Pointer,
            byte_size,
            scalar_size(ScalarType::Pointer),
            LocalBinding::PointerArray {
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
        self.ensure_scope_name_available(name)?;
        let slot = self.local_slots.len();
        let offset = align_to(self.next_local_offset, alignment);
        self.next_local_offset = offset + byte_size;
        self.local_slots.push(LocalSlot {
            offset,
            scalar_type,
            byte_size,
        });
        self.insert_scope_binding(name, binding)?;
        Ok(slot)
    }

    fn ensure_scope_name_available(&self, name: &str) -> CompileResult<()> {
        let Some(scope) = self.scopes.last() else {
            return Err(CompileError::new("internal error: no local scope"));
        };
        if scope.contains_key(name) {
            return Err(CompileError::new(format!(
                "duplicate local declaration: {name}"
            )));
        }
        Ok(())
    }

    fn insert_scope_binding(&mut self, name: &str, binding: LocalBinding) -> CompileResult<()> {
        let Some(scope) = self.scopes.last_mut() else {
            return Err(CompileError::new("internal error: no local scope"));
        };
        scope.insert(name.to_string(), binding);
        Ok(())
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

    fn named_label(&mut self, name: &str) -> usize {
        if let Some(label) = self.named_labels.get(name) {
            return *label;
        }
        let label = self.fresh_label();
        self.named_labels.insert(name.to_owned(), label);
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
            Expr::Call { callee, args } => self.lower_call_expr(callee, args),
            Expr::IndirectCall { callee, args } => self.lower_indirect_call_expr(callee, args),
            Expr::Identifier(name) => self.lower_identifier_expr(name),
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
            Expr::Assignment { target, value } => self.lower_assignment_expr(target, value),
            Expr::PostIncrement { target, decrement } => {
                self.lower_post_increment_expr(target, *decrement)
            }
            Expr::Unary { op, expr } => Ok(LoweredExpr::Unary {
                op: *op,
                expr: Box::new(self.lower_expr(expr)?),
            }),
            Expr::Cast { target, expr, .. } => self.lower_cast_expr(*target, expr),
            Expr::Conditional {
                condition,
                then_expr,
                else_expr,
            } => self.lower_conditional_expr(condition, then_expr, else_expr),
            Expr::Binary { op, left, right } => self.lower_binary_expr(*op, left, right),
        }
    }

    fn lower_call_expr(&self, callee: &str, args: &[Expr]) -> CompileResult<LoweredExpr> {
        if self.callee_is_pointer_binding(callee) {
            return Ok(LoweredExpr::IndirectCall {
                callee: Box::new(self.lower_identifier_expr(callee)?),
                args: args
                    .iter()
                    .map(|arg| self.lower_expr(arg))
                    .collect::<CompileResult<Vec<_>>>()?,
            });
        }
        Ok(LoweredExpr::Call {
            callee: callee.to_owned(),
            args: args
                .iter()
                .enumerate()
                .map(|(index, arg)| self.lower_call_arg(callee, index, arg))
                .collect::<CompileResult<Vec<_>>>()?,
            return_type: self.direct_call_return_type(callee),
        })
    }

    fn lower_call_arg(&self, callee: &str, index: usize, arg: &Expr) -> CompileResult<LoweredExpr> {
        call_args::lower(self, callee, index, arg)
    }

    fn direct_call_return_type(&self, callee: &str) -> ScalarType {
        if self.pointer_return_functions.contains_key(callee)
            || builtin_calls::returns_pointer(callee)
        {
            ScalarType::Pointer
        } else {
            ScalarType::Int
        }
    }

    fn callee_is_pointer_binding(&self, callee: &str) -> bool {
        if let Some(binding) = self.local_binding(callee) {
            return matches!(
                binding,
                LocalBinding::Scalar {
                    scalar_type: ScalarType::Pointer,
                    ..
                } | LocalBinding::StaticScalar {
                    scalar_type: ScalarType::Pointer,
                    ..
                }
            );
        }
        self.global_bindings
            .get(callee)
            .and_then(GlobalBinding::scalar_type)
            == Some(ScalarType::Pointer)
    }

    fn lower_indirect_call_expr(&self, callee: &Expr, args: &[Expr]) -> CompileResult<LoweredExpr> {
        let callee = if let Expr::Dereference { pointer } = callee {
            pointer.as_ref()
        } else {
            callee
        };
        let callee = self.lower_expr(callee)?;
        if lowered_expr_scalar_type(&callee) != Some(ScalarType::Pointer) {
            return Err(CompileError::new("indirect call requires a pointer callee"));
        }
        Ok(LoweredExpr::IndirectCall {
            callee: Box::new(callee),
            args: args
                .iter()
                .map(|arg| self.lower_expr(arg))
                .collect::<CompileResult<Vec<_>>>()?,
        })
    }

    fn lower_cast_expr(&self, target: ScalarType, expr: &Expr) -> CompileResult<LoweredExpr> {
        let expr = if target == ScalarType::Pointer {
            self.lower_pointer_cast_expr(expr)?
        } else {
            self.lower_expr(expr)?
        };
        Ok(LoweredExpr::Cast {
            target,
            expr: Box::new(expr),
        })
    }

    fn lower_pointer_cast_expr(&self, expr: &Expr) -> CompileResult<LoweredExpr> {
        match self.lower_expr(expr) {
            Ok(lowered) => Ok(lowered),
            Err(error) => {
                if let Expr::Identifier(name) = expr {
                    return Ok(LoweredExpr::GlobalAddress { name: name.clone() });
                }
                Err(error)
            }
        }
    }

    fn lower_identifier_expr(&self, name: &str) -> CompileResult<LoweredExpr> {
        if let Some(binding) = self.local_binding(name) {
            return self.lower_local_identifier_expr(&binding);
        }
        if let Some(scalar_type) = self
            .global_bindings
            .get(name)
            .and_then(GlobalBinding::scalar_type)
        {
            return Ok(LoweredExpr::Global {
                name: name.to_owned(),
                scalar_type,
            });
        }
        if self
            .global_bindings
            .get(name)
            .is_some_and(GlobalBinding::is_addressable_array)
        {
            return Ok(LoweredExpr::GlobalAddress {
                name: name.to_owned(),
            });
        }
        if let Some(value) = self.constants.get(name) {
            return Ok(LoweredExpr::Integer(*value));
        }
        if self.function_names.contains(name) {
            return Ok(LoweredExpr::GlobalAddress {
                name: name.to_owned(),
            });
        }
        Err(CompileError::new(format!(
            "unknown local or global: {name}"
        )))
    }

    fn lower_local_identifier_expr(&self, binding: &LocalBinding) -> CompileResult<LoweredExpr> {
        match binding {
            LocalBinding::Scalar {
                slot, scalar_type, ..
            } => Ok(LoweredExpr::Local {
                offset: self.local_offset(*slot)?,
                scalar_type: *scalar_type,
            }),
            LocalBinding::StaticScalar {
                global_name,
                scalar_type,
                ..
            } => Ok(LoweredExpr::Global {
                name: global_name.clone(),
                scalar_type: *scalar_type,
            }),
            LocalBinding::CharArray { slot, length } => Ok(LoweredExpr::LocalAddress {
                offset: self.local_offset(*slot)?,
                byte_size: *length,
            }),
            LocalBinding::CharMatrix {
                slot,
                rows,
                columns,
            } => Ok(LoweredExpr::LocalAddress {
                offset: self.local_offset(*slot)?,
                byte_size: local_char_matrix_byte_size(*rows, *columns)?,
            }),
            LocalBinding::IntArray { slot, length } => Ok(LoweredExpr::LocalAddress {
                offset: self.local_offset(*slot)?,
                byte_size: local_int_array_byte_size(*length)?,
            }),
            LocalBinding::ShortArray { slot, length, .. } => Ok(LoweredExpr::LocalAddress {
                offset: self.local_offset(*slot)?,
                byte_size: local_short_array_byte_size(*length)?,
            }),
            LocalBinding::PointerArray { slot, length } => Ok(LoweredExpr::LocalAddress {
                offset: self.local_offset(*slot)?,
                byte_size: local_pointer_array_byte_size(*length)?,
            }),
            LocalBinding::StructObject {
                slot, byte_size, ..
            } => Ok(LoweredExpr::LocalAddress {
                offset: self.local_offset(*slot)?,
                byte_size: *byte_size,
            }),
            LocalBinding::VaList { slot } => Ok(LoweredExpr::LocalAddress {
                offset: self.local_offset(*slot)?,
                byte_size: scalar_size(ScalarType::VaList),
            }),
        }
    }

    fn lower_assignment_expr(&self, target: &LValue, value: &Expr) -> CompileResult<LoweredExpr> {
        let target = self.lower_lvalue(target)?;
        Ok(LoweredExpr::Assign {
            target,
            value: Box::new(self.lower_expr(value)?),
        })
    }

    fn lower_conditional_expr(
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

    fn lower_binary_expr(
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
        Ok(LoweredExpr::Binary {
            op,
            left: Box::new(self.lower_expr(left)?),
            right: Box::new(self.lower_expr(right)?),
        })
    }

    fn lower_pointer_offset_expr(
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

    fn lower_pointer_difference_expr(
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

    fn lower_assignment(&mut self, target: &LValue, value: &Expr) -> CompileResult<()> {
        if let Some(struct_target) = self.resolve_struct_lvalue_address(target)? {
            let source = self.resolve_struct_address(value)?;
            if source.struct_name != struct_target.struct_name {
                return Err(CompileError::new("incompatible struct assignment"));
            }
            self.push_struct_copy(&struct_target, &source)?;
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
                        LocalBinding::StaticScalar {
                            global_name,
                            scalar_type,
                            ..
                        } => Ok(LoweredLValue::Global {
                            name: global_name,
                            scalar_type,
                        }),
                        LocalBinding::CharArray { .. }
                        | LocalBinding::CharMatrix { .. }
                        | LocalBinding::IntArray { .. }
                        | LocalBinding::ShortArray { .. } => Err(CompileError::new(
                            "assignment to local array is not supported",
                        )),
                        LocalBinding::PointerArray { .. } => Err(CompileError::new(
                            "assignment to local pointer array is not supported",
                        )),
                        LocalBinding::StructObject { .. } => Err(CompileError::new(
                            "direct assignment to local struct object is not supported",
                        )),
                        LocalBinding::VaList { .. } => {
                            Err(CompileError::new("assignment to va_list is not supported"))
                        }
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
                if let Some(LocalBinding::StructObject {
                    slot,
                    struct_name,
                    byte_size,
                }) = self.local_binding(name)
                {
                    return Ok(Some(StructAddress {
                        pointer: LoweredExpr::LocalAddress {
                            offset: self.local_offset(slot)?,
                            byte_size,
                        },
                        offset: 0,
                        struct_name,
                    }));
                }
                if let Some(GlobalBinding::StructObject { struct_name, .. }) =
                    self.global_bindings.get(name)
                {
                    return Ok(Some(StructAddress {
                        pointer: LoweredExpr::GlobalAddress { name: name.clone() },
                        offset: 0,
                        struct_name: struct_name.clone(),
                    }));
                }
                Ok(None)
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
            LValue::Subscript { array, index } => {
                if let Some(address) =
                    self.resolve_struct_array_field_subscript_address(array, index)?
                {
                    return Ok(Some(address));
                }
                if let Ok(address) = self.resolve_pointer_struct_subscript_address(array, index) {
                    return Ok(Some(address));
                }
                Ok(None)
            }
        }
    }

    fn lower_sizeof_expr(&self, expr: &Expr) -> CompileResult<LoweredExpr> {
        sizeof_expr::lower(self, expr)
    }

    fn lower_subscript(&self, array: &Expr, index: &Expr) -> CompileResult<LoweredExpr> {
        if let Some(subscript) = self.lower_global_array_subscript_expr(array, index)? {
            return Ok(subscript);
        }
        if let Some(subscript) = self.lower_field_array_subscript_expr(array, index)? {
            return Ok(subscript);
        }
        if let Some(subscript) = self.lower_struct_array_subscript_expr(array, index)? {
            return Ok(subscript);
        }
        if let Some(subscript) = self.lower_local_array_subscript_expr(array, index)? {
            return Ok(subscript);
        }
        self.lower_pointer_subscript_expr(array, index)
    }

    fn lower_subscript_lvalue(&self, array: &Expr, index: &Expr) -> CompileResult<LoweredLValue> {
        if let Some(subscript) = self.lower_global_array_subscript_lvalue(array, index)? {
            return Ok(subscript);
        }
        if let Some(subscript) = self.lower_field_array_subscript_lvalue(array, index)? {
            return Ok(subscript);
        }
        if let Some(subscript) = self.lower_local_array_subscript_lvalue(array, index)? {
            return Ok(subscript);
        }
        self.lower_pointer_subscript_lvalue(array, index)
    }

    fn lower_global_array_subscript_expr(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredExpr>> {
        if let Some(pointer) = self.resolve_global_int_matrix_row(array, index)? {
            return Ok(Some(pointer));
        }
        if let Some(pointer) = self.resolve_global_short_matrix_row(array, index)? {
            return Ok(Some(pointer));
        }
        if let Some(pointer) = self.resolve_global_byte_matrix_row(array, index)? {
            return Ok(Some(pointer));
        }
        if let Expr::Identifier(name) = array
            && self.global_bindings.get(name) == Some(&GlobalBinding::UnsignedCharArray)
        {
            return Ok(Some(LoweredExpr::GlobalByteSubscript {
                name: name.clone(),
                index: Box::new(self.lower_expr(index)?),
            }));
        }
        if let Expr::Identifier(name) = array
            && self.global_bindings.get(name) == Some(&GlobalBinding::IntArray)
        {
            return Ok(Some(LoweredExpr::GlobalIntSubscript {
                name: name.clone(),
                index: Box::new(self.lower_expr(index)?),
            }));
        }
        if let Some((pointer, element_unsigned)) = self.resolve_global_short_array(array) {
            return Ok(Some(Self::pointer_subscript_expr(
                pointer,
                self.lower_expr(index)?,
                ScalarType::Int,
                2,
                element_unsigned,
            )));
        }
        if let Some(pointer) = self.resolve_global_pointer_matrix_row(array, index)? {
            return Ok(Some(pointer));
        }
        if let Expr::Identifier(name) = array
            && matches!(
                self.global_bindings.get(name),
                Some(GlobalBinding::PointerArray { .. })
            )
        {
            return Ok(Some(LoweredExpr::GlobalPointerSubscript {
                name: name.clone(),
                index: Box::new(self.lower_expr(index)?),
            }));
        }
        Ok(None)
    }

    fn lower_field_array_subscript_expr(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredExpr>> {
        if let Some((pointer, element_type, element_byte_size, element_unsigned)) =
            self.resolve_array_field_subscript(array)?
        {
            return Ok(Some(Self::pointer_subscript_expr(
                pointer,
                self.lower_expr(index)?,
                element_type,
                element_byte_size,
                element_unsigned,
            )));
        }
        if let Some((pointer, flat_index, element_type, element_byte_size, element_unsigned)) =
            self.resolve_nested_array_field_subscript(array, index)?
        {
            return Ok(Some(Self::pointer_subscript_expr(
                pointer,
                flat_index,
                element_type,
                element_byte_size,
                element_unsigned,
            )));
        }
        Ok(None)
    }

    fn lower_struct_array_subscript_expr(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredExpr>> {
        if let Some(pointer) = self.resolve_global_struct_matrix_row(array, index)? {
            return Ok(Some(pointer));
        }
        if let Some(address) = self.resolve_global_struct_subscript_address(array, index)? {
            return Ok(Some(address.pointer));
        }
        self.resolve_local_char_matrix_row(array, index)
    }

    fn lower_local_array_subscript_expr(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredExpr>> {
        if let Some(pointer) = self.resolve_local_char_array(array)? {
            return Ok(Some(Self::pointer_subscript_expr(
                pointer,
                self.lower_expr(index)?,
                ScalarType::Int,
                1,
                true,
            )));
        }
        if let Some((pointer, element_unsigned)) = self.resolve_local_short_array(array)? {
            return Ok(Some(Self::pointer_subscript_expr(
                pointer,
                self.lower_expr(index)?,
                ScalarType::Int,
                2,
                element_unsigned,
            )));
        }
        if let Some(pointer) = self.resolve_local_pointer_array(array)? {
            return Ok(Some(Self::pointer_subscript_expr(
                pointer,
                self.lower_expr(index)?,
                ScalarType::Pointer,
                scalar_size(ScalarType::Pointer),
                false,
            )));
        }
        Ok(None)
    }

    fn lower_pointer_subscript_expr(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<LoweredExpr> {
        let pointer = self.lower_expr(array)?;
        if lowered_expr_scalar_type(&pointer) != Some(ScalarType::Pointer)
            && self.pointer_referent_for_expr(array).is_err()
        {
            return Err(CompileError::new(
                "only pointer and global byte-array subscripts are supported",
            ));
        }
        let (element_type, element_byte_size) = self.pointer_subscript_layout(array);
        Ok(Self::pointer_subscript_expr(
            pointer,
            self.lower_expr(index)?,
            element_type,
            element_byte_size,
            false,
        ))
    }

    fn lower_global_array_subscript_lvalue(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredLValue>> {
        if let Some(pointer) = self.resolve_global_byte_matrix_row(array, index)? {
            return Ok(Some(Self::pointer_subscript_lvalue(
                pointer,
                LoweredExpr::Integer(0),
                ScalarType::Int,
                1,
                true,
            )));
        }
        if let Some(pointer) = self.resolve_global_short_matrix_row(array, index)? {
            return Ok(Some(Self::pointer_subscript_lvalue(
                pointer,
                LoweredExpr::Integer(0),
                ScalarType::Int,
                2,
                false,
            )));
        }
        if let Expr::Identifier(name) = array
            && self.global_bindings.get(name) == Some(&GlobalBinding::UnsignedCharArray)
        {
            return Ok(Some(LoweredLValue::GlobalByteSubscript {
                name: name.clone(),
                index: Box::new(self.lower_expr(index)?),
            }));
        }
        if let Expr::Identifier(name) = array
            && self.global_bindings.get(name) == Some(&GlobalBinding::IntArray)
        {
            return Ok(Some(LoweredLValue::GlobalIntSubscript {
                name: name.clone(),
                index: Box::new(self.lower_expr(index)?),
            }));
        }
        if let Some((pointer, element_unsigned)) = self.resolve_global_short_array(array) {
            return Ok(Some(Self::pointer_subscript_lvalue(
                pointer,
                self.lower_expr(index)?,
                ScalarType::Int,
                2,
                element_unsigned,
            )));
        }
        if let Some(pointer) = self.resolve_global_pointer_matrix_row(array, index)? {
            return Ok(Some(Self::pointer_subscript_lvalue(
                pointer,
                LoweredExpr::Integer(0),
                ScalarType::Pointer,
                scalar_size(ScalarType::Pointer),
                false,
            )));
        }
        if let Expr::Identifier(name) = array
            && matches!(
                self.global_bindings.get(name),
                Some(GlobalBinding::PointerArray { .. })
            )
        {
            return Ok(Some(LoweredLValue::GlobalPointerSubscript {
                name: name.clone(),
                index: Box::new(self.lower_expr(index)?),
            }));
        }
        Ok(None)
    }

    fn lower_field_array_subscript_lvalue(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredLValue>> {
        if let Some((pointer, element_type, element_byte_size, element_unsigned)) =
            self.resolve_array_field_subscript(array)?
        {
            return Ok(Some(Self::pointer_subscript_lvalue(
                pointer,
                self.lower_expr(index)?,
                element_type,
                element_byte_size,
                element_unsigned,
            )));
        }
        if let Some((pointer, flat_index, element_type, element_byte_size, element_unsigned)) =
            self.resolve_nested_array_field_subscript(array, index)?
        {
            return Ok(Some(Self::pointer_subscript_lvalue(
                pointer,
                flat_index,
                element_type,
                element_byte_size,
                element_unsigned,
            )));
        }
        Ok(None)
    }

    fn lower_local_array_subscript_lvalue(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredLValue>> {
        if let Some(pointer) = self.resolve_local_char_array(array)? {
            return Ok(Some(Self::pointer_subscript_lvalue(
                pointer,
                self.lower_expr(index)?,
                ScalarType::Int,
                1,
                true,
            )));
        }
        if let Some((pointer, element_unsigned)) = self.resolve_local_short_array(array)? {
            return Ok(Some(Self::pointer_subscript_lvalue(
                pointer,
                self.lower_expr(index)?,
                ScalarType::Int,
                2,
                element_unsigned,
            )));
        }
        if let Some(pointer) = self.resolve_local_pointer_array(array)? {
            return Ok(Some(Self::pointer_subscript_lvalue(
                pointer,
                self.lower_expr(index)?,
                ScalarType::Pointer,
                scalar_size(ScalarType::Pointer),
                false,
            )));
        }
        Ok(None)
    }

    fn lower_pointer_subscript_lvalue(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<LoweredLValue> {
        let pointer = self.lower_expr(array)?;
        if lowered_expr_scalar_type(&pointer) != Some(ScalarType::Pointer)
            && self.pointer_referent_for_expr(array).is_err()
        {
            return Err(CompileError::new(
                "assignment to non-pointer subscript targets is not supported",
            ));
        }
        let (element_type, element_byte_size) = self.pointer_subscript_layout(array);
        Ok(Self::pointer_subscript_lvalue(
            pointer,
            self.lower_expr(index)?,
            element_type,
            element_byte_size,
            false,
        ))
    }

    fn pointer_subscript_expr(
        pointer: LoweredExpr,
        index: LoweredExpr,
        element_type: ScalarType,
        element_byte_size: usize,
        element_unsigned: bool,
    ) -> LoweredExpr {
        LoweredExpr::PointerSubscript {
            pointer: Box::new(pointer),
            index: Box::new(index),
            element_type,
            element_byte_size,
            element_unsigned,
        }
    }

    fn pointer_subscript_lvalue(
        pointer: LoweredExpr,
        index: LoweredExpr,
        element_type: ScalarType,
        element_byte_size: usize,
        element_unsigned: bool,
    ) -> LoweredLValue {
        LoweredLValue::PointerSubscript {
            pointer: Box::new(pointer),
            index: Box::new(index),
            element_type,
            element_byte_size,
            element_unsigned,
        }
    }

    fn pointer_subscript_layout(&self, array: &Expr) -> (ScalarType, usize) {
        let element_type = self.pointer_subscript_element_type(array);
        let element_byte_size = self
            .pointer_referent_for_expr(array)
            .ok()
            .and_then(|referent| pointer_arithmetic::byte_size(&referent))
            .unwrap_or_else(|| scalar_size(element_type));
        (element_type, element_byte_size)
    }

    fn pointer_subscript_element_type(&self, array: &Expr) -> ScalarType {
        if self
            .pointer_referent_for_expr(array)
            .is_ok_and(|referent| pointer_arithmetic::is_pointer(&referent))
        {
            ScalarType::Pointer
        } else {
            ScalarType::Int
        }
    }

    fn lower_address_of(&self, target: &LValue) -> CompileResult<LoweredExpr> {
        match target {
            LValue::Subscript { array, index } => self.lower_address_of_subscript(array, index),
            LValue::Identifier(name) => self.lower_address_of_identifier(name),
            LValue::Member {
                base,
                field,
                dereference,
            } => self.lower_address_of_member(base, field, *dereference),
        }
    }

    fn lower_address_of_subscript(&self, array: &Expr, index: &Expr) -> CompileResult<LoweredExpr> {
        if let Some(pointer) = self.resolve_global_int_matrix_row(array, index)? {
            return Ok(pointer);
        }
        if let Some(pointer) = self.resolve_global_short_matrix_row(array, index)? {
            return Ok(pointer);
        }
        if let Some(pointer) = self.resolve_global_byte_matrix_row(array, index)? {
            return Ok(pointer);
        }
        if let Some((pointer, _element_unsigned)) = self.resolve_global_short_array(array) {
            return Ok(LoweredExpr::PointerOffset {
                pointer: Box::new(pointer),
                index: Box::new(self.lower_expr(index)?),
                byte_size: 2,
            });
        }
        if let Some((pointer, _element_type, element_byte_size, _element_unsigned)) =
            self.resolve_array_field_subscript(array)?
        {
            return Ok(LoweredExpr::PointerOffset {
                pointer: Box::new(pointer),
                index: Box::new(self.lower_expr(index)?),
                byte_size: element_byte_size,
            });
        }
        if let Some((pointer, flat_index, _element_type, element_byte_size, _element_unsigned)) =
            self.resolve_nested_array_field_subscript(array, index)?
        {
            return Ok(LoweredExpr::PointerOffset {
                pointer: Box::new(pointer),
                index: Box::new(flat_index),
                byte_size: element_byte_size,
            });
        }
        if let Some(pointer) = self.resolve_local_char_array(array)? {
            return Ok(LoweredExpr::PointerOffset {
                pointer: Box::new(pointer),
                index: Box::new(self.lower_expr(index)?),
                byte_size: 1,
            });
        }
        if let Some((pointer, _element_unsigned)) = self.resolve_local_short_array(array)? {
            return Ok(LoweredExpr::PointerOffset {
                pointer: Box::new(pointer),
                index: Box::new(self.lower_expr(index)?),
                byte_size: 2,
            });
        }
        self.lower_address_of_struct_subscript(array, index)?
            .map_or_else(|| self.lower_address_of_pointer_subscript(array, index), Ok)
    }

    fn lower_address_of_struct_subscript(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredExpr>> {
        if let Some(address) = self.resolve_global_struct_subscript_address(array, index)? {
            return Ok(Some(address.pointer));
        }
        if let Some(address) = self.resolve_struct_array_field_subscript_address(array, index)? {
            return Ok(Some(address.pointer));
        }
        Ok(self
            .resolve_pointer_struct_subscript_address(array, index)
            .ok()
            .map(|address| address.pointer))
    }

    fn lower_address_of_pointer_subscript(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<LoweredExpr> {
        let pointer = self.lower_expr(array)?;
        if lowered_expr_scalar_type(&pointer) != Some(ScalarType::Pointer) {
            return Err(CompileError::new(
                "address of subscript requires a pointer base",
            ));
        }
        let (_element_type, element_byte_size) = self.pointer_subscript_layout(array);
        Ok(LoweredExpr::PointerOffset {
            pointer: Box::new(pointer),
            index: Box::new(self.lower_expr(index)?),
            byte_size: element_byte_size,
        })
    }

    fn lower_address_of_identifier(&self, name: &str) -> CompileResult<LoweredExpr> {
        let Some(binding) = self.local_binding(name) else {
            if self.global_bindings.contains_key(name) {
                return Ok(LoweredExpr::GlobalAddress {
                    name: name.to_owned(),
                });
            }
            return Err(CompileError::new("unsupported address-of target"));
        };
        self.lower_address_of_local_binding(&binding)
    }

    fn lower_address_of_local_binding(&self, binding: &LocalBinding) -> CompileResult<LoweredExpr> {
        let (slot, byte_size) = match binding {
            LocalBinding::StaticScalar { global_name, .. } => {
                return Ok(LoweredExpr::GlobalAddress {
                    name: global_name.clone(),
                });
            }
            LocalBinding::Scalar {
                slot, scalar_type, ..
            } => (*slot, scalar_size(*scalar_type)),
            LocalBinding::CharArray { slot, length } => (*slot, *length),
            LocalBinding::IntArray { slot, length } => (*slot, local_int_array_byte_size(*length)?),
            LocalBinding::ShortArray { slot, length, .. } => {
                (*slot, local_short_array_byte_size(*length)?)
            }
            LocalBinding::CharMatrix {
                slot,
                rows,
                columns,
            } => (*slot, local_char_matrix_byte_size(*rows, *columns)?),
            LocalBinding::PointerArray { slot, length } => {
                (*slot, local_pointer_array_byte_size(*length)?)
            }
            LocalBinding::StructObject {
                slot, byte_size, ..
            } => (*slot, *byte_size),
            LocalBinding::VaList { slot } => (*slot, scalar_size(ScalarType::VaList)),
        };
        Ok(LoweredExpr::LocalAddress {
            offset: self.local_offset(slot)?,
            byte_size,
        })
    }

    fn lower_address_of_member(
        &self,
        base: &Expr,
        field: &str,
        dereference: bool,
    ) -> CompileResult<LoweredExpr> {
        let member = self.resolve_member_access(base, field, dereference)?;
        Ok(pointer_field_address(member.pointer, member.offset))
    }

    fn lower_member_expr(
        &self,
        base: &Expr,
        field: &str,
        dereference: bool,
    ) -> CompileResult<LoweredExpr> {
        let member = self.resolve_member_access(base, field, dereference)?;
        let (scalar_type, byte_size, is_unsigned) = match member.field_type {
            FieldType::Scalar(field) => (field.scalar_type, field.byte_size, field.is_unsigned),
            FieldType::Pointer { .. } => {
                (ScalarType::Pointer, scalar_size(ScalarType::Pointer), false)
            }
            FieldType::Array { .. } | FieldType::StructArray { .. } => {
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
            byte_size,
            is_unsigned,
        })
    }

    fn lower_member_lvalue(
        &self,
        base: &Expr,
        field: &str,
        dereference: bool,
    ) -> CompileResult<LoweredLValue> {
        let member = self.resolve_member_access(base, field, dereference)?;
        let (scalar_type, byte_size, is_unsigned) = match member.field_type {
            FieldType::Scalar(field) => (field.scalar_type, field.byte_size, field.is_unsigned),
            FieldType::Pointer { .. } => {
                (ScalarType::Pointer, scalar_size(ScalarType::Pointer), false)
            }
            FieldType::Array { .. } | FieldType::StructArray { .. } => {
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
            byte_size,
            is_unsigned,
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
            if lowered_expr_scalar_type(&pointer) != Some(ScalarType::Pointer)
                && !self.expr_is_pointer_return_call(base)
            {
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
    ) -> CompileResult<Option<ArrayFieldSubscript>> {
        let Expr::Member {
            base,
            field,
            dereference,
        } = array
        else {
            return Ok(None);
        };
        let member = self.resolve_member_access(base, field, *dereference)?;
        let FieldType::Array {
            element_type,
            element_size,
            element_unsigned,
            ..
        } = member.field_type
        else {
            return Ok(None);
        };
        Ok(Some((
            pointer_field_address(member.pointer, member.offset),
            element_type,
            element_size,
            element_unsigned,
        )))
    }

    fn resolve_nested_array_field_subscript(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<NestedArrayFieldSubscript>> {
        let Expr::Subscript {
            array: nested_array,
            index: row_index,
        } = array
        else {
            return Ok(None);
        };
        let Expr::Member {
            base,
            field,
            dereference,
        } = nested_array.as_ref()
        else {
            return Ok(None);
        };
        let member = self.resolve_member_access(base, field, *dereference)?;
        let FieldType::Array {
            element_type,
            element_size,
            element_unsigned,
            columns: Some(columns),
            ..
        } = member.field_type
        else {
            return Ok(None);
        };
        let columns = i64::try_from(columns)
            .map_err(|_| CompileError::new("struct array column count does not fit i64"))?;
        let row_offset = LoweredExpr::Binary {
            op: BinaryOp::Mul,
            left: Box::new(self.lower_expr(row_index)?),
            right: Box::new(LoweredExpr::Integer(columns)),
        };
        let flat_index = LoweredExpr::Binary {
            op: BinaryOp::Add,
            left: Box::new(row_offset),
            right: Box::new(self.lower_expr(index)?),
        };
        Ok(Some((
            pointer_field_address(member.pointer, member.offset),
            flat_index,
            element_type,
            element_size,
            element_unsigned,
        )))
    }

    fn resolve_struct_array_field_subscript_address(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<StructAddress>> {
        let Expr::Member {
            base,
            field,
            dereference,
        } = array
        else {
            return Ok(None);
        };
        let member = self.resolve_member_access(base, field, *dereference)?;
        let FieldType::StructArray { struct_name, .. } = member.field_type else {
            return Ok(None);
        };
        let byte_size = self.struct_layout(&struct_name)?.size;
        Ok(Some(StructAddress {
            pointer: LoweredExpr::PointerOffset {
                pointer: Box::new(pointer_field_address(member.pointer, member.offset)),
                index: Box::new(self.lower_expr(index)?),
                byte_size,
            },
            offset: 0,
            struct_name,
        }))
    }

    fn resolve_local_pointer_array(&self, array: &Expr) -> CompileResult<Option<LoweredExpr>> {
        let Expr::Identifier(name) = array else {
            return Ok(None);
        };
        let Some(LocalBinding::PointerArray { slot, length }) = self.local_binding(name) else {
            return Ok(None);
        };
        Ok(Some(LoweredExpr::LocalAddress {
            offset: self.local_offset(slot)?,
            byte_size: local_pointer_array_byte_size(length)?,
        }))
    }

    fn resolve_local_short_array(
        &self,
        array: &Expr,
    ) -> CompileResult<Option<(LoweredExpr, bool)>> {
        let Expr::Identifier(name) = array else {
            return Ok(None);
        };
        let Some(LocalBinding::ShortArray {
            slot,
            length,
            is_unsigned,
        }) = self.local_binding(name)
        else {
            return Ok(None);
        };
        Ok(Some((
            LoweredExpr::LocalAddress {
                offset: self.local_offset(slot)?,
                byte_size: local_short_array_byte_size(length)?,
            },
            is_unsigned,
        )))
    }

    fn resolve_local_char_array(&self, array: &Expr) -> CompileResult<Option<LoweredExpr>> {
        let binding = if let Expr::Identifier(name) = array {
            self.local_binding(name)
        } else {
            None
        };
        local_array::char_array_pointer(array, binding.as_ref(), |slot| self.local_offset(slot))
    }

    fn resolve_global_short_array(&self, array: &Expr) -> Option<(LoweredExpr, bool)> {
        let Expr::Identifier(name) = array else {
            return None;
        };
        let Some(GlobalBinding::ShortArray {
            is_unsigned,
            columns: None,
        }) = self.global_bindings.get(name)
        else {
            return None;
        };
        Some((
            LoweredExpr::GlobalAddress { name: name.clone() },
            *is_unsigned,
        ))
    }

    fn resolve_local_char_matrix_row(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredExpr>> {
        let Expr::Identifier(name) = array else {
            return Ok(None);
        };
        let Some(LocalBinding::CharMatrix {
            slot,
            rows,
            columns,
        }) = self.local_binding(name)
        else {
            return Ok(None);
        };
        Ok(Some(LoweredExpr::PointerOffset {
            pointer: Box::new(LoweredExpr::LocalAddress {
                offset: self.local_offset(slot)?,
                byte_size: local_char_matrix_byte_size(rows, columns)?,
            }),
            index: Box::new(self.lower_expr(index)?),
            byte_size: columns,
        }))
    }

    fn resolve_global_byte_matrix_row(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredExpr>> {
        let Expr::Identifier(name) = array else {
            return Ok(None);
        };
        let Some(GlobalBinding::UnsignedCharMatrix { columns }) = self.global_bindings.get(name)
        else {
            return Ok(None);
        };
        Ok(Some(LoweredExpr::PointerOffset {
            pointer: Box::new(LoweredExpr::GlobalAddress { name: name.clone() }),
            index: Box::new(self.lower_expr(index)?),
            byte_size: *columns,
        }))
    }

    fn resolve_global_int_matrix_row(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredExpr>> {
        let Expr::Identifier(name) = array else {
            return Ok(None);
        };
        let Some(GlobalBinding::IntMatrix { columns }) = self.global_bindings.get(name) else {
            return Ok(None);
        };
        let byte_size = columns
            .checked_mul(scalar_size(ScalarType::Int))
            .ok_or_else(|| CompileError::new("global int matrix row size overflow"))?;
        Ok(Some(LoweredExpr::PointerOffset {
            pointer: Box::new(LoweredExpr::GlobalAddress { name: name.clone() }),
            index: Box::new(self.lower_expr(index)?),
            byte_size,
        }))
    }

    fn resolve_global_short_matrix_row(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredExpr>> {
        let Expr::Identifier(name) = array else {
            return Ok(None);
        };
        let Some(GlobalBinding::ShortArray {
            columns: Some(columns),
            ..
        }) = self.global_bindings.get(name)
        else {
            return Ok(None);
        };
        let byte_size = columns
            .checked_mul(2)
            .ok_or_else(|| CompileError::new("global short matrix row size overflow"))?;
        Ok(Some(LoweredExpr::PointerOffset {
            pointer: Box::new(LoweredExpr::GlobalAddress { name: name.clone() }),
            index: Box::new(self.lower_expr(index)?),
            byte_size,
        }))
    }

    fn resolve_global_pointer_matrix_row(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredExpr>> {
        let Expr::Identifier(name) = array else {
            return Ok(None);
        };
        let Some(GlobalBinding::PointerArray {
            columns: Some(columns),
            ..
        }) = self.global_bindings.get(name)
        else {
            return Ok(None);
        };
        let byte_size = columns
            .checked_mul(scalar_size(ScalarType::Pointer))
            .ok_or_else(|| CompileError::new("global pointer matrix row size overflow"))?;
        Ok(Some(LoweredExpr::PointerOffset {
            pointer: Box::new(LoweredExpr::GlobalAddress { name: name.clone() }),
            index: Box::new(self.lower_expr(index)?),
            byte_size,
        }))
    }

    fn resolve_global_struct_matrix_row(
        &self,
        array: &Expr,
        index: &Expr,
    ) -> CompileResult<Option<LoweredExpr>> {
        let Expr::Identifier(name) = array else {
            return Ok(None);
        };
        let Some(GlobalBinding::StructArray {
            byte_size,
            columns: Some(columns),
            ..
        }) = self.global_bindings.get(name)
        else {
            return Ok(None);
        };
        let row_size = columns
            .checked_mul(*byte_size)
            .ok_or_else(|| CompileError::new("global struct matrix row size overflow"))?;
        Ok(Some(LoweredExpr::PointerOffset {
            pointer: Box::new(LoweredExpr::GlobalAddress { name: name.clone() }),
            index: Box::new(self.lower_expr(index)?),
            byte_size: row_size,
        }))
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
        if let Expr::Dereference { pointer } = expr {
            let struct_name = self.pointer_referent_for_expr(pointer)?;
            return Ok(StructAddress {
                pointer: self.lower_expr(pointer)?,
                offset: 0,
                struct_name,
            });
        }
        if let Expr::Subscript { array, index } = expr {
            if let Some(address) = self.resolve_global_struct_subscript_address(array, index)? {
                return Ok(address);
            }
            if let Some(address) =
                self.resolve_struct_array_field_subscript_address(array, index)?
            {
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
            columns,
            ..
        }) = self.global_bindings.get(name)
        else {
            return Ok(None);
        };
        if columns.is_some() {
            return Ok(None);
        }
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

    fn pointer_referent_for_identifier(&self, name: &str) -> Option<String> {
        if let Some(binding) = self.local_binding(name) {
            return match binding {
                LocalBinding::Scalar {
                    referent: Some(referent),
                    ..
                }
                | LocalBinding::StaticScalar {
                    referent: Some(referent),
                    ..
                } => Some(referent),
                LocalBinding::ShortArray { .. } => Some("short".to_owned()),
                _ => None,
            };
        }
        if let Some(GlobalBinding::Pointer {
            referent: Some(referent),
        }) = self.global_bindings.get(name)
        {
            return Some(referent.clone());
        }
        if let Some(GlobalBinding::StructArray { struct_name, .. }) = self.global_bindings.get(name)
        {
            return Some(struct_name.clone());
        }
        if matches!(
            self.global_bindings.get(name),
            Some(GlobalBinding::ShortArray { .. })
        ) {
            return Some("short".to_owned());
        }
        None
    }

    fn pointer_referent_for_expr(&self, expr: &Expr) -> CompileResult<String> {
        pointer_referent::for_expr(self, expr)
    }

    fn pointer_referent_stride(&self, referent: &str) -> CompileResult<usize> {
        pointer_arithmetic::byte_size(referent)
            .or_else(|| self.structs.get(referent).map(|layout| layout.size))
            .ok_or_else(|| CompileError::new("unknown pointer arithmetic referent"))
    }

    fn pointer_difference_stride(
        &self,
        left_referent: &str,
        right_referent: &str,
    ) -> CompileResult<usize> {
        pointer_arithmetic::difference_stride(
            self.pointer_referent_stride(left_referent)?,
            self.pointer_referent_stride(right_referent)?,
        )
    }

    fn expr_is_pointer_return_call(&self, expr: &Expr) -> bool {
        matches!(
            expr,
            Expr::Call { callee, .. } if self.pointer_return_functions.contains_key(callee)
        )
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
                FieldType::Scalar(field) => self.push_struct_scalar_copy(
                    target,
                    source,
                    target_offset,
                    source_offset,
                    field.scalar_type,
                    field.byte_size,
                ),
                FieldType::Pointer { .. } => self.push_struct_scalar_copy(
                    target,
                    source,
                    target_offset,
                    source_offset,
                    ScalarType::Pointer,
                    scalar_size(ScalarType::Pointer),
                ),
                FieldType::Array {
                    element_type,
                    element_size,
                    length,
                    ..
                } => {
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
                            element_size,
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
                FieldType::StructArray {
                    struct_name,
                    length,
                } => {
                    let element_size = self.struct_layout(&struct_name)?.size;
                    for index in 0..length {
                        let element_offset = index
                            .checked_mul(element_size)
                            .and_then(|offset| target_offset.checked_add(offset))
                            .ok_or_else(|| CompileError::new("struct array offset overflow"))?;
                        let source_element_offset = index
                            .checked_mul(element_size)
                            .and_then(|offset| source_offset.checked_add(offset))
                            .ok_or_else(|| CompileError::new("struct array offset overflow"))?;
                        self.push_struct_copy(
                            &StructAddress {
                                pointer: target.pointer.clone(),
                                offset: element_offset,
                                struct_name: struct_name.clone(),
                            },
                            &StructAddress {
                                pointer: source.pointer.clone(),
                                offset: source_element_offset,
                                struct_name: struct_name.clone(),
                            },
                        )?;
                    }
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
        byte_size: usize,
    ) {
        self.push_store(
            LoweredLValue::PointerField {
                pointer: Box::new(target.pointer.clone()),
                offset: target_offset,
                scalar_type,
                byte_size,
                is_unsigned: false,
            },
            LoweredExpr::PointerField {
                pointer: Box::new(source.pointer.clone()),
                offset: source_offset,
                scalar_type,
                byte_size,
                is_unsigned: false,
            },
        );
    }

    fn lower_post_increment_statement(
        &mut self,
        target: &LValue,
        decrement: bool,
    ) -> CompileResult<()> {
        let lowered = self.lower_lvalue(target)?;
        ensure_post_increment_scalar(&lowered)?;
        let increment = self.post_increment_amount(target, &lowered, decrement)?;
        let current = lowered_lvalue_to_expr(&lowered);
        self.push_store(
            lowered,
            LoweredExpr::Binary {
                op: BinaryOp::Add,
                left: Box::new(current),
                right: Box::new(LoweredExpr::Integer(increment)),
            },
        );
        Ok(())
    }

    fn lower_post_increment_expr(
        &self,
        target: &LValue,
        decrement: bool,
    ) -> CompileResult<LoweredExpr> {
        let lowered = self.lower_lvalue(target)?;
        ensure_post_increment_scalar(&lowered)?;
        let increment = self.post_increment_amount(target, &lowered, decrement)?;
        Ok(LoweredExpr::PostIncrement {
            target: lowered,
            increment,
        })
    }

    fn post_increment_amount(
        &self,
        target: &LValue,
        lowered: &LoweredLValue,
        decrement: bool,
    ) -> CompileResult<i64> {
        let amount = if lowered_lvalue_scalar_type(lowered) == ScalarType::Pointer {
            self.pointer_referent_for_lvalue(target)?
                .map_or(Ok(1), |referent| {
                    let stride = self.pointer_referent_stride(&referent)?;
                    i64::try_from(stride)
                        .map_err(|_| CompileError::new("pointer stride does not fit i64"))
                })?
        } else {
            1
        };
        Ok(if decrement { -amount } else { amount })
    }

    fn pointer_referent_for_lvalue(&self, target: &LValue) -> CompileResult<Option<String>> {
        match target {
            LValue::Identifier(name) => Ok(self.pointer_referent_for_identifier(name)),
            LValue::Member {
                base,
                field,
                dereference,
            } => {
                let member = self.resolve_member_access(base, field, *dereference)?;
                if let FieldType::Pointer { referent } = member.field_type {
                    Ok(referent)
                } else {
                    Ok(None)
                }
            }
            LValue::Subscript { .. } => Ok(None),
        }
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
        | LoweredExpr::Call {
            return_type: scalar_type,
            ..
        }
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
        LoweredExpr::Assign { target, .. } | LoweredExpr::PostIncrement { target, .. } => {
            Some(lowered_lvalue_scalar_type(target))
        }
        LoweredExpr::Integer(_)
        | LoweredExpr::DoubleLiteral(_)
        | LoweredExpr::StringLiteral(_)
        | LoweredExpr::IndirectCall { .. }
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
            element_byte_size,
            element_unsigned,
        } => LoweredExpr::PointerSubscript {
            pointer: pointer.clone(),
            index: index.clone(),
            element_type: *element_type,
            element_byte_size: *element_byte_size,
            element_unsigned: *element_unsigned,
        },
        LoweredLValue::PointerField {
            pointer,
            offset,
            scalar_type,
            byte_size,
            is_unsigned,
        } => LoweredExpr::PointerField {
            pointer: pointer.clone(),
            offset: *offset,
            scalar_type: *scalar_type,
            byte_size: *byte_size,
            is_unsigned: *is_unsigned,
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
        ScalarType::Double | ScalarType::Pointer | ScalarType::VaList => Err(CompileError::new(
            "non-integer cast is not an integer constant expression",
        )),
    }
}

fn zero_expr_for(scalar_type: ScalarType) -> LoweredExpr {
    match scalar_type {
        ScalarType::Double => LoweredExpr::DoubleLiteral("0.0".to_string()),
        ScalarType::Int | ScalarType::LongLong | ScalarType::Pointer | ScalarType::VaList => {
            LoweredExpr::Integer(0)
        }
    }
}

fn local_char_array_initializer_values(
    initializer: &LocalCharArrayInitializer,
    length: usize,
) -> CompileResult<Vec<u8>> {
    match initializer {
        LocalCharArrayInitializer::StringLiteral(value) => {
            local_char_array_string_initializer_values(value, length)
        }
        LocalCharArrayInitializer::Bytes(values) => {
            local_char_array_braced_initializer_values(values, length)
        }
    }
}

fn local_char_array_string_initializer_values(
    value: &str,
    length: usize,
) -> CompileResult<Vec<u8>> {
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

fn local_char_array_braced_initializer_values(
    values: &[u8],
    length: usize,
) -> CompileResult<Vec<u8>> {
    if values.len() > length {
        return Err(CompileError::new(
            "local char array initializer is too large",
        ));
    }
    let mut values = values.to_vec();
    values.resize(length, 0);
    Ok(values)
}

fn local_char_matrix_initializer_values(
    values: &[String],
    rows: usize,
    columns: usize,
) -> CompileResult<Vec<u8>> {
    if values.len() > rows {
        return Err(CompileError::new(
            "local char matrix initializer has too many rows",
        ));
    }
    let mut bytes = Vec::with_capacity(local_char_matrix_byte_size(rows, columns)?);
    for value in values {
        bytes.extend(local_char_array_string_initializer_values(value, columns)?);
    }
    bytes.resize(local_char_matrix_byte_size(rows, columns)?, 0);
    Ok(bytes)
}

fn local_char_matrix_byte_size(rows: usize, columns: usize) -> CompileResult<usize> {
    rows.checked_mul(columns)
        .ok_or_else(|| CompileError::new("local char matrix size overflow"))
}

fn local_int_array_byte_size(length: usize) -> CompileResult<usize> {
    length
        .checked_mul(scalar_size(ScalarType::Int))
        .ok_or_else(|| CompileError::new("local int array size overflow"))
}

fn local_short_array_byte_size(length: usize) -> CompileResult<usize> {
    length
        .checked_mul(2)
        .ok_or_else(|| CompileError::new("local short array size overflow"))
}

fn local_pointer_array_byte_size(length: usize) -> CompileResult<usize> {
    length
        .checked_mul(scalar_size(ScalarType::Pointer))
        .ok_or_else(|| CompileError::new("local pointer array size overflow"))
}

fn struct_alignment(layout: &StructLayout) -> usize {
    layout.size.clamp(1, 8)
}

const fn scalar_size(scalar_type: ScalarType) -> usize {
    match scalar_type {
        ScalarType::Int => 4,
        ScalarType::LongLong | ScalarType::Double | ScalarType::Pointer => 8,
        ScalarType::VaList => 24,
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
        LoweredExpr::Call { callee, args, .. } => {
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
        LoweredExpr::IndirectCall { callee, args } => {
            inline_constant_calls_in_expr(callee, constants);
            for arg in args {
                inline_constant_calls_in_expr(arg, constants);
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
