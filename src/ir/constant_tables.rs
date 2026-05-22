use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Constant, Function, PointerReturnFunction, ReturnType, ScalarType};
use std::collections::{HashMap, HashSet};

pub(in crate::ir) fn lower_constants(
    constants: &[Constant],
) -> CompileResult<HashMap<String, i64>> {
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

pub(in crate::ir) fn lower_pointer_return_functions(
    functions: &[PointerReturnFunction],
) -> HashMap<String, Option<String>> {
    functions
        .iter()
        .map(|function| (function.name.clone(), function.referent.clone()))
        .collect()
}

pub(in crate::ir) fn lower_function_names(
    functions: &[Function],
    function_prototypes: &[String],
) -> HashSet<String> {
    functions
        .iter()
        .map(|function| function.name.clone())
        .chain(function_prototypes.iter().cloned())
        .collect()
}

pub(in crate::ir) fn lower_function_return_types(
    functions: &[Function],
) -> HashMap<String, ScalarType> {
    functions
        .iter()
        .filter_map(|function| {
            function_return_type(function.return_type)
                .map(|return_type| (function.name.clone(), return_type))
        })
        .collect()
}

const fn function_return_type(return_type: ReturnType) -> Option<ScalarType> {
    match return_type {
        ReturnType::Int => Some(ScalarType::Int),
        ReturnType::Pointer => Some(ScalarType::Pointer),
        ReturnType::Double => Some(ScalarType::Double),
        ReturnType::Void => None,
    }
}
