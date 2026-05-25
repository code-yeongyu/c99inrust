use crate::diagnostics::{CompileError, CompileResult};

use super::global_array_compound_literals::{
    GlobalArrayCompoundLiteralBacking, global_array_compound_literal_initializer,
};
use super::global_pointer_compound_array_element_addresses::{
    element_address_globals, element_address_offset_globals, element_address_subtract_globals,
};
use super::integer_initializer::eval_integer_initializer_expr_with_constants;
use super::{BinaryOp, Constant, Expr, Global, GlobalInitializer};

pub(super) fn array_compound_literal_globals(
    name: &str,
    referent: Option<String>,
    expr: &Expr,
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
        } => array_globals(
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
        )
        .map(Some),
        Expr::Binary {
            op: BinaryOp::Add,
            left,
            right,
        } => offset_globals(name, referent, left, right, constants),
        Expr::Binary {
            op: BinaryOp::Sub,
            left,
            right,
        } => subtract_globals(name, referent, left, right, constants),
        Expr::AddressOf { .. } => element_address_globals(name, referent, expr, constants),
        _ => Ok(None),
    }
}

fn offset_globals(
    name: &str,
    referent: Option<String>,
    left: &Expr,
    right: &Expr,
    constants: &[Constant],
) -> CompileResult<Option<Vec<Global>>> {
    if let Some(globals) = array_offset_globals(name, referent.clone(), left, right, constants)? {
        return Ok(Some(globals));
    }
    if let Some(globals) =
        element_address_offset_globals(name, referent.clone(), left, right, constants)?
    {
        return Ok(Some(globals));
    }
    if let Some(globals) = array_offset_globals(name, referent.clone(), right, left, constants)? {
        return Ok(Some(globals));
    }
    element_address_offset_globals(name, referent, right, left, constants)
}

fn subtract_globals(
    name: &str,
    referent: Option<String>,
    left: &Expr,
    right: &Expr,
    constants: &[Constant],
) -> CompileResult<Option<Vec<Global>>> {
    if let Some(globals) = array_subtract_globals(name, referent.clone(), left, right, constants)? {
        return Ok(Some(globals));
    }
    element_address_subtract_globals(name, referent, left, right, constants)
}

fn array_offset_globals(
    name: &str,
    referent: Option<String>,
    array: &Expr,
    offset: &Expr,
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
        compound_pointer_add_delta(0, compound_pointer_delta(offset, constants)?)?,
    )
    .map(Some)
}

fn array_subtract_globals(
    name: &str,
    referent: Option<String>,
    array: &Expr,
    offset: &Expr,
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
        compound_pointer_subtract_delta(0, compound_pointer_delta(offset, constants)?)?,
    )
    .map(Some)
}

fn array_globals(
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
    let mut backing = Global::new(backing_name.clone(), initializer);
    backing.is_static = true;
    let pointer = Global::new(
        name.to_owned(),
        GlobalInitializer::PointerName {
            referent,
            value: backing_name,
        },
    );
    Ok(vec![backing, pointer])
}

pub(super) fn array_globals_at_index(
    name: &str,
    referent: Option<String>,
    backing: GlobalArrayCompoundLiteralBacking,
    values: &[Expr],
    constants: &[Constant],
    index: usize,
) -> CompileResult<Vec<Global>> {
    if values.len() > backing.length {
        return Err(CompileError::new(
            "global array compound literal initializer is too large",
        ));
    }
    let backing_name = compound_backing_name(name);
    let initializer = global_array_compound_literal_initializer(backing, values, constants)?;
    let mut backing = Global::new(backing_name.clone(), initializer);
    backing.is_static = true;
    let pointer = Global::new(
        name.to_owned(),
        GlobalInitializer::PointerSubscriptAddress {
            referent,
            base: backing_name,
            index,
        },
    );
    Ok(vec![backing, pointer])
}

pub(super) fn compound_pointer_offset(expr: &Expr, constants: &[Constant]) -> CompileResult<usize> {
    let index = eval_integer_initializer_expr_with_constants(expr, constants)?.to_i64_trunc()?;
    if index < 0 {
        return Err(CompileError::new(
            "global compound literal pointer offset must be nonnegative",
        ));
    }
    usize::try_from(index)
        .map_err(|_| CompileError::new("global compound literal pointer offset is too large"))
}

pub(super) fn compound_pointer_delta(expr: &Expr, constants: &[Constant]) -> CompileResult<isize> {
    let delta = eval_integer_initializer_expr_with_constants(expr, constants)?.to_i64_trunc()?;
    isize::try_from(delta)
        .map_err(|_| CompileError::new("global compound literal pointer offset is too large"))
}

pub(super) fn compound_pointer_add_delta(base: usize, delta: isize) -> CompileResult<usize> {
    if delta >= 0 {
        let delta = usize::try_from(delta).map_err(|_| {
            CompileError::new("global compound literal pointer offset is too large")
        })?;
        return base.checked_add(delta).ok_or_else(|| {
            CompileError::new("global compound literal pointer offset is too large")
        });
    }
    base.checked_sub(pointer_delta_magnitude(delta)?)
        .ok_or_else(|| CompileError::new("global compound literal pointer offset is negative"))
}

pub(super) fn compound_pointer_subtract_delta(base: usize, delta: isize) -> CompileResult<usize> {
    if delta >= 0 {
        let delta = usize::try_from(delta).map_err(|_| {
            CompileError::new("global compound literal pointer offset is too large")
        })?;
        return base.checked_sub(delta).ok_or_else(|| {
            CompileError::new("global compound literal pointer offset is negative")
        });
    }
    base.checked_add(pointer_delta_magnitude(delta)?)
        .ok_or_else(|| CompileError::new("global compound literal pointer offset is too large"))
}

fn pointer_delta_magnitude(delta: isize) -> CompileResult<usize> {
    delta
        .checked_neg()
        .and_then(|magnitude| usize::try_from(magnitude).ok())
        .ok_or_else(|| CompileError::new("global compound literal pointer offset is too large"))
}

fn compound_backing_name(name: &str) -> String {
    format!("__c99inrust_compound_{name}")
}
