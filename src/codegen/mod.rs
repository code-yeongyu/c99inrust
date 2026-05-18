use std::fmt::{self, Write as _};

use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::{Instruction, LoweredExpr, LoweredFunction, LoweredProgram};
use crate::parser::{BinaryOp, ScalarType, UnaryOp};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ValueWidth {
    I32,
    I64,
    F64,
}

const TEMPORARY_BYTES: usize = 8;

#[derive(Clone, Copy)]
struct BinaryExpr<'a> {
    op: BinaryOp,
    left: &'a LoweredExpr,
    right: &'a LoweredExpr,
}

#[derive(Clone, Copy)]
struct ConditionalExpr<'a> {
    condition: &'a LoweredExpr,
    then_expr: &'a LoweredExpr,
    else_expr: &'a LoweredExpr,
}

const fn scalar_width(scalar_type: ScalarType) -> ValueWidth {
    match scalar_type {
        ScalarType::Int => ValueWidth::I32,
        ScalarType::LongLong | ScalarType::Pointer => ValueWidth::I64,
        ScalarType::Double => ValueWidth::F64,
    }
}

fn expr_width(expr: &LoweredExpr) -> ValueWidth {
    match expr {
        LoweredExpr::Cast { target, .. } => scalar_width(*target),
        LoweredExpr::DoubleLiteral(_) => ValueWidth::F64,
        LoweredExpr::StringLiteral(_) => ValueWidth::I64,
        LoweredExpr::Local { scalar_type, .. } => scalar_width(*scalar_type),
        LoweredExpr::Unary { op, expr } => match op {
            UnaryOp::LogicalNot => ValueWidth::I32,
            UnaryOp::Plus | UnaryOp::Minus | UnaryOp::BitNot => expr_width(expr),
        },
        LoweredExpr::Conditional {
            then_expr,
            else_expr,
            ..
        } => expr_width(then_expr).max(expr_width(else_expr)),
        LoweredExpr::Binary { op, left, right } => binary_result_width(*op, left, right),
        LoweredExpr::Call { .. } | LoweredExpr::Integer(_) => ValueWidth::I32,
    }
}

fn binary_result_width(op: BinaryOp, left: &LoweredExpr, right: &LoweredExpr) -> ValueWidth {
    if binary_returns_i32(op) {
        ValueWidth::I32
    } else {
        binary_operand_width(op, left, right)
    }
}

fn binary_operand_width(op: BinaryOp, left: &LoweredExpr, right: &LoweredExpr) -> ValueWidth {
    if matches!(op, BinaryOp::LogicalAnd | BinaryOp::LogicalOr) {
        ValueWidth::I32
    } else {
        expr_width(left).max(expr_width(right))
    }
}

