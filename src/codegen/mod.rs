use std::fmt::{self, Write as _};

use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::{Instruction, LoweredExpr, LoweredFunction, LoweredProgram};
use crate::parser::{BinaryOp, UnaryOp};

macro_rules! write_assembly {
    ($assembly:expr, $($argument:tt)*) => {
        write_assembly($assembly, format_args!($($argument)*))
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Aarch64AppleDarwin,
    X86_64AppleDarwin,
    X86_64UnknownLinuxGnu,
}

impl Target {
    #[must_use]
    pub const fn native() -> Self {
        if cfg!(all(target_arch = "aarch64", target_os = "macos")) {
            Self::Aarch64AppleDarwin
        } else if cfg!(all(target_arch = "x86_64", target_os = "macos")) {
            Self::X86_64AppleDarwin
        } else {
            Self::X86_64UnknownLinuxGnu
        }
    }

    /// Parses a supported backend target triple.
    ///
    /// # Errors
    ///
    /// Returns an error when `value` is not `native` or one of the explicit
    /// target triples supported by the current backend.
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

/// Emits target assembly for a lowered program.
///
/// # Errors
///
/// Returns an error when the program has no functions or an expression cannot
/// be represented by the current `int`-only backend.
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
                emit_x86_64_function(function, target, &mut assembly)?;
            }
        }
    }
    if target == Target::X86_64UnknownLinuxGnu {
        assembly.push_str(".section .note.GNU-stack,\"\",@progbits\n");
    }
    Ok(assembly)
}

fn write_assembly(assembly: &mut String, arguments: fmt::Arguments<'_>) -> CompileResult<()> {
    assembly
        .write_fmt(arguments)
        .map_err(|_| CompileError::new("failed to format assembly"))
}

struct LabelAllocator<'a> {
    function: &'a str,
    target: Target,
    next_label: usize,
}

impl<'a> LabelAllocator<'a> {
    fn new(function: &'a LoweredFunction, target: Target) -> Self {
        Self {
            function: &function.name,
            target,
            next_label: next_available_label(function),
        }
    }

    fn fresh(&mut self) -> String {
        let label = self.next_label;
        self.next_label += 1;
        branch_label(self.function, label, self.target)
    }
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
    let call_frame_bytes = if function_uses_call(function) { 8 } else { 0 };
    let preserved_temp_bytes = if function_uses_aarch64_preserved_temp(function) {
        8
    } else {
        0
    };
    let stack_bytes = align_to(
        local_bytes + (temporary_count * 4) + call_frame_bytes + preserved_temp_bytes,
        16,
    );
    let link_register_offset = if call_frame_bytes > 0 {
        Some(stack_bytes - 8)
    } else {
        None
    };
    let preserved_temp_offset = if preserved_temp_bytes > 0 {
        Some(stack_bytes - call_frame_bytes - 8)
    } else {
        None
    };
    let mut labels = LabelAllocator::new(function, target);
    let shared_epilogue = if should_share_aarch64_epilogue(function, stack_bytes) {
        Some(labels.fresh())
    } else {
        None
    };
    write_assembly!(assembly, ".globl {label}\n")?;
    assembly.push_str(".p2align 2\n");
    write_assembly!(assembly, "{label}:\n")?;
    emit_aarch64_prologue(
        preserved_temp_offset,
        link_register_offset,
        stack_bytes,
        assembly,
    )?;
    for instruction in &function.instructions {
        match instruction {
            Instruction::StoreLocal { slot, value } => {
                emit_aarch64_store_local(*slot, value, temporary_base, &mut labels, assembly)?;
                write_assembly!(assembly, "\tstr w0, [sp, #{}]\n", local_offset(*slot))?;
            }
            Instruction::JumpIfZero { condition, label } => {
                let target_label = branch_label(&function.name, *label, target);
                emit_aarch64_jump_if_zero(
                    condition,
                    &target_label,
                    temporary_base,
                    &mut labels,
                    assembly,
                )?;
            }
            Instruction::Jump { label } => {
                write_assembly!(
                    assembly,
                    "\tb {}\n",
                    branch_label(&function.name, *label, target)
                )?;
            }
            Instruction::Label { label } => {
                write_assembly!(
                    assembly,
                    "{}:\n",
                    branch_label(&function.name, *label, target)
                )?;
            }
            Instruction::Return(expr) => {
                emit_aarch64_expr(expr, temporary_base, 0, &mut labels, assembly)?;
                if let Some(label) = shared_epilogue.as_ref() {
                    write_assembly!(assembly, "\tb {label}\n")?;
                } else {
                    emit_aarch64_epilogue(
                        preserved_temp_offset,
                        link_register_offset,
                        stack_bytes,
                        assembly,
                    )?;
                }
            }
        }
    }
    if let Some(label) = shared_epilogue {
        write_assembly!(assembly, "{label}:\n")?;
        emit_aarch64_epilogue(
            preserved_temp_offset,
            link_register_offset,
            stack_bytes,
            assembly,
        )?;
    }
    Ok(())
}

