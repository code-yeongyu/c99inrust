use crate::diagnostics::{CompileError, CompileResult};

use super::{
    BinaryOp, Constant, Expr, GlobalStructInitializerAddress, LValue, ScalarType,
    eval_integer_initializer_expr_with_constants,
};

pub(super) fn constant_value(name: &str, constants: &[Constant]) -> Option<i64> {
    constants
        .iter()
        .rev()
        .find(|constant| constant.name == name)
        .map(|constant| constant.value)
}

pub(super) fn address_from_expr(
    expr: &Expr,
    constants: &[Constant],
) -> CompileResult<Option<GlobalStructInitializerAddress>> {
    match expr {
        Expr::Cast { expr, .. } => address_from_expr(expr, constants),
        Expr::Identifier(name) => {
            Ok(constant_value(name, constants)
                .is_none()
                .then(|| GlobalStructInitializerAddress {
                    base: name.clone(),
                    index: None,
                }))
        }
        Expr::Subscript { array, index } => {
            address_from_subscript(array, index, constants).map(Some)
        }
        Expr::AddressOf { target } => address_from_lvalue(target, constants).map(Some),
        Expr::Binary {
            op: BinaryOp::Add,
            left,
            right,
        } => offset_address_expr(left, right, constants, 1).and_then(|address| {
            address.map_or_else(
                || offset_address_expr(right, left, constants, 1),
                |address| Ok(Some(address)),
            )
        }),
        Expr::Binary {
            op: BinaryOp::Sub,
            left,
            right,
        } => offset_address_expr(left, right, constants, -1),
        _ => Ok(None),
    }
}

pub(super) fn string_pointer_from_expr(
    expr: &Expr,
    constants: &[Constant],
) -> CompileResult<Option<(String, usize, Option<ScalarType>)>> {
    match expr {
        Expr::Cast { target, expr, .. } => string_pointer_from_expr(expr, constants).map(|value| {
            value.map(|(value, byte_offset, _cast_target)| (value, byte_offset, Some(*target)))
        }),
        Expr::StringLiteral(value) => Ok(Some((value.clone(), 0, None))),
        Expr::AddressOf {
            target: LValue::Subscript { array, index },
        } => string_subscript_address(array, index, constants),
        Expr::Binary {
            op: BinaryOp::Add,
            left,
            right,
        } => offset_string_expr(left, right, constants, 1).and_then(|value| {
            value.map_or_else(
                || offset_string_expr(right, left, constants, 1),
                |value| Ok(Some(value)),
            )
        }),
        Expr::Binary {
            op: BinaryOp::Sub,
            left,
            right,
        } => offset_string_expr(left, right, constants, -1),
        _ => Ok(None),
    }
}

pub(super) fn address_from_lvalue(
    target: &LValue,
    constants: &[Constant],
) -> CompileResult<GlobalStructInitializerAddress> {
    match target {
        LValue::Identifier(base) => Ok(GlobalStructInitializerAddress {
            base: base.clone(),
            index: None,
        }),
        LValue::Subscript { array, index } => address_from_subscript(array, index, constants),
        LValue::Member { .. }
        | LValue::ScalarCompoundLiteral { .. }
        | LValue::StructCompoundLiteral { .. } => Err(CompileError::new(
            "unsupported global struct initializer address",
        )),
    }
}

pub(super) fn address_from_subscript(
    array: &Expr,
    index: &Expr,
    constants: &[Constant],
) -> CompileResult<GlobalStructInitializerAddress> {
    let Expr::Identifier(base) = array else {
        return Err(CompileError::new(
            "unsupported global struct initializer address",
        ));
    };
    let index = eval_integer_initializer_expr_with_constants(index, constants)?.to_i64_trunc()?;
    Ok(GlobalStructInitializerAddress {
        base: base.clone(),
        index: Some(usize::try_from(index).map_err(|_| {
            CompileError::new("global struct initializer address index is negative")
        })?),
    })
}

fn string_subscript_address(
    array: &Expr,
    index: &Expr,
    constants: &[Constant],
) -> CompileResult<Option<(String, usize, Option<ScalarType>)>> {
    let Expr::StringLiteral(value) = array else {
        return Ok(None);
    };
    let index = eval_integer_initializer_expr_with_constants(index, constants)?.to_i64_trunc()?;
    Ok(Some((
        value.clone(),
        usize::try_from(index)
            .map_err(|_| CompileError::new("global struct string pointer offset is negative"))?,
        None,
    )))
}

fn offset_address_expr(
    address_expr: &Expr,
    offset_expr: &Expr,
    constants: &[Constant],
    direction: i64,
) -> CompileResult<Option<GlobalStructInitializerAddress>> {
    let Some(mut address) = address_from_expr(address_expr, constants)? else {
        return Ok(None);
    };
    let offset = eval_integer_initializer_expr_with_constants(offset_expr, constants)?
        .to_i64_trunc()?
        .checked_mul(direction)
        .ok_or_else(|| CompileError::new("global struct initializer address overflow"))?;
    add_address_offset(&mut address, offset)?;
    Ok(Some(address))
}

fn offset_string_expr(
    string_expr: &Expr,
    offset_expr: &Expr,
    constants: &[Constant],
    direction: i64,
) -> CompileResult<Option<(String, usize, Option<ScalarType>)>> {
    let Some((value, byte_offset, cast_target)) = string_pointer_from_expr(string_expr, constants)?
    else {
        return Ok(None);
    };
    let offset = eval_integer_initializer_expr_with_constants(offset_expr, constants)?
        .to_i64_trunc()?
        .checked_mul(direction)
        .ok_or_else(|| CompileError::new("global struct string pointer offset overflow"))?;
    add_string_offset(byte_offset, offset)
        .map(|byte_offset| Some((value, byte_offset, cast_target)))
}

fn add_address_offset(
    address: &mut GlobalStructInitializerAddress,
    offset: i64,
) -> CompileResult<()> {
    let index = i64::try_from(address.index.unwrap_or(0))
        .map_err(|_| CompileError::new("global struct initializer address index is too large"))?;
    let index = index
        .checked_add(offset)
        .ok_or_else(|| CompileError::new("global struct initializer address overflow"))?;
    address.index =
        Some(usize::try_from(index).map_err(|_| {
            CompileError::new("global struct initializer address index is negative")
        })?);
    Ok(())
}

fn add_string_offset(byte_offset: usize, offset: i64) -> CompileResult<usize> {
    let byte_offset = i64::try_from(byte_offset)
        .map_err(|_| CompileError::new("global struct string pointer offset is too large"))?;
    let byte_offset = byte_offset
        .checked_add(offset)
        .ok_or_else(|| CompileError::new("global struct string pointer offset overflow"))?;
    usize::try_from(byte_offset)
        .map_err(|_| CompileError::new("global struct string pointer offset is negative"))
}
