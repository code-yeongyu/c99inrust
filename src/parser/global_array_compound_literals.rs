use crate::diagnostics::{CompileError, CompileResult};

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
    if backing.element_byte_size == 1 {
        return byte_array_compound_literal_initializer(backing, values, constants);
    }
    if backing.element_type == ScalarType::Int {
        return int_array_compound_literal_initializer(backing.length, values, constants);
    }
    if matches!(
        backing.element_type,
        ScalarType::Double | ScalarType::LongDouble
    ) {
        return real_array_compound_literal_initializer(backing, values, constants);
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
    let value = eval_integer_initializer_expr_with_constants(expr, constants)?.to_i64_trunc()?;
    i32::try_from(value)
        .map_err(|_| CompileError::new("global compound literal integer does not fit i32"))
}

fn byte_initializer_value(
    expr: &Expr,
    constants: &[Constant],
    is_unsigned: bool,
) -> CompileResult<u8> {
    let value = eval_integer_initializer_expr_with_constants(expr, constants)?.to_i64_trunc()?;
    if is_unsigned || value >= 0 {
        return u8::try_from(value)
            .map_err(|_| CompileError::new("global compound literal byte does not fit u8"));
    }
    i8::try_from(value)
        .map(|signed| signed.to_ne_bytes()[0])
        .map_err(|_| CompileError::new("global compound literal byte does not fit i8"))
}

fn real_initializer_value(expr: &Expr, constants: &[Constant]) -> CompileResult<String> {
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