const fn binary_returns_i32(op: BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::Less
            | BinaryOp::LessEqual
            | BinaryOp::Greater
            | BinaryOp::GreaterEqual
            | BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::LogicalAnd
            | BinaryOp::LogicalOr
    )
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
    let frame = Aarch64Frame::new(function);
    let mut labels = LabelAllocator::new(function, target);
    let shared_epilogue = if should_share_aarch64_epilogue(function, frame.stack_bytes) {
        Some(labels.fresh())
    } else {
        None
    };
    write_assembly!(assembly, ".globl {label}\n")?;
    assembly.push_str(".p2align 2\n");
    write_assembly!(assembly, "{label}:\n")?;
    emit_aarch64_prologue(
        frame.preserved_temp_offset,
        frame.link_register_offset,
        frame.stack_bytes,
        assembly,
    )?;
    emit_aarch64_parameter_stores(function, assembly)?;
    for instruction in &function.instructions {
        match instruction {
            Instruction::StoreLocal {
                slot,
                offset,
                scalar_type,
                value,
            } => {
                emit_aarch64_store_local(
                    *slot,
                    *offset,
                    *scalar_type,
                    value,
                    frame.temporary_base,
                    &mut labels,
                    assembly,
                )?;
                emit_aarch64_store_result(scalar_width(*scalar_type), *offset, assembly)?;
            }
            Instruction::JumpIfZero { condition, label } => {
                let target_label = branch_label(&function.name, *label, target);
                emit_aarch64_jump_if_zero(
                    condition,
                    &target_label,
                    frame.temporary_base,
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
            Instruction::Eval(expr) => {
                emit_aarch64_expr(expr, frame.temporary_base, 0, &mut labels, assembly)?;
            }
            Instruction::Return(expr) => {
                emit_aarch64_return(
                    expr.as_ref(),
                    Aarch64Epilogue {
                        preserved_temp_offset: frame.preserved_temp_offset,
                        link_register_offset: frame.link_register_offset,
                        stack_bytes: frame.stack_bytes,
                        shared_label: shared_epilogue.as_deref(),
                    },
                    frame.temporary_base,
                    &mut labels,
                    assembly,
                )?;
            }
        }
    }
    if let Some(label) = shared_epilogue {
        write_assembly!(assembly, "{label}:\n")?;
        emit_aarch64_epilogue(
            frame.preserved_temp_offset,
            frame.link_register_offset,
            frame.stack_bytes,
            assembly,
        )?;
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct Aarch64Frame {
    temporary_base: usize,
    stack_bytes: usize,
    link_register_offset: Option<usize>,
    preserved_temp_offset: Option<usize>,
}

impl Aarch64Frame {
    fn new(function: &LoweredFunction) -> Self {
        let temporary_count = function
            .instructions
            .iter()
            .map(instruction_depth)
            .max()
            .unwrap_or(0);
        let local_bytes = local_stack_bytes(function);
        let temporary_base = align_to(local_bytes, TEMPORARY_BYTES);
        let call_frame_bytes = if function_uses_call(function) { 8 } else { 0 };
        let preserved_temp_bytes = if function_uses_aarch64_preserved_temp(function) {
            8
        } else {
            0
        };
        let stack_bytes = align_to(
            temporary_base
                + (temporary_count * TEMPORARY_BYTES)
                + call_frame_bytes
                + preserved_temp_bytes,
            16,
        );
        Self {
            temporary_base,
            stack_bytes,
            link_register_offset: (call_frame_bytes > 0).then(|| stack_bytes - 8),
            preserved_temp_offset: (preserved_temp_bytes > 0)
                .then(|| stack_bytes - call_frame_bytes - 8),
        }
    }
}

#[derive(Clone, Copy)]
struct Aarch64Epilogue<'a> {
    preserved_temp_offset: Option<usize>,
    link_register_offset: Option<usize>,
    stack_bytes: usize,
    shared_label: Option<&'a str>,
}

fn emit_aarch64_return(
    expr: Option<&LoweredExpr>,
    epilogue: Aarch64Epilogue<'_>,
    temporary_base: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    if let Some(expr) = expr {
        emit_aarch64_expr(expr, temporary_base, 0, labels, assembly)?;
    }
    if let Some(label) = epilogue.shared_label {
        write_assembly!(assembly, "\tb {label}\n")?;
        return Ok(());
    }
    emit_aarch64_epilogue(
        epilogue.preserved_temp_offset,
        epilogue.link_register_offset,
        epilogue.stack_bytes,
        assembly,
    )
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
    let local_bytes = local_stack_bytes(function);
    let temporary_base = align_to(local_bytes, TEMPORARY_BYTES);
    let stack_bytes = align_to(temporary_base + (temporary_count * TEMPORARY_BYTES), 16);
    let mut labels = LabelAllocator::new(function, target);
    write_assembly!(assembly, ".globl {label}\n")?;
    write_assembly!(assembly, "{label}:\n")?;
    assembly.push_str("\tpushq %rbp\n");
    assembly.push_str("\tmovq %rsp, %rbp\n");
    if stack_bytes > 0 {
        write_assembly!(assembly, "\tsubq ${stack_bytes}, %rsp\n")?;
    }
    emit_x86_64_parameter_stores(function, assembly)?;
    for instruction in &function.instructions {
        match instruction {
            Instruction::StoreLocal {
                slot: _,
                offset,
                scalar_type,
                value,
            } => {
                emit_x86_64_expr(value, temporary_base, 0, target, &mut labels, assembly)?;
                emit_x86_64_store_result(scalar_width(*scalar_type), *offset, assembly)?;
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
            Instruction::Eval(expr) => {
                emit_x86_64_expr(expr, temporary_base, 0, target, &mut labels, assembly)?;
            }
            Instruction::Return(expr) => {
                if let Some(expr) = expr {
                    emit_x86_64_expr(expr, temporary_base, 0, target, &mut labels, assembly)?;
                }
                assembly.push_str("\tleave\n");
                assembly.push_str("\tret\n");
            }
        }
    }
    Ok(())
}

fn emit_aarch64_parameter_stores(
    function: &LoweredFunction,
    assembly: &mut String,
) -> CompileResult<()> {
    const REGISTERS: [&str; 8] = ["w0", "w1", "w2", "w3", "w4", "w5", "w6", "w7"];
    let Some(registers) = REGISTERS.get(..function.parameter_count) else {
        return Err(CompileError::new("too many function parameters"));
    };
    for (slot, register) in registers.iter().enumerate() {
        write_assembly!(
            assembly,
            "\tstr {register}, [sp, #{}]\n",
            local_offset(function, slot)?
        )?;
    }
    Ok(())
}

fn emit_x86_64_parameter_stores(
    function: &LoweredFunction,
    assembly: &mut String,
) -> CompileResult<()> {
    const REGISTERS: [&str; 6] = ["%edi", "%esi", "%edx", "%ecx", "%r8d", "%r9d"];
    let Some(registers) = REGISTERS.get(..function.parameter_count) else {
        return Err(CompileError::new("too many function parameters"));
    };
    for (slot, register) in registers.iter().enumerate() {
        write_assembly!(
            assembly,
            "\tmovl {register}, {}(%rbp)\n",
            x86_stack_offset(local_offset(function, slot)?, ValueWidth::I32)
        )?;
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
    emit_aarch64_expr_with_width(
        expr,
        expr_width(expr),
        temporary_base,
        depth,
        labels,
        assembly,
    )
}

fn emit_aarch64_expr_with_width(
    expr: &LoweredExpr,
    target_width: ValueWidth,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let natural_width = expr_width(expr);
    emit_aarch64_expr_natural(expr, temporary_base, depth, labels, assembly)?;
    emit_aarch64_width_adjustment(natural_width, target_width, assembly);
    Ok(())
}

fn emit_aarch64_expr_natural(
    expr: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    match expr {
        LoweredExpr::Call { callee, args } => {
            emit_aarch64_call(callee, args, temporary_base, depth, labels, assembly)
        }
        LoweredExpr::Integer(value) => emit_aarch64_i32_to_register(*value, "w0", assembly),
        LoweredExpr::DoubleLiteral(value) => {
            emit_aarch64_load_double_literal(value, labels, assembly)
        }
        LoweredExpr::StringLiteral(value) => {
            emit_aarch64_load_string_address(value, labels, assembly)
        }
        LoweredExpr::Local {
            offset,
            scalar_type,
        } => emit_aarch64_load_temporary(scalar_width(*scalar_type), *offset, assembly),
        LoweredExpr::Unary { op, expr } => {
            emit_aarch64_expr(expr, temporary_base, depth, labels, assembly)?;
            let width = expr_width(expr);
            match op {
                UnaryOp::Plus => {}
                UnaryOp::Minus => match width {
                    ValueWidth::I32 => assembly.push_str("\tneg w0, w0\n"),
                    ValueWidth::I64 => assembly.push_str("\tneg x0, x0\n"),
                    ValueWidth::F64 => assembly.push_str("\tfneg d0, d0\n"),
                },
                UnaryOp::BitNot => match width {
                    ValueWidth::I32 => assembly.push_str("\tmvn w0, w0\n"),
                    ValueWidth::I64 => assembly.push_str("\tmvn x0, x0\n"),
                    ValueWidth::F64 => {
                        return Err(CompileError::new("unsupported double bitwise operator"));
                    }
                },
                UnaryOp::LogicalNot => {
                    emit_aarch64_compare_result_to_zero(width, assembly)?;
                    assembly.push_str("\tcset w0, eq\n");
                }
            }
            Ok(())
        }
        LoweredExpr::Cast { target, expr } => emit_aarch64_expr_with_width(
            expr,
            scalar_width(*target),
            temporary_base,
            depth,
            labels,
            assembly,
        ),
        LoweredExpr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => emit_aarch64_conditional(
            ConditionalExpr {
                condition,
                then_expr,
                else_expr,
            },
            expr_width(expr),
            temporary_base,
            depth,
            labels,
            assembly,
        ),
        LoweredExpr::Binary { op, left, right } => emit_aarch64_binary_expr(
            BinaryExpr {
                op: *op,
                left,
                right,
            },
            temporary_base,
            depth,
            labels,
            assembly,
        ),
    }
}

fn emit_aarch64_binary_expr(
    binary: BinaryExpr<'_>,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    if binary.op == BinaryOp::LogicalAnd {
        return emit_aarch64_logical_and(
            binary.left,
            binary.right,
            temporary_base,
            depth,
            labels,
            assembly,
        );
    }
    if binary.op == BinaryOp::LogicalOr {
        return emit_aarch64_logical_or(
            binary.left,
            binary.right,
            temporary_base,
            depth,
            labels,
            assembly,
        );
    }
    let operand_width = binary_operand_width(binary.op, binary.left, binary.right);
    let temporary_offset = temporary_base + (depth * TEMPORARY_BYTES);
    emit_aarch64_expr_with_width(
        binary.left,
        operand_width,
        temporary_base,
        depth + 1,
        labels,
        assembly,
    )?;
    if expr_is_direct_call(binary.right) {
        emit_aarch64_move_result_to_register("19", operand_width, assembly)?;
        emit_aarch64_expr_with_width(
            binary.right,
            operand_width,
            temporary_base,
            depth + 1,
            labels,
            assembly,
        )?;
        emit_aarch64_move_result_to_register("1", operand_width, assembly)?;
        emit_aarch64_move_register_to_result("19", operand_width, assembly)?;
    } else {
        emit_aarch64_store_temporary(operand_width, temporary_offset, assembly)?;
        emit_aarch64_expr_with_width(
            binary.right,
            operand_width,
            temporary_base,
            depth + 1,
            labels,
            assembly,
        )?;
        emit_aarch64_move_result_to_register("1", operand_width, assembly)?;
        emit_aarch64_load_temporary(operand_width, temporary_offset, assembly)?;
    }
    emit_aarch64_binary_op(binary.op, operand_width, assembly)?;
    Ok(())
}

fn emit_aarch64_width_adjustment(
    actual_width: ValueWidth,
    target_width: ValueWidth,
    assembly: &mut String,
) {
    match (actual_width, target_width) {
        (ValueWidth::I32, ValueWidth::I64) => assembly.push_str("\tsxtw x0, w0\n"),
        (ValueWidth::I32, ValueWidth::F64) => assembly.push_str("\tscvtf d0, w0\n"),
        (ValueWidth::I64, ValueWidth::F64) => assembly.push_str("\tscvtf d0, x0\n"),
        (ValueWidth::F64, ValueWidth::I32) => assembly.push_str("\tfcvtzs w0, d0\n"),
        (ValueWidth::F64, ValueWidth::I64) => assembly.push_str("\tfcvtzs x0, d0\n"),
        _ => {}
    }
}

fn emit_aarch64_call(
    callee: &str,
    args: &[LoweredExpr],
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    const REGISTERS: [&str; 8] = ["0", "1", "2", "3", "4", "5", "6", "7"];
    let Some(registers) = REGISTERS.get(..args.len()) else {
        return Err(CompileError::new("too many function call arguments"));
    };
    let arg_depth = depth + args.len();
    for (index, arg) in args.iter().enumerate() {
        let offset = temporary_base + ((depth + index) * TEMPORARY_BYTES);
        let width = expr_width(arg);
        emit_aarch64_expr_with_width(arg, width, temporary_base, arg_depth, labels, assembly)?;
        emit_aarch64_store_temporary(width, offset, assembly)?;
    }
    for (index, (arg, register)) in args.iter().zip(registers.iter()).enumerate() {
        let offset = temporary_base + ((depth + index) * TEMPORARY_BYTES);
        emit_aarch64_load_temporary_to_register(expr_width(arg), offset, register, assembly)?;
    }
    write_assembly!(assembly, "\tbl {}\n", label_name(callee, labels.target))
}

fn emit_aarch64_conditional(
    expr: ConditionalExpr<'_>,
    result_width: ValueWidth,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let else_label = labels.fresh();
    let end_label = labels.fresh();
    emit_aarch64_expr(expr.condition, temporary_base, depth, labels, assembly)?;
    emit_aarch64_compare_result_to_zero(expr_width(expr.condition), assembly)?;
    write_assembly!(assembly, "\tb.eq {else_label}\n")?;
    emit_aarch64_expr_with_width(
        expr.then_expr,
        result_width,
        temporary_base,
        depth,
        labels,
        assembly,
    )?;
    write_assembly!(assembly, "\tb {end_label}\n")?;
    write_assembly!(assembly, "{else_label}:\n")?;
    emit_aarch64_expr_with_width(
        expr.else_expr,
        result_width,
        temporary_base,
        depth,
        labels,
        assembly,
    )?;
    write_assembly!(assembly, "{end_label}:\n")?;
    Ok(())
}

fn emit_aarch64_move_result_to_register(
    register: &str,
    width: ValueWidth,
    assembly: &mut String,
) -> CompileResult<()> {
    if width == ValueWidth::F64 {
        return write_assembly!(assembly, "\tfmov d{register}, d0\n");
    }
    let prefix = aarch64_register_prefix(width);
    write_assembly!(assembly, "\tmov {prefix}{register}, {prefix}0\n")
}

fn emit_aarch64_move_register_to_result(
    register: &str,
    width: ValueWidth,
    assembly: &mut String,
) -> CompileResult<()> {
    if width == ValueWidth::F64 {
        return write_assembly!(assembly, "\tfmov d0, d{register}\n");
    }
    let prefix = aarch64_register_prefix(width);
    write_assembly!(assembly, "\tmov {prefix}0, {prefix}{register}\n")
}

fn emit_aarch64_store_temporary(
    width: ValueWidth,
    offset: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    let register = aarch64_result_register(width);
    write_assembly!(assembly, "\tstr {register}, [sp, #{offset}]\n")
}

fn emit_aarch64_store_result(
    width: ValueWidth,
    offset: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    let register = aarch64_result_register(width);
    write_assembly!(assembly, "\tstr {register}, [sp, #{offset}]\n")
}

fn emit_aarch64_load_temporary(
    width: ValueWidth,
    offset: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    let register = aarch64_result_register(width);
    write_assembly!(assembly, "\tldr {register}, [sp, #{offset}]\n")
}

fn emit_aarch64_load_temporary_to_register(
    width: ValueWidth,
    offset: usize,
    register: &str,
    assembly: &mut String,
) -> CompileResult<()> {
    let prefix = aarch64_register_prefix(width);
    write_assembly!(assembly, "\tldr {prefix}{register}, [sp, #{offset}]\n")
}

fn emit_aarch64_compare_result_to_zero(
    width: ValueWidth,
    assembly: &mut String,
) -> CompileResult<()> {
    match width {
        ValueWidth::I32 | ValueWidth::I64 => {
            let register = aarch64_result_register(width);
            write_assembly!(assembly, "\tcmp {register}, #0\n")
        }
        ValueWidth::F64 => {
            assembly.push_str("\tfcmp d0, #0.0\n");
            Ok(())
        }
    }
}

fn emit_aarch64_binary_op(
    op: BinaryOp,
    width: ValueWidth,
    assembly: &mut String,
) -> CompileResult<()> {
    match (op, width) {
        (BinaryOp::Mul, ValueWidth::I32) => assembly.push_str("\tmul w0, w0, w1\n"),
        (BinaryOp::Mul, ValueWidth::I64) => assembly.push_str("\tmul x0, x0, x1\n"),
        (BinaryOp::Mul, ValueWidth::F64) => assembly.push_str("\tfmul d0, d0, d1\n"),
        (BinaryOp::Div, ValueWidth::I32) => assembly.push_str("\tsdiv w0, w0, w1\n"),
        (BinaryOp::Div, ValueWidth::I64) => assembly.push_str("\tsdiv x0, x0, x1\n"),
        (BinaryOp::Div, ValueWidth::F64) => assembly.push_str("\tfdiv d0, d0, d1\n"),
        (BinaryOp::Mod, ValueWidth::I32) => {
            assembly.push_str("\tsdiv w2, w0, w1\n");
            assembly.push_str("\tmsub w0, w2, w1, w0\n");
        }
        (BinaryOp::Mod, ValueWidth::I64) => {
            assembly.push_str("\tsdiv x2, x0, x1\n");
            assembly.push_str("\tmsub x0, x2, x1, x0\n");
        }
        (BinaryOp::Add, ValueWidth::I32) => assembly.push_str("\tadd w0, w0, w1\n"),
        (BinaryOp::Add, ValueWidth::I64) => assembly.push_str("\tadd x0, x0, x1\n"),
        (BinaryOp::Add, ValueWidth::F64) => assembly.push_str("\tfadd d0, d0, d1\n"),
        (BinaryOp::Sub, ValueWidth::I32) => assembly.push_str("\tsub w0, w0, w1\n"),
        (BinaryOp::Sub, ValueWidth::I64) => assembly.push_str("\tsub x0, x0, x1\n"),
        (BinaryOp::Sub, ValueWidth::F64) => assembly.push_str("\tfsub d0, d0, d1\n"),
        (BinaryOp::ShiftLeft, ValueWidth::I32) => assembly.push_str("\tlsl w0, w0, w1\n"),
        (BinaryOp::ShiftLeft, ValueWidth::I64) => assembly.push_str("\tlsl x0, x0, x1\n"),
        (BinaryOp::ShiftRight, ValueWidth::I32) => assembly.push_str("\tasr w0, w0, w1\n"),
        (BinaryOp::ShiftRight, ValueWidth::I64) => assembly.push_str("\tasr x0, x0, x1\n"),
        (BinaryOp::Less, _) => emit_aarch64_comparison("lt", width, assembly)?,
        (BinaryOp::LessEqual, _) => emit_aarch64_comparison("le", width, assembly)?,
        (BinaryOp::Greater, _) => emit_aarch64_comparison("gt", width, assembly)?,
        (BinaryOp::GreaterEqual, _) => emit_aarch64_comparison("ge", width, assembly)?,
        (BinaryOp::Equal, _) => emit_aarch64_comparison("eq", width, assembly)?,
        (BinaryOp::NotEqual, _) => emit_aarch64_comparison("ne", width, assembly)?,
        (BinaryOp::BitAnd, ValueWidth::I32) => assembly.push_str("\tand w0, w0, w1\n"),
        (BinaryOp::BitAnd, ValueWidth::I64) => assembly.push_str("\tand x0, x0, x1\n"),
        (BinaryOp::BitXor, ValueWidth::I32) => assembly.push_str("\teor w0, w0, w1\n"),
        (BinaryOp::BitXor, ValueWidth::I64) => assembly.push_str("\teor x0, x0, x1\n"),
        (BinaryOp::BitOr, ValueWidth::I32) => assembly.push_str("\torr w0, w0, w1\n"),
        (BinaryOp::BitOr, ValueWidth::I64) => assembly.push_str("\torr x0, x0, x1\n"),
        (
            BinaryOp::Mod
            | BinaryOp::ShiftLeft
            | BinaryOp::ShiftRight
            | BinaryOp::BitAnd
            | BinaryOp::BitXor
            | BinaryOp::BitOr,
            ValueWidth::F64,
        ) => return Err(CompileError::new("unsupported double operator")),
        (BinaryOp::LogicalAnd | BinaryOp::LogicalOr, _) => {}
    }
    Ok(())
}

fn emit_aarch64_load_double_literal(
    value: &str,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = labels.fresh();
    write_assembly!(assembly, "\tadrp x16, {label}@PAGE\n")?;
    write_assembly!(assembly, "\tldr d0, [x16, {label}@PAGEOFF]\n")?;
    emit_double_literal_data(&label, double_literal_bits(value)?, labels.target, assembly)
}

fn emit_aarch64_load_string_address(
    value: &str,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = labels.fresh();
    write_assembly!(assembly, "\tadrp x0, {label}@PAGE\n")?;
    write_assembly!(assembly, "\tadd x0, x0, {label}@PAGEOFF\n")?;
    emit_string_literal_data(&label, value, labels.target, assembly)
}

const fn aarch64_register_prefix(width: ValueWidth) -> &'static str {
    match width {
        ValueWidth::I32 => "w",
        ValueWidth::I64 => "x",
        ValueWidth::F64 => "d",
    }
}

const fn aarch64_result_register(width: ValueWidth) -> &'static str {
    match width {
        ValueWidth::I32 => "w0",
        ValueWidth::I64 => "x0",
        ValueWidth::F64 => "d0",
    }
}

fn emit_aarch64_store_local(
    _slot: usize,
    offset: usize,
    scalar_type: ScalarType,
    value: &LoweredExpr,
    temporary_base: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    if scalar_type != ScalarType::Int {
        return emit_aarch64_expr(value, temporary_base, 0, labels, assembly);
    }
    if let LoweredExpr::Binary { op, left, right } = value
        && let (
            LoweredExpr::Local {
                offset: local_offset,
                ..
            },
            LoweredExpr::Integer(value),
        ) = (left.as_ref(), right.as_ref())
        && *local_offset == offset
        && let Some((instruction, immediate)) = aarch64_update_immediate(*op, *value)
    {
        write_assembly!(assembly, "\tldr w0, [sp, #{offset}]\n")?;
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
    let width = expr_width(left).max(expr_width(right));
    emit_aarch64_expr_with_width(left, width, temporary_base, 1, labels, assembly)?;
    emit_aarch64_store_temporary(width, temporary_base, assembly)?;
    emit_aarch64_expr_with_width(right, width, temporary_base, 1, labels, assembly)?;
    emit_aarch64_move_result_to_register("1", width, assembly)?;
    emit_aarch64_load_temporary(width, temporary_base, assembly)?;
    emit_aarch64_compare_result_to_rhs(width, assembly)?;
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
    emit_x86_64_expr_with_width(
        expr,
        expr_width(expr),
        temporary_base,
        depth,
        target,
        labels,
        assembly,
    )
}

fn emit_x86_64_expr_with_width(
    expr: &LoweredExpr,
    target_width: ValueWidth,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let natural_width = expr_width(expr);
    emit_x86_64_expr_natural(expr, temporary_base, depth, target, labels, assembly)?;
    emit_x86_64_width_adjustment(natural_width, target_width, assembly);
    Ok(())
}

fn emit_x86_64_expr_natural(
    expr: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    match expr {
        LoweredExpr::Call { callee, args } => emit_x86_64_call(
            callee,
            args,
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        ),
        LoweredExpr::Integer(value) => {
            let value = i32::try_from(*value)
                .map_err(|_| CompileError::new("integer literal does not fit i32"))?;
            write_assembly!(assembly, "\tmovl ${value}, %eax\n")?;
            Ok(())
        }
        LoweredExpr::DoubleLiteral(value) => {
            emit_x86_64_load_double_literal(value, target, labels, assembly)
        }
        LoweredExpr::StringLiteral(value) => {
            emit_x86_64_load_string_address(value, target, labels, assembly)
        }
        LoweredExpr::Local {
            offset,
            scalar_type,
        } => emit_x86_64_load_temporary(scalar_width(*scalar_type), *offset, assembly),
        LoweredExpr::Unary { op, expr } => {
            emit_x86_64_expr(expr, temporary_base, depth, target, labels, assembly)?;
            let width = expr_width(expr);
            match op {
                UnaryOp::Plus => {}
                UnaryOp::Minus => match width {
                    ValueWidth::I32 => assembly.push_str("\tnegl %eax\n"),
                    ValueWidth::I64 => assembly.push_str("\tnegq %rax\n"),
                    ValueWidth::F64 => emit_x86_64_negate_f64(target, labels, assembly)?,
                },
                UnaryOp::BitNot => match width {
                    ValueWidth::I32 => assembly.push_str("\tnotl %eax\n"),
                    ValueWidth::I64 => assembly.push_str("\tnotq %rax\n"),
                    ValueWidth::F64 => {
                        return Err(CompileError::new("unsupported double bitwise operator"));
                    }
                },
                UnaryOp::LogicalNot => {
                    emit_x86_64_compare_result_to_zero(width, assembly);
                    assembly.push_str("\tsete %al\n");
                    assembly.push_str("\tmovzbl %al, %eax\n");
                }
            }
            Ok(())
        }
        LoweredExpr::Cast {
            target: scalar_type,
            expr,
        } => emit_x86_64_expr_with_width(
            expr,
            scalar_width(*scalar_type),
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        ),
        LoweredExpr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => emit_x86_64_conditional(
            ConditionalExpr {
                condition,
                then_expr,
                else_expr,
            },
            expr_width(expr),
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        ),
        LoweredExpr::Binary { op, left, right } => emit_x86_64_binary_expr(
            BinaryExpr {
                op: *op,
                left,
                right,
            },
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        ),
    }
}

fn emit_x86_64_binary_expr(
    binary: BinaryExpr<'_>,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    if binary.op == BinaryOp::LogicalAnd {
        return emit_x86_64_logical_and(
            binary.left,
            binary.right,
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        );
    }
    if binary.op == BinaryOp::LogicalOr {
        return emit_x86_64_logical_or(
            binary.left,
            binary.right,
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        );
    }
    let operand_width = binary_operand_width(binary.op, binary.left, binary.right);
    let temporary_offset = temporary_base + (depth * TEMPORARY_BYTES);
    emit_x86_64_expr_with_width(
        binary.left,
        operand_width,
        temporary_base,
        depth + 1,
        target,
        labels,
        assembly,
    )?;
    emit_x86_64_store_temporary(operand_width, temporary_offset, assembly)?;
    emit_x86_64_expr_with_width(
        binary.right,
        operand_width,
        temporary_base,
        depth + 1,
        target,
        labels,
        assembly,
    )?;
    emit_x86_64_move_result_to_rhs(operand_width, assembly);
    emit_x86_64_load_temporary(operand_width, temporary_offset, assembly)?;
    emit_x86_64_binary_op(binary.op, operand_width, assembly)?;
    Ok(())
}

fn emit_x86_64_width_adjustment(
    actual_width: ValueWidth,
    target_width: ValueWidth,
    assembly: &mut String,
) {
    match (actual_width, target_width) {
        (ValueWidth::I32, ValueWidth::I64) => assembly.push_str("\tcltq\n"),
        (ValueWidth::I32, ValueWidth::F64) => assembly.push_str("\tcvtsi2sdl %eax, %xmm0\n"),
        (ValueWidth::I64, ValueWidth::F64) => assembly.push_str("\tcvtsi2sdq %rax, %xmm0\n"),
        (ValueWidth::F64, ValueWidth::I32) => assembly.push_str("\tcvttsd2sil %xmm0, %eax\n"),
        (ValueWidth::F64, ValueWidth::I64) => assembly.push_str("\tcvttsd2siq %xmm0, %rax\n"),
        _ => {}
    }
}

fn emit_x86_64_call(
    callee: &str,
    args: &[LoweredExpr],
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    const MAX_REGISTER_ARGS: usize = 6;
    if args.len() > MAX_REGISTER_ARGS {
        return Err(CompileError::new("too many function call arguments"));
    }
    let arg_depth = depth + args.len();
    for (index, arg) in args.iter().enumerate() {
        let offset = temporary_base + ((depth + index) * TEMPORARY_BYTES);
        let width = expr_width(arg);
        emit_x86_64_expr_with_width(
            arg,
            width,
            temporary_base,
            arg_depth,
            target,
            labels,
            assembly,
        )?;
        emit_x86_64_store_temporary(width, offset, assembly)?;
    }
    for (index, arg) in args.iter().enumerate() {
        let offset = temporary_base + ((depth + index) * TEMPORARY_BYTES);
        let width = expr_width(arg);
        let register = x86_64_argument_register(index, width)?;
        emit_x86_64_load_temporary_to_register(width, offset, register, assembly)?;
    }
    write_assembly!(assembly, "\tcall {}\n", label_name(callee, target))
}

fn emit_x86_64_conditional(
    expr: ConditionalExpr<'_>,
    result_width: ValueWidth,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let else_label = labels.fresh();
    let end_label = labels.fresh();
    emit_x86_64_expr(
        expr.condition,
        temporary_base,
        depth,
        target,
        labels,
        assembly,
    )?;
    emit_x86_64_compare_result_to_zero(expr_width(expr.condition), assembly);
    write_assembly!(assembly, "\tje {else_label}\n")?;
    emit_x86_64_expr_with_width(
        expr.then_expr,
        result_width,
        temporary_base,
        depth,
        target,
        labels,
        assembly,
    )?;
    write_assembly!(assembly, "\tjmp {end_label}\n")?;
    write_assembly!(assembly, "{else_label}:\n")?;
    emit_x86_64_expr_with_width(
        expr.else_expr,
        result_width,
        temporary_base,
        depth,
        target,
        labels,
        assembly,
    )?;
    write_assembly!(assembly, "{end_label}:\n")?;
    Ok(())
}

fn emit_x86_64_store_temporary(
    width: ValueWidth,
    offset: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    let suffix = x86_64_instruction_suffix(width);
    let register = x86_64_result_register(width);
    write_assembly!(
        assembly,
        "\tmov{suffix} {register}, {}(%rbp)\n",
        x86_stack_offset(offset, width)
    )
}

fn emit_x86_64_store_result(
    width: ValueWidth,
    offset: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    let suffix = x86_64_instruction_suffix(width);
    let register = x86_64_result_register(width);
    write_assembly!(
        assembly,
        "\tmov{suffix} {register}, {}(%rbp)\n",
        x86_stack_offset(offset, width)
    )
}

fn emit_x86_64_load_temporary(
    width: ValueWidth,
    offset: usize,
    assembly: &mut String,
) -> CompileResult<()> {
    let suffix = x86_64_instruction_suffix(width);
    let register = x86_64_result_register(width);
    write_assembly!(
        assembly,
        "\tmov{suffix} {}(%rbp), {register}\n",
        x86_stack_offset(offset, width)
    )
}

fn emit_x86_64_load_temporary_to_register(
    width: ValueWidth,
    offset: usize,
    register: &str,
    assembly: &mut String,
) -> CompileResult<()> {
    let suffix = x86_64_instruction_suffix(width);
    write_assembly!(
        assembly,
        "\tmov{suffix} {}(%rbp), {register}\n",
        x86_stack_offset(offset, width)
    )
}

fn emit_x86_64_move_result_to_rhs(width: ValueWidth, assembly: &mut String) {
    match width {
        ValueWidth::I32 => assembly.push_str("\tmovl %eax, %ecx\n"),
        ValueWidth::I64 => assembly.push_str("\tmovq %rax, %rcx\n"),
        ValueWidth::F64 => assembly.push_str("\tmovsd %xmm0, %xmm1\n"),
    }
}

fn emit_x86_64_compare_result_to_zero(width: ValueWidth, assembly: &mut String) {
    match width {
        ValueWidth::I32 => assembly.push_str("\tcmpl $0, %eax\n"),
        ValueWidth::I64 => assembly.push_str("\tcmpq $0, %rax\n"),
        ValueWidth::F64 => {
            assembly.push_str("\txorpd %xmm1, %xmm1\n");
            assembly.push_str("\tucomisd %xmm1, %xmm0\n");
        }
    }
}

fn emit_x86_64_binary_op(
    op: BinaryOp,
    width: ValueWidth,
    assembly: &mut String,
) -> CompileResult<()> {
    match (op, width) {
        (BinaryOp::Mul, ValueWidth::I32) => assembly.push_str("\timull %ecx, %eax\n"),
        (BinaryOp::Mul, ValueWidth::I64) => assembly.push_str("\timulq %rcx, %rax\n"),
        (BinaryOp::Mul, ValueWidth::F64) => assembly.push_str("\tmulsd %xmm1, %xmm0\n"),
        (BinaryOp::Div, ValueWidth::I32) => {
            assembly.push_str("\tcltd\n");
            assembly.push_str("\tidivl %ecx\n");
        }
        (BinaryOp::Div, ValueWidth::I64) => {
            assembly.push_str("\tcqto\n");
            assembly.push_str("\tidivq %rcx\n");
        }
        (BinaryOp::Div, ValueWidth::F64) => assembly.push_str("\tdivsd %xmm1, %xmm0\n"),
        (BinaryOp::Mod, ValueWidth::I32) => {
            assembly.push_str("\tcltd\n");
            assembly.push_str("\tidivl %ecx\n");
            assembly.push_str("\tmovl %edx, %eax\n");
        }
        (BinaryOp::Mod, ValueWidth::I64) => {
            assembly.push_str("\tcqto\n");
            assembly.push_str("\tidivq %rcx\n");
            assembly.push_str("\tmovq %rdx, %rax\n");
        }
        (BinaryOp::Add, ValueWidth::I32) => assembly.push_str("\taddl %ecx, %eax\n"),
        (BinaryOp::Add, ValueWidth::I64) => assembly.push_str("\taddq %rcx, %rax\n"),
        (BinaryOp::Add, ValueWidth::F64) => assembly.push_str("\taddsd %xmm1, %xmm0\n"),
        (BinaryOp::Sub, ValueWidth::I32) => assembly.push_str("\tsubl %ecx, %eax\n"),
        (BinaryOp::Sub, ValueWidth::I64) => assembly.push_str("\tsubq %rcx, %rax\n"),
        (BinaryOp::Sub, ValueWidth::F64) => assembly.push_str("\tsubsd %xmm1, %xmm0\n"),
        (BinaryOp::ShiftLeft, ValueWidth::I32) => assembly.push_str("\tsall %cl, %eax\n"),
        (BinaryOp::ShiftLeft, ValueWidth::I64) => assembly.push_str("\tsalq %cl, %rax\n"),
        (BinaryOp::ShiftRight, ValueWidth::I32) => assembly.push_str("\tsarl %cl, %eax\n"),
        (BinaryOp::ShiftRight, ValueWidth::I64) => assembly.push_str("\tsarq %cl, %rax\n"),
        (BinaryOp::Less, _) => emit_x86_64_comparison("setl", width, assembly)?,
        (BinaryOp::LessEqual, _) => emit_x86_64_comparison("setle", width, assembly)?,
        (BinaryOp::Greater, _) => emit_x86_64_comparison("setg", width, assembly)?,
        (BinaryOp::GreaterEqual, _) => emit_x86_64_comparison("setge", width, assembly)?,
        (BinaryOp::Equal, _) => emit_x86_64_comparison("sete", width, assembly)?,
        (BinaryOp::NotEqual, _) => emit_x86_64_comparison("setne", width, assembly)?,
        (BinaryOp::BitAnd, ValueWidth::I32) => assembly.push_str("\tandl %ecx, %eax\n"),
        (BinaryOp::BitAnd, ValueWidth::I64) => assembly.push_str("\tandq %rcx, %rax\n"),
        (BinaryOp::BitXor, ValueWidth::I32) => assembly.push_str("\txorl %ecx, %eax\n"),
        (BinaryOp::BitXor, ValueWidth::I64) => assembly.push_str("\txorq %rcx, %rax\n"),
        (BinaryOp::BitOr, ValueWidth::I32) => assembly.push_str("\torl %ecx, %eax\n"),
        (BinaryOp::BitOr, ValueWidth::I64) => assembly.push_str("\torq %rcx, %rax\n"),
        (
            BinaryOp::Mod
            | BinaryOp::ShiftLeft
            | BinaryOp::ShiftRight
            | BinaryOp::BitAnd
            | BinaryOp::BitXor
            | BinaryOp::BitOr,
            ValueWidth::F64,
        ) => return Err(CompileError::new("unsupported double operator")),
        (BinaryOp::LogicalAnd | BinaryOp::LogicalOr, _) => {}
    }
    Ok(())
}

fn emit_x86_64_load_double_literal(
    value: &str,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = labels.fresh();
    write_assembly!(assembly, "\tmovsd {label}(%rip), %xmm0\n")?;
    emit_double_literal_data(&label, double_literal_bits(value)?, target, assembly)
}

fn emit_x86_64_load_string_address(
    value: &str,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = labels.fresh();
    write_assembly!(assembly, "\tleaq {label}(%rip), %rax\n")?;
    emit_string_literal_data(&label, value, target, assembly)
}

fn emit_x86_64_negate_f64(
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = labels.fresh();
    write_assembly!(assembly, "\txorpd {label}(%rip), %xmm0\n")?;
    emit_double_literal_data(&label, 0x8000_0000_0000_0000, target, assembly)
}

fn x86_64_argument_register(index: usize, width: ValueWidth) -> CompileResult<&'static str> {
    const I32_REGISTERS: [&str; 6] = ["%edi", "%esi", "%edx", "%ecx", "%r8d", "%r9d"];
    const I64_REGISTERS: [&str; 6] = ["%rdi", "%rsi", "%rdx", "%rcx", "%r8", "%r9"];
    const F64_REGISTERS: [&str; 8] = [
        "%xmm0", "%xmm1", "%xmm2", "%xmm3", "%xmm4", "%xmm5", "%xmm6", "%xmm7",
    ];
    let registers = match width {
        ValueWidth::I32 => I32_REGISTERS.as_slice(),
        ValueWidth::I64 => I64_REGISTERS.as_slice(),
        ValueWidth::F64 => F64_REGISTERS.as_slice(),
    };
    registers
        .get(index)
        .copied()
        .ok_or_else(|| CompileError::new("too many function call arguments"))
}

