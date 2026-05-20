use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::Token;

use super::{BinaryOp, Constant, Expr, InitializerNumber, Parser, ScalarType, UnaryOp};

pub(super) fn parse_integer_initializer(tokens: &[Token]) -> CompileResult<i64> {
    parse_integer_initializer_with_constants(tokens, &[])
}

pub(super) fn parse_integer_initializer_with_constants(
    tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<i64> {
    parse_integer_initializer_with_context(tokens, constants, &[])
}

pub(super) fn parse_integer_initializer_with_context(
    tokens: &[Token],
    constants: &[Constant],
    sizeof_symbols: &[(String, usize)],
) -> CompileResult<i64> {
    if tokens.is_empty() {
        return Err(CompileError::new("expected global integer initializer"));
    }
    let mut parser = Parser {
        tokens,
        index: 0,
        known_structs: &[],
        known_constants: constants,
        known_scalar_typedefs: &[],
        known_pointer_typedefs: &[],
    };
    let expr = parser.expression()?;
    if let Some(token) = parser.peek() {
        return Err(CompileError::new("unsupported global integer initializer")
            .at(token.line, token.column));
    }
    eval_integer_initializer_expr_with_context(&expr, constants, sizeof_symbols)?.to_i64_trunc()
}

pub(super) fn eval_integer_initializer_expr_with_constants(
    expr: &Expr,
    constants: &[Constant],
) -> CompileResult<InitializerNumber> {
    eval_integer_initializer_expr_with_context(expr, constants, &[])
}

fn eval_integer_initializer_expr_with_context(
    expr: &Expr,
    constants: &[Constant],
    sizeof_symbols: &[(String, usize)],
) -> CompileResult<InitializerNumber> {
    match expr {
        Expr::Integer(value) => Ok(InitializerNumber::integer(*value)),
        Expr::DoubleLiteral(value) => InitializerNumber::decimal(value),
        Expr::Unary { op, expr } => {
            let value =
                eval_integer_initializer_expr_with_context(expr, constants, sizeof_symbols)?;
            match op {
                UnaryOp::Plus => Ok(value),
                UnaryOp::Minus => value.checked_neg(),
                UnaryOp::BitNot => {
                    let value = value.to_i128_integer()?;
                    InitializerNumber::new(!value, 1)
                }
                UnaryOp::LogicalNot => {
                    let value = value.to_i128_integer()?;
                    Ok(InitializerNumber::integer(i64::from(value == 0)))
                }
            }
        }
        Expr::Cast { target, expr, .. } => {
            let value =
                eval_integer_initializer_expr_with_context(expr, constants, sizeof_symbols)?;
            match target {
                ScalarType::Bool => Ok(InitializerNumber::integer(i64::from(
                    value.to_i64_trunc()? != 0,
                ))),
                ScalarType::Int
                | ScalarType::LongLong
                | ScalarType::Pointer
                | ScalarType::VaList => Ok(InitializerNumber::integer(value.to_i64_trunc()?)),
                ScalarType::Double => Ok(value),
            }
        }
        Expr::Binary { op, left, right } => {
            let left = eval_integer_initializer_expr_with_context(left, constants, sizeof_symbols)?;
            let right =
                eval_integer_initializer_expr_with_context(right, constants, sizeof_symbols)?;
            eval_integer_binary_initializer_expr(*op, left, right)
        }
        Expr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => {
            if eval_integer_initializer_expr_with_context(condition, constants, sizeof_symbols)?
                .to_i128_integer()?
                == 0
            {
                eval_integer_initializer_expr_with_context(else_expr, constants, sizeof_symbols)
            } else {
                eval_integer_initializer_expr_with_context(then_expr, constants, sizeof_symbols)
            }
        }
        Expr::Identifier(name) => constants
            .iter()
            .rev()
            .find(|constant| constant.name == *name)
            .map(|constant| InitializerNumber::integer(constant.value))
            .ok_or_else(|| {
                CompileError::new(format!("identifier {name} is not an integer initializer"))
            }),
        Expr::Call { callee, .. } => Err(CompileError::new(format!(
            "call to {callee} is not an integer initializer"
        ))),
        Expr::IndirectCall { .. } => Err(CompileError::new(
            "indirect call is not an integer initializer",
        )),
        Expr::SizeOfExpr { expr } => eval_sizeof_initializer_expr(expr, sizeof_symbols),
        Expr::StringLiteral(_)
        | Expr::AddressOf { .. }
        | Expr::Dereference { .. }
        | Expr::Member { .. }
        | Expr::Subscript { .. }
        | Expr::Assignment { .. }
        | Expr::PostIncrement { .. } => {
            Err(CompileError::new("unsupported global integer initializer"))
        }
    }
}

fn eval_sizeof_initializer_expr(
    expr: &Expr,
    sizeof_symbols: &[(String, usize)],
) -> CompileResult<InitializerNumber> {
    let Expr::Identifier(name) = expr else {
        return Err(CompileError::new("unsupported global sizeof initializer"));
    };
    let Some((_name, size)) = sizeof_symbols
        .iter()
        .rev()
        .find(|(candidate, _size)| candidate == name)
    else {
        return Err(CompileError::new(format!(
            "unknown global sizeof initializer: {name}"
        )));
    };
    i64::try_from(*size)
        .map(InitializerNumber::integer)
        .map_err(|_| CompileError::new("global sizeof initializer is too large"))
}

fn eval_integer_binary_initializer_expr(
    op: BinaryOp,
    left: InitializerNumber,
    right: InitializerNumber,
) -> CompileResult<InitializerNumber> {
    match op {
        BinaryOp::Mul => left.checked_mul(right),
        BinaryOp::Div => left.checked_div(right),
        BinaryOp::Mod => left.checked_rem(right),
        BinaryOp::Add => left.checked_add(right),
        BinaryOp::Sub => left.checked_sub(right),
        BinaryOp::ShiftLeft => left.checked_shl(right),
        BinaryOp::ShiftRight => left.checked_shr(right),
        BinaryOp::BitAnd => {
            InitializerNumber::new(left.to_i128_integer()? & right.to_i128_integer()?, 1)
        }
        BinaryOp::BitXor => {
            InitializerNumber::new(left.to_i128_integer()? ^ right.to_i128_integer()?, 1)
        }
        BinaryOp::BitOr => {
            InitializerNumber::new(left.to_i128_integer()? | right.to_i128_integer()?, 1)
        }
        BinaryOp::Less => Ok(InitializerNumber::integer(i64::from(
            left.to_i128_integer()? < right.to_i128_integer()?,
        ))),
        BinaryOp::LessEqual => Ok(InitializerNumber::integer(i64::from(
            left.to_i128_integer()? <= right.to_i128_integer()?,
        ))),
        BinaryOp::Greater => Ok(InitializerNumber::integer(i64::from(
            left.to_i128_integer()? > right.to_i128_integer()?,
        ))),
        BinaryOp::GreaterEqual => Ok(InitializerNumber::integer(i64::from(
            left.to_i128_integer()? >= right.to_i128_integer()?,
        ))),
        BinaryOp::Equal => Ok(InitializerNumber::integer(i64::from(
            left.to_i128_integer()? == right.to_i128_integer()?,
        ))),
        BinaryOp::NotEqual => Ok(InitializerNumber::integer(i64::from(
            left.to_i128_integer()? != right.to_i128_integer()?,
        ))),
        BinaryOp::LogicalAnd => Ok(InitializerNumber::integer(i64::from(
            left.to_i128_integer()? != 0 && right.to_i128_integer()? != 0,
        ))),
        BinaryOp::LogicalOr => Ok(InitializerNumber::integer(i64::from(
            left.to_i128_integer()? != 0 || right.to_i128_integer()? != 0,
        ))),
    }
}
