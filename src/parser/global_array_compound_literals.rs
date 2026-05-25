use crate::diagnostics::{CompileError, CompileResult};

use super::global_string_initializers::string_pointer_initializer_expr;
use super::{
    Constant, Expr, GlobalInitializer, ScalarType, UnaryOp,
    eval_integer_initializer_expr_with_constants,
};

#[derive(Clone, Copy)]
pub(super) struct GlobalArrayCompoundLiteralBacking {
    pub(super) element_type: ScalarType,
    pub(super) element_byte_size: usize,
    pub(super) element_unsigned: bool,
    pub(super) length: usize,
}

pub(super) fn global_array_compound_literal_initializer(
    backing: GlobalArrayCompoundLiteralBacking,
    values: &[Expr],
    constants: &[Constant],
) -> CompileResult<GlobalInitializer> {
    if backing.element_type == ScalarType::Bool {
        return bool_array_compound_literal_initializer(backing.length, values, constants);
    }
    if backing.element_byte_size == 1 {
        return byte_array_compound_literal_initializer(backing, values, constants);
    }
    if backing.element_type == ScalarType::Int && backing.element_byte_size == 2 {
        return short_array_compound_literal_initializer(backing, values, constants);
    }
    if backing.element_type == ScalarType::Int {
        return int_array_compound_literal_initializer(backing.length, values, constants);
    }
    if backing.element_type == ScalarType::LongLong {
        return long_long_array_compound_literal_initializer(backing.length, values, constants);
    }
    if matches!(
        backing.element_type,
        ScalarType::ComplexFloat
            | ScalarType::ComplexDouble
            | ScalarType::ComplexLongDouble
            | ScalarType::Double
            | ScalarType::LongDouble
    ) {
        return real_array_compound_literal_initializer(backing, values, constants);
    }
    if backing.element_type == ScalarType::Pointer {
        return pointer_array_compound_literal_initializer(backing.length, values, constants);
    }
    Err(CompileError::new(
        "unsupported global array compound literal element type",
    ))
}

fn int_array_compound_literal_initializer(
    length: usize,
    values: &[Expr],
    constants: &[Constant],
) -> CompileResult<GlobalInitializer> {
    let mut array_values = Vec::with_capacity(length);
    for value in values {
        array_values.push(int_initializer_value(value, constants)?);
    }
    array_values.resize(length, 0);
    Ok(GlobalInitializer::IntArray(array_values))
}

fn byte_array_compound_literal_initializer(
    backing: GlobalArrayCompoundLiteralBacking,
    values: &[Expr],
    constants: &[Constant],
) -> CompileResult<GlobalInitializer> {
    let mut array_values = Vec::with_capacity(backing.length);
    for value in values {
        array_values.push(byte_initializer_value(
            value,
            constants,
            backing.element_unsigned,
        )?);
    }
    array_values.resize(backing.length, 0);
    Ok(GlobalInitializer::UnsignedCharArray {
        values: array_values,
        is_unsigned: backing.element_unsigned,
    })
}

fn bool_array_compound_literal_initializer(
    length: usize,
    values: &[Expr],
    constants: &[Constant],
) -> CompileResult<GlobalInitializer> {
    let mut array_values = Vec::with_capacity(length);
    for value in values {
        array_values.push(u8::from(integer_initializer_value(value, constants)? != 0));
    }
    array_values.resize(length, 0);
    Ok(GlobalInitializer::BoolArray(array_values))
}

fn short_array_compound_literal_initializer(
    backing: GlobalArrayCompoundLiteralBacking,
    values: &[Expr],
    constants: &[Constant],
) -> CompileResult<GlobalInitializer> {
    let mut array_values = Vec::with_capacity(backing.length);
    for value in values {
        array_values.push(int_initializer_value(value, constants)?);
    }
    array_values.resize(backing.length, 0);
    Ok(GlobalInitializer::ShortArray {
        values: array_values,
        is_unsigned: backing.element_unsigned,
        columns: None,
    })
}

fn long_long_array_compound_literal_initializer(
    length: usize,
    values: &[Expr],
    constants: &[Constant],
) -> CompileResult<GlobalInitializer> {
    let mut array_values = Vec::with_capacity(length);
    for value in values {
        array_values.push(integer_initializer_value(value, constants)?);
    }
    array_values.resize(length, 0);
    Ok(GlobalInitializer::LongLongArray(array_values))
}

fn pointer_array_compound_literal_initializer(
    length: usize,
    values: &[Expr],
    constants: &[Constant],
) -> CompileResult<GlobalInitializer> {
    let mut array_values = Vec::with_capacity(length);
    for value in values {
        array_values.push(pointer_initializer_value(value, constants)?);
    }
    array_values.resize(length, None);
    Ok(GlobalInitializer::PointerStringArray {
        referent: Some("char".to_owned()),
        values: array_values,
        length,
    })
}

fn real_array_compound_literal_initializer(
    backing: GlobalArrayCompoundLiteralBacking,
    values: &[Expr],
    constants: &[Constant],
) -> CompileResult<GlobalInitializer> {
    let values = values
        .iter()
        .map(|value| real_initializer_value(value, constants))
        .collect::<CompileResult<Vec<_>>>()?;
    Ok(GlobalInitializer::ScalarArrayValues {
        scalar_type: backing.element_type,
        length: backing.length,
        values,
    })
}

fn int_initializer_value(expr: &Expr, constants: &[Constant]) -> CompileResult<i32> {
    let value = integer_initializer_value(expr, constants)?;
    i32::try_from(value)
        .map_err(|_| CompileError::new("global compound literal integer does not fit i32"))
}

fn byte_initializer_value(
    expr: &Expr,
    constants: &[Constant],
    is_unsigned: bool,
) -> CompileResult<u8> {
    let value = integer_initializer_value(expr, constants)?;
    if is_unsigned || value >= 0 {
        return u8::try_from(value)
            .map_err(|_| CompileError::new("global compound literal byte does not fit u8"));
    }
    i8::try_from(value)
        .map(|signed| signed.to_ne_bytes()[0])
        .map_err(|_| CompileError::new("global compound literal byte does not fit i8"))
}

fn pointer_initializer_value(
    expr: &Expr,
    constants: &[Constant],
) -> CompileResult<Option<(String, usize)>> {
    if matches!(expr, Expr::Integer(0) | Expr::LongInteger(0)) {
        return Ok(None);
    }
    string_pointer_initializer_expr(expr, constants)?.map_or_else(
        || {
            Err(CompileError::new(
                "unsupported global pointer compound literal value",
            ))
        },
        |value| Ok(Some(value)),
    )
}

pub(super) fn real_initializer_value(expr: &Expr, constants: &[Constant]) -> CompileResult<String> {
    match expr {
        Expr::DoubleLiteral(value) => Ok(value.clone()),
        Expr::Integer(value) | Expr::LongInteger(value) => Ok(value.to_string()),
        Expr::Unary {
            op: UnaryOp::Plus,
            expr,
        } => real_initializer_value(expr, constants),
        Expr::Unary {
            op: UnaryOp::Minus,
            expr,
        } => real_initializer_value(expr, constants).map(|value| format!("-{value}")),
        expr => eval_integer_initializer_expr_with_constants(expr, constants)?
            .to_i64_trunc()
            .map(|value| value.to_string()),
    }
}

fn integer_initializer_value(expr: &Expr, constants: &[Constant]) -> CompileResult<i64> {
    eval_integer_initializer_expr_with_constants(expr, constants)?.to_i64_trunc()
}
