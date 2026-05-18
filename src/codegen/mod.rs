use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::{Instruction, LoweredExpr, LoweredFunction, LoweredProgram};
use crate::parser::{BinaryOp, UnaryOp};

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
        match target {
            Target::Aarch64AppleDarwin => emit_aarch64_function(function, target, &mut assembly)?,
            Target::X86_64AppleDarwin | Target::X86_64UnknownLinuxGnu => {
                emit_x86_64_function(function, target, &mut assembly)?
            }
        }
    }
    if target == Target::X86_64UnknownLinuxGnu {
        assembly.push_str(".section .note.GNU-stack,\"\",@progbits\n");
    }
    Ok(assembly)
}

fn emit_aarch64_function(
    function: &LoweredFunction,
    target: Target,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = label_name(&function.name, target);
    let temporary_count = function
        .instructions
        .iter()
        .map(instruction_depth)
        .max()
        .unwrap_or(0);
    let local_bytes = function.local_count * 4;
    let temporary_base = local_bytes;
    let stack_bytes = align_to(local_bytes + (temporary_count * 4), 16);
    assembly.push_str(&format!(".globl {label}\n"));
    assembly.push_str(".p2align 2\n");
    assembly.push_str(&format!("{label}:\n"));
    if stack_bytes > 0 {
        assembly.push_str(&format!("\tsub sp, sp, #{stack_bytes}\n"));
    }
    for instruction in &function.instructions {
        match instruction {
            Instruction::StoreLocal { slot, value } => {
                emit_aarch64_expr(value, temporary_base, 0, assembly)?;
                assembly.push_str(&format!("\tstr w0, [sp, #{}]\n", local_offset(*slot)));
            }
            Instruction::Return(expr) => {
                emit_aarch64_expr(expr, temporary_base, 0, assembly)?;
                if stack_bytes > 0 {
                    assembly.push_str(&format!("\tadd sp, sp, #{stack_bytes}\n"));
                }
                assembly.push_str("\tret\n");
            }
        }
    }
    Ok(())
}

fn emit_x86_64_function(
    function: &LoweredFunction,
    target: Target,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = label_name(&function.name, target);
    let stack_bytes = align_to(function.local_count * 4, 16);
    assembly.push_str(&format!(".globl {label}\n"));
    assembly.push_str(&format!("{label}:\n"));
    assembly.push_str("\tpushq %rbp\n");
    assembly.push_str("\tmovq %rsp, %rbp\n");
    if stack_bytes > 0 {
        assembly.push_str(&format!("\tsubq ${stack_bytes}, %rsp\n"));
    }
    for instruction in &function.instructions {
        match instruction {
            Instruction::StoreLocal { slot, value } => {
                emit_x86_64_expr(value, assembly)?;
                assembly.push_str(&format!("\tmovl %eax, {}(%rbp)\n", x86_local_offset(*slot)));
            }
            Instruction::Return(expr) => {
                emit_x86_64_expr(expr, assembly)?;
                assembly.push_str("\tleave\n");
                assembly.push_str("\tret\n");
            }
        }
    }
    Ok(())
}

fn emit_aarch64_expr(
    expr: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    match expr {
        LoweredExpr::Integer(value) => emit_aarch64_i32(*value, assembly),
        LoweredExpr::Local(slot) => {
            assembly.push_str(&format!("\tldr w0, [sp, #{}]\n", local_offset(*slot)));
            Ok(())
        }
        LoweredExpr::Unary { op, expr } => {
            emit_aarch64_expr(expr, temporary_base, depth, assembly)?;
            match op {
                UnaryOp::Plus => {}
                UnaryOp::Minus => assembly.push_str("\tneg w0, w0\n"),
                UnaryOp::BitNot => assembly.push_str("\tmvn w0, w0\n"),
                UnaryOp::LogicalNot => {
                    assembly.push_str("\tcmp w0, #0\n");
                    assembly.push_str("\tcset w0, eq\n");
                }
            }
            Ok(())
        }
        LoweredExpr::Binary { op, left, right } => {
            let temporary_offset = temporary_base + (depth * 4);
            emit_aarch64_expr(left, temporary_base, depth + 1, assembly)?;
            assembly.push_str(&format!("\tstr w0, [sp, #{temporary_offset}]\n"));
            emit_aarch64_expr(right, temporary_base, depth + 1, assembly)?;
            assembly.push_str("\tmov w1, w0\n");
            assembly.push_str(&format!("\tldr w0, [sp, #{temporary_offset}]\n"));
            match op {
                BinaryOp::Mul => assembly.push_str("\tmul w0, w0, w1\n"),
                BinaryOp::Div => assembly.push_str("\tsdiv w0, w0, w1\n"),
                BinaryOp::Mod => {
                    assembly.push_str("\tsdiv w2, w0, w1\n");
                    assembly.push_str("\tmsub w0, w2, w1, w0\n");
                }
                BinaryOp::Add => assembly.push_str("\tadd w0, w0, w1\n"),
                BinaryOp::Sub => assembly.push_str("\tsub w0, w0, w1\n"),
                BinaryOp::ShiftLeft => assembly.push_str("\tlsl w0, w0, w1\n"),
                BinaryOp::ShiftRight => assembly.push_str("\tasr w0, w0, w1\n"),
                BinaryOp::BitAnd => assembly.push_str("\tand w0, w0, w1\n"),
                BinaryOp::BitXor => assembly.push_str("\teor w0, w0, w1\n"),
                BinaryOp::BitOr => assembly.push_str("\torr w0, w0, w1\n"),
            }
            Ok(())
        }
    }
}

