use crate::diagnostics::{CompileError, CompileResult};
use crate::front_end::lexer::{Keyword, Token};

use super::global_array_compound_literals::{
    GlobalArrayCompoundLiteralBacking, global_array_compound_literal_initializer,
};
use super::global_scalar_compound_literals::scalar_compound_literal_globals;
use super::global_specifiers::global_specifiers_are_pointer;
use super::integer_initializer::eval_integer_initializer_expr_with_constants;
use super::token_scan::{
    previous_identifier_index, token_has_keyword, token_identifier, top_level_punctuator_index,
};
use super::{
    Constant, Expr, Global, GlobalInitializer, GlobalStructInitializerValue, LValue,
    LocalStructInitializerValue, Parser, StructLayout, pointer_referent_from_specifiers,
};

pub(super) fn parse_global_pointer_compound_literal(
    tokens: &[Token],
    known_structs: &[StructLayout],
    constants: &[Constant],
) -> CompileResult<Option<Vec<Global>>> {
    let Some(declaration) = tokens.get(..tokens.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let Some(end_index) = top_level_punctuator_index(declaration, "=") else {
        return Ok(None);
    };
    if top_level_punctuator_index(&declaration[..end_index], "[").is_some() {
        return Ok(None);
    }
    let Some(name_index) = previous_identifier_index(declaration, end_index) else {
        return Ok(None);
    };
    if !global_specifiers_are_pointer(&declaration[..name_index]) {
        return Ok(None);
    }
    let name = token_identifier(&declaration[name_index])
        .ok_or_else(|| CompileError::new("expected global pointer name"))?
        .to_owned();
    let referent = pointer_referent_from_specifiers(&declaration[..name_index]);
    let expr = parse_initializer_expr(&declaration[end_index + 1..], known_structs, constants)?;
    let Some(mut globals) = compound_literal_globals(&name, referent, expr, constants)? else {
        return Ok(None);
    };
    if token_has_keyword(declaration, Keyword::Static) {
        let Some(pointer) = globals.last_mut() else {
            return Err(CompileError::new("missing compound literal pointer global"));
        };
        pointer.is_static = true;
    }
    Ok(Some(globals))
}

fn parse_initializer_expr(
    tokens: &[Token],
    known_structs: &[StructLayout],
    constants: &[Constant],
) -> CompileResult<Expr> {
    let mut parser = Parser {
        tokens,
        index: 0,
        known_structs,
        known_constants: constants,
        known_scalar_typedefs: &[],
        known_pointer_typedefs: &[],
        known_function_pointer_typedefs: &[],
    };
    let expr = parser.expression()?;
    if let Some(token) = parser.peek() {
        return Err(
            CompileError::new("unsupported global compound literal pointer initializer")
                .at(token.line, token.column),
        );
    }
    Ok(expr)
}

fn compound_literal_globals(
    name: &str,
    referent: Option<String>,
    expr: Expr,
    constants: &[Constant],
) -> CompileResult<Option<Vec<Global>>> {
    match expr {
        Expr::ArrayCompoundLiteral {
            element_type,
            element_byte_size,
            element_unsigned,
            length,
            values,
            ..
        } => array_compound_literal_globals(
            name,
            referent,
            GlobalArrayCompoundLiteralBacking {
                element_type,
                element_byte_size,
                element_unsigned,
                length,
            },
            &values,
            constants,
        )
        .map(Some),
        Expr::AddressOf {
            target:
                LValue::ScalarCompoundLiteral {
                    scalar_type,
                    referent: scalar_referent,
                    value,
                },
        } => scalar_compound_literal_globals(
            name,
            referent,
            scalar_type,
            scalar_referent.as_deref(),
            value.as_ref(),
            constants,
        )
        .map(Some),
        Expr::AddressOf {
            target:
                LValue::StructCompoundLiteral {
                    struct_name,
                    values,
                },
        } => struct_compound_literal_globals(name, referent, struct_name, &values, constants)
            .map(Some),
        _ => Ok(None),
    }
}

fn array_compound_literal_globals(
    name: &str,
    referent: Option<String>,
    backing: GlobalArrayCompoundLiteralBacking,
    values: &[Expr],
    constants: &[Constant],
) -> CompileResult<Vec<Global>> {
    if values.len() > backing.length {
        return Err(CompileError::new(
            "global array compound literal initializer is too large",
        ));
    }
    let backing_name = compound_backing_name(name);
    let initializer = global_array_compound_literal_initializer(backing, values, constants)?;
    Ok(pointer_with_backing(
        name,
        referent,
        backing_name,
        initializer,
    ))
}

fn struct_compound_literal_globals(
    name: &str,
    referent: Option<String>,
    struct_name: String,
    values: &[LocalStructInitializerValue],
    constants: &[Constant],
) -> CompileResult<Vec<Global>> {
    let backing_name = compound_backing_name(name);
    let values = values
        .iter()
        .map(|value| global_struct_value(value, constants))
        .collect::<CompileResult<Vec<_>>>()?;
    Ok(pointer_with_backing(
        name,
        referent,
        backing_name,
        GlobalInitializer::StructObject {
            struct_name,
            values,
        },
    ))
}

fn pointer_with_backing(
    name: &str,
    referent: Option<String>,
    backing_name: String,
    backing_initializer: GlobalInitializer,
) -> Vec<Global> {
    let mut backing = Global::new(backing_name.clone(), backing_initializer);
    backing.is_static = true;
    let pointer = Global::new(
        name.to_owned(),
        GlobalInitializer::PointerName {
            referent,
            value: backing_name,
        },
    );
    vec![backing, pointer]
}

fn global_struct_value(
    value: &LocalStructInitializerValue,
    constants: &[Constant],
) -> CompileResult<GlobalStructInitializerValue> {
    match value {
        LocalStructInitializerValue::Expr(expr) => match expr {
            Expr::StringLiteral(value) => Ok(GlobalStructInitializerValue::String(value.clone())),
            expr => int_initializer_value(expr, constants)
                .map(i64::from)
                .map(GlobalStructInitializerValue::Integer),
        },
        LocalStructInitializerValue::Nested(values) => values
            .iter()
            .map(|value| global_struct_value(value, constants))
            .collect::<CompileResult<Vec<_>>>()
            .map(GlobalStructInitializerValue::Nested),
    }
}

fn int_initializer_value(expr: &Expr, constants: &[Constant]) -> CompileResult<i32> {
    let value = integer_initializer_value(expr, constants)?;
    i32::try_from(value)
        .map_err(|_| CompileError::new("global compound literal integer does not fit i32"))
}

fn integer_initializer_value(expr: &Expr, constants: &[Constant]) -> CompileResult<i64> {
    eval_integer_initializer_expr_with_constants(expr, constants)?.to_i64_trunc()
}

fn compound_backing_name(name: &str) -> String {
    format!("__c99inrust_compound_{name}")
}