const fn x86_64_instruction_suffix(width: ValueWidth) -> &'static str {
    match width {
        ValueWidth::I32 => "l",
        ValueWidth::I64 => "q",
        ValueWidth::F64 => "sd",
    }
}

const fn x86_64_result_register(width: ValueWidth) -> &'static str {
    match width {
        ValueWidth::I32 => "%eax",
        ValueWidth::I64 => "%rax",
        ValueWidth::F64 => "%xmm0",
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

fn emit_aarch64_comparison(
    condition: &str,
    width: ValueWidth,
    assembly: &mut String,
) -> CompileResult<()> {
    emit_aarch64_compare_result_to_rhs(width, assembly)?;
    write_assembly!(assembly, "\tcset w0, {condition}\n")
}

fn emit_x86_64_comparison(
    instruction: &str,
    width: ValueWidth,
    assembly: &mut String,
) -> CompileResult<()> {
    match width {
        ValueWidth::I32 => assembly.push_str("\tcmpl %ecx, %eax\n"),
        ValueWidth::I64 => assembly.push_str("\tcmpq %rcx, %rax\n"),
        ValueWidth::F64 => {
            let condition = match instruction {
                "setl" => "setb",
                "setle" => "setbe",
                "setg" => "seta",
                "setge" => "setae",
                "sete" => "sete",
                "setne" => "setne",
                _ => return Err(CompileError::new("unsupported comparison operator")),
            };
            assembly.push_str("\tucomisd %xmm1, %xmm0\n");
            write_assembly!(assembly, "\t{condition} %al\n")?;
            assembly.push_str("\tmovzbl %al, %eax\n");
            return Ok(());
        }
    }
    write_assembly!(assembly, "\t{instruction} %al\n")?;
    assembly.push_str("\tmovzbl %al, %eax\n");
    Ok(())
}

fn emit_aarch64_compare_result_to_rhs(
    width: ValueWidth,
    assembly: &mut String,
) -> CompileResult<()> {
    match width {
        ValueWidth::I32 | ValueWidth::I64 => {
            let prefix = aarch64_register_prefix(width);
            write_assembly!(assembly, "\tcmp {prefix}0, {prefix}1\n")
        }
        ValueWidth::F64 => {
            assembly.push_str("\tfcmp d0, d1\n");
            Ok(())
        }
    }
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
        Instruction::StoreLocal { value, .. }
        | Instruction::Eval(value)
        | Instruction::Return(Some(value)) => expr_depth(value),
        Instruction::JumpIfZero { condition, .. } => expr_depth(condition),
        Instruction::Return(None) | Instruction::Jump { .. } | Instruction::Label { .. } => 0,
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
        Instruction::StoreLocal { .. } | Instruction::Eval(_) | Instruction::Return(_) => None,
        Instruction::JumpIfZero { label, .. }
        | Instruction::Jump { label }
        | Instruction::Label { label } => Some(*label),
    }
}