fn emit_x86_64_expr(expr: &LoweredExpr, assembly: &mut String) -> CompileResult<()> {
    match expr {
        LoweredExpr::Integer(value) => {
            let value = i32::try_from(*value)
                .map_err(|_| CompileError::new("integer literal does not fit i32"))?;
            assembly.push_str(&format!("\tmovl ${value}, %eax\n"));
            Ok(())
        }
        LoweredExpr::Local(slot) => {
            assembly.push_str(&format!("\tmovl {}(%rbp), %eax\n", x86_local_offset(*slot)));
            Ok(())
        }
        LoweredExpr::Unary { op, expr } => {
            emit_x86_64_expr(expr, assembly)?;
            match op {
                UnaryOp::Plus => {}
                UnaryOp::Minus => assembly.push_str("\tnegl %eax\n"),
                UnaryOp::BitNot => assembly.push_str("\tnotl %eax\n"),
                UnaryOp::LogicalNot => {
                    assembly.push_str("\tcmpl $0, %eax\n");
                    assembly.push_str("\tsete %al\n");
                    assembly.push_str("\tmovzbl %al, %eax\n");
                }
            }
            Ok(())
        }
        LoweredExpr::Binary { op, left, right } => {
            emit_x86_64_expr(left, assembly)?;
            assembly.push_str("\tpushq %rax\n");
            emit_x86_64_expr(right, assembly)?;
            assembly.push_str("\tmovl %eax, %ecx\n");
            assembly.push_str("\tpopq %rax\n");
            match op {
                BinaryOp::Mul => assembly.push_str("\timull %ecx, %eax\n"),
                BinaryOp::Div => {
                    assembly.push_str("\tcltd\n");
                    assembly.push_str("\tidivl %ecx\n");
                }
                BinaryOp::Mod => {
                    assembly.push_str("\tcltd\n");
                    assembly.push_str("\tidivl %ecx\n");
                    assembly.push_str("\tmovl %edx, %eax\n");
                }
                BinaryOp::Add => assembly.push_str("\taddl %ecx, %eax\n"),
                BinaryOp::Sub => assembly.push_str("\tsubl %ecx, %eax\n"),
                BinaryOp::ShiftLeft => assembly.push_str("\tsall %cl, %eax\n"),
                BinaryOp::ShiftRight => assembly.push_str("\tsarl %cl, %eax\n"),
                BinaryOp::BitAnd => assembly.push_str("\tandl %ecx, %eax\n"),
                BinaryOp::BitXor => assembly.push_str("\txorl %ecx, %eax\n"),
                BinaryOp::BitOr => assembly.push_str("\torl %ecx, %eax\n"),
            }
            Ok(())
        }
    }
}

fn emit_aarch64_i32(value: i64, assembly: &mut String) -> CompileResult<()> {
    let value =
        i32::try_from(value).map_err(|_| CompileError::new("integer literal does not fit i32"))?;
    let bits = u32::from_ne_bytes(value.to_ne_bytes());
    let low = bits & 0xffff;
    let high = (bits >> 16) & 0xffff;
    assembly.push_str(&format!("\tmovz w0, #{low}\n"));
    if high != 0 {
        assembly.push_str(&format!("\tmovk w0, #{high}, lsl #16\n"));
    }
    Ok(())
}

fn instruction_depth(instruction: &Instruction) -> usize {
    match instruction {
        Instruction::StoreLocal { value, .. } | Instruction::Return(value) => expr_depth(value),
    }
}

fn expr_depth(expr: &LoweredExpr) -> usize {
    match expr {
        LoweredExpr::Integer(_) | LoweredExpr::Local(_) => 0,
        LoweredExpr::Unary { expr, .. } => expr_depth(expr),
        LoweredExpr::Binary { left, right, .. } => 1 + expr_depth(left).max(expr_depth(right)),
    }
}

fn local_offset(slot: usize) -> usize {
    slot * 4
}

fn x86_local_offset(slot: usize) -> String {
    format!("-{}", (slot + 1) * 4)
}

fn align_to(value: usize, alignment: usize) -> usize {
    let remainder = value % alignment;
    if remainder == 0 {
        value
    } else {
        value + (alignment - remainder)
    }
}

fn label_name(name: &str, target: Target) -> String {
    match target {
        Target::Aarch64AppleDarwin | Target::X86_64AppleDarwin => format!("_{name}"),
        Target::X86_64UnknownLinuxGnu => name.to_string(),
    }
}
