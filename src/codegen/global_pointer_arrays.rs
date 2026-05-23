use super::data_literals::{
    emit_string_literal_data_returning_to, global_string_label, label_name,
};
use super::target::Target;
use crate::diagnostics::CompileResult;

pub(in crate::codegen) fn emit_pointer_string_array_global(
    name: &str,
    values: &[Option<(String, usize)>],
    length: usize,
    target: Target,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = label_name(name, target);
    assembly.push_str(".p2align 3\n");
    write_assembly!(assembly, "{label}:\n")?;
    for (index, value) in values.iter().enumerate() {
        emit_pointer_entry(name, index, value.as_ref(), target, assembly)?;
    }
    for _index in values.len()..length {
        assembly.push_str("\t.quad 0\n");
    }
    for (index, value) in values.iter().enumerate() {
        if let Some((text, _byte_offset)) = value {
            let string_label = global_string_label(name, index, target);
            emit_string_literal_data_returning_to(
                &string_label,
                text,
                target,
                ".data\n",
                assembly,
            )?;
        }
    }
    Ok(())
}

pub(in crate::codegen) fn emit_pointer_name_array_global(
    name: &str,
    values: &[Option<(String, usize)>],
    length: usize,
    target: Target,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = label_name(name, target);
    assembly.push_str(".p2align 3\n");
    write_assembly!(assembly, "{label}:\n")?;
    for value in values {
        emit_name_pointer_entry(value.as_ref(), target, assembly)?;
    }
    for _index in values.len()..length {
        assembly.push_str("\t.quad 0\n");
    }
    Ok(())
}

fn emit_pointer_entry(
    name: &str,
    index: usize,
    value: Option<&(String, usize)>,
    target: Target,
    assembly: &mut String,
) -> CompileResult<()> {
    let Some((_text, byte_offset)) = value else {
        assembly.push_str("\t.quad 0\n");
        return Ok(());
    };
    let string_label = global_string_label(name, index, target);
    if *byte_offset == 0 {
        write_assembly!(assembly, "\t.quad {string_label}\n")
    } else {
        write_assembly!(assembly, "\t.quad {string_label}+{byte_offset}\n")
    }
}

fn emit_name_pointer_entry(
    value: Option<&(String, usize)>,
    target: Target,
    assembly: &mut String,
) -> CompileResult<()> {
    let Some((base, byte_offset)) = value else {
        assembly.push_str("\t.quad 0\n");
        return Ok(());
    };
    let base_label = label_name(base, target);
    if *byte_offset == 0 {
        write_assembly!(assembly, "\t.quad {base_label}\n")
    } else {
        write_assembly!(assembly, "\t.quad {base_label}+{byte_offset}\n")
    }
}
