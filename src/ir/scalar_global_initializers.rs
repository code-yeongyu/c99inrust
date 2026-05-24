use super::{GlobalBinding, LoweredGlobalInitializer, scalar_size};
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Global, GlobalInitializer, ScalarType};
use std::collections::HashMap;

pub(in crate::ir) fn lower_scalar_global_initializer(
    global: &Global,
    constants: &HashMap<String, i64>,
) -> CompileResult<Option<(LoweredGlobalInitializer, GlobalBinding)>> {
    match &global.initializer {
        GlobalInitializer::Int(value) => Ok(Some((
            LoweredGlobalInitializer::Int(i32::try_from(*value).map_err(|_| {
                CompileError::new(format!(
                    "global int initializer does not fit i32: {}",
                    global.name
                ))
            })?),
            GlobalBinding::Int,
        ))),
        GlobalInitializer::Bool(value) => Ok(Some((
            LoweredGlobalInitializer::UnsignedCharArray(vec![u8::from(*value != 0)]),
            GlobalBinding::Scalar(ScalarType::Bool),
        ))),
        GlobalInitializer::LongLong(value) => Ok(Some(lower_long_long_global_initializer(*value))),
        GlobalInitializer::Double(value) => Ok(Some((
            LoweredGlobalInitializer::Double(value.clone()),
            GlobalBinding::Scalar(ScalarType::Double),
        ))),
        GlobalInitializer::ComplexReal { scalar_type, real } => Ok(Some((
            LoweredGlobalInitializer::RealThenZero {
                real: real.clone(),
                byte_len: scalar_size(*scalar_type),
            },
            GlobalBinding::Scalar(*scalar_type),
        ))),
        GlobalInitializer::ScalarZero(scalar_type) => Ok(Some((
            LoweredGlobalInitializer::ZeroBytes(scalar_size(*scalar_type)),
            GlobalBinding::Scalar(*scalar_type),
        ))),
        GlobalInitializer::IntConstant(name) => {
            lower_int_constant_global(name, &global.name, constants).map(Some)
        }
        _ => Ok(None),
    }
}

const fn lower_long_long_global_initializer(
    value: i64,
) -> (LoweredGlobalInitializer, GlobalBinding) {
    (
        LoweredGlobalInitializer::LongLong(value),
        GlobalBinding::LongLong,
    )
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
