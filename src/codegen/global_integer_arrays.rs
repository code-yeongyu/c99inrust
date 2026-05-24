use crate::diagnostics::{CompileError, CompileResult};

pub(in crate::codegen) fn emit_int_array_global(
    label: &str,
    values: &[i32],
    assembly: &mut String,
) -> CompileResult<()> {
    assembly.push_str(".p2align 2\n");
    write_assembly!(assembly, "{label}:\n")?;
    if values.iter().all(|value| *value == 0) {
        let byte_len = values
            .len()
            .checked_mul(4)
            .ok_or_else(|| CompileError::new("global int-array size overflow"))?;
        write_assembly!(assembly, "\t.zero {byte_len}\n")
    } else {
        emit_int_values(values, assembly)
    }
}

pub(in crate::codegen) fn emit_short_array_global(
    label: &str,
    values: &[i32],
    assembly: &mut String,
) -> CompileResult<()> {
    assembly.push_str(".p2align 1\n");
    write_assembly!(assembly, "{label}:\n")?;
    if values.iter().all(|value| *value == 0) {
        let byte_len = values
            .len()
            .checked_mul(2)
            .ok_or_else(|| CompileError::new("global short-array size overflow"))?;
        write_assembly!(assembly, "\t.zero {byte_len}\n")
    } else {
        emit_short_values(values, assembly)
    }
}

pub(in crate::codegen) fn emit_long_long_array_global(
    label: &str,
    values: &[i64],
    assembly: &mut String,
) -> CompileResult<()> {
    assembly.push_str(".p2align 3\n");
    write_assembly!(assembly, "{label}:\n")?;
    if values.iter().all(|value| *value == 0) {
        let byte_len = values
            .len()
            .checked_mul(8)
            .ok_or_else(|| CompileError::new("global long-long-array size overflow"))?;
        write_assembly!(assembly, "\t.zero {byte_len}\n")
    } else {
        emit_long_long_values(values, assembly)
    }
}

fn emit_int_values(values: &[i32], assembly: &mut String) -> CompileResult<()> {
    emit_integer_values(".long", values, assembly)
}

fn emit_short_values(values: &[i32], assembly: &mut String) -> CompileResult<()> {
    emit_integer_values(".short", values, assembly)
}

fn emit_long_long_values(values: &[i64], assembly: &mut String) -> CompileResult<()> {
    emit_integer_values(".quad", values, assembly)
}

fn emit_integer_values<T: std::fmt::Display>(
    directive: &str,
    values: &[T],
    assembly: &mut String,
) -> CompileResult<()> {
    write_assembly!(assembly, "\t{directive} ")?;
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
