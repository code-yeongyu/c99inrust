use std::collections::HashMap;

use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Expr, GlobalPointerAddress, LValue};

pub(in crate::ir) fn from_lvalue(
    target: &LValue,
    constants: &HashMap<String, i64>,
) -> CompileResult<Option<GlobalPointerAddress>> {
    match target {
        LValue::Member {
            base,
            field,
            dereference: false,
        } => member_expr_address(base, field, constants),
        LValue::Subscript { array, index } => member_subscript_address(array, index, constants),
        LValue::Identifier(_)
        | LValue::ScalarCompoundLiteral { .. }
        | LValue::StructCompoundLiteral { .. }
        | LValue::Member { .. } => Ok(None),
    }
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
            element_index: None,
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
                element_index: None,
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

fn member_subscript_address(
    array: &Expr,
    index: &Expr,
    constants: &HashMap<String, i64>,
) -> CompileResult<Option<GlobalPointerAddress>> {
    let Expr::Member {
        base,
        field,
        dereference: false,
    } = array
    else {
        return Ok(None);
    };
    let Some(mut address) = member_expr_address(base, field, constants)? else {
        return Ok(None);
    };
    let element_index = super::static_local::eval_with_constants(index, constants)?;
    address.element_index = Some(
        usize::try_from(element_index)
            .map_err(|_| CompileError::new("static local member pointer index is negative"))?,
    );
    Ok(Some(address))
}
