use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{
    Constant, Function, FunctionPrototype, PointerReturnFunction, ReturnType, ScalarType,
};
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
    function_prototypes: &[FunctionPrototype],
) -> HashSet<String> {
    functions
        .iter()
        .map(|function| function.name.clone())
        .chain(
            function_prototypes
                .iter()
                .map(|prototype| prototype.name.clone()),
        )
        .collect()
}

pub(in crate::ir) fn lower_function_return_types(
    functions: &[Function],
    function_prototypes: &[FunctionPrototype],
) -> HashMap<String, ScalarType> {
    function_prototypes
        .iter()
        .filter_map(|prototype| {
            function_return_type(prototype.return_type)
                .map(|return_type| (prototype.name.clone(), return_type))
        })
        .chain(functions.iter().filter_map(|function| {
            function_return_type(function.return_type)
                .map(|return_type| (function.name.clone(), return_type))
        }))
        .collect()
}

const fn function_return_type(return_type: ReturnType) -> Option<ScalarType> {
    match return_type {
        ReturnType::Int => Some(ScalarType::Int),
        ReturnType::Pointer => Some(ScalarType::Pointer),
        ReturnType::ComplexFloat => Some(ScalarType::ComplexFloat),
        ReturnType::ComplexDouble => Some(ScalarType::ComplexDouble),
        ReturnType::ComplexLongDouble => Some(ScalarType::ComplexLongDouble),
        ReturnType::Double => Some(ScalarType::Double),
        ReturnType::LongDouble => Some(ScalarType::LongDouble),
        ReturnType::Void => None,
    }
}
