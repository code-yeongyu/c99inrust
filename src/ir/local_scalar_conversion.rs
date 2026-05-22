use super::LoweredExpr;
use crate::parser::BinaryOp;

pub(in crate::ir) fn narrow_local_scalar_value(
    referent: Option<&str>,
    value: LoweredExpr,
) -> LoweredExpr {
    match referent {
        Some("byte") => masked_integer(value, 255),
        Some("char") => signed_narrow_integer(value, 255, 128, 256),
        Some("unsigned short") => masked_integer(value, 65_535),
        Some("short") => signed_narrow_integer(value, 65_535, 32_768, 65_536),
        _ => value,
    }
}

pub(in crate::ir) fn local_scalar_referent_size(referent: Option<&str>) -> Option<usize> {
    match referent {
        Some("byte" | "char") => Some(1),
        Some("unsigned short" | "short") => Some(2),
        _ => None,
    }
}

fn signed_narrow_integer(expr: LoweredExpr, mask: i64, sign_bit: i64, range: i64) -> LoweredExpr {
    let masked = masked_integer(expr, mask);
    LoweredExpr::Conditional {
        condition: Box::new(LoweredExpr::Binary {
            op: BinaryOp::GreaterEqual,
            left: Box::new(masked.clone()),
            right: Box::new(LoweredExpr::Integer(sign_bit)),
        }),
        then_expr: Box::new(LoweredExpr::Binary {
            op: BinaryOp::Sub,
            left: Box::new(masked.clone()),
            right: Box::new(LoweredExpr::Integer(range)),
        }),
        else_expr: Box::new(masked),
    }
}

fn masked_integer(expr: LoweredExpr, mask: i64) -> LoweredExpr {
    LoweredExpr::Binary {
        op: BinaryOp::BitAnd,
        left: Box::new(expr),
        right: Box::new(LoweredExpr::Integer(mask)),
    }
}
