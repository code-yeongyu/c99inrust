use super::data_literals::{
    emit_string_literal_data_returning_to, global_string_label, label_name,
};
use super::global_pointer_arrays::emit_pointer_string_array_global;
use super::global_real_scalars::{emit_double_global, emit_real_then_zero_global};
use super::struct_globals;
use super::target::Target;
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::{LoweredGlobal, LoweredGlobalInitializer};

pub(in crate::codegen) fn emit_globals(
    globals: &[LoweredGlobal],
    target: Target,
    assembly: &mut String,
) -> CompileResult<()> {
    if globals.is_empty() {
        return Ok(());
    }
    assembly.push_str(".data\n");
    for global in globals {
        let label = label_name(&global.name, target);
        if !global.is_static {
            write_assembly!(assembly, ".globl {label}\n")?;
        }
        match &global.initializer {
            LoweredGlobalInitializer::Int(value) => {
                assembly.push_str(".p2align 2\n");
                write_assembly!(assembly, "{label}:\n")?;
                write_assembly!(assembly, "\t.long {value}\n")?;
            }
            LoweredGlobalInitializer::LongLong(value) => {
                assembly.push_str(".p2align 3\n");
                write_assembly!(assembly, "{label}:\n")?;
                write_assembly!(assembly, "\t.quad {value}\n")?;
            }
            LoweredGlobalInitializer::Double(value) => {
                emit_double_global(&label, value, assembly)?;
            }
            LoweredGlobalInitializer::RealThenZero { real, byte_len } => {
                emit_real_then_zero_global(&label, real, *byte_len, assembly)?;
            }
            LoweredGlobalInitializer::IntArray(values) => {
                if values.iter().all(|value| *value == 0) {
                    let byte_len = values
                        .len()
                        .checked_mul(4)
                        .ok_or_else(|| CompileError::new("global int-array size overflow"))?;
                    assembly.push_str(".p2align 2\n");
                    write_assembly!(assembly, "{label}:\n")?;
                    write_assembly!(assembly, "\t.zero {byte_len}\n")?;
                    continue;
                }
                assembly.push_str(".p2align 2\n");
                write_assembly!(assembly, "{label}:\n")?;
                emit_int_values(values, assembly)?;
            }
            LoweredGlobalInitializer::ShortArray(values) => {
                if values.iter().all(|value| *value == 0) {
                    let byte_len = values
                        .len()
                        .checked_mul(2)
                        .ok_or_else(|| CompileError::new("global short-array size overflow"))?;
                    assembly.push_str(".p2align 1\n");
                    write_assembly!(assembly, "{label}:\n")?;
                    write_assembly!(assembly, "\t.zero {byte_len}\n")?;
                    continue;
                }
                assembly.push_str(".p2align 1\n");
                write_assembly!(assembly, "{label}:\n")?;
                emit_short_values(values, assembly)?;
            }
            LoweredGlobalInitializer::PointerNull => {
                assembly.push_str(".p2align 3\n");
                write_assembly!(assembly, "{label}:\n")?;
                assembly.push_str("\t.quad 0\n");
            }
            LoweredGlobalInitializer::PointerString(value, byte_offset) => {
                emit_pointer_string_global(&global.name, value, *byte_offset, target, assembly)?;
            }
            LoweredGlobalInitializer::PointerGlobalOffset { base, byte_offset } => {
                emit_pointer_global_offset(&global.name, base, *byte_offset, target, assembly)?;
            }
            LoweredGlobalInitializer::PointerArray(length) => {
                let byte_len = length
                    .checked_mul(8)
                    .ok_or_else(|| CompileError::new("global pointer-array size overflow"))?;
                assembly.push_str(".p2align 3\n");
                write_assembly!(assembly, "{label}:\n")?;
                write_assembly!(assembly, "\t.zero {byte_len}\n")?;
            }
            LoweredGlobalInitializer::PointerStringArray { values, length } => {
                emit_pointer_string_array_global(&global.name, values, *length, target, assembly)?;
            }
            LoweredGlobalInitializer::PointerNameArray { values, length } => {
                emit_pointer_name_array_global(&global.name, values, *length, target, assembly)?;
            }
            LoweredGlobalInitializer::StructArray { byte_len, values } => {
                struct_globals::emit(&global.name, *byte_len, values, target, assembly)?;
            }
            LoweredGlobalInitializer::ZeroBytes(byte_len) => {
                assembly.push_str(".p2align 3\n");
                write_assembly!(assembly, "{label}:\n")?;
                write_assembly!(assembly, "\t.zero {byte_len}\n")?;
            }
            LoweredGlobalInitializer::UnsignedCharArray(values) => {
                write_assembly!(assembly, "{label}:\n")?;
                emit_byte_values(values, assembly)?;
            }
        }
    }
    Ok(())
}

