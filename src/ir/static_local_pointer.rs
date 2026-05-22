use std::collections::HashMap;

use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{BinaryOp, Expr, LValue, ScalarType};

use super::{LoweredGlobalInitializer, pointer_arithmetic, scalar_size, static_local};

pub(in crate::ir) fn initializer(
    initializer_expr: Option<&Expr>,
    constants: &HashMap<String, i64>,
    referent: Option<&str>,
) -> CompileResult<LoweredGlobalInitializer> {
    let Some(expr) = initializer_expr else {
        return Ok(LoweredGlobalInitializer::PointerNull);
    };
    match expr {
        Expr::Integer(0) | Expr::LongInteger(0) => Ok(LoweredGlobalInitializer::PointerNull),
        Expr::Identifier(name) => identifier_initializer(name, constants),
        Expr::StringLiteral(value) => Ok(LoweredGlobalInitializer::PointerString(value.clone())),
        Expr::AddressOf {
            target: LValue::Identifier(name),
        } => Ok(global_offset(name, 0, None)?),
        Expr::AddressOf {
            target: LValue::Subscript { array, index },
        } => {
            if let Some(initializer) = subscript_initializer(array, index, constants, referent)? {
                return Ok(initializer);
            }
            Err(pointer_error())
        }
        Expr::Cast { expr, .. } => initializer(Some(expr), constants, referent),
        Expr::Binary {
            op: BinaryOp::Add,
            left,
            right,
        } => decay_initializer(left, right, constants, referent)
            .and_then(|value| {
                value.map_or_else(
                    || decay_initializer(right, left, constants, referent),
                    |initializer| Ok(Some(initializer)),
                )
            })?
            .ok_or_else(pointer_error),
        _ => {
            let value = static_local::eval_with_constants(expr, constants)?;
            if value == 0 {
                return Ok(LoweredGlobalInitializer::PointerNull);
            }
            Err(pointer_error())
        }
    }
}

fn identifier_initializer(
    name: &str,
    constants: &HashMap<String, i64>,
) -> CompileResult<LoweredGlobalInitializer> {
    if let Some(value) = constants.get(name) {
        if *value == 0 {
            return Ok(LoweredGlobalInitializer::PointerNull);
        }
        return Err(pointer_error());
    }
    global_offset(name, 0, None)
}

fn subscript_initializer(
    array: &Expr,
    index: &Expr,
    constants: &HashMap<String, i64>,
    referent: Option<&str>,
) -> CompileResult<Option<LoweredGlobalInitializer>> {
    let Expr::Identifier(base) = array else {
        return Ok(None);
    };
    if constants.contains_key(base) {
        return Ok(None);
    }
    global_offset(
        base,
        static_local::eval_with_constants(index, constants)?,
        referent,
    )
    .map(Some)
}

fn decay_initializer(
    base: &Expr,
    index: &Expr,
    constants: &HashMap<String, i64>,
    referent: Option<&str>,
) -> CompileResult<Option<LoweredGlobalInitializer>> {
    let Expr::Identifier(base) = base else {
        return Ok(None);
    };
    if constants.contains_key(base) {
        return Ok(None);
    }
    global_offset(
        base,
        static_local::eval_with_constants(index, constants)?,
        referent,
    )
    .map(Some)
}

fn global_offset(
    base: &str,
    index: i64,
    referent: Option<&str>,
) -> CompileResult<LoweredGlobalInitializer> {
    let index = usize::try_from(index)
        .map_err(|_| CompileError::new("static local pointer offset must be nonnegative"))?;
    let stride = referent
        .and_then(pointer_arithmetic::byte_size)
        .unwrap_or_else(|| scalar_size(ScalarType::Int));
    let byte_offset = index
        .checked_mul(stride)
        .ok_or_else(|| CompileError::new("static local pointer offset overflow"))?;
    Ok(LoweredGlobalInitializer::PointerGlobalOffset {
        base: base.to_owned(),
        byte_offset,
    })
}

fn pointer_error() -> CompileError {
    CompileError::new("static local pointer initializer must be null")
}
