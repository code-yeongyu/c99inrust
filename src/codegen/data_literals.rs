use super::target::Target;
use crate::diagnostics::{CompileError, CompileResult};

pub(in crate::codegen) fn double_literal_bits(value: &str) -> CompileResult<u64> {
    value
        .parse::<f64>()
        .map(f64::to_bits)
        .map_err(|_| CompileError::new(format!("invalid double literal: {value}")))
}

pub(in crate::codegen) fn emit_double_literal_data(
    label: &str,
    bits: u64,
    target: Target,
    assembly: &mut String,
) -> CompileResult<()> {
    match target {
        Target::Aarch64AppleDarwin | Target::X86_64AppleDarwin => {
            assembly.push_str("\t.section __TEXT,__literal8,8byte_literals\n");
        }
        Target::X86_64UnknownLinuxGnu => {
            assembly.push_str("\t.section .rodata.cst8,\"aM\",@progbits,8\n");
        }
    }
    assembly.push_str("\t.p2align 3\n");
    write_assembly!(assembly, "{label}:\n")?;
    write_assembly!(assembly, "\t.quad 0x{bits:016x}\n")?;
    assembly.push_str(".text\n");
    Ok(())
}

pub(in crate::codegen) fn emit_string_literal_data(
    label: &str,
    value: &str,
    target: Target,
    assembly: &mut String,
) -> CompileResult<()> {
    emit_string_literal_data_returning_to(label, value, target, ".text\n", assembly)
}

pub(in crate::codegen) fn emit_string_literal_data_returning_to(
    label: &str,
    value: &str,
    target: Target,
    return_section: &str,
    assembly: &mut String,
) -> CompileResult<()> {
    match target {
        Target::Aarch64AppleDarwin | Target::X86_64AppleDarwin => {
            assembly.push_str("\t.section __TEXT,__cstring,cstring_literals\n");
        }
        Target::X86_64UnknownLinuxGnu => {
            assembly.push_str("\t.section .rodata\n");
        }
    }
    write_assembly!(assembly, "{label}:\n")?;
    assembly.push_str("\t.byte ");
    let mut first = true;
    for byte in value.as_bytes().iter().copied().chain([0]) {
        if first {
            first = false;
        } else {
            assembly.push(',');
        }
        write_assembly!(assembly, "{byte}")?;
    }
    assembly.push('\n');
    assembly.push_str(return_section);
    Ok(())
}
pub(in crate::codegen) fn label_name(name: &str, target: Target) -> String {
    match target {
        Target::Aarch64AppleDarwin | Target::X86_64AppleDarwin => format!("_{name}"),
        Target::X86_64UnknownLinuxGnu => name.to_string(),
    }
}

pub(in crate::codegen) fn global_string_label(name: &str, index: usize, target: Target) -> String {
    match target {
        Target::Aarch64AppleDarwin | Target::X86_64AppleDarwin => {
            format!("L{name}_str{index}")
        }
        Target::X86_64UnknownLinuxGnu => format!(".L{name}_str{index}"),
    }
}

pub(in crate::codegen) fn branch_label(function: &str, label: usize, target: Target) -> String {
    match target {
        Target::Aarch64AppleDarwin | Target::X86_64AppleDarwin => format!("L{function}_{label}"),
        Target::X86_64UnknownLinuxGnu => format!(".L{function}_{label}"),
    }
}
