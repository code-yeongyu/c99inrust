use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::Token;

use super::integer_initializer::eval_integer_initializer_expr_with_constants;
use super::token_scan::{
    matching_top_level_brace, token_is_punctuator, top_level_punctuator_index,
};
use super::{BinaryOp, Constant, Expr, LValue, Parser};

struct StringPointerInitializer {
    value: String,
    byte_offset: i64,
}

pub(super) fn parse_string_array_initializer(tokens: &[Token]) -> CompileResult<Vec<String>> {
    let Some(first) = tokens.first() else {
        return Err(CompileError::new(
            "expected global pointer-array initializer",
        ));
    };
    if !token_is_punctuator(first, "{") {
        return Err(
            CompileError::new("expected global pointer-array initializer")
                .at(first.line, first.column),
        );
    }
    let Some(close_brace) = matching_top_level_brace(tokens, 0) else {
        return Err(
            CompileError::new("unterminated global pointer-array initializer")
                .at(first.line, first.column),
        );
    };
    if let Some(token) = tokens.get(close_brace + 1) {
        return Err(
            CompileError::new("unsupported global pointer-array initializer")
                .at(token.line, token.column),
        );
    }

    let mut values = Vec::new();
    let mut start = 1usize;
    while start < close_brace {
        let item = &tokens[start..close_brace];
        let item_len = top_level_punctuator_index(item, ",").unwrap_or(item.len());
        if item_len == 0 {
            return Err(
                CompileError::new("expected global pointer-array initializer value")
                    .at(tokens[start].line, tokens[start].column),
            );
        }
        values.push(parse_string_initializer(&item[..item_len])?);
        start += item_len;
        if start < close_brace {
            start += 1;
        }
    }
    Ok(values)
}

pub(super) fn parse_string_initializer(tokens: &[Token]) -> CompileResult<String> {
    if tokens.is_empty() {
        return Err(CompileError::new("expected global string initializer"));
    }
    let mut parser = Parser {
        tokens,
        index: 0,
        known_structs: &[],
        known_constants: &[],
        known_scalar_typedefs: &[],
        known_pointer_typedefs: &[],
        known_function_pointer_typedefs: &[],
    };
    let expr = parser.expression()?;
    if let Some(token) = parser.peek() {
        return Err(
            CompileError::new("unsupported global string initializer").at(token.line, token.column)
        );
    }
    let Expr::StringLiteral(value) = expr else {
        return Err(CompileError::new("expected global string initializer"));
    };
    Ok(value)
}

pub(super) fn parse_string_pointer_initializer(
    tokens: &[Token],
    constants: &[Constant],
) -> CompileResult<Option<(String, usize)>> {
    let expr = parse_initializer_expr(tokens, constants)?;
    string_pointer_initializer_expr(&expr, constants)
}

pub(super) fn string_pointer_initializer_expr(
    expr: &Expr,
    constants: &[Constant],
) -> CompileResult<Option<(String, usize)>> {
    string_pointer_initializer(expr, constants)?
        .map(|initializer| {
            usize::try_from(initializer.byte_offset)
                .map(|byte_offset| (initializer.value, byte_offset))
                .map_err(|_| CompileError::new("global string pointer offset must be nonnegative"))
        })
        .transpose()
}

fn parse_initializer_expr(tokens: &[Token], constants: &[Constant]) -> CompileResult<Expr> {
    let mut parser = Parser {
        tokens,
        index: 0,
        known_structs: &[],
        known_constants: constants,
        known_scalar_typedefs: &[],
        known_pointer_typedefs: &[],
        known_function_pointer_typedefs: &[],
    };
    let expr = parser.expression()?;
    if let Some(token) = parser.peek() {
        return Err(
            CompileError::new("unsupported global string pointer initializer")
                .at(token.line, token.column),
        );
    }
    Ok(expr)
}

fn string_pointer_initializer(
    expr: &Expr,
    constants: &[Constant],
) -> CompileResult<Option<StringPointerInitializer>> {
    match expr {
        Expr::StringLiteral(value) => Ok(Some(StringPointerInitializer {
            value: value.clone(),
            byte_offset: 0,
        })),
        Expr::AddressOf {
            target: LValue::Subscript { array, index },
        } => string_subscript_initializer(array, index, constants),
        Expr::Cast { expr, .. } => string_pointer_initializer(expr, constants),
        Expr::Binary {
            op: BinaryOp::Add,
            left,
            right,
        } => add_string_pointer_initializer(left, right, constants).and_then(|initializer| {
            initializer.map_or_else(
                || add_string_pointer_initializer(right, left, constants),
                |initializer| Ok(Some(initializer)),
            )
        }),
        Expr::Binary {
            op: BinaryOp::Sub,
            left,
            right,
        } => subtract_string_pointer_initializer(left, right, constants),
        _ => Ok(None),
    }
}

fn string_subscript_initializer(
    array: &Expr,
    index: &Expr,
    constants: &[Constant],
) -> CompileResult<Option<StringPointerInitializer>> {
    let Expr::StringLiteral(value) = array else {
        return Ok(None);
    };
    Ok(Some(StringPointerInitializer {
        value: value.clone(),
        byte_offset: integer_offset(index, constants)?,
    }))
}

fn add_string_pointer_initializer(
    base: &Expr,
    offset: &Expr,
    constants: &[Constant],
) -> CompileResult<Option<StringPointerInitializer>> {
    string_pointer_initializer(base, constants)?
        .map(|initializer| offset_string_pointer(initializer, integer_offset(offset, constants)?))
        .transpose()
}

fn subtract_string_pointer_initializer(
    base: &Expr,
    offset: &Expr,
    constants: &[Constant],
) -> CompileResult<Option<StringPointerInitializer>> {
    let offset = integer_offset(offset, constants)?
        .checked_neg()
        .ok_or_else(|| CompileError::new("global string pointer offset overflow"))?;
    string_pointer_initializer(base, constants)?
        .map(|initializer| offset_string_pointer(initializer, offset))
        .transpose()
}

fn offset_string_pointer(
    initializer: StringPointerInitializer,
    offset: i64,
) -> CompileResult<StringPointerInitializer> {
    Ok(StringPointerInitializer {
        value: initializer.value,
        byte_offset: initializer
            .byte_offset
            .checked_add(offset)
            .ok_or_else(|| CompileError::new("global string pointer offset overflow"))?,
    })
}

fn integer_offset(expr: &Expr, constants: &[Constant]) -> CompileResult<i64> {
    eval_integer_initializer_expr_with_constants(expr, constants)?.to_i64_trunc()
}
