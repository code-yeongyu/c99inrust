use crate::diagnostics::{CompileError, CompileResult};

use super::global_array_compound_literals::real_initializer_value;
use super::integer_initializer::eval_integer_initializer_expr_with_constants;
use super::{Constant, Expr, Global, GlobalInitializer, ScalarType};

pub(super) fn scalar_compound_literal_globals(
    name: &str,
    referent: Option<String>,
    scalar_type: ScalarType,
    scalar_referent: Option<&str>,
    value: &Expr,
    constants: &[Constant],
) -> CompileResult<Vec<Global>> {
    let backing_name = compound_backing_name(name);
    let initializer =
        scalar_compound_literal_initializer(scalar_type, scalar_referent, value, constants)?;
    Ok(pointer_with_backing(
        name,
        referent,
        backing_name,
        initializer,
    ))
}

fn scalar_compound_literal_initializer(
    scalar_type: ScalarType,
    scalar_referent: Option<&str>,
    value: &Expr,
    constants: &[Constant],
) -> CompileResult<GlobalInitializer> {
    match scalar_referent {
        Some("byte" | "char") => Ok(GlobalInitializer::UnsignedCharArray {
            values: vec![byte_initializer_value(value, constants)?],
            is_unsigned: scalar_referent == Some("byte"),
        }),
        Some("unsigned short" | "short") => Ok(GlobalInitializer::ShortArray {
            values: vec![int_initializer_value(value, constants)?],
            is_unsigned: scalar_referent == Some("unsigned short"),
            columns: None,
        }),
        _ => scalar_compound_scalar_initializer(scalar_type, value, constants),
    }
}

fn scalar_compound_scalar_initializer(
    scalar_type: ScalarType,
    value: &Expr,
    constants: &[Constant],
) -> CompileResult<GlobalInitializer> {
    match scalar_type {
        ScalarType::Bool => Ok(GlobalInitializer::Bool(integer_initializer_value(
            value, constants,
        )?)),
        ScalarType::Int => Ok(GlobalInitializer::Int(integer_initializer_value(
            value, constants,
        )?)),
        ScalarType::LongLong => Ok(GlobalInitializer::LongLong(integer_initializer_value(
            value, constants,
        )?)),
        ScalarType::Double | ScalarType::LongDouble => Ok(GlobalInitializer::Double(
            real_initializer_value(value, constants)?,
        )),
        ScalarType::ComplexFloat | ScalarType::ComplexDouble | ScalarType::ComplexLongDouble => {
            Ok(GlobalInitializer::ComplexReal {
                scalar_type,
                real: real_initializer_value(value, constants)?,
            })
        }
        ScalarType::Pointer | ScalarType::VaList => Err(CompileError::new(
            "unsupported global scalar compound literal element type",
        )),
    }
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

fn int_initializer_value(expr: &Expr, constants: &[Constant]) -> CompileResult<i32> {
    let value = integer_initializer_value(expr, constants)?;
    i32::try_from(value)
        .map_err(|_| CompileError::new("global compound literal integer does not fit i32"))
}

fn byte_initializer_value(expr: &Expr, constants: &[Constant]) -> CompileResult<u8> {
    let value = integer_initializer_value(expr, constants)?;
    u8::try_from(value)
        .or_else(|_| i8::try_from(value).map(|signed| signed.to_ne_bytes()[0]))
        .map_err(|_| CompileError::new("global compound literal byte does not fit u8"))
}

fn integer_initializer_value(expr: &Expr, constants: &[Constant]) -> CompileResult<i64> {
    eval_integer_initializer_expr_with_constants(expr, constants)?.to_i64_trunc()
}

fn compound_backing_name(name: &str) -> String {
    format!("__c99inrust_compound_{name}")
}
