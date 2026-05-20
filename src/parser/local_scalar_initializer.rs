use super::{BinaryOp, Expr, ScalarType};

pub(super) fn local_scalar_initializer(
    scalar_type: ScalarType,
    type_includes_char: bool,
    type_includes_short: bool,
    type_is_unsigned: bool,
    expr: Expr,
) -> Expr {
    if scalar_type != ScalarType::Int {
        return expr;
    }
    if type_includes_char {
        return narrowed(expr, 255, 128, 256, type_is_unsigned);
    }
    if type_includes_short {
        return narrowed(expr, 65_535, 32_768, 65_536, type_is_unsigned);
    }
    expr
}

fn narrowed(
    expr: Expr,
    mask: i64,
    sign_bit: i64,
    signed_range: i64,
    type_is_unsigned: bool,
) -> Expr {
    let masked = bit_mask(expr, mask);
    if type_is_unsigned {
        return masked;
    }
    Expr::Conditional {
        condition: Box::new(Expr::Binary {
            op: BinaryOp::GreaterEqual,
            left: Box::new(masked.clone()),
            right: Box::new(Expr::Integer(sign_bit)),
        }),
        then_expr: Box::new(Expr::Binary {
            op: BinaryOp::Sub,
            left: Box::new(masked.clone()),
            right: Box::new(Expr::Integer(signed_range)),
        }),
        else_expr: Box::new(masked),
    }
}

fn bit_mask(expr: Expr, mask: i64) -> Expr {
    Expr::Binary {
        op: BinaryOp::BitAnd,
        left: Box::new(expr),
        right: Box::new(Expr::Integer(mask)),
    }
}
