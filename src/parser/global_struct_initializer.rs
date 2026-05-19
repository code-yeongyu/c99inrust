use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::Token;

use super::{
    Constant, Expr, GlobalStructInitializerAddress, GlobalStructInitializerValue, LValue, Parser,
    StructLayout, eval_integer_initializer_expr_with_constants, matching_top_level_brace,
    token_is_punctuator, top_level_comma_ranges,
};

pub(super) fn parse(
    tokens: &[Token],
    known_structs: &[StructLayout],
    constants: &[Constant],
) -> CompileResult<Vec<Vec<GlobalStructInitializerValue>>> {
    let Some(first) = tokens.first() else {
        return Err(CompileError::new(
            "expected global struct-array initializer",
        ));
    };
    if !token_is_punctuator(first, "{") {
        return Err(
            CompileError::new("expected global struct-array initializer")
                .at(first.line, first.column),
        );
    }
    let Some(close_brace) = matching_top_level_brace(tokens, 0) else {
        return Err(
            CompileError::new("unterminated global struct-array initializer")
                .at(first.line, first.column),
        );
    };
    if let Some(token) = tokens.get(close_brace + 1) {
        return Err(
            CompileError::new("unsupported global struct-array initializer")
                .at(token.line, token.column),
        );
    }

    let mut rows = Vec::new();
    for (start, end) in top_level_comma_ranges(&tokens[1..close_brace]) {
        if start == end {
            continue;
        }
        let row = &tokens[(start + 1)..=end];
        rows.push(parse_row(row, known_structs, constants)?);
    }
    Ok(rows)
}

fn parse_row(
    tokens: &[Token],
    known_structs: &[StructLayout],
    constants: &[Constant],
) -> CompileResult<Vec<GlobalStructInitializerValue>> {
    let Some(first) = tokens.first() else {
        return Err(CompileError::new("expected global struct initializer"));
    };
    if !token_is_punctuator(first, "{") {
        return Err(
            CompileError::new("expected global struct initializer").at(first.line, first.column)
        );
    }
    let Some(close_brace) = matching_top_level_brace(tokens, 0) else {
        return Err(CompileError::new("unterminated global struct initializer")
            .at(first.line, first.column));
    };
    if let Some(token) = tokens.get(close_brace + 1) {
        return Err(
            CompileError::new("unsupported global struct initializer").at(token.line, token.column)
        );
    }

    let mut values = Vec::new();
    for (start, end) in top_level_comma_ranges(&tokens[1..close_brace]) {
        if start == end {
            continue;
        }
        values.push(parse_value(
            &tokens[(start + 1)..=end],
            known_structs,
            constants,
        )?);
    }
    Ok(values)
}

fn parse_value(
    tokens: &[Token],
    known_structs: &[StructLayout],
    constants: &[Constant],
) -> CompileResult<GlobalStructInitializerValue> {
    let mut parser = Parser {
        tokens,
        index: 0,
        known_structs,
        known_constants: constants,
        known_scalar_typedefs: &[],
        known_pointer_typedefs: &[],
    };
    let expr = parser.expression()?;
    if let Some(token) = parser.peek() {
        return Err(
            CompileError::new("unsupported global struct initializer value")
                .at(token.line, token.column),
        );
    }
    value_from_expr(&expr, constants)
}

fn value_from_expr(
    expr: &Expr,
    constants: &[Constant],
) -> CompileResult<GlobalStructInitializerValue> {
    match expr {
        Expr::Cast { expr, .. } => value_from_expr(expr, constants),
        Expr::StringLiteral(value) => Ok(GlobalStructInitializerValue::String(value.clone())),
        Expr::AddressOf { target } => {
            address_from_lvalue(target, constants).map(GlobalStructInitializerValue::Address)
        }
        _ => eval_integer_initializer_expr_with_constants(expr, constants)
            .and_then(super::InitializerNumber::to_i64_trunc)
            .map(GlobalStructInitializerValue::Integer),
    }
}

fn address_from_lvalue(
    target: &LValue,
    constants: &[Constant],
) -> CompileResult<GlobalStructInitializerAddress> {
    match target {
        LValue::Identifier(base) => Ok(GlobalStructInitializerAddress {
            base: base.clone(),
            index: None,
        }),
        LValue::Subscript { array, index } => {
            let Expr::Identifier(base) = array.as_ref() else {
                return Err(CompileError::new(
                    "unsupported global struct initializer address",
                ));
            };
            let index =
                eval_integer_initializer_expr_with_constants(index, constants)?.to_i64_trunc()?;
            Ok(GlobalStructInitializerAddress {
                base: base.clone(),
                index: Some(usize::try_from(index).map_err(|_| {
                    CompileError::new("global struct initializer address index is negative")
                })?),
            })
        }
        LValue::Member { .. } => Err(CompileError::new(
            "unsupported global struct initializer address",
        )),
    }
}
