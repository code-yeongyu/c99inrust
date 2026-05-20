use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Constant, Function, PointerReturnFunction};
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
