use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Expr, FieldType};

use super::{LoweredExpr, LoweringContext, doom_alloc, pointer_field_address};

pub(super) fn lower(
    context: &LoweringContext,
    callee: &str,
    index: usize,
    arg: &Expr,
) -> CompileResult<LoweredExpr> {
    if callee == "Z_Malloc"
        && index == 0
        && let Some(arg) = doom_alloc::widened_size_arg(arg)
    {
        return context.lower_expr(&arg);
    }
    if let Some(pointer) = decay_struct_array_row(context, arg)? {
        return Ok(pointer);
    }
    context.lower_expr(arg)
}

fn decay_struct_array_row(
    context: &LoweringContext,
    arg: &Expr,
) -> CompileResult<Option<LoweredExpr>> {
    let Expr::Subscript { array, index } = arg else {
        return Ok(None);
    };
    let Expr::Member {
        base,
        field,
        dereference,
    } = array.as_ref()
    else {
        return Ok(None);
    };
    let member = context.resolve_member_access(base, field, *dereference)?;
    let FieldType::Array {
        element_size,
        columns: Some(columns),
        ..
    } = member.field_type
    else {
        return Ok(None);
    };
    let row_byte_size = columns
        .checked_mul(element_size)
        .ok_or_else(|| CompileError::new("struct array row size overflow"))?;
    Ok(Some(LoweredExpr::PointerOffset {
        pointer: Box::new(pointer_field_address(member.pointer, member.offset)),
        index: Box::new(context.lower_expr(index)?),
        byte_size: row_byte_size,
    }))
}
