use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{BinaryOp, Expr, ScalarType, UnaryOp};

/// Evaluates a constant integer expression.
///
/// # Errors
///
/// Returns an error when the expression is not constant or overflows the current
/// checked integer model.
pub fn const_eval(expr: &Expr) -> CompileResult<i64> {
    match expr {
        Expr::Call { callee, .. } => Err(CompileError::new(format!(
            "call to {callee} is not a constant expression"
        ))),
        Expr::IndirectCall { .. } => Err(CompileError::new(
            "indirect call is not a constant expression",
        )),
        Expr::Identifier(name) => Err(CompileError::new(format!(
            "identifier {name} is not a constant expression"
        ))),
        Expr::Integer(value) => Ok(*value),
        Expr::DoubleLiteral(_) => Err(CompileError::new(
            "double literal is not an integer constant expression",
        )),
        Expr::StringLiteral(_) => Err(CompileError::new(
            "string literal is not an integer constant expression",
        )),
        Expr::SizeOfExpr { .. } => Err(CompileError::new(
            "sizeof expression is not an integer constant expression",
        )),
        Expr::Subscript { .. } => Err(CompileError::new(
            "subscript expression is not an integer constant expression",
        )),
        Expr::Dereference { .. } => Err(CompileError::new(
            "dereference expression is not an integer constant expression",
        )),
        Expr::AddressOf { .. } => Err(CompileError::new(
            "address expression is not an integer constant expression",
        )),
        Expr::Member { .. } => Err(CompileError::new(
            "member expression is not an integer constant expression",
        )),
        Expr::Assignment { .. } => Err(CompileError::new(
            "assignment expression is not an integer constant expression",
        )),
        Expr::PostIncrement { .. } => Err(CompileError::new(
            "post-increment expression is not an integer constant expression",
        )),
        Expr::Unary { op, expr } => {
            let value = const_eval(expr)?;
            match op {
                UnaryOp::Plus => Ok(value),
                UnaryOp::Minus => value
                    .checked_neg()
                    .ok_or_else(|| CompileError::new("integer overflow in unary minus")),
                UnaryOp::BitNot => Ok(!value),
                UnaryOp::LogicalNot => Ok(i64::from(value == 0)),
            }
        }
        Expr::Cast { target, expr, .. } => {
            let value = const_eval(expr)?;
            cast_const_value(*target, value)
        }
        Expr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => {
            if const_eval(condition)? == 0 {
                const_eval(else_expr)
            } else {
                const_eval(then_expr)
            }
        }
        Expr::Binary { op, left, right } => {
            let left = const_eval(left)?;
            if *op == BinaryOp::LogicalAnd && left == 0 {
                return Ok(0);
            }
            if *op == BinaryOp::LogicalOr && left != 0 {
                return Ok(1);
            }
            let right = const_eval(right)?;
            eval_binary(*op, left, right)
        }
    }
}
pub(in crate::ir) fn cast_const_value(target: ScalarType, value: i64) -> CompileResult<i64> {
    match target {
        ScalarType::Int => i32::try_from(value)
            .map(i64::from)
            .map_err(|_| CompileError::new("integer cast result does not fit i32")),
        ScalarType::LongLong => Ok(value),
        ScalarType::Double | ScalarType::Pointer | ScalarType::VaList => Err(CompileError::new(
            "non-integer cast is not an integer constant expression",
        )),
    }
}
pub(in crate::ir) fn eval_binary(op: BinaryOp, left: i64, right: i64) -> CompileResult<i64> {
    match op {
        BinaryOp::Mul => left
            .checked_mul(right)
            .ok_or_else(|| CompileError::new("integer overflow in multiplication")),
        BinaryOp::Div => {
            if right == 0 {
                return Err(CompileError::new("division by zero"));
            }
            left.checked_div(right)
                .ok_or_else(|| CompileError::new("integer overflow in division"))
        }
        BinaryOp::Mod => {
            if right == 0 {
                return Err(CompileError::new("modulo by zero"));
            }
            left.checked_rem(right)
                .ok_or_else(|| CompileError::new("integer overflow in modulo"))
        }
        BinaryOp::Add => left
            .checked_add(right)
            .ok_or_else(|| CompileError::new("integer overflow in addition")),
        BinaryOp::Sub => left
            .checked_sub(right)
            .ok_or_else(|| CompileError::new("integer overflow in subtraction")),
        BinaryOp::ShiftLeft => shift_count(right).and_then(|count| {
            left.checked_shl(count)
                .ok_or_else(|| CompileError::new("integer overflow in left shift"))
        }),
        BinaryOp::ShiftRight => shift_count(right).and_then(|count| {
            left.checked_shr(count)
                .ok_or_else(|| CompileError::new("integer overflow in right shift"))
        }),
        BinaryOp::Less => Ok(i64::from(left < right)),
        BinaryOp::LessEqual => Ok(i64::from(left <= right)),
        BinaryOp::Greater => Ok(i64::from(left > right)),
        BinaryOp::GreaterEqual => Ok(i64::from(left >= right)),
        BinaryOp::Equal => Ok(i64::from(left == right)),
        BinaryOp::NotEqual => Ok(i64::from(left != right)),
        BinaryOp::LogicalAnd => Ok(i64::from(left != 0 && right != 0)),
        BinaryOp::LogicalOr => Ok(i64::from(left != 0 || right != 0)),
        BinaryOp::BitAnd => Ok(left & right),
        BinaryOp::BitXor => Ok(left ^ right),
        BinaryOp::BitOr => Ok(left | right),
    }
}
pub(in crate::ir) fn shift_count(value: i64) -> CompileResult<u32> {
    let count =
        u32::try_from(value).map_err(|_| CompileError::new("shift count must be non-negative"))?;
    if count >= i64::BITS {
        return Err(CompileError::new("shift count is too large"));
    }
    Ok(count)
}
