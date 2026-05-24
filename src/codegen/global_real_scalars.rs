use super::data_literals::double_literal_bits;
use crate::diagnostics::{CompileError, CompileResult};
use crate::parser::ScalarType;

pub(in crate::codegen) fn emit_double_global(
    label: &str,
    value: &str,
    assembly: &mut String,
) -> CompileResult<()> {
    assembly.push_str(".p2align 3\n");
    write_assembly!(assembly, "{label}:\n")?;
    let bits = double_literal_bits(value)?;
    write_assembly!(assembly, "\t.quad 0x{bits:016x}\n")
}

pub(in crate::codegen) fn emit_real_then_zero_global(
    label: &str,
    real: &str,
    byte_len: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    let real_len = real_part_byte_len(byte_len);
    if real_len == 4 {
        emit_float_global(label, real, assembly)?;
    } else {
        emit_double_global(label, real, assembly)?;
    }
    if byte_len > real_len {
        write_assembly!(assembly, "\t.zero {}\n", byte_len - real_len)?;
    }
    Ok(())
}

pub(in crate::codegen) fn emit_real_array_global(
    label: &str,
    scalar_type: ScalarType,
    values: &[String],
    length: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    let byte_len = scalar_byte_len(scalar_type);
    let real_len = real_part_byte_len(byte_len);
    if real_len == 4 {
        assembly.push_str(".p2align 2\n");
    } else {
        assembly.push_str(".p2align 3\n");
    }
    write_assembly!(assembly, "{label}:\n")?;
    for value in values {
        emit_real_value(value, real_len, assembly)?;
        if byte_len > real_len {
            write_assembly!(assembly, "\t.zero {}\n", byte_len - real_len)?;
        }
    }
    let zero_tail = length
        .checked_sub(values.len())
        .and_then(|count| count.checked_mul(byte_len))
        .ok_or_else(|| CompileError::new("global real-array size overflow"))?;
    if zero_tail > 0 {
        write_assembly!(assembly, "\t.zero {zero_tail}\n")?;
    }
    Ok(())
}

fn emit_float_global(label: &str, value: &str, assembly: &mut String) -> CompileResult<()> {
    assembly.push_str(".p2align 2\n");
    write_assembly!(assembly, "{label}:\n")?;
    let bits = float_literal_bits(value)?;
    write_assembly!(assembly, "\t.long 0x{bits:08x}\n")
}

fn emit_real_value(value: &str, real_len: usize, assembly: &mut String) -> CompileResult<()> {
    if real_len == 4 {
        let bits = float_literal_bits(value)?;
        write_assembly!(assembly, "\t.long 0x{bits:08x}\n")
    } else {
        let bits = double_literal_bits(value)?;
        write_assembly!(assembly, "\t.quad 0x{bits:016x}\n")
    }
}

fn float_literal_bits(value: &str) -> CompileResult<u32> {
    value
        .parse::<f32>()
        .map(f32::to_bits)
        .map_err(|_| CompileError::new(format!("invalid float literal: {value}")))
}

const fn real_part_byte_len(byte_len: usize) -> usize {
    if byte_len == 8 { 4 } else { 8 }
}

const fn scalar_byte_len(scalar_type: ScalarType) -> usize {
    match scalar_type {
        ScalarType::ComplexDouble => 16,
        ScalarType::ComplexLongDouble => 2 * long_double_size(),
        ScalarType::LongDouble => long_double_size(),
        ScalarType::ComplexFloat
        | ScalarType::Double
        | ScalarType::Bool
        | ScalarType::Int
        | ScalarType::LongLong
        | ScalarType::Pointer
        | ScalarType::VaList => 8,
    }
}

const fn long_double_size() -> usize {
    if cfg!(all(target_arch = "x86_64", not(target_os = "macos"))) {
        16
    } else {
        8
    }
}
