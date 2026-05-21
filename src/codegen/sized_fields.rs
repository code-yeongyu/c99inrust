use super::aarch64_addressing::aarch64_result_register;
use super::widths::{ValueWidth, width_bytes};
use super::x86_64_addressing::{x86_64_instruction_suffix, x86_64_result_register};
use crate::diagnostics::{CompileError, CompileResult};

use std::fmt;

pub(in crate::codegen) fn emit_aarch64_load(
    byte_size: usize,
    width: ValueWidth,
    is_unsigned: bool,
    base_register: &str,
    offset: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    if byte_size == 1 && width == ValueWidth::I32 && is_unsigned {
        return write_assembly(
            assembly,
            format_args!("\tldrb w0, [{base_register}, #{offset}]\n"),
        );
    }
    if byte_size == 1 && width == ValueWidth::I32 {
        return write_assembly(
            assembly,
            format_args!("\tldrsb w0, [{base_register}, #{offset}]\n"),
        );
    }
    if byte_size == 2 && width == ValueWidth::I32 && is_unsigned {
        return write_assembly(
            assembly,
            format_args!("\tldrh w0, [{base_register}, #{offset}]\n"),
        );
    }
    if byte_size == 2 && width == ValueWidth::I32 {
        return write_assembly(
            assembly,
            format_args!("\tldrsh w0, [{base_register}, #{offset}]\n"),
        );
    }
    reject_mismatched_width(byte_size, width)?;
    write_assembly(
        assembly,
        format_args!(
            "\tldr {}, [{base_register}, #{}]\n",
            aarch64_result_register(width),
            offset
        ),
    )
}

pub(in crate::codegen) fn emit_aarch64_store(
    byte_size: usize,
    width: ValueWidth,
    base_register: &str,
    offset: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    if byte_size == 1 && width == ValueWidth::I32 {
        return write_assembly(
            assembly,
            format_args!("\tstrb w0, [{base_register}, #{offset}]\n"),
        );
    }
    if byte_size == 2 && width == ValueWidth::I32 {
        return write_assembly(
            assembly,
            format_args!("\tstrh w0, [{base_register}, #{offset}]\n"),
        );
    }
    reject_mismatched_width(byte_size, width)?;
    write_assembly(
        assembly,
        format_args!(
            "\tstr {}, [{base_register}, #{}]\n",
            aarch64_result_register(width),
            offset
        ),
    )
}

pub(in crate::codegen) fn emit_x86_64_load(
    byte_size: usize,
    width: ValueWidth,
    is_unsigned: bool,
    base_register: &str,
    offset: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    if byte_size == 1 && width == ValueWidth::I32 && is_unsigned {
        return write_assembly(
            assembly,
            format_args!("\tmovzbl {offset}({base_register}), %eax\n"),
        );
    }
    if byte_size == 1 && width == ValueWidth::I32 {
        return write_assembly(
            assembly,
            format_args!("\tmovsbl {offset}({base_register}), %eax\n"),
        );
    }
    if byte_size == 2 && width == ValueWidth::I32 && is_unsigned {
        return write_assembly(
            assembly,
            format_args!("\tmovzwl {offset}({base_register}), %eax\n"),
        );
    }
    if byte_size == 2 && width == ValueWidth::I32 {
        return write_assembly(
            assembly,
            format_args!("\tmovswl {offset}({base_register}), %eax\n"),
        );
    }
    reject_mismatched_width(byte_size, width)?;
    write_assembly(
        assembly,
        format_args!(
            "\tmov{} {}({}), {}\n",
            x86_64_instruction_suffix(width),
            offset,
            base_register,
            x86_64_result_register(width)
        ),
    )
}

pub(in crate::codegen) fn emit_x86_64_store(
    byte_size: usize,
    width: ValueWidth,
    base_register: &str,
    offset: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    if byte_size == 1 && width == ValueWidth::I32 {
        return write_assembly(
            assembly,
            format_args!("\tmovb %al, {offset}({base_register})\n"),
        );
    }
    if byte_size == 2 && width == ValueWidth::I32 {
        return write_assembly(
            assembly,
            format_args!("\tmovw %ax, {offset}({base_register})\n"),
        );
    }
    reject_mismatched_width(byte_size, width)?;
    write_assembly(
        assembly,
        format_args!(
            "\tmov{} {}, {}({})\n",
            x86_64_instruction_suffix(width),
            x86_64_result_register(width),
            offset,
            base_register
        ),
    )
}

fn reject_mismatched_width(byte_size: usize, width: ValueWidth) -> CompileResult<()> {
    if byte_size == width_bytes(width) {
        Ok(())
    } else {
        Err(CompileError::new("unsupported pointer field byte size"))
    }
}

fn write_assembly(assembly: &mut String, arguments: fmt::Arguments<'_>) -> CompileResult<()> {
    super::write_assembly(assembly, arguments)
}
