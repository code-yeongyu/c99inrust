use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::ScalarType;

use super::{POINTER_REFERENT, scalar_size};

pub(in crate::ir) fn byte_size(referent: &str) -> Option<usize> {
    if is_pointer(referent) || is_function_pointer(referent) {
        Some(scalar_size(ScalarType::Pointer))
    } else if matches!(referent, "byte" | "char") {
        Some(1)
    } else if matches!(referent, "short" | "unsigned short") {
        Some(2)
    } else if referent == "int" {
        Some(scalar_size(ScalarType::Int))
    } else if referent == "long long" {
        Some(scalar_size(ScalarType::LongLong))
    } else if referent == "float" {
        Some(4)
    } else if referent == "double" {
        Some(scalar_size(ScalarType::Double))
    } else if referent == "long double" {
        Some(scalar_size(ScalarType::LongDouble))
    } else if referent == "float _Complex" {
        Some(scalar_size(ScalarType::ComplexFloat))
    } else if referent == "double _Complex" {
        Some(scalar_size(ScalarType::ComplexDouble))
    } else if referent == "long double _Complex" {
        Some(scalar_size(ScalarType::ComplexLongDouble))
    } else {
        None
    }
}

pub(in crate::ir) fn is_pointer(referent: &str) -> bool {
    referent.starts_with(POINTER_REFERENT)
}

pub(in crate::ir) fn is_function_pointer(referent: &str) -> bool {
    referent.starts_with("function ")
}

pub(in crate::ir) fn is_unsigned_integer(referent: &str) -> bool {
    matches!(referent, "byte" | "unsigned short")
}

pub(in crate::ir) fn nested_referent(referent: Option<&str>) -> String {
    let mut nested = POINTER_REFERENT.to_owned();
    if let Some(referent) = referent {
        nested.push_str(referent);
    }
    nested
}

pub(in crate::ir) fn difference_stride(left: usize, right: usize) -> CompileResult<usize> {
    if left == right {
        Ok(left)
    } else {
        Err(CompileError::new(
            "pointer subtraction requires matching referent sizes",
        ))
    }
}
