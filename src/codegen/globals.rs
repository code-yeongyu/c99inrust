use super::data_literals::{
    emit_string_literal_data_returning_to, global_string_label, label_name,
};
use super::global_integer_arrays::{
    emit_int_array_global, emit_long_long_array_global, emit_short_array_global,
};
use super::global_pointer_arrays::{
    emit_pointer_name_array_global, emit_pointer_string_array_global,
};
use super::global_real_scalars::{
    emit_double_global, emit_real_array_global, emit_real_then_zero_global,
};
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
        emit_global_initializer(&global.name, &label, &global.initializer, target, assembly)?;
    }
    Ok(())
}

fn emit_global_initializer(
    name: &str,
    label: &str,
    initializer: &LoweredGlobalInitializer,
    target: Target,
    assembly: &mut String,
) -> CompileResult<()> {
    match initializer {
        LoweredGlobalInitializer::Int(value) => {
            assembly.push_str(".p2align 2\n");
            write_assembly!(assembly, "{label}:\n")?;
            write_assembly!(assembly, "\t.long {value}\n")
        }
        LoweredGlobalInitializer::LongLong(value) => {
            assembly.push_str(".p2align 3\n");
            write_assembly!(assembly, "{label}:\n")?;
            write_assembly!(assembly, "\t.quad {value}\n")
        }
        LoweredGlobalInitializer::Double(value) => emit_double_global(label, value, assembly),
        LoweredGlobalInitializer::RealThenZero { real, byte_len } => {
            emit_real_then_zero_global(label, real, *byte_len, assembly)
        }
        LoweredGlobalInitializer::RealArray {
            scalar_type,
            length,
            values,
        } => emit_real_array_global(label, *scalar_type, values, *length, assembly),
        LoweredGlobalInitializer::LongLongArray(values) => {
            emit_long_long_array_global(label, values, assembly)
        }
        LoweredGlobalInitializer::IntArray(values) => {
            emit_int_array_global(label, values, assembly)
        }
        LoweredGlobalInitializer::ShortArray(values) => {
            emit_short_array_global(label, values, assembly)
        }
        LoweredGlobalInitializer::PointerNull => {
            assembly.push_str(".p2align 3\n");
            write_assembly!(assembly, "{label}:\n")?;
            assembly.push_str("\t.quad 0\n");
            Ok(())
        }
        LoweredGlobalInitializer::PointerString(value, byte_offset) => {
            emit_pointer_string_global(name, value, *byte_offset, target, assembly)
        }
        LoweredGlobalInitializer::PointerGlobalOffset { base, byte_offset } => {
            emit_pointer_global_offset(name, base, *byte_offset, target, assembly)
        }
        LoweredGlobalInitializer::PointerArray(length) => {
            emit_pointer_array_global(label, *length, assembly)
        }
        LoweredGlobalInitializer::PointerStringArray { values, length } => {
            emit_pointer_string_array_global(name, values, *length, target, assembly)
        }
        LoweredGlobalInitializer::PointerNameArray { values, length } => {
            emit_pointer_name_array_global(name, values, *length, target, assembly)
        }
        LoweredGlobalInitializer::StructArray { byte_len, values } => {
            struct_globals::emit(name, *byte_len, values, target, assembly)
        }
        LoweredGlobalInitializer::ZeroBytes(byte_len) => {
            assembly.push_str(".p2align 3\n");
            write_assembly!(assembly, "{label}:\n")?;
            write_assembly!(assembly, "\t.zero {byte_len}\n")
        }
        LoweredGlobalInitializer::UnsignedCharArray(values) => {
            write_assembly!(assembly, "{label}:\n")?;
            emit_byte_values(values, assembly)
        }
    }
}

fn emit_pointer_array_global(
    label: &str,
    length: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    let byte_len = length
        .checked_mul(8)
        .ok_or_else(|| CompileError::new("global pointer-array size overflow"))?;
    assembly.push_str(".p2align 3\n");
    write_assembly!(assembly, "{label}:\n")?;
    write_assembly!(assembly, "\t.zero {byte_len}\n")
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
