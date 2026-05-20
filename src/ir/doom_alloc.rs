use crate::parser::{BinaryOp, Expr};

pub(in crate::ir) fn widened_size_arg(expr: &Expr) -> Option<Expr> {
    let Expr::Binary { op, left, right } = expr else {
        return None;
    };
    if *op != BinaryOp::Mul {
        return None;
    }
    if matches!(left.as_ref(), Expr::Integer(4)) {
        return Some(Expr::Binary {
            op: BinaryOp::Mul,
            left: Box::new(Expr::Integer(8)),
            right: right.clone(),
        });
    }
    if matches!(right.as_ref(), Expr::Integer(4)) {
        return Some(Expr::Binary {
            op: BinaryOp::Mul,
            left: left.clone(),
            right: Box::new(Expr::Integer(8)),
        });
    }
    None
}
