use std::collections::HashMap;

use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{BinaryOp, Expr, LValue, ScalarType};

use super::{LoweredGlobalInitializer, pointer_arithmetic, scalar_size, static_local};

#[derive(Clone, Copy)]
struct AddressConstant<'a> {
    base: &'a str,
    index: i64,
}

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
        Expr::Cast { expr, .. } => initializer(Some(expr), constants, referent),
        _ => {
            if let Some(address) = address_constant(expr, constants)? {
                return global_offset(address.base, address.index, referent);
            }
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

fn address_constant<'a>(
    expr: &'a Expr,
    constants: &HashMap<String, i64>,
) -> CompileResult<Option<AddressConstant<'a>>> {
    match expr {
        Expr::Identifier(name) if !constants.contains_key(name) => Ok(Some(AddressConstant {
            base: name,
            index: 0,
        })),
        Expr::AddressOf {
            target: LValue::Identifier(name),
        } => Ok(Some(AddressConstant {
            base: name,
            index: 0,
        })),
        Expr::AddressOf {
            target: LValue::Subscript { array, index },
        } => subscript_address_constant(array, index, constants),
        Expr::Cast { expr, .. } => address_constant(expr, constants),
        Expr::Binary {
            op: BinaryOp::Add,
            left,
            right,
        } => add_address_constant(left, right, constants).and_then(|address| {
            address.map_or_else(
                || add_address_constant(right, left, constants),
                |address| Ok(Some(address)),
            )
        }),
        Expr::Binary {
            op: BinaryOp::Sub,
            left,
            right,
        } => subtract_address_constant(left, right, constants),
        _ => Ok(None),
    }
}

fn subscript_address_constant<'a>(
    array: &'a Expr,
    index: &Expr,
    constants: &HashMap<String, i64>,
) -> CompileResult<Option<AddressConstant<'a>>> {
    let Expr::Identifier(base) = array else {
        return Ok(None);
    };
    if constants.contains_key(base) {
        return Ok(None);
    }
    Ok(Some(AddressConstant {
        base,
        index: static_local::eval_with_constants(index, constants)?,
    }))
}

fn add_address_constant<'a>(
    base: &'a Expr,
    index: &Expr,
    constants: &HashMap<String, i64>,
) -> CompileResult<Option<AddressConstant<'a>>> {
    address_constant(base, constants)?
        .map(|address| {
            offset_address(
                address,
                static_local::eval_with_constants(index, constants)?,
            )
        })
        .transpose()
}

fn subtract_address_constant<'a>(
    base: &'a Expr,
    index: &Expr,
    constants: &HashMap<String, i64>,
) -> CompileResult<Option<AddressConstant<'a>>> {
    let offset = static_local::eval_with_constants(index, constants)?
        .checked_neg()
        .ok_or_else(|| CompileError::new("static local pointer offset overflow"))?;
    address_constant(base, constants)?
        .map(|address| offset_address(address, offset))
        .transpose()
}

fn offset_address(address: AddressConstant<'_>, offset: i64) -> CompileResult<AddressConstant<'_>> {
    Ok(AddressConstant {
        base: address.base,
        index: address
            .index
            .checked_add(offset)
            .ok_or_else(|| CompileError::new("static local pointer offset overflow"))?,
    })
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
