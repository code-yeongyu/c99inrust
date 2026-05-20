use super::data_literals::{
    emit_string_literal_data_returning_to, global_string_label, label_name,
};
use super::globals::emit_byte_values;
use super::target::Target;
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::{LoweredStructInitializerScalar, LoweredStructInitializerValue};

use std::fmt;

pub(in crate::codegen) fn emit(
    name: &str,
    byte_len: usize,
    values: &[LoweredStructInitializerValue],
    target: Target,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = label_name(name, target);
    assembly.push_str(".p2align 3\n");
    write_assembly(assembly, format_args!("{label}:\n"))?;
    let mut cursor = 0usize;
    let mut strings = Vec::new();
    for value in values {
        if value.byte_offset < cursor {
            return Err(CompileError::new("overlapping global struct initializer"));
        }
        emit_zero_padding(value.byte_offset - cursor, assembly)?;
        let byte_size = emit_scalar(name, strings.len(), &value.value, target, assembly)?;
        cursor = value
            .byte_offset
            .checked_add(byte_size)
            .ok_or_else(|| CompileError::new("global struct initializer size overflow"))?;
        if let Some(string) = struct_initializer_string(&value.value) {
            strings.push(string.to_owned());
        }
    }
    if cursor > byte_len {
        return Err(CompileError::new("global struct initializer exceeds size"));
    }
    emit_zero_padding(byte_len - cursor, assembly)?;
    for (index, value) in strings.iter().enumerate() {
        let string_label = global_string_label(name, index, target);
        emit_string_literal_data_returning_to(&string_label, value, target, ".data\n", assembly)?;
    }
    Ok(())
}

fn emit_scalar(
    name: &str,
    string_index: usize,
    value: &LoweredStructInitializerScalar,
    target: Target,
    assembly: &mut String,
) -> CompileResult<usize> {
    match value {
        LoweredStructInitializerScalar::Int { value, byte_size } => {
            emit_integer(*value, *byte_size, assembly)?;
            Ok(*byte_size)
        }
        LoweredStructInitializerScalar::IntString { byte_size, .. } => {
            if *byte_size != 4 {
                return Err(CompileError::new(
                    "global struct string address requires int-sized field",
                ));
            }
            let string_label = global_string_label(name, string_index, target);
            write_assembly(assembly, format_args!("\t.long {string_label}\n"))?;
            Ok(4)
        }
        LoweredStructInitializerScalar::LongLong(value)
        | LoweredStructInitializerScalar::PointerInteger(value) => {
            write_assembly(assembly, format_args!("\t.quad {value}\n"))?;
            Ok(8)
        }
        LoweredStructInitializerScalar::Bytes { values, byte_len } => {
            emit_byte_values(values, assembly)?;
            Ok(*byte_len)
        }
        LoweredStructInitializerScalar::PointerNull => {
            assembly.push_str("\t.quad 0\n");
            Ok(8)
        }
        LoweredStructInitializerScalar::PointerString(_) => {
            let string_label = global_string_label(name, string_index, target);
            write_assembly(assembly, format_args!("\t.quad {string_label}\n"))?;
            Ok(8)
        }
        LoweredStructInitializerScalar::PointerGlobalOffset { base, byte_offset } => {
            let base_label = label_name(base, target);
            if *byte_offset == 0 {
                write_assembly(assembly, format_args!("\t.quad {base_label}\n"))?;
            } else {
                write_assembly(
                    assembly,
                    format_args!("\t.quad {base_label}+{byte_offset}\n"),
                )?;
            }
            Ok(8)
        }
    }
}

fn emit_integer(value: i32, byte_size: usize, assembly: &mut String) -> CompileResult<()> {
    match byte_size {
        1 => write_assembly(assembly, format_args!("\t.byte {value}\n")),
        2 => write_assembly(assembly, format_args!("\t.short {value}\n")),
        4 => write_assembly(assembly, format_args!("\t.long {value}\n")),
        _ => Err(CompileError::new(
            "unsupported global struct int field byte size",
        )),
    }
}

fn struct_initializer_string(value: &LoweredStructInitializerScalar) -> Option<&str> {
    match value {
        LoweredStructInitializerScalar::IntString { value, .. }
        | LoweredStructInitializerScalar::PointerString(value) => Some(value),
        LoweredStructInitializerScalar::Int { .. }
        | LoweredStructInitializerScalar::LongLong(_)
        | LoweredStructInitializerScalar::Bytes { .. }
        | LoweredStructInitializerScalar::PointerNull
        | LoweredStructInitializerScalar::PointerInteger(_)
        | LoweredStructInitializerScalar::PointerGlobalOffset { .. } => None,
    }
}

fn emit_zero_padding(byte_len: usize, assembly: &mut String) -> CompileResult<()> {
    if byte_len == 0 {
        return Ok(());
    }
    write_assembly(assembly, format_args!("\t.zero {byte_len}\n"))
}

fn write_assembly(assembly: &mut String, arguments: fmt::Arguments<'_>) -> CompileResult<()> {
    super::write_assembly(assembly, arguments)
}