fn expr_depth(expr: &LoweredExpr) -> usize {
    match expr {
        LoweredExpr::Integer(_)
        | LoweredExpr::DoubleLiteral(_)
        | LoweredExpr::StringLiteral(_)
        | LoweredExpr::Local { .. } => 0,
        LoweredExpr::Call { args, .. } => call_arg_depth(args),
        LoweredExpr::Cast { expr, .. } | LoweredExpr::Unary { expr, .. } => expr_depth(expr),
        LoweredExpr::Binary {
            op: BinaryOp::LogicalAnd | BinaryOp::LogicalOr,
            left,
            right,
        } => expr_depth(left).max(expr_depth(right)),
        LoweredExpr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => expr_depth(condition)
            .max(expr_depth(then_expr))
            .max(expr_depth(else_expr)),
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
        | Instruction::Eval(value)
        | Instruction::Return(Some(value)) => expr_needs_preserved_temp(value),
        Instruction::Return(None) | Instruction::Jump { .. } | Instruction::Label { .. } => false,
    }
}

fn expr_needs_preserved_temp(expr: &LoweredExpr) -> bool {
    match expr {
        LoweredExpr::Binary {
            op: BinaryOp::LogicalAnd | BinaryOp::LogicalOr,
            left,
            right,
        } => expr_needs_preserved_temp(left) || expr_needs_preserved_temp(right),
        LoweredExpr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => {
            expr_needs_preserved_temp(condition)
                || expr_needs_preserved_temp(then_expr)
                || expr_needs_preserved_temp(else_expr)
        }
        LoweredExpr::Binary { left, right, .. } => {
            expr_is_direct_call(right)
                || expr_needs_preserved_temp(left)
                || expr_needs_preserved_temp(right)
        }
        LoweredExpr::Cast { expr, .. } | LoweredExpr::Unary { expr, .. } => {
            expr_needs_preserved_temp(expr)
        }
        LoweredExpr::Call { args, .. } => args.iter().any(expr_needs_preserved_temp),
        LoweredExpr::Integer(_)
        | LoweredExpr::DoubleLiteral(_)
        | LoweredExpr::StringLiteral(_)
        | LoweredExpr::Local { .. } => false,
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
        | Instruction::Eval(value)
        | Instruction::Return(Some(value)) => expr_uses_call(value),
        Instruction::Return(None) | Instruction::Jump { .. } | Instruction::Label { .. } => false,
    }
}

