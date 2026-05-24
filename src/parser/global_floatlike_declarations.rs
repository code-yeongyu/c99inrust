use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::{Keyword, Token, TokenKind};

use super::token_scan::{
    previous_identifier_index, token_identifier, token_is_punctuator, top_level_punctuator_index,
};
use super::{
    Constant, Expr, Global, GlobalInitializer, Parser, ScalarType, UnaryOp,
    parse_integer_initializer_with_context,
};

pub(super) fn parse_global_floatlike_scalar(
    tokens: &[Token],
    constants: &[Constant],
    sizeof_symbols: &[(String, usize)],
) -> CompileResult<Option<Global>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let end_index = top_level_punctuator_index(declaration, "=").unwrap_or(declaration.len());
    if top_level_punctuator_index(&declaration[..end_index], "[").is_some() {
        return Ok(None);
    }
    let Some(name_index) = previous_identifier_index(declaration, end_index) else {
        return Ok(None);
    };
    let specifiers = &declaration[..name_index];
    let Some(scalar_type) = global_floatlike_scalar_type(specifiers, false) else {
        return Ok(None);
    };
    if declaration
        .get(end_index + 1)
        .is_some_and(|token| token_is_punctuator(token, "{"))
    {
        return Ok(None);
    }
    let initializer = if end_index == declaration.len() {
        GlobalInitializer::ScalarZero(scalar_type)
    } else {
        let real = parse_global_real_initializer(
            &declaration[end_index + 1..],
            constants,
            sizeof_symbols,
        )?;
        if scalar_type == ScalarType::Double || scalar_type == ScalarType::LongDouble {
            GlobalInitializer::Double(real)
        } else {
            GlobalInitializer::ComplexReal { scalar_type, real }
        }
    };
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global scalar name"))?
        .to_owned();
    Ok(Some(Global::new(name, initializer)))
}

pub(super) fn global_floatlike_scalar_type(
    specifiers: &[Token],
    allow_extern: bool,
) -> Option<ScalarType> {
    let mut saw_complex = false;
    let mut saw_double = false;
    let mut saw_float = false;
    let mut long_count = 0usize;
    for token in specifiers {
        match token.kind {
            TokenKind::Keyword(Keyword::Extern) if allow_extern => {}
            TokenKind::Keyword(
                Keyword::Static | Keyword::Const | Keyword::Volatile | Keyword::Register,
            ) => {}
            TokenKind::Keyword(Keyword::Complex) => saw_complex = true,
            TokenKind::Keyword(Keyword::Double) => saw_double = true,
            TokenKind::Keyword(Keyword::Float) => saw_float = true,
            TokenKind::Keyword(Keyword::Long) => long_count += 1,
            _ => return None,
        }
    }
    if saw_complex {
        if saw_float {
            Some(ScalarType::ComplexFloat)
        } else if long_count == 0 {
            Some(ScalarType::ComplexDouble)
        } else {
            Some(ScalarType::ComplexLongDouble)
        }
    } else if saw_double && long_count == 0 {
        Some(ScalarType::Double)
    } else if saw_double {
        Some(ScalarType::LongDouble)
    } else {
        None
    }
}

fn parse_global_real_initializer(
    tokens: &[Token],
    constants: &[Constant],
    sizeof_symbols: &[(String, usize)],
) -> CompileResult<String> {
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
    if parser.index != tokens.len() {
        return Err(CompileError::new("unsupported global real initializer"));
    }
    match expr {
        Expr::DoubleLiteral(value) => Ok(value),
        Expr::Integer(value) | Expr::LongInteger(value) => Ok(value.to_string()),
        Expr::Unary {
            op: UnaryOp::Minus,
            expr,
        } => parse_negative_global_real_initializer(*expr),
        _ => parse_integer_initializer_with_context(tokens, constants, sizeof_symbols)
            .map(|value| value.to_string()),
    }
}

fn parse_negative_global_real_initializer(expr: Expr) -> CompileResult<String> {
    match expr {
        Expr::DoubleLiteral(value) => Ok(format!("-{value}")),
        Expr::Integer(value) | Expr::LongInteger(value) => Ok(value
            .checked_neg()
            .ok_or_else(|| CompileError::new("global real initializer overflow"))?
            .to_string()),
        _ => Err(CompileError::new("unsupported global real initializer")),
    }
}
