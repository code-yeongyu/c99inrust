use crate::diagnostics::{CompileError, CompileResult};

use super::global_array_compound_literals::GlobalArrayCompoundLiteralBacking;
use super::global_pointer_compound_array_literals::{
    array_globals_at_index, compound_pointer_offset,
};
use super::{Constant, Expr, Global, LValue};

pub(super) fn element_address_globals(
    name: &str,
    referent: Option<String>,
    pointer: &Expr,
    constants: &[Constant],
) -> CompileResult<Option<Vec<Global>>> {
    let Some((array, index)) = element_address_parts(pointer) else {
        return Ok(None);
    };
    let index = compound_pointer_offset(index, constants)?;
    array_element_globals_at_index(name, referent, array, index, constants)
}

pub(super) fn element_address_offset_globals(
    name: &str,
    referent: Option<String>,
    pointer: &Expr,
    offset: &Expr,
    constants: &[Constant],
) -> CompileResult<Option<Vec<Global>>> {
    let Some((array, base_index)) = element_address_parts(pointer) else {
        return Ok(None);
    };
    let index = compound_pointer_offset(base_index, constants)?
        .checked_add(compound_pointer_offset(offset, constants)?)
        .ok_or_else(|| CompileError::new("global compound literal pointer offset is too large"))?;
    array_element_globals_at_index(name, referent, array, index, constants)
}

pub(super) fn element_address_subtract_globals(
    name: &str,
    referent: Option<String>,
    pointer: &Expr,
    offset: &Expr,
    constants: &[Constant],
) -> CompileResult<Option<Vec<Global>>> {
    let Some((array, base_index)) = element_address_parts(pointer) else {
        return Ok(None);
    };
    let index = compound_pointer_offset(base_index, constants)?
        .checked_sub(compound_pointer_offset(offset, constants)?)
        .ok_or_else(|| CompileError::new("global compound literal pointer offset is negative"))?;
    array_element_globals_at_index(name, referent, array, index, constants)
}

fn element_address_parts(pointer: &Expr) -> Option<(&Expr, &Expr)> {
    let Expr::AddressOf {
        target: LValue::Subscript { array, index },
    } = pointer
    else {
        return None;
    };
    matches!(array.as_ref(), Expr::ArrayCompoundLiteral { .. }).then_some((array, index))
}

fn array_element_globals_at_index(
    name: &str,
    referent: Option<String>,
    array: &Expr,
    index: usize,
    constants: &[Constant],
) -> CompileResult<Option<Vec<Global>>> {
    let Expr::ArrayCompoundLiteral {
        element_type,
        element_byte_size,
        element_unsigned,
        length,
        values,
        ..
    } = array
    else {
        return Ok(None);
    };
    array_globals_at_index(
        name,
        referent,
        GlobalArrayCompoundLiteralBacking {
            element_type: *element_type,
            element_byte_size: *element_byte_size,
            element_unsigned: *element_unsigned,
            length: *length,
        },
        values,
        constants,
        index,
    )
    .map(Some)
}