fn emit_x86_64_function(
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
    let mut labels = LabelAllocator::new(function, target);
    write_assembly!(assembly, ".globl {label}\n")?;
    write_assembly!(assembly, "{label}:\n")?;
    assembly.push_str("\tpushq %rbp\n");
    assembly.push_str("\tmovq %rsp, %rbp\n");
    if stack_bytes > 0 {
        write_assembly!(assembly, "\tsubq ${stack_bytes}, %rsp\n")?;
    }
    for instruction in &function.instructions {
        match instruction {
            Instruction::StoreLocal { slot, value } => {
                emit_x86_64_expr(value, temporary_base, 0, target, &mut labels, assembly)?;
                write_assembly!(assembly, "\tmovl %eax, {}(%rbp)\n", x86_local_offset(*slot))?;
            }
            Instruction::JumpIfZero { condition, label } => {
                emit_x86_64_expr(condition, temporary_base, 0, target, &mut labels, assembly)?;
                assembly.push_str("\tcmpl $0, %eax\n");
                write_assembly!(
                    assembly,
                    "\tje {}\n",
                    branch_label(&function.name, *label, target)
                )?;
            }
            Instruction::Jump { label } => {
                write_assembly!(
                    assembly,
                    "\tjmp {}\n",
                    branch_label(&function.name, *label, target)
                )?;
            }
            Instruction::Label { label } => {
                write_assembly!(
                    assembly,
                    "{}:\n",
                    branch_label(&function.name, *label, target)
                )?;
            }
            Instruction::Return(expr) => {
                emit_x86_64_expr(expr, temporary_base, 0, target, &mut labels, assembly)?;
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
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    match expr {
        LoweredExpr::Call { callee } => {
            write_assembly!(assembly, "\tbl {}\n", label_name(callee, labels.target))?;
            Ok(())
        }
        LoweredExpr::Integer(value) => emit_aarch64_i32_to_register(*value, "w0", assembly),
        LoweredExpr::Local(slot) => {
            write_assembly!(assembly, "\tldr w0, [sp, #{}]\n", local_offset(*slot))?;
            Ok(())
        }
        LoweredExpr::Unary { op, expr } => {
            emit_aarch64_expr(expr, temporary_base, depth, labels, assembly)?;
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
            if *op == BinaryOp::LogicalAnd {
                return emit_aarch64_logical_and(
                    left,
                    right,
                    temporary_base,
                    depth,
                    labels,
                    assembly,
                );
            }
            if *op == BinaryOp::LogicalOr {
                return emit_aarch64_logical_or(
                    left,
                    right,
                    temporary_base,
                    depth,
                    labels,
                    assembly,
                );
            }
            let temporary_offset = temporary_base + (depth * 4);
            emit_aarch64_expr(left, temporary_base, depth + 1, labels, assembly)?;
            if expr_is_direct_call(right) {
                assembly.push_str("\tmov w19, w0\n");
                emit_aarch64_expr(right, temporary_base, depth + 1, labels, assembly)?;
                assembly.push_str("\tmov w1, w0\n");
                assembly.push_str("\tmov w0, w19\n");
            } else {
                write_assembly!(assembly, "\tstr w0, [sp, #{temporary_offset}]\n")?;
                emit_aarch64_expr(right, temporary_base, depth + 1, labels, assembly)?;
                assembly.push_str("\tmov w1, w0\n");
                write_assembly!(assembly, "\tldr w0, [sp, #{temporary_offset}]\n")?;
            }
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
                BinaryOp::Less => emit_aarch64_comparison("lt", assembly)?,
                BinaryOp::LessEqual => emit_aarch64_comparison("le", assembly)?,
                BinaryOp::Greater => emit_aarch64_comparison("gt", assembly)?,
                BinaryOp::GreaterEqual => emit_aarch64_comparison("ge", assembly)?,
                BinaryOp::Equal => emit_aarch64_comparison("eq", assembly)?,
                BinaryOp::NotEqual => emit_aarch64_comparison("ne", assembly)?,
                BinaryOp::LogicalAnd | BinaryOp::LogicalOr => {}
                BinaryOp::BitAnd => assembly.push_str("\tand w0, w0, w1\n"),
                BinaryOp::BitXor => assembly.push_str("\teor w0, w0, w1\n"),
                BinaryOp::BitOr => assembly.push_str("\torr w0, w0, w1\n"),
            }
            Ok(())
        }
    }
}

fn emit_aarch64_store_local(
    slot: usize,
    value: &LoweredExpr,
    temporary_base: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    if let LoweredExpr::Binary { op, left, right } = value
        && let (LoweredExpr::Local(local_slot), LoweredExpr::Integer(value)) =
            (left.as_ref(), right.as_ref())
        && *local_slot == slot
        && let Some((instruction, immediate)) = aarch64_update_immediate(*op, *value)
    {
        write_assembly!(assembly, "\tldr w0, [sp, #{}]\n", local_offset(slot))?;
        write_assembly!(assembly, "\t{instruction} w0, w0, #{immediate}\n")?;
        return Ok(());
    }
    emit_aarch64_expr(value, temporary_base, 0, labels, assembly)
}

fn emit_aarch64_prologue(
    preserved_temp_offset: Option<usize>,
    link_register_offset: Option<usize>,
    stack_bytes: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    if stack_bytes > 0 {
        write_assembly!(assembly, "\tsub sp, sp, #{stack_bytes}\n")?;
    }
    if let Some(offset) = link_register_offset {
        write_assembly!(assembly, "\tstr x30, [sp, #{offset}]\n")?;
    }
    if let Some(offset) = preserved_temp_offset {
        write_assembly!(assembly, "\tstr x19, [sp, #{offset}]\n")?;
    }
    Ok(())
}

fn emit_aarch64_epilogue(
    preserved_temp_offset: Option<usize>,
    link_register_offset: Option<usize>,
    stack_bytes: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    if let Some(offset) = preserved_temp_offset {
        write_assembly!(assembly, "\tldr x19, [sp, #{offset}]\n")?;
    }
    if let Some(offset) = link_register_offset {
        write_assembly!(assembly, "\tldr x30, [sp, #{offset}]\n")?;
    }
    if stack_bytes > 0 {
        write_assembly!(assembly, "\tadd sp, sp, #{stack_bytes}\n")?;
    }
    assembly.push_str("\tret\n");
    Ok(())
}

fn emit_aarch64_jump_if_zero(
    condition: &LoweredExpr,
    target_label: &str,
    temporary_base: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    if let LoweredExpr::Binary { op, left, right } = condition
        && let Some(branch) = aarch64_zero_branch_for_comparison(*op)
    {
        emit_aarch64_compare(left, right, temporary_base, labels, assembly)?;
        write_assembly!(assembly, "\t{branch} {target_label}\n")?;
        return Ok(());
    }
    emit_aarch64_expr(condition, temporary_base, 0, labels, assembly)?;
    assembly.push_str("\tcmp w0, #0\n");
    write_assembly!(assembly, "\tb.eq {target_label}\n")
}

fn emit_aarch64_compare(
    left: &LoweredExpr,
    right: &LoweredExpr,
    temporary_base: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    emit_aarch64_expr(left, temporary_base, 1, labels, assembly)?;
    if let LoweredExpr::Integer(value) = right {
        emit_aarch64_i32_to_register(*value, "w1", assembly)?;
    } else {
        write_assembly!(assembly, "\tstr w0, [sp, #{temporary_base}]\n")?;
        emit_aarch64_expr(right, temporary_base, 1, labels, assembly)?;
        assembly.push_str("\tmov w1, w0\n");
        write_assembly!(assembly, "\tldr w0, [sp, #{temporary_base}]\n")?;
    }
    assembly.push_str("\tcmp w0, w1\n");
    Ok(())
}

fn emit_x86_64_expr(
    expr: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    match expr {
        LoweredExpr::Call { callee } => {
            write_assembly!(assembly, "\tcall {}\n", label_name(callee, target))?;
            Ok(())
        }
        LoweredExpr::Integer(value) => {
            let value = i32::try_from(*value)
                .map_err(|_| CompileError::new("integer literal does not fit i32"))?;
            write_assembly!(assembly, "\tmovl ${value}, %eax\n")?;
            Ok(())
        }
        LoweredExpr::Local(slot) => {
            write_assembly!(assembly, "\tmovl {}(%rbp), %eax\n", x86_local_offset(*slot))?;
            Ok(())
        }
        LoweredExpr::Unary { op, expr } => {
            emit_x86_64_expr(expr, temporary_base, depth, target, labels, assembly)?;
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
            if *op == BinaryOp::LogicalAnd {
                return emit_x86_64_logical_and(
                    left,
                    right,
                    temporary_base,
                    depth,
                    target,
                    labels,
                    assembly,
                );
            }
            if *op == BinaryOp::LogicalOr {
                return emit_x86_64_logical_or(
                    left,
                    right,
                    temporary_base,
                    depth,
                    target,
                    labels,
                    assembly,
                );
            }
            let temporary_offset = temporary_base + (depth * 4);
            emit_x86_64_expr(left, temporary_base, depth + 1, target, labels, assembly)?;
            write_assembly!(
                assembly,
                "\tmovl %eax, {}(%rbp)\n",
                x86_stack_offset(temporary_offset)
            )?;
            emit_x86_64_expr(right, temporary_base, depth + 1, target, labels, assembly)?;
            assembly.push_str("\tmovl %eax, %ecx\n");
            write_assembly!(
                assembly,
                "\tmovl {}(%rbp), %eax\n",
                x86_stack_offset(temporary_offset)
            )?;
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
                BinaryOp::Less => emit_x86_64_comparison("setl", assembly)?,
                BinaryOp::LessEqual => emit_x86_64_comparison("setle", assembly)?,
                BinaryOp::Greater => emit_x86_64_comparison("setg", assembly)?,
                BinaryOp::GreaterEqual => emit_x86_64_comparison("setge", assembly)?,
                BinaryOp::Equal => emit_x86_64_comparison("sete", assembly)?,
                BinaryOp::NotEqual => emit_x86_64_comparison("setne", assembly)?,
                BinaryOp::LogicalAnd | BinaryOp::LogicalOr => {}
                BinaryOp::BitAnd => assembly.push_str("\tandl %ecx, %eax\n"),
                BinaryOp::BitXor => assembly.push_str("\txorl %ecx, %eax\n"),
                BinaryOp::BitOr => assembly.push_str("\torl %ecx, %eax\n"),
            }
            Ok(())
        }
    }
}

fn emit_aarch64_logical_and(
    left: &LoweredExpr,
    right: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let false_label = labels.fresh();
    let end_label = labels.fresh();
    emit_aarch64_expr(left, temporary_base, depth, labels, assembly)?;
    assembly.push_str("\tcmp w0, #0\n");
    write_assembly!(assembly, "\tb.eq {false_label}\n")?;
    emit_aarch64_expr(right, temporary_base, depth, labels, assembly)?;
    assembly.push_str("\tcmp w0, #0\n");
    write_assembly!(assembly, "\tb.eq {false_label}\n")?;
    emit_aarch64_i32_to_register(1, "w0", assembly)?;
    write_assembly!(assembly, "\tb {end_label}\n")?;
    write_assembly!(assembly, "{false_label}:\n")?;
    emit_aarch64_i32_to_register(0, "w0", assembly)?;
    write_assembly!(assembly, "{end_label}:\n")?;
    Ok(())
}

fn emit_aarch64_logical_or(
    left: &LoweredExpr,
    right: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let true_label = labels.fresh();
    let end_label = labels.fresh();
    emit_aarch64_expr(left, temporary_base, depth, labels, assembly)?;
    assembly.push_str("\tcmp w0, #0\n");
    write_assembly!(assembly, "\tb.ne {true_label}\n")?;
    emit_aarch64_expr(right, temporary_base, depth, labels, assembly)?;
    assembly.push_str("\tcmp w0, #0\n");
    write_assembly!(assembly, "\tb.ne {true_label}\n")?;
    emit_aarch64_i32_to_register(0, "w0", assembly)?;
    write_assembly!(assembly, "\tb {end_label}\n")?;
    write_assembly!(assembly, "{true_label}:\n")?;
    emit_aarch64_i32_to_register(1, "w0", assembly)?;
    write_assembly!(assembly, "{end_label}:\n")?;
    Ok(())
}

fn emit_x86_64_logical_and(
    left: &LoweredExpr,
    right: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let false_label = labels.fresh();
    let end_label = labels.fresh();
    emit_x86_64_expr(left, temporary_base, depth, target, labels, assembly)?;
    assembly.push_str("\tcmpl $0, %eax\n");
    write_assembly!(assembly, "\tje {false_label}\n")?;
    emit_x86_64_expr(right, temporary_base, depth, target, labels, assembly)?;
    assembly.push_str("\tcmpl $0, %eax\n");
    write_assembly!(assembly, "\tje {false_label}\n")?;
    assembly.push_str("\tmovl $1, %eax\n");
    write_assembly!(assembly, "\tjmp {end_label}\n")?;
    write_assembly!(assembly, "{false_label}:\n")?;
    assembly.push_str("\tmovl $0, %eax\n");
    write_assembly!(assembly, "{end_label}:\n")?;
    Ok(())
}

fn emit_x86_64_logical_or(
    left: &LoweredExpr,
    right: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let true_label = labels.fresh();
    let end_label = labels.fresh();
    emit_x86_64_expr(left, temporary_base, depth, target, labels, assembly)?;
    assembly.push_str("\tcmpl $0, %eax\n");
    write_assembly!(assembly, "\tjne {true_label}\n")?;
    emit_x86_64_expr(right, temporary_base, depth, target, labels, assembly)?;
    assembly.push_str("\tcmpl $0, %eax\n");
    write_assembly!(assembly, "\tjne {true_label}\n")?;
    assembly.push_str("\tmovl $0, %eax\n");
    write_assembly!(assembly, "\tjmp {end_label}\n")?;
    write_assembly!(assembly, "{true_label}:\n")?;
    assembly.push_str("\tmovl $1, %eax\n");
    write_assembly!(assembly, "{end_label}:\n")?;
    Ok(())
}

fn emit_aarch64_comparison(condition: &str, assembly: &mut String) -> CompileResult<()> {
    assembly.push_str("\tcmp w0, w1\n");
    write_assembly!(assembly, "\tcset w0, {condition}\n")
}

fn emit_x86_64_comparison(instruction: &str, assembly: &mut String) -> CompileResult<()> {
    assembly.push_str("\tcmpl %ecx, %eax\n");
    write_assembly!(assembly, "\t{instruction} %al\n")?;
    assembly.push_str("\tmovzbl %al, %eax\n");
    Ok(())
}

fn emit_aarch64_i32_to_register(
    value: i64,
    register: &str,
    assembly: &mut String,
) -> CompileResult<()> {
    let value =
        i32::try_from(value).map_err(|_| CompileError::new("integer literal does not fit i32"))?;
    let bits = u32::from_ne_bytes(value.to_ne_bytes());
    let low = bits & 0xffff;
    let high = (bits >> 16) & 0xffff;
    write_assembly!(assembly, "\tmovz {register}, #{low}\n")?;
    if high != 0 {
        write_assembly!(assembly, "\tmovk {register}, #{high}, lsl #16\n")?;
    }
    Ok(())
}

const fn aarch64_zero_branch_for_comparison(op: BinaryOp) -> Option<&'static str> {
    match op {
        BinaryOp::Less => Some("b.ge"),
        BinaryOp::LessEqual => Some("b.gt"),
        BinaryOp::Greater => Some("b.le"),
        BinaryOp::GreaterEqual => Some("b.lt"),
        BinaryOp::Equal => Some("b.ne"),
        BinaryOp::NotEqual => Some("b.eq"),
        BinaryOp::Mul
        | BinaryOp::Div
        | BinaryOp::Mod
        | BinaryOp::Add
        | BinaryOp::Sub
        | BinaryOp::ShiftLeft
        | BinaryOp::ShiftRight
        | BinaryOp::LogicalAnd
        | BinaryOp::LogicalOr
        | BinaryOp::BitAnd
        | BinaryOp::BitXor
        | BinaryOp::BitOr => None,
    }
}

