use super::aarch64_expr::emit_aarch64_expr_with_width;
use super::frames::{Aarch64VariadicFrame, LabelAllocator};
use super::stack_helpers::local_offset;
use super::widths::{TEMPORARY_BYTES, ValueWidth, scalar_width};
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::{LoweredExpr, LoweredFunction};
use crate::parser::ScalarType;

const AARCH64_VARIADIC_GP_REGISTERS: [&str; 8] = ["x0", "x1", "x2", "x3", "x4", "x5", "x6", "x7"];
const AARCH64_VARIADIC_GP_SAVE_BYTES: usize = 64;
const AARCH64_VARIADIC_FP_REGISTERS: [&str; 8] = ["d0", "d1", "d2", "d3", "d4", "d5", "d6", "d7"];
const AARCH64_VARIADIC_FP_OFFSET: usize = AARCH64_VARIADIC_GP_SAVE_BYTES;
const AARCH64_VARIADIC_FP_SAVE_BYTES: usize = 64;
const AARCH64_VARIADIC_REGISTER_SAVE_BYTES: usize =
    AARCH64_VARIADIC_GP_SAVE_BYTES + AARCH64_VARIADIC_FP_SAVE_BYTES;

pub(in crate::codegen) fn aarch64_variadic_frame(
    function: &LoweredFunction,
    stack_bytes: usize,
) -> CompileResult<Option<Aarch64VariadicFrame>> {
    let Some(slot) = function.variadic_save_slot else {
        return Ok(None);
    };
    let register_save_offset = local_offset(function, slot)?;
    let named_gp_args = function
        .parameter_count
        .min(AARCH64_VARIADIC_GP_REGISTERS.len());
    let stack_named_args = function
        .parameter_count
        .saturating_sub(AARCH64_VARIADIC_GP_REGISTERS.len());
    Ok(Some(Aarch64VariadicFrame {
        gp_offset: named_gp_args
            .checked_mul(TEMPORARY_BYTES)
            .ok_or_else(|| CompileError::new("variadic gp offset overflow"))?,
        fp_offset: AARCH64_VARIADIC_FP_OFFSET
            .checked_add(
                function
                    .parameter_count
                    .min(AARCH64_VARIADIC_FP_REGISTERS.len())
                    .checked_mul(TEMPORARY_BYTES)
                    .ok_or_else(|| CompileError::new("variadic fp offset overflow"))?,
            )
            .ok_or_else(|| CompileError::new("variadic fp offset overflow"))?,
        overflow_arg_offset: stack_bytes
            .checked_add(
                stack_named_args
                    .checked_mul(TEMPORARY_BYTES)
                    .ok_or_else(|| CompileError::new("variadic overflow offset overflow"))?,
            )
            .ok_or_else(|| CompileError::new("variadic overflow offset overflow"))?,
        register_save_offset,
        register_save_size: AARCH64_VARIADIC_REGISTER_SAVE_BYTES,
    }))
}

pub(in crate::codegen) fn emit_aarch64_variadic_register_saves(
    function: &LoweredFunction,
    frame: Option<Aarch64VariadicFrame>,
    assembly: &mut String,
) -> CompileResult<()> {
    let Some(frame) = frame else {
        return Ok(());
    };
    for (index, register) in AARCH64_VARIADIC_GP_REGISTERS.iter().enumerate() {
        let offset = frame
            .register_save_offset
            .checked_add(index * TEMPORARY_BYTES)
            .ok_or_else(|| CompileError::new("variadic register save offset overflow"))?;
        let end = frame
            .register_save_offset
            .checked_add(frame.register_save_size)
            .ok_or_else(|| CompileError::new("variadic register save offset overflow"))?;
        if offset >= end {
            return Err(CompileError::new("variadic register save overflow"));
        }
        write_assembly!(assembly, "\tstr {register}, [sp, #{offset}]\n")?;
    }
    for (index, register) in AARCH64_VARIADIC_FP_REGISTERS.iter().enumerate() {
        let offset = frame
            .register_save_offset
            .checked_add(AARCH64_VARIADIC_FP_OFFSET)
            .and_then(|offset| offset.checked_add(index * TEMPORARY_BYTES))
            .ok_or_else(|| CompileError::new("variadic fp register save offset overflow"))?;
        let end = frame
            .register_save_offset
            .checked_add(frame.register_save_size)
            .ok_or_else(|| CompileError::new("variadic fp register save offset overflow"))?;
        if offset >= end {
            return Err(CompileError::new("variadic fp register save overflow"));
        }
        write_assembly!(assembly, "\tstr {register}, [sp, #{offset}]\n")?;
    }
    if function.parameter_count > AARCH64_VARIADIC_GP_REGISTERS.len() {
        return Err(CompileError::new("too many variadic named parameters"));
    }
    Ok(())
}

