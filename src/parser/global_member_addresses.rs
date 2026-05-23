use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::Token;

use super::{
    Constant, Expr, GlobalPointerAddress, LValue, Parser, StructLayout,
    eval_integer_initializer_expr_with_constants,
};

pub(super) fn parse_global_member_address(
    tokens: &[Token],
    known_structs: &[StructLayout],
    constants: &[Constant],
) -> CompileResult<Option<GlobalPointerAddress>> {
    let mut parser = Parser {
        tokens,
        index: 0,
        known_structs,
        known_constants: constants,
        known_scalar_typedefs: &[],
        known_pointer_typedefs: &[],
    };
    let Ok(expr) = parser.expression() else {
        return Ok(None);
    };
    if parser.peek().is_some() {
        return Ok(None);
    }
    let Expr::AddressOf { target } = expr else {
        return Ok(None);
    };
    member_lvalue_address(&target, constants)
}

fn member_lvalue_address(
    target: &LValue,
    constants: &[Constant],
) -> CompileResult<Option<GlobalPointerAddress>> {
    match target {
        LValue::Member {
            base,
            field,
            dereference: false,
        } => member_expr_address(base, field, constants),
        LValue::Subscript { array, index } => member_subscript_address(array, index, constants),
        LValue::Identifier(_) | LValue::ScalarCompoundLiteral { .. } | LValue::Member { .. } => {
            Ok(None)
        }
    }
}

fn member_expr_address(
    base: &Expr,
    field: &str,
    constants: &[Constant],
) -> CompileResult<Option<GlobalPointerAddress>> {
    let Some(mut address) = base_expr_address(base, constants)? else {
        return Ok(None);
    };
    address.fields.push(field.to_owned());
    Ok(Some(address))
}

fn base_expr_address(
    expr: &Expr,
    constants: &[Constant],
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
            Ok(Some(GlobalPointerAddress {
                base: base.clone(),
                index: member_index(index, constants)?,
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
    constants: &[Constant],
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
    address.element_index = Some(member_index(index, constants)?);
    Ok(Some(address))
}

fn member_index(index: &Expr, constants: &[Constant]) -> CompileResult<usize> {
    let value = eval_integer_initializer_expr_with_constants(index, constants)?.to_i64_trunc()?;
    usize::try_from(value)
        .map_err(|_| CompileError::new("global member pointer address index is negative"))
}
