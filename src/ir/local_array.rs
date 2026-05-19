use crate::diagnostics::CompileResult;
use crate::parser::Expr;

use super::{LocalBinding, LoweredExpr};

pub(super) fn char_array_pointer<F>(
    array: &Expr,
    binding: Option<&LocalBinding>,
    local_offset: F,
) -> CompileResult<Option<LoweredExpr>>
where
    F: FnOnce(usize) -> CompileResult<usize>,
{
    let Expr::Identifier(_) = array else {
        return Ok(None);
    };
    let Some(LocalBinding::CharArray { slot, length }) = binding else {
        return Ok(None);
    };
    Ok(Some(LoweredExpr::LocalAddress {
        offset: local_offset(*slot)?,
        byte_size: *length,
    }))
}
