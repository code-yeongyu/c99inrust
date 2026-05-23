use std::collections::HashMap;

use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{BinaryOp, Expr, LValue, StructLayout};

use super::{
    GlobalBinding, LoweredGlobalInitializer, global_address_offsets, static_local,
    static_member_addresses, static_pointer_address_values,
};
use static_pointer_address_values::AddressConstant;

pub(in crate::ir) fn initializer(
    initializer_expr: Option<&Expr>,
    constants: &HashMap<String, i64>,
    referent: Option<&str>,
    structs: &HashMap<String, StructLayout>,
    global_bindings: &HashMap<String, GlobalBinding>,
) -> CompileResult<LoweredGlobalInitializer> {
    let Some(expr) = initializer_expr else {
        return Ok(LoweredGlobalInitializer::PointerNull);
    };
    match expr {
        Expr::Integer(0) | Expr::LongInteger(0) => Ok(LoweredGlobalInitializer::PointerNull),
        Expr::Identifier(name) => identifier_initializer(name, constants),
        Expr::StringLiteral(value) => Ok(LoweredGlobalInitializer::PointerString(value.clone(), 0)),
        Expr::Cast { expr, .. } => {
            initializer(Some(expr), constants, referent, structs, global_bindings)
        }
        _ => {
            if let Some(address) =
                address_constant(expr, constants, referent, structs, global_bindings)?
            {
                return static_pointer_address_values::initializer(address);
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
    static_pointer_address_values::initializer(static_pointer_address_values::global(name, 0))
}

fn address_constant(
    expr: &Expr,
    constants: &HashMap<String, i64>,
    referent: Option<&str>,
    structs: &HashMap<String, StructLayout>,
    global_bindings: &HashMap<String, GlobalBinding>,
) -> CompileResult<Option<AddressConstant>> {
    match expr {
        Expr::Identifier(name) if !constants.contains_key(name) => {
            Ok(Some(static_pointer_address_values::global(name, 0)))
        }
        Expr::StringLiteral(value) => Ok(Some(static_pointer_address_values::string(value, 0))),
        Expr::AddressOf {
            target: LValue::Identifier(name),
        } => Ok(Some(static_pointer_address_values::global(name, 0))),
        Expr::AddressOf {
            target: target @ LValue::Subscript { array, index },
        } => subscript_address_constant(array, index, constants, referent, structs).and_then(
            |address| {
                address.map_or_else(
                    || {
                        member_address_constant(
                            target,
                            referent,
                            constants,
                            structs,
                            global_bindings,
                        )
                    },
                    |address| Ok(Some(address)),
                )
            },
        ),
        Expr::AddressOf { target } => {
            member_address_constant(target, referent, constants, structs, global_bindings)
        }
        Expr::Cast { expr, .. } => {
            address_constant(expr, constants, referent, structs, global_bindings)
        }
        Expr::Binary {
            op: BinaryOp::Add,
            left,
            right,
        } => add_address_constant(left, right, constants, referent, structs, global_bindings)
            .and_then(|address| {
                address.map_or_else(
                    || {
                        add_address_constant(
                            right,
                            left,
                            constants,
                            referent,
                            structs,
                            global_bindings,
                        )
                    },
                    |address| Ok(Some(address)),
                )
            }),
        Expr::Binary {
            op: BinaryOp::Sub,
            left,
            right,
        } => subtract_address_constant(left, right, constants, referent, structs, global_bindings),
        _ => Ok(None),
    }
}

fn subscript_address_constant(
    array: &Expr,
    index: &Expr,
    constants: &HashMap<String, i64>,
    referent: Option<&str>,
    structs: &HashMap<String, StructLayout>,
) -> CompileResult<Option<AddressConstant>> {
    let offset = static_local::eval_with_constants(index, constants)?;
    match array {
        Expr::Identifier(base) if !constants.contains_key(base) => {
            Ok(Some(static_pointer_address_values::global(
                base,
                static_pointer_address_values::scaled_offset(offset, referent, structs)?,
            )))
        }
        Expr::StringLiteral(value) => {
            Ok(Some(static_pointer_address_values::string(value, offset)))
        }
        _ => Ok(None),
    }
}

fn member_address_constant(
    target: &LValue,
    referent: Option<&str>,
    constants: &HashMap<String, i64>,
    structs: &HashMap<String, StructLayout>,
    global_bindings: &HashMap<String, GlobalBinding>,
) -> CompileResult<Option<AddressConstant>> {
    let Some(address) = static_member_addresses::from_lvalue(target, constants)? else {
        return Ok(None);
    };
    let (base, byte_offset) =
        global_address_offsets::resolve(referent, &address, structs, global_bindings)?;
    let byte_offset = i64::try_from(byte_offset)
        .map_err(|_| CompileError::new("static local member pointer offset is too large"))?;
    Ok(Some(AddressConstant::Global { base, byte_offset }))
}

fn add_address_constant(
    base: &Expr,
    index: &Expr,
    constants: &HashMap<String, i64>,
    referent: Option<&str>,
    structs: &HashMap<String, StructLayout>,
    global_bindings: &HashMap<String, GlobalBinding>,
) -> CompileResult<Option<AddressConstant>> {
    address_constant(base, constants, referent, structs, global_bindings)?
        .map(|address| {
            static_pointer_address_values::offset(
                address,
                static_local::eval_with_constants(index, constants)?,
                referent,
                structs,
            )
        })
        .transpose()
}

fn subtract_address_constant(
    base: &Expr,
    index: &Expr,
    constants: &HashMap<String, i64>,
    referent: Option<&str>,
    structs: &HashMap<String, StructLayout>,
    global_bindings: &HashMap<String, GlobalBinding>,
) -> CompileResult<Option<AddressConstant>> {
    let offset = static_local::eval_with_constants(index, constants)?
        .checked_neg()
        .ok_or_else(|| CompileError::new("static local pointer offset overflow"))?;
    address_constant(base, constants, referent, structs, global_bindings)?
        .map(|address| static_pointer_address_values::offset(address, offset, referent, structs))
        .transpose()
}

fn pointer_error() -> CompileError {
    CompileError::new("static local pointer initializer must be null")
}
