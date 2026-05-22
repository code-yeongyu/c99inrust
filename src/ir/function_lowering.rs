use super::{
    GlobalBinding, Instruction, LoweredFunction, LoweredGlobal, LoweringContext,
    LoweringContextInputs, VARIADIC_REGISTER_SAVE_BYTES, scalar_size,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Function, ReturnType, ScalarType, StructLayout};
use std::collections::{HashMap, HashSet};

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
    let function_return_types = HashMap::new();
    let function_names = HashSet::new();
    lower_function_with_globals(
        function,
        &structs,
        &global_bindings,
        &constants,
        &pointer_return_functions,
        &function_return_types,
        &function_names,
    )
    .map(|lowered| lowered.function)
}

pub(in crate::ir) struct LoweredFunctionWithStatics {
    pub(in crate::ir) function: LoweredFunction,
    pub(in crate::ir) static_globals: Vec<LoweredGlobal>,
}

pub(in crate::ir) fn lower_function_with_globals(
    function: &Function,
    structs: &HashMap<String, StructLayout>,
    global_bindings: &HashMap<String, GlobalBinding>,
    constants: &HashMap<String, i64>,
    pointer_return_functions: &HashMap<String, Option<String>>,
    function_return_types: &HashMap<String, ScalarType>,
    function_names: &HashSet<String>,
) -> CompileResult<LoweredFunctionWithStatics> {
    let mut context = LoweringContext::new(
        &function.name,
        function.return_type,
        LoweringContextInputs {
            structs,
            global_bindings,
            constants,
            pointer_return_functions,
            function_return_types,
            function_names,
        },
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
            VARIADIC_REGISTER_SAVE_BYTES,
            scalar_size(ScalarType::Pointer),
        )?)
    } else {
        None
    };
    for statement in &function.statements {
        context.lower_statement(statement)?;
    }
    if matches!(
        function.return_type,
        ReturnType::Int | ReturnType::Pointer | ReturnType::Double
    ) && !context.has_return
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
pub(in crate::ir) const fn ends_with_return(instructions: &[Instruction]) -> bool {
    matches!(instructions.last(), Some(Instruction::Return(_)))
}
