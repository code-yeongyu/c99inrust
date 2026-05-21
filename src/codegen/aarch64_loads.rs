use super::aarch64_addressing::aarch64_result_register;
use super::aarch64_expr::emit_aarch64_expr_with_width;
use super::aarch64_temporaries::{
    emit_aarch64_load_temporary_to_register, emit_aarch64_store_temporary,
};
use super::data_literals::{
    double_literal_bits, emit_double_literal_data, emit_string_literal_data, label_name,
};
use super::frames::LabelAllocator;
use super::stack_helpers::memory_scale_shift_for_byte_size;
use super::target::Target;
use super::widths::{PointerSubscriptExpr, TEMPORARY_BYTES, ValueWidth, scalar_width};
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::LoweredExpr;

pub(in crate::codegen) fn emit_aarch64_load_double_literal(
    value: &str,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = labels.fresh();
    write_assembly!(assembly, "\tadrp x16, {label}@PAGE\n")?;
    write_assembly!(assembly, "\tldr d0, [x16, {label}@PAGEOFF]\n")?;
    emit_double_literal_data(&label, double_literal_bits(value)?, labels.target, assembly)
}

pub(in crate::codegen) fn emit_aarch64_load_string_address(
    value: &str,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = labels.fresh();
    write_assembly!(assembly, "\tadrp x0, {label}@PAGE\n")?;
    write_assembly!(assembly, "\tadd x0, x0, {label}@PAGEOFF\n")?;
    emit_string_literal_data(&label, value, labels.target, assembly)
}

pub(in crate::codegen) fn emit_aarch64_load_global(
    name: &str,
    width: ValueWidth,
    target: Target,
    assembly: &mut String,
) -> CompileResult<()> {
    if target != Target::Aarch64AppleDarwin {
        return Err(CompileError::new("unsupported AArch64 global target"));
    }
    let label = label_name(name, target);
    let register = aarch64_result_register(width);
    write_assembly!(assembly, "\tadrp x16, {label}@PAGE\n")?;
    write_assembly!(assembly, "\tldr {register}, [x16, {label}@PAGEOFF]\n")
}

pub(in crate::codegen) fn emit_aarch64_load_global_f32_as_f64(
    name: &str,
    target: Target,
    assembly: &mut String,
) -> CompileResult<()> {
    if target != Target::Aarch64AppleDarwin {
        return Err(CompileError::new("unsupported AArch64 global target"));
    }
    let label = label_name(name, target);
    write_assembly!(assembly, "\tadrp x16, {label}@PAGE\n")?;
    write_assembly!(assembly, "\tldr s0, [x16, {label}@PAGEOFF]\n")?;
    assembly.push_str("\tfcvt d0, s0\n");
    Ok(())
}

pub(in crate::codegen) fn emit_aarch64_store_global(
    name: &str,
    width: ValueWidth,
    target: Target,
    assembly: &mut String,
) -> CompileResult<()> {
    if target != Target::Aarch64AppleDarwin {
        return Err(CompileError::new("unsupported AArch64 global target"));
    }
    let label = label_name(name, target);
    let register = aarch64_result_register(width);
    write_assembly!(assembly, "\tadrp x16, {label}@PAGE\n")?;
    write_assembly!(assembly, "\tstr {register}, [x16, {label}@PAGEOFF]\n")
}

pub(in crate::codegen) fn emit_aarch64_load_global_byte_subscript(
    name: &str,
    index: &LoweredExpr,
    is_unsigned: bool,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = label_name(name, labels.target);
    emit_aarch64_expr_with_width(
        index,
        ValueWidth::I32,
        temporary_base,
        depth,
        labels,
        assembly,
    )?;
    write_assembly!(assembly, "\tadrp x16, {label}@PAGE\n")?;
    write_assembly!(assembly, "\tadd x16, x16, {label}@PAGEOFF\n")?;
    if is_unsigned {
        assembly.push_str("\tldrb w0, [x16, w0, sxtw]\n");
    } else {
        assembly.push_str("\tldrsb w0, [x16, w0, sxtw]\n");
    }
    Ok(())
}

pub(in crate::codegen) fn emit_aarch64_load_global_int_subscript(
    name: &str,
    index: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = label_name(name, labels.target);
    emit_aarch64_expr_with_width(
        index,
        ValueWidth::I32,
        temporary_base,
        depth,
        labels,
        assembly,
    )?;
    write_assembly!(assembly, "\tadrp x16, {label}@PAGE\n")?;
    write_assembly!(assembly, "\tadd x16, x16, {label}@PAGEOFF\n")?;
    assembly.push_str("\tldr w0, [x16, w0, sxtw #2]\n");
    Ok(())
}

pub(in crate::codegen) fn emit_aarch64_load_global_pointer_subscript(
    name: &str,
    index: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = label_name(name, labels.target);
    emit_aarch64_expr_with_width(
        index,
        ValueWidth::I32,
        temporary_base,
        depth,
        labels,
        assembly,
    )?;
    write_assembly!(assembly, "\tadrp x16, {label}@PAGE\n")?;
    write_assembly!(assembly, "\tadd x16, x16, {label}@PAGEOFF\n")?;
    assembly.push_str("\tldr x0, [x16, w0, sxtw #3]\n");
    Ok(())
}

pub(in crate::codegen) fn emit_aarch64_load_pointer_subscript(
    subscript: PointerSubscriptExpr<'_>,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let base_offset = temporary_base + (depth * TEMPORARY_BYTES);
    let width = scalar_width(subscript.element_type);
    emit_aarch64_expr_with_width(
        subscript.pointer,
        ValueWidth::I64,
        temporary_base,
        depth + 1,
        labels,
        assembly,
    )?;
    emit_aarch64_store_temporary(ValueWidth::I64, base_offset, assembly)?;
    emit_aarch64_expr_with_width(
        subscript.index,
        ValueWidth::I32,
        temporary_base,
        depth + 1,
        labels,
        assembly,
    )?;
    emit_aarch64_load_temporary_to_register(ValueWidth::I64, base_offset, "16", assembly)?;
    if subscript.element_byte_size == 1 && width == ValueWidth::I32 && subscript.element_unsigned {
        return write_assembly!(assembly, "\tldrb w0, [x16, w0, sxtw]\n");
    }
    if subscript.element_byte_size == 1 && width == ValueWidth::I32 {
        return write_assembly!(assembly, "\tldrsb w0, [x16, w0, sxtw]\n");
    }
    if subscript.element_byte_size == 2 && width == ValueWidth::I32 && subscript.element_unsigned {
        return write_assembly!(assembly, "\tldrh w0, [x16, w0, sxtw #1]\n");
    }
    if subscript.element_byte_size == 2 && width == ValueWidth::I32 {
        return write_assembly!(assembly, "\tldrsh w0, [x16, w0, sxtw #1]\n");
    }
    let Some(shift) = memory_scale_shift_for_byte_size(subscript.element_byte_size) else {
        return Err(CompileError::new(
            "unsupported pointer subscript element size",
        ));
    };
    write_assembly!(
        assembly,
        "\tldr {}, [x16, w0, sxtw #{}]\n",
        aarch64_result_register(width),
        shift
    )
}