fn expr_uses_call(expr: &LoweredExpr) -> bool {
    match expr {
        LoweredExpr::Call { .. } => true,
        LoweredExpr::Integer(_)
        | LoweredExpr::DoubleLiteral(_)
        | LoweredExpr::StringLiteral(_)
        | LoweredExpr::Local { .. } => false,
        LoweredExpr::Cast { expr, .. } | LoweredExpr::Unary { expr, .. } => expr_uses_call(expr),
        LoweredExpr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => expr_uses_call(condition) || expr_uses_call(then_expr) || expr_uses_call(else_expr),
        LoweredExpr::Binary { left, right, .. } => expr_uses_call(left) || expr_uses_call(right),
    }
}

fn local_offset(function: &LoweredFunction, slot: usize) -> CompileResult<usize> {
    function
        .local_slots
        .get(slot)
        .map(|local_slot| local_slot.offset)
        .ok_or_else(|| CompileError::new("internal error: missing local slot"))
}

fn x86_stack_offset(byte_offset: usize, width: ValueWidth) -> String {
    format!("-{}", byte_offset + width_bytes(width))
}

fn local_stack_bytes(function: &LoweredFunction) -> usize {
    function
        .local_slots
        .iter()
        .map(|local_slot| local_slot.offset + width_bytes(scalar_width(local_slot.scalar_type)))
        .max()
        .unwrap_or(0)
}

fn double_literal_bits(value: &str) -> CompileResult<u64> {
    value
        .parse::<f64>()
        .map(f64::to_bits)
        .map_err(|_| CompileError::new(format!("invalid double literal: {value}")))
}

fn emit_double_literal_data(
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

fn emit_string_literal_data(
    label: &str,
    value: &str,
    target: Target,
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
    assembly.push_str(".text\n");
    Ok(())
}

const fn align_to(value: usize, alignment: usize) -> usize {
    let remainder = value % alignment;
    if remainder == 0 {
        value
    } else {
        value + (alignment - remainder)
    }
}

fn call_arg_depth(args: &[LoweredExpr]) -> usize {
    if args.is_empty() {
        0
    } else {
        args.len() + args.iter().map(expr_depth).max().unwrap_or(0)
    }
}

const fn width_bytes(width: ValueWidth) -> usize {
    match width {
        ValueWidth::I32 => 4,
        ValueWidth::I64 | ValueWidth::F64 => 8,
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
