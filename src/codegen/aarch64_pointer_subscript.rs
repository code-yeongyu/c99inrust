use super::aarch64_addressing::aarch64_result_register;
use super::aarch64_expr::emit_aarch64_expr_with_width;
use super::aarch64_temporaries::{
    emit_aarch64_load_temporary, emit_aarch64_load_temporary_to_register,
    emit_aarch64_store_temporary,
};
use super::frames::LabelAllocator;
use super::stack_helpers::memory_scale_shift_for_byte_size;
use super::widths::{PointerSubscriptExpr, TEMPORARY_BYTES, ValueWidth, scalar_width};
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::LoweredExpr;

pub(in crate::codegen) fn emit_aarch64_store_pointer_subscript(
    subscript: PointerSubscriptExpr<'_>,
    value: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let width = scalar_width(subscript.element_type);
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
    let base_offset = temporary_base + ((depth + 1) * TEMPORARY_BYTES);
    emit_aarch64_expr_with_width(value, width, temporary_base, depth, labels, assembly)?;
    emit_aarch64_store_temporary(width, value_offset, assembly)?;
    emit_aarch64_expr_with_width(
        subscript.pointer,
        ValueWidth::I64,
        temporary_base,
        depth + 2,
        labels,
        assembly,
    )?;
    emit_aarch64_store_temporary(ValueWidth::I64, base_offset, assembly)?;
    emit_aarch64_expr_with_width(
        subscript.index,
        ValueWidth::I32,
        temporary_base,
        depth + 2,
        labels,
        assembly,
    )?;
    assembly.push_str("\tmov w17, w0\n");
    emit_aarch64_load_temporary_to_register(ValueWidth::I64, base_offset, "16", assembly)?;
    emit_aarch64_load_temporary(width, value_offset, assembly)?;
    if subscript.element_byte_size == 4 && width == ValueWidth::F64 {
        assembly.push_str("\tfcvt s0, d0\n");
        return write_assembly!(assembly, "\tstr s0, [x16, w17, sxtw #2]\n");
    }
    if subscript.element_byte_size == 1 && width == ValueWidth::I32 {
        return write_assembly!(assembly, "\tstrb w0, [x16, w17, sxtw]\n");
    }
    if subscript.element_byte_size == 2 && width == ValueWidth::I32 {
        return write_assembly!(assembly, "\tstrh w0, [x16, w17, sxtw #1]\n");
    }
    let Some(shift) = memory_scale_shift_for_byte_size(subscript.element_byte_size) else {
        return Err(CompileError::new(
            "unsupported pointer subscript element size",
        ));
    };
    write_assembly!(
        assembly,
        "\tstr {}, [x16, w17, sxtw #{}]\n",
        aarch64_result_register(width),
        shift
    )
}

pub(in crate::codegen) fn emit_aarch64_load_pointer_subscript_result(
    element_byte_size: usize,
    width: ValueWidth,
    element_unsigned: bool,
    assembly: &mut String,
) -> CompileResult<()> {
    if element_byte_size == 1 && width == ValueWidth::I32 && element_unsigned {
        return write_assembly!(assembly, "\tldrb w0, [x16, w17, sxtw]\n");
    }
    if element_byte_size == 1 && width == ValueWidth::I32 {
        return write_assembly!(assembly, "\tldrsb w0, [x16, w17, sxtw]\n");
    }
    if element_byte_size == 2 && width == ValueWidth::I32 && element_unsigned {
        return write_assembly!(assembly, "\tldrh w0, [x16, w17, sxtw #1]\n");
    }
    if element_byte_size == 2 && width == ValueWidth::I32 {
        return write_assembly!(assembly, "\tldrsh w0, [x16, w17, sxtw #1]\n");
    }
    if element_byte_size == 4 && width == ValueWidth::F64 {
        assembly.push_str("\tldr s0, [x16, w17, sxtw #2]\n");
        assembly.push_str("\tfcvt d0, s0\n");
        return Ok(());
    }
    let Some(shift) = memory_scale_shift_for_byte_size(element_byte_size) else {
        return Err(CompileError::new(
            "unsupported pointer subscript element size",
        ));
    };
    write_assembly!(
        assembly,
        "\tldr {}, [x16, w17, sxtw #{}]\n",
        aarch64_result_register(width),
        shift
    )
}

pub(in crate::codegen) fn emit_aarch64_store_pointer_subscript_result(
    element_byte_size: usize,
    width: ValueWidth,
    assembly: &mut String,
) -> CompileResult<()> {
    if element_byte_size == 4 && width == ValueWidth::F64 {
        assembly.push_str("\tfcvt s0, d0\n");
        return write_assembly!(assembly, "\tstr s0, [x16, w17, sxtw #2]\n");
    }
    if element_byte_size == 1 && width == ValueWidth::I32 {
        return write_assembly!(assembly, "\tstrb w0, [x16, w17, sxtw]\n");
    }
    if element_byte_size == 2 && width == ValueWidth::I32 {
        return write_assembly!(assembly, "\tstrh w0, [x16, w17, sxtw #1]\n");
    }
    let Some(shift) = memory_scale_shift_for_byte_size(element_byte_size) else {
        return Err(CompileError::new(
            "unsupported pointer subscript element size",
        ));
    };
    write_assembly!(
        assembly,
        "\tstr {}, [x16, w17, sxtw #{}]\n",
        aarch64_result_register(width),
        shift
    )
}