pub(in crate::codegen) fn emit_pointer_string_global(
    name: &str,
    value: &str,
    byte_offset: usize,
    target: Target,
    assembly: &mut String,
) -> CompileResult<()> {
    let string_label = global_string_label(name, 0, target);
    let label = label_name(name, target);
    assembly.push_str(".p2align 3\n");
    write_assembly!(assembly, "{label}:\n")?;
    if byte_offset == 0 {
        write_assembly!(assembly, "\t.quad {string_label}\n")?;
    } else {
        write_assembly!(assembly, "\t.quad {string_label}+{byte_offset}\n")?;
    }
    emit_string_literal_data_returning_to(&string_label, value, target, ".data\n", assembly)
}

pub(in crate::codegen) fn emit_pointer_global_offset(
    name: &str,
    base: &str,
    byte_offset: usize,
    target: Target,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = label_name(name, target);
    let base_label = label_name(base, target);
    assembly.push_str(".p2align 3\n");
    write_assembly!(assembly, "{label}:\n")?;
    if byte_offset == 0 {
        write_assembly!(assembly, "\t.quad {base_label}\n")
    } else {
        write_assembly!(assembly, "\t.quad {base_label}+{byte_offset}\n")
    }
}

pub(in crate::codegen) fn emit_pointer_name_array_global(
    name: &str,
    values: &[String],
    length: usize,
    target: Target,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = label_name(name, target);
    assembly.push_str(".p2align 3\n");
    write_assembly!(assembly, "{label}:\n")?;
    for value in values {
        let value_label = label_name(value, target);
        write_assembly!(assembly, "\t.quad {value_label}\n")?;
    }
    for _index in values.len()..length {
        assembly.push_str("\t.quad 0\n");
    }
    Ok(())
}

pub(in crate::codegen) fn emit_int_values(
    values: &[i32],
    assembly: &mut String,
) -> CompileResult<()> {
    assembly.push_str("\t.long ");
    let mut first = true;
    for value in values {
        if first {
            first = false;
        } else {
            assembly.push(',');
        }
        write_assembly!(assembly, "{value}")?;
    }
    assembly.push('\n');
    Ok(())
}

pub(in crate::codegen) fn emit_short_values(
    values: &[i32],
    assembly: &mut String,
) -> CompileResult<()> {
    assembly.push_str("\t.short ");
    let mut first = true;
    for value in values {
        if first {
            first = false;
        } else {
            assembly.push(',');
        }
        write_assembly!(assembly, "{value}")?;
    }
    assembly.push('\n');
    Ok(())
}

pub(in crate::codegen) fn emit_byte_values(
    values: &[u8],
    assembly: &mut String,
) -> CompileResult<()> {
    assembly.push_str("\t.byte ");
    let mut first = true;
    for value in values {
        if first {
            first = false;
        } else {
            assembly.push(',');
        }
        write_assembly!(assembly, "{value}")?;
    }
    assembly.push('\n');
    Ok(())
}
