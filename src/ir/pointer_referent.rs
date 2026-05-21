use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{BinaryOp, Expr, FieldType, ScalarType};

use super::{GlobalBinding, LocalBinding, LoweringContext, POINTER_REFERENT, pointer_arithmetic};

pub(in crate::ir) fn for_expr(context: &LoweringContext, expr: &Expr) -> CompileResult<String> {
    if let Expr::Identifier(name) = expr
        && let Some(referent) = context.pointer_referent_for_identifier(name)
    {
        return Ok(referent);
    }
    if matches!(expr, Expr::StringLiteral(_)) {
        return Ok("char".to_owned());
    }
    if let Expr::Call { callee, .. } = expr
        && let Some(Some(referent)) = context.pointer_return_functions.get(callee)
    {
        return Ok(referent.clone());
    }
    if let Expr::Member {
        base,
        field,
        dereference,
    } = expr
    {
        let member = context.resolve_member_access(base, field, *dereference)?;
        if let FieldType::Pointer {
            referent: Some(referent),
        } = member.field_type
        {
            return Ok(referent);
        }
    }
    if let Expr::Cast {
        target: ScalarType::Pointer,
        referent: Some(referent),
        ..
    } = expr
    {
        return Ok(referent.clone());
    }
    if let Expr::Dereference { pointer } = expr {
        return nested_referent(context, pointer);
    }
    if let Expr::PostIncrement { target, .. } = expr
        && let Some(referent) = context.pointer_referent_for_lvalue(target)?
    {
        return Ok(referent);
    }
    if let Expr::Subscript { array, .. } = expr {
        if let Some(referent) = array_referent(context, array) {
            return Ok(referent);
        }
        return nested_referent(context, array);
    }
    if let Expr::Binary { op, left, right } = expr {
        return binary_referent(context, *op, left, right);
    }
    Err(CompileError::new(
        "pointer member access requires a typed pointer",
    ))
}

fn array_referent(context: &LoweringContext, array: &Expr) -> Option<String> {
    if let Expr::Identifier(name) = array
        && matches!(
            context.local_binding(name),
            Some(LocalBinding::CharMatrix { .. })
        )
    {
        return Some("char".to_owned());
    }
    if let Expr::Identifier(name) = array
        && matches!(
            context.local_binding(name),
            Some(LocalBinding::IntMatrix { .. })
        )
    {
        return Some("int".to_owned());
    }
    if let Expr::Identifier(name) = array
        && let Some(GlobalBinding::UnsignedCharMatrix { is_unsigned, .. }) =
            context.global_bindings.get(name)
    {
        return Some(if *is_unsigned { "byte" } else { "char" }.to_owned());
    }
    if let Expr::Identifier(name) = array
        && matches!(
            context.global_bindings.get(name),
            Some(GlobalBinding::ShortArray {
                columns: Some(_),
                ..
            })
        )
    {
        return Some("short".to_owned());
    }
    if let Expr::Identifier(name) = array
        && let Some(GlobalBinding::PointerArray {
            referent, columns, ..
        }) = context.global_bindings.get(name)
    {
        if columns.is_some() {
            return Some(pointer_arithmetic::nested_referent(referent.as_deref()));
        }
        return referent.clone();
    }
    if let Expr::Identifier(name) = array
        && let Some(GlobalBinding::StructArray {
            struct_name,
            columns: Some(_),
            ..
        }) = context.global_bindings.get(name)
    {
        return Some(struct_name.clone());
    }
    None
}

fn nested_referent(context: &LoweringContext, expr: &Expr) -> CompileResult<String> {
    let referent = for_expr(context, expr)?;
    referent
        .strip_prefix(POINTER_REFERENT)
        .filter(|nested| !nested.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| CompileError::new("pointer member access requires a typed pointer"))
}

fn binary_referent(
    context: &LoweringContext,
    op: BinaryOp,
    left: &Expr,
    right: &Expr,
) -> CompileResult<String> {
    if op == BinaryOp::Add {
        return for_expr(context, left).or_else(|_error| for_expr(context, right));
    }
    if op == BinaryOp::Sub {
        return for_expr(context, left);
    }
    Err(CompileError::new(
        "pointer member access requires a typed pointer",
    ))
}
