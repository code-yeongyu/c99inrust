use std::collections::HashMap;

use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Expr, GlobalPointerAddress, LValue};

pub(in crate::ir) fn from_lvalue(
    target: &LValue,
    constants: &HashMap<String, i64>,
) -> CompileResult<Option<GlobalPointerAddress>> {
    let LValue::Member {
        base,
        field,
        dereference: false,
    } = target
    else {
        return Ok(None);
    };
    member_expr_address(base, field, constants)
}

fn member_expr_address(
    base: &Expr,
    field: &str,
    constants: &HashMap<String, i64>,
) -> CompileResult<Option<GlobalPointerAddress>> {
    let Some(mut address) = base_expr_address(base, constants)? else {
        return Ok(None);
    };
    address.fields.push(field.to_owned());
    Ok(Some(address))
}

fn base_expr_address(
    expr: &Expr,
    constants: &HashMap<String, i64>,
) -> CompileResult<Option<GlobalPointerAddress>> {
    match expr {
        Expr::Identifier(base) => Ok(Some(GlobalPointerAddress {
            base: base.clone(),
            index: 0,
            fields: Vec::new(),
        })),
        Expr::Subscript { array, index } => {
            let Expr::Identifier(base) = array.as_ref() else {
                return Ok(None);
            };
            let index =
                usize::try_from(super::static_local::eval_with_constants(index, constants)?)
                    .map_err(|_| {
                        CompileError::new("static local member pointer index is negative")
                    })?;
            Ok(Some(GlobalPointerAddress {
                base: base.clone(),
                index,
                fields: Vec::new(),
            }))
        }
        Expr::Member {
            base,
            field,
            dereference: false,
        } => member_expr_address(base, field, constants),
        _ => Ok(None),
    }
}
