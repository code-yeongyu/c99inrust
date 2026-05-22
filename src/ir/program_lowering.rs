use super::{
    LoweredProgram, constant_return_functions, inline_constant_calls, lower_constants,
    lower_function_names, lower_function_return_types, lower_function_with_globals, lower_globals,
    lower_pointer_return_functions,
};
use crate::diagnostics::CompileResult;
use crate::parser::Program;
use std::collections::HashMap;

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
    let function_return_types = lower_function_return_types(&program.functions);
    let function_names = lower_function_names(&program.functions, &program.function_prototypes);
    let mut functions = Vec::with_capacity(program.functions.len());
    for function in &program.functions {
        let lowered = lower_function_with_globals(
            function,
            &structs,
            &global_bindings,
            &constants,
            &pointer_return_functions,
            &function_return_types,
            &function_names,
        )?;
        globals.extend(lowered.static_globals);
        functions.push(lowered.function);
    }
    let constant_returns = constant_return_functions(&functions);
    inline_constant_calls(&mut functions, &constant_returns);
    Ok(LoweredProgram { globals, functions })
}
