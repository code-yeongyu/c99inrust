use super::{BinaryOp, Expr, ScalarType};

pub(super) fn local_scalar_initializer(
    scalar_type: ScalarType,
    type_includes_char: bool,
    type_is_unsigned: bool,
    expr: Expr,
) -> Expr {
    if scalar_type != ScalarType::Int || !type_includes_char {
        return expr;
    }
    let masked = byte_mask(expr);
    if type_is_unsigned {
        return masked;
    }
    Expr::Conditional {
        condition: Box::new(Expr::Binary {
            op: BinaryOp::GreaterEqual,
            left: Box::new(masked.clone()),
            right: Box::new(Expr::Integer(128)),
        }),
        then_expr: Box::new(Expr::Binary {
            op: BinaryOp::Sub,
            left: Box::new(masked.clone()),
            right: Box::new(Expr::Integer(256)),
        }),
        else_expr: Box::new(masked),
    }
}

fn byte_mask(expr: Expr) -> Expr {
    Expr::Binary {
        op: BinaryOp::BitAnd,
        left: Box::new(expr),
        right: Box::new(Expr::Integer(255)),
    }
}
