use super::LoweredExpr;
use crate::parser::BinaryOp;

pub(in crate::ir) struct ComplexBinaryLanes {
    pub(in crate::ir) a: LoweredExpr,
    pub(in crate::ir) b: LoweredExpr,
    pub(in crate::ir) c: LoweredExpr,
    pub(in crate::ir) d: LoweredExpr,
}

pub(in crate::ir) fn complex_arithmetic_lane_expr(
    op: BinaryOp,
    lanes: ComplexBinaryLanes,
    index: i64,
    imaginary_index: i64,
) -> Option<LoweredExpr> {
    Some(match op {
        BinaryOp::Mul => complex_mul_lane_expr(lanes, index, imaginary_index),
        BinaryOp::Div => complex_div_lane_expr(lanes, index, imaginary_index),
        _ => return None,
    })
}

fn complex_mul_lane_expr(
    lanes: ComplexBinaryLanes,
    index: i64,
    imaginary_index: i64,
) -> LoweredExpr {
    if index == 0 {
        binary(
            BinaryOp::Sub,
            binary(BinaryOp::Mul, lanes.a, lanes.c),
            binary(BinaryOp::Mul, lanes.b, lanes.d),
        )
    } else if index == imaginary_index {
        binary(
            BinaryOp::Add,
            binary(BinaryOp::Mul, lanes.a, lanes.d),
            binary(BinaryOp::Mul, lanes.b, lanes.c),
        )
    } else {
        zero_lane_expr()
    }
}

fn complex_div_lane_expr(
    lanes: ComplexBinaryLanes,
    index: i64,
    imaginary_index: i64,
) -> LoweredExpr {
    let denominator = binary(
        BinaryOp::Add,
        binary(BinaryOp::Mul, lanes.c.clone(), lanes.c.clone()),
        binary(BinaryOp::Mul, lanes.d.clone(), lanes.d.clone()),
    );
    if index == 0 {
        binary(
            BinaryOp::Div,
            binary(
                BinaryOp::Add,
                binary(BinaryOp::Mul, lanes.a, lanes.c),
                binary(BinaryOp::Mul, lanes.b, lanes.d),
            ),
            denominator,
        )
    } else if index == imaginary_index {
        binary(
            BinaryOp::Div,
            binary(
                BinaryOp::Sub,
                binary(BinaryOp::Mul, lanes.b, lanes.c),
                binary(BinaryOp::Mul, lanes.a, lanes.d),
            ),
            denominator,
        )
    } else {
        zero_lane_expr()
    }
}

fn binary(op: BinaryOp, left: LoweredExpr, right: LoweredExpr) -> LoweredExpr {
    LoweredExpr::Binary {
        op,
        left: Box::new(left),
        right: Box::new(right),
    }
}

fn zero_lane_expr() -> LoweredExpr {
    LoweredExpr::DoubleLiteral("0.0".to_owned())
}
