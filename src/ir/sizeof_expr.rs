use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::{Expr, ScalarType};

use super::{
    GlobalBinding, LocalBinding, LoweredExpr, LoweringContext, local_char_matrix_byte_size,
    local_int_array_byte_size, local_pointer_array_byte_size, local_short_array_byte_size,
    lowered_expr_scalar_type, scalar_size,
};

pub(in crate::ir) fn lower(context: &LoweringContext, expr: &Expr) -> CompileResult<LoweredExpr> {
    if let Some(size) = expression_size(context, expr)? {
        return integer(size);
    }

    let lowered = context.lower_expr(expr)?;
    let size = lowered_expr_scalar_type(&lowered).map_or(4, scalar_size);
    integer(size)
}

fn expression_size(context: &LoweringContext, expr: &Expr) -> CompileResult<Option<usize>> {
    match expr {
        Expr::Identifier(name) => identifier_size(context, name),
        Expr::Dereference { pointer } => pointer_element_size(context, pointer).map(Some),
        Expr::Subscript { array, .. } => Ok(subscript_size(context, array)),
        _ => Ok(None),
    }
}

fn subscript_size(context: &LoweringContext, array: &Expr) -> Option<usize> {
    if let Expr::Identifier(name) = array {
        if let Some(LocalBinding::CharMatrix { columns, .. }) = context.local_binding(name) {
            return Some(columns);
        }
        if let Some(GlobalBinding::UnsignedCharMatrix { columns, .. }) =
            context.global_bindings.get(name)
        {
            return Some(*columns);
        }
    }
    pointer_element_size(context, array).ok()
}

fn identifier_size(context: &LoweringContext, name: &str) -> CompileResult<Option<usize>> {
    if let Some(binding) = context.local_binding(name) {
        return local_binding_size(&binding).map(Some);
    }
    match context.global_bindings.get(name) {
        Some(GlobalBinding::StructArray {
            byte_size,
            length: Some(length),
            ..
        }) => byte_size
            .checked_mul(*length)
            .ok_or_else(|| CompileError::new("sizeof global array overflow"))
            .map(Some),
        Some(GlobalBinding::StructObject { byte_size, .. }) => Ok(Some(*byte_size)),
        _ => Ok(None),
    }
}

fn local_binding_size(binding: &LocalBinding) -> CompileResult<usize> {
    match binding {
        LocalBinding::Scalar { scalar_type, .. }
        | LocalBinding::StaticScalar { scalar_type, .. } => Ok(scalar_size(*scalar_type)),
        LocalBinding::CharArray { length, .. } => Ok(*length),
        LocalBinding::CharMatrix { rows, columns, .. } => {
            local_char_matrix_byte_size(*rows, *columns)
        }
        LocalBinding::IntArray { length, .. } => local_int_array_byte_size(*length),
        LocalBinding::ShortArray { length, .. } => local_short_array_byte_size(*length),
        LocalBinding::PointerArray { length, .. } => local_pointer_array_byte_size(*length),
        LocalBinding::StructObject { byte_size, .. } => Ok(*byte_size),
        LocalBinding::StructArray {
            byte_size, length, ..
        } => byte_size
            .checked_mul(*length)
            .ok_or_else(|| CompileError::new("sizeof local struct array overflow")),
        LocalBinding::VaList { .. } => Ok(scalar_size(ScalarType::VaList)),
    }
}

fn pointer_element_size(context: &LoweringContext, pointer: &Expr) -> CompileResult<usize> {
    let referent = context.pointer_referent_for_expr(pointer)?;
    context.pointer_referent_stride(&referent)
}

fn integer(size: usize) -> CompileResult<LoweredExpr> {
    i64::try_from(size)
        .map(LoweredExpr::Integer)
        .map_err(|_| CompileError::new("sizeof result does not fit i64"))
}
