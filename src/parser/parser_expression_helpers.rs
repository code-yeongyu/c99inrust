use super::{BinaryOp, CompileError, CompileResult, Expr, LValue, Statement};

pub(super) fn lvalue_from_expr(expr: Expr) -> CompileResult<LValue> {
    match expr {
        Expr::Identifier(name) => Ok(LValue::Identifier(name)),
        Expr::Subscript { array, index } => Ok(LValue::Subscript { array, index }),
        Expr::Dereference { pointer } => Ok(LValue::Subscript {
            array: pointer,
            index: Box::new(Expr::Integer(0)),
        }),
        Expr::Member {
            base,
            field,
            dereference,
        } => Ok(LValue::Member {
            base,
            field,
            dereference,
        }),
        _ => Err(CompileError::new("unsupported assignment target")),
    }
}

pub(super) fn prefix_update_expr(expr: Expr, op: BinaryOp) -> CompileResult<Expr> {
    let target = lvalue_from_expr(expr.clone())?;
    Ok(Expr::Assignment {
        target,
        value: Box::new(Expr::Binary {
            op,
            left: Box::new(expr),
            right: Box::new(Expr::Integer(1)),
        }),
    })
}

pub(super) fn statement_from_expression(expr: Expr) -> Statement {
    match expr {
        Expr::Assignment { target, value } => Statement::Assignment {
            target,
            value: *value,
        },
        _ => Statement::Expression(expr),
    }
}
