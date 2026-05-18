use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::LoweredProgram;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Aarch64AppleDarwin,
    X86_64AppleDarwin,
    X86_64UnknownLinuxGnu,
}

impl Target {
    #[must_use]
    pub fn native() -> Self {
        if cfg!(all(target_arch = "aarch64", target_os = "macos")) {
            Self::Aarch64AppleDarwin
        } else if cfg!(all(target_arch = "x86_64", target_os = "macos")) {
            Self::X86_64AppleDarwin
        } else {
            Self::X86_64UnknownLinuxGnu
        }
    }

    pub fn parse(value: &str) -> CompileResult<Self> {
        match value {
            "aarch64-apple-darwin" => Ok(Self::Aarch64AppleDarwin),
            "x86_64-apple-darwin" => Ok(Self::X86_64AppleDarwin),
            "x86_64-unknown-linux-gnu" => Ok(Self::X86_64UnknownLinuxGnu),
            "native" => Ok(Self::native()),
            _ => Err(CompileError::new(format!("unsupported target: {value}"))),
        }
    }
}

pub fn emit_assembly(program: &LoweredProgram, target: Target) -> CompileResult<String> {
    if program.functions.is_empty() {
        return Err(CompileError::new("program has no functions"));
    }
    let mut assembly = String::new();
    assembly.push_str(".text\n");
    for function in &program.functions {
        let label = label_name(&function.name, target);
        assembly.push_str(&format!(".globl {label}\n"));
        match target {
            Target::Aarch64AppleDarwin => {
                let value = i32::try_from(function.return_value)
                    .map_err(|_| CompileError::new("return value does not fit i32"))?;
                assembly.push_str(".p2align 2\n");
                assembly.push_str(&format!("{label}:\n"));
                assembly.push_str(&format!("\tmov w0, #{value}\n"));
                assembly.push_str("\tret\n");
            }
            Target::X86_64AppleDarwin | Target::X86_64UnknownLinuxGnu => {
                let value = i32::try_from(function.return_value)
                    .map_err(|_| CompileError::new("return value does not fit i32"))?;
                assembly.push_str(&format!("{label}:\n"));
                assembly.push_str(&format!("\tmovl ${value}, %eax\n"));
                assembly.push_str("\tret\n");
            }
        }
    }
    Ok(assembly)
}

fn label_name(name: &str, target: Target) -> String {
    match target {
        Target::Aarch64AppleDarwin | Target::X86_64AppleDarwin => format!("_{name}"),
        Target::X86_64UnknownLinuxGnu => name.to_string(),
    }
}