const fn aarch64_update_immediate(op: BinaryOp, value: i64) -> Option<(&'static str, u64)> {
    let magnitude = value.unsigned_abs();
    if magnitude > 4095 {
        return None;
    }
    match (op, value >= 0) {
        (BinaryOp::Add, true) | (BinaryOp::Sub, false) => Some(("add", magnitude)),
        (BinaryOp::Add, false) | (BinaryOp::Sub, true) => Some(("sub", magnitude)),
        _ => None,
    }
}

fn instruction_depth(instruction: &Instruction) -> usize {
    match instruction {
        Instruction::StoreLocal { value, .. } | Instruction::Return(value) => expr_depth(value),
        Instruction::JumpIfZero { condition, .. } => expr_depth(condition),
        Instruction::Jump { .. } | Instruction::Label { .. } => 0,
    }
}

fn next_available_label(function: &LoweredFunction) -> usize {
    function
        .instructions
        .iter()
        .filter_map(instruction_label)
        .max()
        .map_or(0, |label| label + 1)
}

fn should_share_aarch64_epilogue(function: &LoweredFunction, stack_bytes: usize) -> bool {
    stack_bytes > 0
        && function
            .instructions
            .iter()
            .filter(|instruction| matches!(instruction, Instruction::Return(_)))
            .take(2)
            .count()
            > 1
}

