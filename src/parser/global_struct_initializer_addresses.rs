use crate::diagnostics::{CompileError, CompileResult};

use super::{
    Constant, Expr, GlobalStructInitializerAddress, LValue,
    eval_integer_initializer_expr_with_constants,
};

pub(super) fn constant_value(name: &str, constants: &[Constant]) -> Option<i64> {
    constants
        .iter()
        .rev()
        .find(|constant| constant.name == name)
        .map(|constant| constant.value)
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
        LValue::Member { .. } | LValue::ScalarCompoundLiteral { .. } => Err(CompileError::new(
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