pub(in crate::codegen) fn emit_aarch64_va_start(
    args: &[LoweredExpr],
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    if args.len() != 2 {
        return Err(CompileError::new("va_start expects two arguments"));
    }
    let Some(frame) = labels.aarch64_variadic else {
        return Err(CompileError::new("va_start used outside variadic function"));
    };
    emit_aarch64_expr_with_width(
        &args[0],
        ValueWidth::I64,
        temporary_base,
        depth,
        labels,
        assembly,
    )?;
    assembly.push_str("\tmov x10, x0\n");
    write_assembly!(assembly, "\tmov w8, #{}\n", frame.gp_offset)?;
    assembly.push_str("\tstr w8, [x10]\n");
    write_assembly!(assembly, "\tmov w8, #{}\n", frame.fp_offset)?;
    assembly.push_str("\tstr w8, [x10, #4]\n");
    write_assembly!(assembly, "\tadd x8, sp, #{}\n", frame.overflow_arg_offset)?;
    assembly.push_str("\tstr x8, [x10, #8]\n");
    write_assembly!(assembly, "\tadd x8, sp, #{}\n", frame.register_save_offset)?;
    assembly.push_str("\tstr x8, [x10, #16]\n");
    assembly.push_str("\tmov w0, #0\n");
    Ok(())
}

pub(in crate::codegen) fn emit_aarch64_va_arg(
    list: &LoweredExpr,
    scalar_type: ScalarType,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let width = scalar_width(scalar_type);
    emit_aarch64_expr_with_width(
        list,
        ValueWidth::I64,
        temporary_base,
        depth,
        labels,
        assembly,
    )?;
    let overflow_label = labels.fresh();
    let load_label = labels.fresh();
    assembly.push_str("\tmov x10, x0\n");
    if width == ValueWidth::F64 {
        return emit_aarch64_va_arg_f64(&overflow_label, &load_label, assembly);
    }
    assembly.push_str("\tldr w11, [x10]\n");
    write_assembly!(
        assembly,
        "\tcmp w11, #{}\n\tb.ge {overflow_label}\n",
        AARCH64_VARIADIC_GP_SAVE_BYTES
    )?;
    assembly.push_str("\tldr x12, [x10, #16]\n");
    assembly.push_str("\tadd x12, x12, x11\n");
    assembly.push_str("\tadd w11, w11, #8\n");
    assembly.push_str("\tstr w11, [x10]\n");
    write_assembly!(assembly, "\tb {load_label}\n")?;
    write_assembly!(assembly, "{overflow_label}:\n")?;
    assembly.push_str("\tldr x12, [x10, #8]\n");
    assembly.push_str("\tadd x11, x12, #8\n");
    assembly.push_str("\tstr x11, [x10, #8]\n");
    write_assembly!(assembly, "{load_label}:\n")?;
    match width {
        ValueWidth::I32 => assembly.push_str("\tldr w0, [x12]\n"),
        ValueWidth::I64 => assembly.push_str("\tldr x0, [x12]\n"),
        ValueWidth::F64 => assembly.push_str("\tldr d0, [x12]\n"),
    }
    Ok(())
}

fn emit_aarch64_va_arg_f64(
    overflow_label: &str,
    load_label: &str,
    assembly: &mut String,
) -> CompileResult<()> {
    assembly.push_str("\tldr w11, [x10, #4]\n");
    write_assembly!(
        assembly,
        "\tcmp w11, #{}\n\tb.ge {overflow_label}\n",
        AARCH64_VARIADIC_REGISTER_SAVE_BYTES
    )?;
    assembly.push_str("\tldr x12, [x10, #16]\n");
    assembly.push_str("\tadd x12, x12, x11\n");
    assembly.push_str("\tadd w11, w11, #8\n");
    assembly.push_str("\tstr w11, [x10, #4]\n");
    write_assembly!(assembly, "\tb {load_label}\n")?;
    write_assembly!(assembly, "{overflow_label}:\n")?;
    assembly.push_str("\tldr x12, [x10, #8]\n");
    assembly.push_str("\tadd x11, x12, #8\n");
    assembly.push_str("\tstr x11, [x10, #8]\n");
    write_assembly!(assembly, "{load_label}:\n")?;
    assembly.push_str("\tldr d0, [x12]\n");
    Ok(())
}