const fn instruction_label(instruction: &Instruction) -> Option<usize> {
    match instruction {
        Instruction::StoreLocal { .. } | Instruction::Return(_) => None,
        Instruction::JumpIfZero { label, .. }
        | Instruction::Jump { label }
        | Instruction::Label { label } => Some(*label),
    }
}

fn expr_depth(expr: &LoweredExpr) -> usize {
    match expr {
        LoweredExpr::Call { .. } | LoweredExpr::Integer(_) | LoweredExpr::Local(_) => 0,
        LoweredExpr::Unary { expr, .. } => expr_depth(expr),
        LoweredExpr::Binary {
            op: BinaryOp::LogicalAnd | BinaryOp::LogicalOr,
            left,
            right,
        } => expr_depth(left).max(expr_depth(right)),
        LoweredExpr::Binary { left, right, .. } => 1 + expr_depth(left).max(expr_depth(right)),
    }
}

fn function_uses_call(function: &LoweredFunction) -> bool {
    function.instructions.iter().any(instruction_uses_call)
}

fn function_uses_aarch64_preserved_temp(function: &LoweredFunction) -> bool {
    function
        .instructions
        .iter()
        .any(instruction_needs_preserved_temp)
}

fn instruction_needs_preserved_temp(instruction: &Instruction) -> bool {
    match instruction {
        Instruction::StoreLocal { value, .. }
        | Instruction::JumpIfZero {
            condition: value, ..
        }
        | Instruction::Return(value) => expr_needs_preserved_temp(value),
        Instruction::Jump { .. } | Instruction::Label { .. } => false,
    }
}

fn expr_needs_preserved_temp(expr: &LoweredExpr) -> bool {
    match expr {
        LoweredExpr::Binary {
            op: BinaryOp::LogicalAnd | BinaryOp::LogicalOr,
            left,
            right,
        } => expr_needs_preserved_temp(left) || expr_needs_preserved_temp(right),
        LoweredExpr::Binary { left, right, .. } => {
            expr_is_direct_call(right)
                || expr_needs_preserved_temp(left)
                || expr_needs_preserved_temp(right)
        }
        LoweredExpr::Unary { expr, .. } => expr_needs_preserved_temp(expr),
        LoweredExpr::Call { .. } | LoweredExpr::Integer(_) | LoweredExpr::Local(_) => false,
    }
}

const fn expr_is_direct_call(expr: &LoweredExpr) -> bool {
    matches!(expr, LoweredExpr::Call { .. })
}

fn instruction_uses_call(instruction: &Instruction) -> bool {
    match instruction {
        Instruction::StoreLocal { value, .. }
        | Instruction::JumpIfZero {
            condition: value, ..
        }
        | Instruction::Return(value) => expr_uses_call(value),
        Instruction::Jump { .. } | Instruction::Label { .. } => false,
    }
}

fn expr_uses_call(expr: &LoweredExpr) -> bool {
    match expr {
        LoweredExpr::Call { .. } => true,
        LoweredExpr::Integer(_) | LoweredExpr::Local(_) => false,
        LoweredExpr::Unary { expr, .. } => expr_uses_call(expr),
        LoweredExpr::Binary { left, right, .. } => expr_uses_call(left) || expr_uses_call(right),
    }
}

const fn local_offset(slot: usize) -> usize {
    slot * 4
}

fn x86_local_offset(slot: usize) -> String {
    format!("-{}", (slot + 1) * 4)
}

fn x86_stack_offset(byte_offset: usize) -> String {
    format!("-{}", byte_offset + 4)
}

const fn align_to(value: usize, alignment: usize) -> usize {
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

fn branch_label(function: &str, label: usize, target: Target) -> String {
    match target {
        Target::Aarch64AppleDarwin | Target::X86_64AppleDarwin => format!("L{function}_{label}"),
        Target::X86_64UnknownLinuxGnu => format!(".L{function}_{label}"),
    }
}
