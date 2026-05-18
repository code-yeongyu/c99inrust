use std::fmt::{self, Write as _};

use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::{
    Instruction, LoweredExpr, LoweredFunction, LoweredGlobal, LoweredGlobalInitializer,
    LoweredLValue, LoweredProgram,
};
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

#[derive(Clone, Copy)]
struct PointerSubscriptExpr<'a> {
    pointer: &'a LoweredExpr,
    index: &'a LoweredExpr,
    element_type: ScalarType,
    element_byte_size: usize,
}

#[derive(Clone, Copy)]
struct PointerOffsetExpr<'a> {
    pointer: &'a LoweredExpr,
    index: &'a LoweredExpr,
    byte_size: usize,
}

#[derive(Clone, Copy)]
struct GlobalByteSubscriptExpr<'a> {
    name: &'a str,
    index: &'a LoweredExpr,
}

#[derive(Clone, Copy)]
struct PointerFieldExpr<'a> {
    pointer: &'a LoweredExpr,
    offset: usize,
    scalar_type: ScalarType,
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
        LoweredExpr::StringLiteral(_)
        | LoweredExpr::LocalAddress { .. }
        | LoweredExpr::GlobalAddress { .. }
        | LoweredExpr::GlobalPointerSubscript { .. }
        | LoweredExpr::PointerOffset { .. }
        | LoweredExpr::PointerFieldAddress { .. } => ValueWidth::I64,
        LoweredExpr::Global { scalar_type, .. } | LoweredExpr::Local { scalar_type, .. } => {
            scalar_width(*scalar_type)
        }
        LoweredExpr::PointerField { scalar_type, .. } => scalar_width(*scalar_type),
        LoweredExpr::GlobalByteSubscript { .. }
        | LoweredExpr::GlobalIntSubscript { .. }
        | LoweredExpr::PointerSubscript {
            element_type: ScalarType::Int,
            ..
        }
        | LoweredExpr::Call { .. }
        | LoweredExpr::IndirectCall { .. }
        | LoweredExpr::Integer(_) => ValueWidth::I32,
        LoweredExpr::PointerSubscript { element_type, .. } => scalar_width(*element_type),
        LoweredExpr::Assign { target, .. } | LoweredExpr::PostIncrement { target } => {
            lowered_lvalue_width(target)
        }
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
    }
}

const fn lowered_lvalue_width(target: &LoweredLValue) -> ValueWidth {
    match target {
        LoweredLValue::Local { scalar_type, .. } | LoweredLValue::Global { scalar_type, .. } => {
            scalar_width(*scalar_type)
        }
        LoweredLValue::GlobalByteSubscript { .. } | LoweredLValue::GlobalIntSubscript { .. } => {
            ValueWidth::I32
        }
        LoweredLValue::GlobalPointerSubscript { .. } => ValueWidth::I64,
        LoweredLValue::PointerSubscript { element_type, .. } => scalar_width(*element_type),
        LoweredLValue::PointerField { scalar_type, .. } => scalar_width(*scalar_type),
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
/// Returns an error when an expression cannot be represented by the current
/// scalar backend.
pub fn emit_assembly(program: &LoweredProgram, target: Target) -> CompileResult<String> {
    let mut assembly = String::new();
    emit_globals(&program.globals, target, &mut assembly)?;
    if program.functions.is_empty() {
        if target == Target::X86_64UnknownLinuxGnu && !assembly.is_empty() {
            assembly.push_str(".section .note.GNU-stack,\"\",@progbits\n");
        }
        return Ok(assembly);
    }
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

fn emit_globals(
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
        write_assembly!(assembly, ".globl {label}\n")?;
        match &global.initializer {
            LoweredGlobalInitializer::Int(value) => {
                assembly.push_str(".p2align 2\n");
                write_assembly!(assembly, "{label}:\n")?;
                write_assembly!(assembly, "\t.long {value}\n")?;
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
            LoweredGlobalInitializer::PointerNull => {
                assembly.push_str(".p2align 3\n");
                write_assembly!(assembly, "{label}:\n")?;
                assembly.push_str("\t.quad 0\n");
            }
            LoweredGlobalInitializer::PointerString(value) => {
                emit_pointer_string_global(&global.name, value, target, assembly)?;
            }
            LoweredGlobalInitializer::PointerArray(length) => {
                let byte_len = length
                    .checked_mul(8)
                    .ok_or_else(|| CompileError::new("global pointer-array size overflow"))?;
                assembly.push_str(".p2align 3\n");
                write_assembly!(assembly, "{label}:\n")?;
                write_assembly!(assembly, "\t.zero {byte_len}\n")?;
            }
            LoweredGlobalInitializer::PointerStringArray(values) => {
                emit_pointer_string_array_global(&global.name, values, target, assembly)?;
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

fn emit_pointer_string_global(
    name: &str,
    value: &str,
    target: Target,
    assembly: &mut String,
) -> CompileResult<()> {
    let string_label = global_string_label(name, 0, target);
    let label = label_name(name, target);
    assembly.push_str(".p2align 3\n");
    write_assembly!(assembly, "{label}:\n")?;
    write_assembly!(assembly, "\t.quad {string_label}\n")?;
    emit_string_literal_data_returning_to(&string_label, value, target, ".data\n", assembly)
}

fn emit_pointer_string_array_global(
    name: &str,
    values: &[String],
    target: Target,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = label_name(name, target);
    assembly.push_str(".p2align 3\n");
    write_assembly!(assembly, "{label}:\n")?;
    for (index, _) in values.iter().enumerate() {
        let string_label = global_string_label(name, index, target);
        write_assembly!(assembly, "\t.quad {string_label}\n")?;
    }
    for (index, value) in values.iter().enumerate() {
        let string_label = global_string_label(name, index, target);
        emit_string_literal_data_returning_to(&string_label, value, target, ".data\n", assembly)?;
    }
    Ok(())
}

fn emit_int_values(values: &[i32], assembly: &mut String) -> CompileResult<()> {
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

fn emit_byte_values(values: &[u8], assembly: &mut String) -> CompileResult<()> {
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
            } => emit_aarch64_store_local_instruction(
                (*slot, *offset, *scalar_type),
                value,
                frame.temporary_base,
                &mut labels,
                assembly,
            )?,
            Instruction::InitLocalBytes { offset, values } => {
                emit_aarch64_init_local_bytes(*offset, values, assembly)?;
            }
            Instruction::InitLocalInts { offset, values } => {
                emit_aarch64_init_local_ints(*offset, values, assembly)?;
            }
            Instruction::StoreGlobal {
                name,
                scalar_type,
                value,
            } => {
                emit_aarch64_expr(value, frame.temporary_base, 0, &mut labels, assembly)?;
                emit_aarch64_store_global(name, scalar_width(*scalar_type), target, assembly)?;
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

fn emit_aarch64_store_local_instruction(
    local: (usize, usize, ScalarType),
    value: &LoweredExpr,
    temporary_base: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let (slot, offset, scalar_type) = local;
    emit_aarch64_store_local(
        slot,
        offset,
        scalar_type,
        value,
        temporary_base,
        labels,
        assembly,
    )?;
    emit_aarch64_store_result(scalar_width(scalar_type), offset, assembly)
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
            Instruction::InitLocalBytes { offset, values } => {
                emit_x86_64_init_local_bytes(*offset, values, assembly)?;
            }
            Instruction::InitLocalInts { offset, values } => {
                emit_x86_64_init_local_ints(*offset, values, assembly)?;
            }
            Instruction::StoreGlobal {
                name,
                scalar_type,
                value,
            } => {
                emit_x86_64_expr(value, temporary_base, 0, target, &mut labels, assembly)?;
                emit_x86_64_store_global(name, scalar_width(*scalar_type), target, assembly)?;
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
    const MAX_REGISTER_ARGS: usize = 8;
    if function.parameter_count > MAX_REGISTER_ARGS {
        return Err(CompileError::new("too many function parameters"));
    }
    for slot in 0..function.parameter_count {
        let Some(local_slot) = function.local_slots.get(slot) else {
            return Err(CompileError::new("internal error: missing parameter slot"));
        };
        let width = scalar_width(local_slot.scalar_type);
        let register = aarch64_parameter_register(slot, width);
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
    const MAX_REGISTER_ARGS: usize = 6;
    if function.parameter_count > MAX_REGISTER_ARGS {
        return Err(CompileError::new("too many function parameters"));
    }
    for slot in 0..function.parameter_count {
        let Some(local_slot) = function.local_slots.get(slot) else {
            return Err(CompileError::new("internal error: missing parameter slot"));
        };
        let width = scalar_width(local_slot.scalar_type);
        let register = x86_64_argument_register(slot, width)?;
        write_assembly!(
            assembly,
            "\tmov{} {register}, {}(%rbp)\n",
            x86_64_instruction_suffix(width),
            x86_stack_offset(local_offset(function, slot)?, width)
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
        expr @ (LoweredExpr::Call { .. } | LoweredExpr::IndirectCall { .. }) => {
            emit_aarch64_call_expr(expr, temporary_base, depth, labels, assembly)
        }
        LoweredExpr::Integer(value) => emit_aarch64_i32_to_register(*value, "w0", assembly),
        LoweredExpr::DoubleLiteral(value) => {
            emit_aarch64_load_double_literal(value, labels, assembly)
        }
        LoweredExpr::StringLiteral(value) => {
            emit_aarch64_load_string_address(value, labels, assembly)
        }
        LoweredExpr::LocalAddress { offset, .. } => {
            write_assembly!(assembly, "\tadd x0, sp, #{offset}\n")
        }
        LoweredExpr::GlobalAddress { name } => {
            let label = label_name(name, labels.target);
            write_assembly!(assembly, "\tadrp x0, {label}@PAGE\n")?;
            write_assembly!(assembly, "\tadd x0, x0, {label}@PAGEOFF\n")
        }
        LoweredExpr::PointerOffset {
            pointer,
            index,
            byte_size,
        } => emit_aarch64_pointer_offset(
            PointerOffsetExpr {
                pointer,
                index,
                byte_size: *byte_size,
            },
            temporary_base,
            depth,
            labels,
            assembly,
        ),
        LoweredExpr::PointerFieldAddress { pointer, offset } => {
            emit_aarch64_expr_with_width(
                pointer,
                ValueWidth::I64,
                temporary_base,
                depth + 1,
                labels,
                assembly,
            )?;
            write_assembly!(assembly, "\tadd x0, x0, #{offset}\n")
        }
        expr @ (LoweredExpr::Global { .. }
        | LoweredExpr::GlobalByteSubscript { .. }
        | LoweredExpr::GlobalIntSubscript { .. }
        | LoweredExpr::GlobalPointerSubscript { .. }
        | LoweredExpr::PointerSubscript { .. }
        | LoweredExpr::PointerField { .. }
        | LoweredExpr::Assign { .. }
        | LoweredExpr::PostIncrement { .. }) => {
            emit_aarch64_memory_expr(expr, temporary_base, depth, labels, assembly)
        }
        LoweredExpr::Local {
            offset,
            scalar_type,
        } => emit_aarch64_load_temporary(scalar_width(*scalar_type), *offset, assembly),
        LoweredExpr::Unary { op, expr } => {
            emit_aarch64_unary_expr(*op, expr, temporary_base, depth, labels, assembly)
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

fn emit_aarch64_memory_expr(
    expr: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    match expr {
        LoweredExpr::Global { name, scalar_type } => {
            emit_aarch64_load_global(name, scalar_width(*scalar_type), labels.target, assembly)
        }
        LoweredExpr::GlobalByteSubscript { name, index } => {
            emit_aarch64_load_global_byte_subscript(
                name,
                index,
                temporary_base,
                depth,
                labels,
                assembly,
            )
        }
        LoweredExpr::GlobalIntSubscript { name, index } => emit_aarch64_load_global_int_subscript(
            name,
            index,
            temporary_base,
            depth,
            labels,
            assembly,
        ),
        LoweredExpr::GlobalPointerSubscript { name, index } => {
            emit_aarch64_load_global_pointer_subscript(
                name,
                index,
                temporary_base,
                depth,
                labels,
                assembly,
            )
        }
        LoweredExpr::PointerSubscript {
            pointer,
            index,
            element_type,
            element_byte_size,
        } => emit_aarch64_load_pointer_subscript(
            PointerSubscriptExpr {
                pointer,
                index,
                element_type: *element_type,
                element_byte_size: *element_byte_size,
            },
            temporary_base,
            depth,
            labels,
            assembly,
        ),
        LoweredExpr::PointerField {
            pointer,
            offset,
            scalar_type,
        } => emit_aarch64_load_pointer_field(
            PointerFieldExpr {
                pointer,
                offset: *offset,
                scalar_type: *scalar_type,
            },
            temporary_base,
            depth,
            labels,
            assembly,
        ),
        LoweredExpr::Assign { target, value } => {
            emit_aarch64_assign(target, value, temporary_base, depth, labels, assembly)
        }
        LoweredExpr::PostIncrement { target } => {
            emit_aarch64_post_increment(target, temporary_base, depth, labels, assembly)
        }
        _ => Err(CompileError::new(
            "internal error: expected aarch64 memory expression",
        )),
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

fn emit_aarch64_call_expr(
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
        LoweredExpr::IndirectCall { callee, args } => {
            emit_aarch64_indirect_call(callee, args, temporary_base, depth, labels, assembly)
        }
        _ => Err(CompileError::new(
            "internal error: expected aarch64 call expression",
        )),
    }
}

fn emit_aarch64_indirect_call(
    callee: &LoweredExpr,
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
    let callee_offset = temporary_base + ((depth + args.len()) * TEMPORARY_BYTES);
    let arg_depth = depth + args.len() + 1;
    for (index, arg) in args.iter().enumerate() {
        let offset = temporary_base + ((depth + index) * TEMPORARY_BYTES);
        let width = expr_width(arg);
        emit_aarch64_expr_with_width(arg, width, temporary_base, arg_depth, labels, assembly)?;
        emit_aarch64_store_temporary(width, offset, assembly)?;
    }
    emit_aarch64_expr_with_width(
        callee,
        ValueWidth::I64,
        temporary_base,
        arg_depth,
        labels,
        assembly,
    )?;
    emit_aarch64_store_temporary(ValueWidth::I64, callee_offset, assembly)?;
    for (index, (arg, register)) in args.iter().zip(registers.iter()).enumerate() {
        let offset = temporary_base + ((depth + index) * TEMPORARY_BYTES);
        emit_aarch64_load_temporary_to_register(expr_width(arg), offset, register, assembly)?;
    }
    emit_aarch64_load_temporary_to_register(ValueWidth::I64, callee_offset, "16", assembly)?;
    write_assembly!(assembly, "\tblr x16\n")
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

fn emit_aarch64_init_local_bytes(
    offset: usize,
    values: &[u8],
    assembly: &mut String,
) -> CompileResult<()> {
    for (index, value) in values.iter().enumerate() {
        let byte_offset = offset
            .checked_add(index)
            .ok_or_else(|| CompileError::new("local byte initializer offset overflow"))?;
        write_assembly!(assembly, "\tmov w16, #{value}\n")?;
        write_assembly!(assembly, "\tstrb w16, [sp, #{byte_offset}]\n")?;
    }
    Ok(())
}

fn emit_aarch64_init_local_ints(
    offset: usize,
    values: &[i32],
    assembly: &mut String,
) -> CompileResult<()> {
    for (index, value) in values.iter().enumerate() {
        let byte_offset = offset
            .checked_add(
                index
                    .checked_mul(4)
                    .ok_or_else(|| CompileError::new("local int initializer offset overflow"))?,
            )
            .ok_or_else(|| CompileError::new("local int initializer offset overflow"))?;
        emit_aarch64_i32_to_register(i64::from(*value), "w16", assembly)?;
        write_assembly!(assembly, "\tstr w16, [sp, #{byte_offset}]\n")?;
    }
    Ok(())
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

fn emit_aarch64_load_global(
    name: &str,
    width: ValueWidth,
    target: Target,
    assembly: &mut String,
) -> CompileResult<()> {
    if target != Target::Aarch64AppleDarwin {
        return Err(CompileError::new("unsupported AArch64 global target"));
    }
    let label = label_name(name, target);
    let register = aarch64_result_register(width);
    write_assembly!(assembly, "\tadrp x16, {label}@PAGE\n")?;
    write_assembly!(assembly, "\tldr {register}, [x16, {label}@PAGEOFF]\n")
}

fn emit_aarch64_store_global(
    name: &str,
    width: ValueWidth,
    target: Target,
    assembly: &mut String,
) -> CompileResult<()> {
    if target != Target::Aarch64AppleDarwin {
        return Err(CompileError::new("unsupported AArch64 global target"));
    }
    let label = label_name(name, target);
    let register = aarch64_result_register(width);
    write_assembly!(assembly, "\tadrp x16, {label}@PAGE\n")?;
    write_assembly!(assembly, "\tstr {register}, [x16, {label}@PAGEOFF]\n")
}

fn emit_aarch64_load_global_byte_subscript(
    name: &str,
    index: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = label_name(name, labels.target);
    emit_aarch64_expr_with_width(
        index,
        ValueWidth::I32,
        temporary_base,
        depth,
        labels,
        assembly,
    )?;
    write_assembly!(assembly, "\tadrp x16, {label}@PAGE\n")?;
    write_assembly!(assembly, "\tadd x16, x16, {label}@PAGEOFF\n")?;
    assembly.push_str("\tldrb w0, [x16, w0, sxtw]\n");
    Ok(())
}

fn emit_aarch64_load_global_int_subscript(
    name: &str,
    index: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = label_name(name, labels.target);
    emit_aarch64_expr_with_width(
        index,
        ValueWidth::I32,
        temporary_base,
        depth,
        labels,
        assembly,
    )?;
    write_assembly!(assembly, "\tadrp x16, {label}@PAGE\n")?;
    write_assembly!(assembly, "\tadd x16, x16, {label}@PAGEOFF\n")?;
    assembly.push_str("\tldr w0, [x16, w0, sxtw #2]\n");
    Ok(())
}

fn emit_aarch64_load_global_pointer_subscript(
    name: &str,
    index: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = label_name(name, labels.target);
    emit_aarch64_expr_with_width(
        index,
        ValueWidth::I32,
        temporary_base,
        depth,
        labels,
        assembly,
    )?;
    write_assembly!(assembly, "\tadrp x16, {label}@PAGE\n")?;
    write_assembly!(assembly, "\tadd x16, x16, {label}@PAGEOFF\n")?;
    assembly.push_str("\tldr x0, [x16, w0, sxtw #3]\n");
    Ok(())
}

fn emit_aarch64_load_pointer_subscript(
    subscript: PointerSubscriptExpr<'_>,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let base_offset = temporary_base + (depth * TEMPORARY_BYTES);
    let width = scalar_width(subscript.element_type);
    emit_aarch64_expr_with_width(
        subscript.pointer,
        ValueWidth::I64,
        temporary_base,
        depth + 1,
        labels,
        assembly,
    )?;
    emit_aarch64_store_temporary(ValueWidth::I64, base_offset, assembly)?;
    emit_aarch64_expr_with_width(
        subscript.index,
        ValueWidth::I32,
        temporary_base,
        depth + 1,
        labels,
        assembly,
    )?;
    emit_aarch64_load_temporary_to_register(ValueWidth::I64, base_offset, "16", assembly)?;
    if subscript.element_byte_size == 1 && width == ValueWidth::I32 {
        return write_assembly!(assembly, "\tldrb w0, [x16, w0, sxtw]\n");
    }
    let Some(shift) = memory_scale_shift_for_byte_size(subscript.element_byte_size) else {
        return Err(CompileError::new(
            "unsupported pointer subscript element size",
        ));
    };
    write_assembly!(
        assembly,
        "\tldr {}, [x16, w0, sxtw #{}]\n",
        aarch64_result_register(width),
        shift
    )
}

fn emit_aarch64_pointer_offset(
    offset: PointerOffsetExpr<'_>,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let base_offset = temporary_base + (depth * TEMPORARY_BYTES);
    emit_aarch64_expr_with_width(
        offset.pointer,
        ValueWidth::I64,
        temporary_base,
        depth + 1,
        labels,
        assembly,
    )?;
    emit_aarch64_store_temporary(ValueWidth::I64, base_offset, assembly)?;
    emit_aarch64_expr_with_width(
        offset.index,
        ValueWidth::I32,
        temporary_base,
        depth + 1,
        labels,
        assembly,
    )?;
    emit_aarch64_load_temporary_to_register(ValueWidth::I64, base_offset, "16", assembly)?;
    if let Some(shift) = memory_scale_shift_for_byte_size(offset.byte_size) {
        if shift == 0 {
            assembly.push_str("\tadd x0, x16, w0, sxtw\n");
        } else {
            write_assembly!(assembly, "\tadd x0, x16, w0, sxtw #{shift}\n")?;
        }
        return Ok(());
    }
    assembly.push_str("\tsxtw x0, w0\n");
    let byte_size = i64::try_from(offset.byte_size)
        .map_err(|_| CompileError::new("pointer offset size does not fit i64"))?;
    emit_aarch64_i32_to_register(byte_size, "x17", assembly)?;
    assembly.push_str("\tmul x0, x0, x17\n");
    assembly.push_str("\tadd x0, x16, x0\n");
    Ok(())
}

fn emit_aarch64_assign(
    target: &LoweredLValue,
    value: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let width = lowered_lvalue_width(target);
    match target {
        LoweredLValue::Local { offset, .. } => {
            emit_aarch64_expr_with_width(value, width, temporary_base, depth, labels, assembly)?;
            emit_aarch64_store_result(width, *offset, assembly)
        }
        LoweredLValue::Global { name, .. } => {
            emit_aarch64_expr_with_width(value, width, temporary_base, depth, labels, assembly)?;
            emit_aarch64_store_global(name, width, labels.target, assembly)
        }
        LoweredLValue::GlobalByteSubscript { name, index } => {
            emit_aarch64_store_global_byte_subscript(
                GlobalByteSubscriptExpr { name, index },
                value,
                temporary_base,
                depth,
                labels,
                assembly,
            )
        }
        LoweredLValue::GlobalIntSubscript { name, index } => {
            emit_aarch64_store_global_int_subscript(
                GlobalByteSubscriptExpr { name, index },
                value,
                temporary_base,
                depth,
                labels,
                assembly,
            )
        }
        LoweredLValue::GlobalPointerSubscript { name, index } => {
            emit_aarch64_store_global_pointer_subscript(
                GlobalByteSubscriptExpr { name, index },
                value,
                temporary_base,
                depth,
                labels,
                assembly,
            )
        }
        LoweredLValue::PointerSubscript {
            pointer,
            index,
            element_type,
            element_byte_size,
        } => emit_aarch64_store_pointer_subscript(
            PointerSubscriptExpr {
                pointer,
                index,
                element_type: *element_type,
                element_byte_size: *element_byte_size,
            },
            value,
            temporary_base,
            depth,
            labels,
            assembly,
        ),
        LoweredLValue::PointerField {
            pointer,
            offset,
            scalar_type,
        } => emit_aarch64_store_pointer_field(
            PointerFieldExpr {
                pointer,
                offset: *offset,
                scalar_type: *scalar_type,
            },
            value,
            temporary_base,
            depth,
            labels,
            assembly,
        ),
    }
}

fn emit_aarch64_post_increment(
    target: &LoweredLValue,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let width = lowered_lvalue_width(target);
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
    match target {
        LoweredLValue::Local { offset, .. } => {
            emit_aarch64_load_temporary(width, *offset, assembly)?;
            emit_aarch64_store_temporary(width, value_offset, assembly)?;
            emit_aarch64_increment_result(width, assembly)?;
            emit_aarch64_store_result(width, *offset, assembly)?;
            emit_aarch64_load_temporary(width, value_offset, assembly)
        }
        LoweredLValue::Global { name, .. } => {
            emit_aarch64_load_global(name, width, labels.target, assembly)?;
            emit_aarch64_store_temporary(width, value_offset, assembly)?;
            emit_aarch64_increment_result(width, assembly)?;
            emit_aarch64_store_global(name, width, labels.target, assembly)?;
            emit_aarch64_load_temporary(width, value_offset, assembly)
        }
        LoweredLValue::PointerField {
            pointer,
            offset,
            scalar_type,
        } => emit_aarch64_post_increment_pointer_field(
            PointerFieldExpr {
                pointer,
                offset: *offset,
                scalar_type: *scalar_type,
            },
            temporary_base,
            depth,
            labels,
            assembly,
        ),
        LoweredLValue::GlobalByteSubscript { .. }
        | LoweredLValue::GlobalIntSubscript { .. }
        | LoweredLValue::GlobalPointerSubscript { .. }
        | LoweredLValue::PointerSubscript { .. } => Err(CompileError::new(
            "post-increment expression supports direct lvalues only",
        )),
    }
}

fn emit_aarch64_post_increment_pointer_field(
    field: PointerFieldExpr<'_>,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let width = scalar_width(field.scalar_type);
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
    let base_offset = temporary_base + ((depth + 1) * TEMPORARY_BYTES);
    emit_aarch64_expr_with_width(
        field.pointer,
        ValueWidth::I64,
        temporary_base,
        depth + 2,
        labels,
        assembly,
    )?;
    emit_aarch64_store_temporary(ValueWidth::I64, base_offset, assembly)?;
    emit_aarch64_load_temporary_to_register(ValueWidth::I64, base_offset, "16", assembly)?;
    write_assembly!(
        assembly,
        "\tldr {}, [x16, #{}]\n",
        aarch64_result_register(width),
        field.offset
    )?;
    emit_aarch64_store_temporary(width, value_offset, assembly)?;
    emit_aarch64_increment_result(width, assembly)?;
    emit_aarch64_load_temporary_to_register(ValueWidth::I64, base_offset, "16", assembly)?;
    write_assembly!(
        assembly,
        "\tstr {}, [x16, #{}]\n",
        aarch64_result_register(width),
        field.offset
    )?;
    emit_aarch64_load_temporary(width, value_offset, assembly)
}

fn emit_aarch64_increment_result(width: ValueWidth, assembly: &mut String) -> CompileResult<()> {
    match width {
        ValueWidth::I32 => {
            assembly.push_str("\tadd w0, w0, #1\n");
            Ok(())
        }
        ValueWidth::I64 => {
            assembly.push_str("\tadd x0, x0, #1\n");
            Ok(())
        }
        ValueWidth::F64 => Err(CompileError::new("unsupported double post-increment")),
    }
}

fn emit_aarch64_store_pointer_subscript(
    subscript: PointerSubscriptExpr<'_>,
    value: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let width = scalar_width(subscript.element_type);
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
    let base_offset = temporary_base + ((depth + 1) * TEMPORARY_BYTES);
    emit_aarch64_expr_with_width(value, width, temporary_base, depth, labels, assembly)?;
    emit_aarch64_store_temporary(width, value_offset, assembly)?;
    emit_aarch64_expr_with_width(
        subscript.pointer,
        ValueWidth::I64,
        temporary_base,
        depth + 2,
        labels,
        assembly,
    )?;
    emit_aarch64_store_temporary(ValueWidth::I64, base_offset, assembly)?;
    emit_aarch64_expr_with_width(
        subscript.index,
        ValueWidth::I32,
        temporary_base,
        depth + 2,
        labels,
        assembly,
    )?;
    assembly.push_str("\tmov w17, w0\n");
    emit_aarch64_load_temporary_to_register(ValueWidth::I64, base_offset, "16", assembly)?;
    emit_aarch64_load_temporary(width, value_offset, assembly)?;
    if subscript.element_byte_size == 1 && width == ValueWidth::I32 {
        return write_assembly!(assembly, "\tstrb w0, [x16, w17, sxtw]\n");
    }
    let Some(shift) = memory_scale_shift_for_byte_size(subscript.element_byte_size) else {
        return Err(CompileError::new(
            "unsupported pointer subscript element size",
        ));
    };
    write_assembly!(
        assembly,
        "\tstr {}, [x16, w17, sxtw #{}]\n",
        aarch64_result_register(width),
        shift
    )
}

fn emit_aarch64_store_global_byte_subscript(
    subscript: GlobalByteSubscriptExpr<'_>,
    value: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
    let label = label_name(subscript.name, labels.target);
    emit_aarch64_expr_with_width(
        value,
        ValueWidth::I32,
        temporary_base,
        depth,
        labels,
        assembly,
    )?;
    emit_aarch64_store_temporary(ValueWidth::I32, value_offset, assembly)?;
    emit_aarch64_expr_with_width(
        subscript.index,
        ValueWidth::I32,
        temporary_base,
        depth + 1,
        labels,
        assembly,
    )?;
    assembly.push_str("\tmov w17, w0\n");
    write_assembly!(assembly, "\tadrp x16, {label}@PAGE\n")?;
    write_assembly!(assembly, "\tadd x16, x16, {label}@PAGEOFF\n")?;
    emit_aarch64_load_temporary(ValueWidth::I32, value_offset, assembly)?;
    assembly.push_str("\tstrb w0, [x16, w17, sxtw]\n");
    Ok(())
}

fn emit_aarch64_store_global_int_subscript(
    subscript: GlobalByteSubscriptExpr<'_>,
    value: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
    let label = label_name(subscript.name, labels.target);
    emit_aarch64_expr_with_width(
        value,
        ValueWidth::I32,
        temporary_base,
        depth,
        labels,
        assembly,
    )?;
    emit_aarch64_store_temporary(ValueWidth::I32, value_offset, assembly)?;
    emit_aarch64_expr_with_width(
        subscript.index,
        ValueWidth::I32,
        temporary_base,
        depth + 1,
        labels,
        assembly,
    )?;
    assembly.push_str("\tmov w17, w0\n");
    write_assembly!(assembly, "\tadrp x16, {label}@PAGE\n")?;
    write_assembly!(assembly, "\tadd x16, x16, {label}@PAGEOFF\n")?;
    emit_aarch64_load_temporary(ValueWidth::I32, value_offset, assembly)?;
    assembly.push_str("\tstr w0, [x16, w17, sxtw #2]\n");
    Ok(())
}

fn emit_aarch64_store_global_pointer_subscript(
    subscript: GlobalByteSubscriptExpr<'_>,
    value: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
    let label = label_name(subscript.name, labels.target);
    emit_aarch64_expr_with_width(
        value,
        ValueWidth::I64,
        temporary_base,
        depth,
        labels,
        assembly,
    )?;
    emit_aarch64_store_temporary(ValueWidth::I64, value_offset, assembly)?;
    emit_aarch64_expr_with_width(
        subscript.index,
        ValueWidth::I32,
        temporary_base,
        depth + 1,
        labels,
        assembly,
    )?;
    assembly.push_str("\tmov w17, w0\n");
    write_assembly!(assembly, "\tadrp x16, {label}@PAGE\n")?;
    write_assembly!(assembly, "\tadd x16, x16, {label}@PAGEOFF\n")?;
    emit_aarch64_load_temporary_to_register(ValueWidth::I64, value_offset, "0", assembly)?;
    assembly.push_str("\tstr x0, [x16, w17, sxtw #3]\n");
    Ok(())
}

fn emit_aarch64_load_pointer_field(
    field: PointerFieldExpr<'_>,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let width = scalar_width(field.scalar_type);
    emit_aarch64_expr_with_width(
        field.pointer,
        ValueWidth::I64,
        temporary_base,
        depth + 1,
        labels,
        assembly,
    )?;
    write_assembly!(
        assembly,
        "\tldr {}, [x0, #{}]\n",
        aarch64_result_register(width),
        field.offset
    )
}

fn emit_aarch64_store_pointer_field(
    field: PointerFieldExpr<'_>,
    value: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let width = scalar_width(field.scalar_type);
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
    emit_aarch64_expr_with_width(value, width, temporary_base, depth, labels, assembly)?;
    emit_aarch64_store_temporary(width, value_offset, assembly)?;
    emit_aarch64_expr_with_width(
        field.pointer,
        ValueWidth::I64,
        temporary_base,
        depth + 1,
        labels,
        assembly,
    )?;
    assembly.push_str("\tmov x16, x0\n");
    emit_aarch64_load_temporary(width, value_offset, assembly)?;
    write_assembly!(
        assembly,
        "\tstr {}, [x16, #{}]\n",
        aarch64_result_register(width),
        field.offset
    )
}

fn emit_aarch64_unary_expr(
    op: UnaryOp,
    expr: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
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

const fn aarch64_register_prefix(width: ValueWidth) -> &'static str {
    match width {
        ValueWidth::I32 => "w",
        ValueWidth::I64 => "x",
        ValueWidth::F64 => "d",
    }
}

fn aarch64_parameter_register(index: usize, width: ValueWidth) -> String {
    let prefix = aarch64_register_prefix(width);
    format!("{prefix}{index}")
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
        expr @ (LoweredExpr::Call { .. } | LoweredExpr::IndirectCall { .. }) => {
            emit_x86_64_call_expr(expr, temporary_base, depth, target, labels, assembly)
        }
        LoweredExpr::Integer(value) => emit_x86_64_integer(*value, assembly),
        LoweredExpr::DoubleLiteral(value) => {
            emit_x86_64_load_double_literal(value, target, labels, assembly)
        }
        LoweredExpr::StringLiteral(value) => {
            emit_x86_64_load_string_address(value, target, labels, assembly)
        }
        LoweredExpr::LocalAddress { offset, byte_size } => write_assembly!(
            assembly,
            "\tleaq {}(%rbp), %rax\n",
            x86_stack_object_offset(*offset, *byte_size)
        ),
        LoweredExpr::GlobalAddress { name } => {
            let label = label_name(name, target);
            write_assembly!(assembly, "\tleaq {label}(%rip), %rax\n")
        }
        expr @ (LoweredExpr::PointerOffset { .. } | LoweredExpr::PointerFieldAddress { .. }) => {
            emit_x86_64_address_expr(expr, temporary_base, depth, target, labels, assembly)
        }
        expr @ (LoweredExpr::Global { .. }
        | LoweredExpr::GlobalByteSubscript { .. }
        | LoweredExpr::GlobalIntSubscript { .. }
        | LoweredExpr::GlobalPointerSubscript { .. }
        | LoweredExpr::PointerSubscript { .. }
        | LoweredExpr::PointerField { .. }
        | LoweredExpr::Assign { .. }
        | LoweredExpr::PostIncrement { .. }) => emit_x86_64_global_or_assignment_expr(
            expr,
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        ),
        LoweredExpr::Local {
            offset,
            scalar_type,
        } => emit_x86_64_load_temporary(scalar_width(*scalar_type), *offset, assembly),
        LoweredExpr::Unary { op, expr } => {
            emit_x86_64_unary_expr(*op, expr, temporary_base, depth, target, labels, assembly)
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

fn emit_x86_64_unary_expr(
    op: UnaryOp,
    expr: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
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

fn emit_x86_64_integer(value: i64, assembly: &mut String) -> CompileResult<()> {
    let value = i32_immediate(value)?;
    write_assembly!(assembly, "\tmovl ${value}, %eax\n")
}

fn emit_x86_64_global_or_assignment_expr(
    expr: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    match expr {
        LoweredExpr::Global { name, scalar_type } => {
            emit_x86_64_load_global(name, scalar_width(*scalar_type), target, assembly)
        }
        LoweredExpr::GlobalByteSubscript { name, index } => emit_x86_64_load_global_byte_subscript(
            name,
            index,
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        ),
        LoweredExpr::GlobalIntSubscript { name, index } => emit_x86_64_load_global_int_subscript(
            name,
            index,
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        ),
        LoweredExpr::GlobalPointerSubscript { name, index } => {
            emit_x86_64_load_global_pointer_subscript(
                name,
                index,
                temporary_base,
                depth,
                target,
                labels,
                assembly,
            )
        }
        LoweredExpr::PointerSubscript {
            pointer,
            index,
            element_type,
            element_byte_size,
        } => emit_x86_64_load_pointer_subscript(
            PointerSubscriptExpr {
                pointer,
                index,
                element_type: *element_type,
                element_byte_size: *element_byte_size,
            },
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        ),
        LoweredExpr::PointerField {
            pointer,
            offset,
            scalar_type,
        } => emit_x86_64_load_pointer_field(
            PointerFieldExpr {
                pointer,
                offset: *offset,
                scalar_type: *scalar_type,
            },
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        ),
        LoweredExpr::Assign {
            target: lvalue,
            value,
        } => emit_x86_64_assign(
            lvalue,
            value,
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        ),
        LoweredExpr::PostIncrement { target: lvalue } => {
            emit_x86_64_post_increment(lvalue, temporary_base, depth, target, labels, assembly)
        }
        _ => Err(CompileError::new(
            "internal error: expected x86-64 global expression",
        )),
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

fn emit_x86_64_call_expr(
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
        LoweredExpr::IndirectCall { callee, args } => emit_x86_64_indirect_call(
            callee,
            args,
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        ),
        _ => Err(CompileError::new(
            "internal error: expected x86_64 call expression",
        )),
    }
}

fn emit_x86_64_indirect_call(
    callee: &LoweredExpr,
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
    let callee_offset = temporary_base + ((depth + args.len()) * TEMPORARY_BYTES);
    let arg_depth = depth + args.len() + 1;
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
    emit_x86_64_expr_with_width(
        callee,
        ValueWidth::I64,
        temporary_base,
        arg_depth,
        target,
        labels,
        assembly,
    )?;
    emit_x86_64_store_temporary(ValueWidth::I64, callee_offset, assembly)?;
    for (index, arg) in args.iter().enumerate() {
        let offset = temporary_base + ((depth + index) * TEMPORARY_BYTES);
        let width = expr_width(arg);
        let register = x86_64_argument_register(index, width)?;
        emit_x86_64_load_temporary_to_register(width, offset, register, assembly)?;
    }
    emit_x86_64_load_temporary_to_register(ValueWidth::I64, callee_offset, "%rax", assembly)?;
    write_assembly!(assembly, "\tcall *%rax\n")
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

fn emit_x86_64_init_local_bytes(
    offset: usize,
    values: &[u8],
    assembly: &mut String,
) -> CompileResult<()> {
    for (index, value) in values.iter().enumerate() {
        let byte_offset = offset
            .checked_add(index)
            .ok_or_else(|| CompileError::new("local byte initializer offset overflow"))?;
        write_assembly!(
            assembly,
            "\tmovb ${value}, {}(%rbp)\n",
            x86_stack_byte_offset(offset, values.len(), byte_offset)
        )?;
    }
    Ok(())
}

fn emit_x86_64_init_local_ints(
    offset: usize,
    values: &[i32],
    assembly: &mut String,
) -> CompileResult<()> {
    let byte_size = values
        .len()
        .checked_mul(4)
        .ok_or_else(|| CompileError::new("local int initializer size overflow"))?;
    for (index, value) in values.iter().enumerate() {
        let byte_offset = offset
            .checked_add(
                index
                    .checked_mul(4)
                    .ok_or_else(|| CompileError::new("local int initializer offset overflow"))?,
            )
            .ok_or_else(|| CompileError::new("local int initializer offset overflow"))?;
        write_assembly!(
            assembly,
            "\tmovl ${value}, {}(%rbp)\n",
            x86_stack_byte_offset(offset, byte_size, byte_offset)
        )?;
    }
    Ok(())
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

fn emit_x86_64_load_global(
    name: &str,
    width: ValueWidth,
    target: Target,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = label_name(name, target);
    let suffix = x86_64_instruction_suffix(width);
    let register = x86_64_result_register(width);
    write_assembly!(assembly, "\tmov{suffix} {label}(%rip), {register}\n")
}

fn emit_x86_64_store_global(
    name: &str,
    width: ValueWidth,
    target: Target,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = label_name(name, target);
    let suffix = x86_64_instruction_suffix(width);
    let register = x86_64_result_register(width);
    write_assembly!(assembly, "\tmov{suffix} {register}, {label}(%rip)\n")
}

fn emit_x86_64_load_global_byte_subscript(
    name: &str,
    index: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = label_name(name, target);
    emit_x86_64_expr_with_width(
        index,
        ValueWidth::I32,
        temporary_base,
        depth,
        target,
        labels,
        assembly,
    )?;
    assembly.push_str("\tcltq\n");
    write_assembly!(assembly, "\tleaq {label}(%rip), %rcx\n")?;
    assembly.push_str("\tmovzbl (%rcx,%rax), %eax\n");
    Ok(())
}

fn emit_x86_64_load_global_int_subscript(
    name: &str,
    index: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = label_name(name, target);
    emit_x86_64_expr_with_width(
        index,
        ValueWidth::I32,
        temporary_base,
        depth,
        target,
        labels,
        assembly,
    )?;
    assembly.push_str("\tcltq\n");
    write_assembly!(assembly, "\tleaq {label}(%rip), %rcx\n")?;
    assembly.push_str("\tmovl (%rcx,%rax,4), %eax\n");
    Ok(())
}

fn emit_x86_64_load_global_pointer_subscript(
    name: &str,
    index: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let label = label_name(name, target);
    emit_x86_64_expr_with_width(
        index,
        ValueWidth::I32,
        temporary_base,
        depth,
        target,
        labels,
        assembly,
    )?;
    assembly.push_str("\tcltq\n");
    write_assembly!(assembly, "\tleaq {label}(%rip), %rcx\n")?;
    assembly.push_str("\tmovq (%rcx,%rax,8), %rax\n");
    Ok(())
}

fn emit_x86_64_load_pointer_subscript(
    subscript: PointerSubscriptExpr<'_>,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let base_offset = temporary_base + (depth * TEMPORARY_BYTES);
    let width = scalar_width(subscript.element_type);
    emit_x86_64_expr_with_width(
        subscript.pointer,
        ValueWidth::I64,
        temporary_base,
        depth + 1,
        target,
        labels,
        assembly,
    )?;
    emit_x86_64_store_temporary(ValueWidth::I64, base_offset, assembly)?;
    emit_x86_64_expr_with_width(
        subscript.index,
        ValueWidth::I32,
        temporary_base,
        depth + 1,
        target,
        labels,
        assembly,
    )?;
    assembly.push_str("\tcltq\n");
    emit_x86_64_load_temporary_to_register(ValueWidth::I64, base_offset, "%rcx", assembly)?;
    if subscript.element_byte_size == 1 && width == ValueWidth::I32 {
        return write_assembly!(assembly, "\tmovzbl (%rcx,%rax,1), %eax\n");
    }
    let Some(scale) = memory_scale_bytes_for_byte_size(subscript.element_byte_size) else {
        return Err(CompileError::new(
            "unsupported pointer subscript element size",
        ));
    };
    write_assembly!(
        assembly,
        "\tmov{} (%rcx,%rax,{}), {}\n",
        x86_64_instruction_suffix(width),
        scale,
        x86_64_result_register(width)
    )
}

fn emit_x86_64_address_expr(
    expr: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    match expr {
        LoweredExpr::PointerOffset {
            pointer,
            index,
            byte_size,
        } => emit_x86_64_pointer_offset(
            PointerOffsetExpr {
                pointer,
                index,
                byte_size: *byte_size,
            },
            temporary_base,
            depth,
            target,
            labels,
            assembly,
        ),
        LoweredExpr::PointerFieldAddress { pointer, offset } => {
            emit_x86_64_expr_with_width(
                pointer,
                ValueWidth::I64,
                temporary_base,
                depth + 1,
                target,
                labels,
                assembly,
            )?;
            write_assembly!(assembly, "\taddq ${offset}, %rax\n")
        }
        _ => Err(CompileError::new(
            "internal error: expected x86-64 address expression",
        )),
    }
}

fn emit_x86_64_pointer_offset(
    offset: PointerOffsetExpr<'_>,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let base_offset = temporary_base + (depth * TEMPORARY_BYTES);
    emit_x86_64_expr_with_width(
        offset.pointer,
        ValueWidth::I64,
        temporary_base,
        depth + 1,
        target,
        labels,
        assembly,
    )?;
    emit_x86_64_store_temporary(ValueWidth::I64, base_offset, assembly)?;
    emit_x86_64_expr_with_width(
        offset.index,
        ValueWidth::I32,
        temporary_base,
        depth + 1,
        target,
        labels,
        assembly,
    )?;
    assembly.push_str("\tcltq\n");
    emit_x86_64_load_temporary_to_register(ValueWidth::I64, base_offset, "%rcx", assembly)?;
    if let Some(scale) = memory_scale_bytes_for_byte_size(offset.byte_size) {
        write_assembly!(assembly, "\tleaq (%rcx,%rax,{scale}), %rax\n")?;
        return Ok(());
    }
    let byte_size = i32::try_from(offset.byte_size)
        .map_err(|_| CompileError::new("pointer offset size does not fit i32"))?;
    write_assembly!(assembly, "\timulq ${byte_size}, %rax\n")?;
    assembly.push_str("\taddq %rcx, %rax\n");
    Ok(())
}

fn emit_x86_64_load_pointer_field(
    field: PointerFieldExpr<'_>,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let width = scalar_width(field.scalar_type);
    emit_x86_64_expr_with_width(
        field.pointer,
        ValueWidth::I64,
        temporary_base,
        depth + 1,
        target,
        labels,
        assembly,
    )?;
    write_assembly!(
        assembly,
        "\tmov{} {}(%rax), {}\n",
        x86_64_instruction_suffix(width),
        field.offset,
        x86_64_result_register(width)
    )
}

fn emit_x86_64_assign(
    target: &LoweredLValue,
    value: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    codegen_target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let width = lowered_lvalue_width(target);
    match target {
        LoweredLValue::Local { offset, .. } => {
            emit_x86_64_expr_with_width(
                value,
                width,
                temporary_base,
                depth,
                codegen_target,
                labels,
                assembly,
            )?;
            emit_x86_64_store_result(width, *offset, assembly)
        }
        LoweredLValue::Global { name, .. } => {
            emit_x86_64_expr_with_width(
                value,
                width,
                temporary_base,
                depth,
                codegen_target,
                labels,
                assembly,
            )?;
            emit_x86_64_store_global(name, width, codegen_target, assembly)
        }
        LoweredLValue::GlobalByteSubscript { name, index } => {
            emit_x86_64_store_global_byte_subscript(
                GlobalByteSubscriptExpr { name, index },
                value,
                temporary_base,
                depth,
                codegen_target,
                labels,
                assembly,
            )
        }
        LoweredLValue::GlobalIntSubscript { name, index } => {
            emit_x86_64_store_global_int_subscript(
                GlobalByteSubscriptExpr { name, index },
                value,
                temporary_base,
                depth,
                codegen_target,
                labels,
                assembly,
            )
        }
        LoweredLValue::GlobalPointerSubscript { name, index } => {
            emit_x86_64_store_global_pointer_subscript(
                GlobalByteSubscriptExpr { name, index },
                value,
                temporary_base,
                depth,
                codegen_target,
                labels,
                assembly,
            )
        }
        LoweredLValue::PointerSubscript {
            pointer,
            index,
            element_type,
            element_byte_size,
        } => emit_x86_64_store_pointer_subscript(
            PointerSubscriptExpr {
                pointer,
                index,
                element_type: *element_type,
                element_byte_size: *element_byte_size,
            },
            value,
            temporary_base,
            depth,
            codegen_target,
            labels,
            assembly,
        ),
        LoweredLValue::PointerField {
            pointer,
            offset,
            scalar_type,
        } => emit_x86_64_store_pointer_field(
            PointerFieldExpr {
                pointer,
                offset: *offset,
                scalar_type: *scalar_type,
            },
            value,
            temporary_base,
            depth,
            codegen_target,
            labels,
            assembly,
        ),
    }
}

fn emit_x86_64_post_increment(
    target: &LoweredLValue,
    temporary_base: usize,
    depth: usize,
    codegen_target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let width = lowered_lvalue_width(target);
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
    match target {
        LoweredLValue::Local { offset, .. } => {
            emit_x86_64_load_temporary(width, *offset, assembly)?;
            emit_x86_64_store_temporary(width, value_offset, assembly)?;
            emit_x86_64_increment_result(width, assembly)?;
            emit_x86_64_store_result(width, *offset, assembly)?;
            emit_x86_64_load_temporary(width, value_offset, assembly)
        }
        LoweredLValue::Global { name, .. } => {
            emit_x86_64_load_global(name, width, codegen_target, assembly)?;
            emit_x86_64_store_temporary(width, value_offset, assembly)?;
            emit_x86_64_increment_result(width, assembly)?;
            emit_x86_64_store_global(name, width, codegen_target, assembly)?;
            emit_x86_64_load_temporary(width, value_offset, assembly)
        }
        LoweredLValue::PointerField {
            pointer,
            offset,
            scalar_type,
        } => emit_x86_64_post_increment_pointer_field(
            PointerFieldExpr {
                pointer,
                offset: *offset,
                scalar_type: *scalar_type,
            },
            temporary_base,
            depth,
            codegen_target,
            labels,
            assembly,
        ),
        LoweredLValue::GlobalByteSubscript { .. }
        | LoweredLValue::GlobalIntSubscript { .. }
        | LoweredLValue::GlobalPointerSubscript { .. }
        | LoweredLValue::PointerSubscript { .. } => Err(CompileError::new(
            "post-increment expression supports direct lvalues only",
        )),
    }
}

fn emit_x86_64_post_increment_pointer_field(
    field: PointerFieldExpr<'_>,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let width = scalar_width(field.scalar_type);
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
    let base_offset = temporary_base + ((depth + 1) * TEMPORARY_BYTES);
    emit_x86_64_expr_with_width(
        field.pointer,
        ValueWidth::I64,
        temporary_base,
        depth + 2,
        target,
        labels,
        assembly,
    )?;
    emit_x86_64_store_temporary(ValueWidth::I64, base_offset, assembly)?;
    emit_x86_64_load_temporary_to_register(ValueWidth::I64, base_offset, "%rcx", assembly)?;
    write_assembly!(
        assembly,
        "\tmov{} {}(%rcx), {}\n",
        x86_64_instruction_suffix(width),
        field.offset,
        x86_64_result_register(width)
    )?;
    emit_x86_64_store_temporary(width, value_offset, assembly)?;
    emit_x86_64_increment_result(width, assembly)?;
    emit_x86_64_load_temporary_to_register(ValueWidth::I64, base_offset, "%rcx", assembly)?;
    write_assembly!(
        assembly,
        "\tmov{} {}, {}(%rcx)\n",
        x86_64_instruction_suffix(width),
        x86_64_result_register(width),
        field.offset
    )?;
    emit_x86_64_load_temporary(width, value_offset, assembly)
}

fn emit_x86_64_increment_result(width: ValueWidth, assembly: &mut String) -> CompileResult<()> {
    match width {
        ValueWidth::I32 => {
            assembly.push_str("\taddl $1, %eax\n");
            Ok(())
        }
        ValueWidth::I64 => {
            assembly.push_str("\taddq $1, %rax\n");
            Ok(())
        }
        ValueWidth::F64 => Err(CompileError::new("unsupported double post-increment")),
    }
}

fn emit_x86_64_store_pointer_subscript(
    subscript: PointerSubscriptExpr<'_>,
    value: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let width = scalar_width(subscript.element_type);
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
    let base_offset = temporary_base + ((depth + 1) * TEMPORARY_BYTES);
    emit_x86_64_expr_with_width(
        value,
        width,
        temporary_base,
        depth,
        target,
        labels,
        assembly,
    )?;
    emit_x86_64_store_temporary(width, value_offset, assembly)?;
    emit_x86_64_expr_with_width(
        subscript.pointer,
        ValueWidth::I64,
        temporary_base,
        depth + 2,
        target,
        labels,
        assembly,
    )?;
    emit_x86_64_store_temporary(ValueWidth::I64, base_offset, assembly)?;
    emit_x86_64_expr_with_width(
        subscript.index,
        ValueWidth::I32,
        temporary_base,
        depth + 2,
        target,
        labels,
        assembly,
    )?;
    assembly.push_str("\tcltq\n");
    assembly.push_str("\tmovq %rax, %rdx\n");
    emit_x86_64_load_temporary_to_register(ValueWidth::I64, base_offset, "%rcx", assembly)?;
    emit_x86_64_load_temporary(width, value_offset, assembly)?;
    if subscript.element_byte_size == 1 && width == ValueWidth::I32 {
        return write_assembly!(assembly, "\tmovb %al, (%rcx,%rdx,1)\n");
    }
    let Some(scale) = memory_scale_bytes_for_byte_size(subscript.element_byte_size) else {
        return Err(CompileError::new(
            "unsupported pointer subscript element size",
        ));
    };
    write_assembly!(
        assembly,
        "\tmov{} {}, (%rcx,%rdx,{})\n",
        x86_64_instruction_suffix(width),
        x86_64_result_register(width),
        scale
    )
}

fn emit_x86_64_store_global_byte_subscript(
    subscript: GlobalByteSubscriptExpr<'_>,
    value: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
    let label = label_name(subscript.name, target);
    emit_x86_64_expr_with_width(
        value,
        ValueWidth::I32,
        temporary_base,
        depth,
        target,
        labels,
        assembly,
    )?;
    emit_x86_64_store_temporary(ValueWidth::I32, value_offset, assembly)?;
    emit_x86_64_expr_with_width(
        subscript.index,
        ValueWidth::I32,
        temporary_base,
        depth + 1,
        target,
        labels,
        assembly,
    )?;
    assembly.push_str("\tcltq\n");
    assembly.push_str("\tmovq %rax, %rdx\n");
    write_assembly!(assembly, "\tleaq {label}(%rip), %rcx\n")?;
    emit_x86_64_load_temporary(ValueWidth::I32, value_offset, assembly)?;
    assembly.push_str("\tmovb %al, (%rcx,%rdx)\n");
    Ok(())
}

fn emit_x86_64_store_global_int_subscript(
    subscript: GlobalByteSubscriptExpr<'_>,
    value: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
    let label = label_name(subscript.name, target);
    emit_x86_64_expr_with_width(
        value,
        ValueWidth::I32,
        temporary_base,
        depth,
        target,
        labels,
        assembly,
    )?;
    emit_x86_64_store_temporary(ValueWidth::I32, value_offset, assembly)?;
    emit_x86_64_expr_with_width(
        subscript.index,
        ValueWidth::I32,
        temporary_base,
        depth + 1,
        target,
        labels,
        assembly,
    )?;
    assembly.push_str("\tcltq\n");
    assembly.push_str("\tmovq %rax, %rdx\n");
    write_assembly!(assembly, "\tleaq {label}(%rip), %rcx\n")?;
    emit_x86_64_load_temporary(ValueWidth::I32, value_offset, assembly)?;
    assembly.push_str("\tmovl %eax, (%rcx,%rdx,4)\n");
    Ok(())
}

fn emit_x86_64_store_global_pointer_subscript(
    subscript: GlobalByteSubscriptExpr<'_>,
    value: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
    let label = label_name(subscript.name, target);
    emit_x86_64_expr_with_width(
        value,
        ValueWidth::I64,
        temporary_base,
        depth,
        target,
        labels,
        assembly,
    )?;
    emit_x86_64_store_temporary(ValueWidth::I64, value_offset, assembly)?;
    emit_x86_64_expr_with_width(
        subscript.index,
        ValueWidth::I32,
        temporary_base,
        depth + 1,
        target,
        labels,
        assembly,
    )?;
    assembly.push_str("\tcltq\n");
    assembly.push_str("\tmovq %rax, %rdx\n");
    write_assembly!(assembly, "\tleaq {label}(%rip), %rcx\n")?;
    emit_x86_64_load_temporary(ValueWidth::I64, value_offset, assembly)?;
    assembly.push_str("\tmovq %rax, (%rcx,%rdx,8)\n");
    Ok(())
}

fn emit_x86_64_store_pointer_field(
    field: PointerFieldExpr<'_>,
    value: &LoweredExpr,
    temporary_base: usize,
    depth: usize,
    target: Target,
    labels: &mut LabelAllocator<'_>,
    assembly: &mut String,
) -> CompileResult<()> {
    let width = scalar_width(field.scalar_type);
    let value_offset = temporary_base + (depth * TEMPORARY_BYTES);
    emit_x86_64_expr_with_width(
        value,
        width,
        temporary_base,
        depth,
        target,
        labels,
        assembly,
    )?;
    emit_x86_64_store_temporary(width, value_offset, assembly)?;
    emit_x86_64_expr_with_width(
        field.pointer,
        ValueWidth::I64,
        temporary_base,
        depth + 1,
        target,
        labels,
        assembly,
    )?;
    assembly.push_str("\tmovq %rax, %rcx\n");
    emit_x86_64_load_temporary(width, value_offset, assembly)?;
    write_assembly!(
        assembly,
        "\tmov{} {}, {}(%rcx)\n",
        x86_64_instruction_suffix(width),
        x86_64_result_register(width),
        field.offset
    )
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
    let value = i32_immediate(value)?;
    let bits = u32::from_ne_bytes(value.to_ne_bytes());
    let low = bits & 0xffff;
    let high = (bits >> 16) & 0xffff;
    write_assembly!(assembly, "\tmovz {register}, #{low}\n")?;
    if high != 0 {
        write_assembly!(assembly, "\tmovk {register}, #{high}, lsl #16\n")?;
    }
    Ok(())
}

fn i32_immediate(value: i64) -> CompileResult<i32> {
    if let Ok(value) = i32::try_from(value) {
        return Ok(value);
    }
    let value =
        u32::try_from(value).map_err(|_| CompileError::new("integer literal does not fit i32"))?;
    Ok(i32::from_ne_bytes(value.to_ne_bytes()))
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
        | Instruction::StoreGlobal { value, .. }
        | Instruction::Eval(value)
        | Instruction::Return(Some(value)) => expr_depth(value),
        Instruction::JumpIfZero { condition, .. } => expr_depth(condition),
        Instruction::Return(None)
        | Instruction::Jump { .. }
        | Instruction::Label { .. }
        | Instruction::InitLocalBytes { .. }
        | Instruction::InitLocalInts { .. } => 0,
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
        Instruction::StoreLocal { .. }
        | Instruction::StoreGlobal { .. }
        | Instruction::Eval(_)
        | Instruction::Return(_)
        | Instruction::InitLocalBytes { .. }
        | Instruction::InitLocalInts { .. } => None,
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
        | LoweredExpr::Global { .. }
        | LoweredExpr::GlobalAddress { .. }
        | LoweredExpr::Local { .. }
        | LoweredExpr::LocalAddress { .. } => 0,
        LoweredExpr::Call { args, .. } => call_arg_depth(args),
        LoweredExpr::IndirectCall { callee, args } => {
            1 + expr_depth(callee).max(call_arg_depth(args))
        }
        LoweredExpr::Cast { expr, .. } | LoweredExpr::Unary { expr, .. } => expr_depth(expr),
        LoweredExpr::GlobalByteSubscript { index, .. }
        | LoweredExpr::GlobalIntSubscript { index, .. }
        | LoweredExpr::GlobalPointerSubscript { index, .. } => expr_depth(index),
        LoweredExpr::PointerSubscript { pointer, index, .. }
        | LoweredExpr::PointerOffset { pointer, index, .. } => {
            pointer_lvalue_address_depth(pointer, index)
        }
        LoweredExpr::PointerFieldAddress { pointer, .. }
        | LoweredExpr::PointerField { pointer, .. } => 1 + expr_depth(pointer),
        LoweredExpr::Assign { target, value } => assign_expr_depth(target, value),
        LoweredExpr::PostIncrement { target } => 1 + lvalue_address_depth(target),
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

fn assign_expr_depth(target: &LoweredLValue, value: &LoweredExpr) -> usize {
    expr_depth(value)
        .max(1)
        .max(1 + lvalue_address_depth(target))
}

fn lvalue_address_depth(target: &LoweredLValue) -> usize {
    match target {
        LoweredLValue::Local { .. } | LoweredLValue::Global { .. } => 0,
        LoweredLValue::GlobalByteSubscript { index, .. }
        | LoweredLValue::GlobalIntSubscript { index, .. }
        | LoweredLValue::GlobalPointerSubscript { index, .. } => expr_depth(index),
        LoweredLValue::PointerSubscript { pointer, index, .. } => {
            pointer_lvalue_address_depth(pointer, index)
        }
        LoweredLValue::PointerField { pointer, .. } => 1 + expr_depth(pointer),
    }
}

fn pointer_lvalue_address_depth(pointer: &LoweredExpr, index: &LoweredExpr) -> usize {
    1 + expr_depth(pointer).max(expr_depth(index))
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
        | Instruction::StoreGlobal { value, .. }
        | Instruction::JumpIfZero {
            condition: value, ..
        }
        | Instruction::Eval(value)
        | Instruction::Return(Some(value)) => expr_needs_preserved_temp(value),
        Instruction::Return(None)
        | Instruction::Jump { .. }
        | Instruction::Label { .. }
        | Instruction::InitLocalBytes { .. }
        | Instruction::InitLocalInts { .. } => false,
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
        LoweredExpr::GlobalByteSubscript { index, .. }
        | LoweredExpr::GlobalIntSubscript { index, .. }
        | LoweredExpr::GlobalPointerSubscript { index, .. } => expr_needs_preserved_temp(index),
        LoweredExpr::PointerSubscript { pointer, index, .. }
        | LoweredExpr::PointerOffset { pointer, index, .. } => {
            expr_needs_preserved_temp(pointer) || expr_needs_preserved_temp(index)
        }
        LoweredExpr::PointerFieldAddress { pointer, .. }
        | LoweredExpr::PointerField { pointer, .. } => expr_needs_preserved_temp(pointer),
        LoweredExpr::Assign { target, value } => {
            lvalue_needs_preserved_temp(target) || expr_needs_preserved_temp(value)
        }
        LoweredExpr::PostIncrement { target } => lvalue_needs_preserved_temp(target),
        LoweredExpr::Call { args, .. } => args.iter().any(expr_needs_preserved_temp),
        LoweredExpr::IndirectCall { callee, args } => {
            expr_needs_preserved_temp(callee) || args.iter().any(expr_needs_preserved_temp)
        }
        LoweredExpr::Integer(_)
        | LoweredExpr::DoubleLiteral(_)
        | LoweredExpr::StringLiteral(_)
        | LoweredExpr::Global { .. }
        | LoweredExpr::GlobalAddress { .. }
        | LoweredExpr::Local { .. }
        | LoweredExpr::LocalAddress { .. } => false,
    }
}

fn lvalue_needs_preserved_temp(target: &LoweredLValue) -> bool {
    match target {
        LoweredLValue::Local { .. } | LoweredLValue::Global { .. } => false,
        LoweredLValue::GlobalByteSubscript { index, .. }
        | LoweredLValue::GlobalIntSubscript { index, .. }
        | LoweredLValue::GlobalPointerSubscript { index, .. } => expr_needs_preserved_temp(index),
        LoweredLValue::PointerSubscript { pointer, index, .. } => {
            expr_needs_preserved_temp(pointer) || expr_needs_preserved_temp(index)
        }
        LoweredLValue::PointerField { pointer, .. } => expr_needs_preserved_temp(pointer),
    }
}

const fn expr_is_direct_call(expr: &LoweredExpr) -> bool {
    matches!(
        expr,
        LoweredExpr::Call { .. } | LoweredExpr::IndirectCall { .. }
    )
}

fn instruction_uses_call(instruction: &Instruction) -> bool {
    match instruction {
        Instruction::StoreLocal { value, .. }
        | Instruction::StoreGlobal { value, .. }
        | Instruction::JumpIfZero {
            condition: value, ..
        }
        | Instruction::Eval(value)
        | Instruction::Return(Some(value)) => expr_uses_call(value),
        Instruction::Return(None)
        | Instruction::Jump { .. }
        | Instruction::Label { .. }
        | Instruction::InitLocalBytes { .. }
        | Instruction::InitLocalInts { .. } => false,
    }
}

fn expr_uses_call(expr: &LoweredExpr) -> bool {
    match expr {
        LoweredExpr::Call { .. } | LoweredExpr::IndirectCall { .. } => true,
        LoweredExpr::Integer(_)
        | LoweredExpr::DoubleLiteral(_)
        | LoweredExpr::StringLiteral(_)
        | LoweredExpr::Global { .. }
        | LoweredExpr::GlobalAddress { .. }
        | LoweredExpr::Local { .. }
        | LoweredExpr::LocalAddress { .. } => false,
        LoweredExpr::Cast { expr, .. } | LoweredExpr::Unary { expr, .. } => expr_uses_call(expr),
        LoweredExpr::GlobalByteSubscript { index, .. }
        | LoweredExpr::GlobalIntSubscript { index, .. }
        | LoweredExpr::GlobalPointerSubscript { index, .. } => expr_uses_call(index),
        LoweredExpr::PointerSubscript { pointer, index, .. }
        | LoweredExpr::PointerOffset { pointer, index, .. } => {
            expr_uses_call(pointer) || expr_uses_call(index)
        }
        LoweredExpr::PointerFieldAddress { pointer, .. }
        | LoweredExpr::PointerField { pointer, .. } => expr_uses_call(pointer),
        LoweredExpr::Assign { target, value } => lvalue_uses_call(target) || expr_uses_call(value),
        LoweredExpr::PostIncrement { target } => lvalue_uses_call(target),
        LoweredExpr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => expr_uses_call(condition) || expr_uses_call(then_expr) || expr_uses_call(else_expr),
        LoweredExpr::Binary { left, right, .. } => expr_uses_call(left) || expr_uses_call(right),
    }
}

fn lvalue_uses_call(target: &LoweredLValue) -> bool {
    match target {
        LoweredLValue::Local { .. } | LoweredLValue::Global { .. } => false,
        LoweredLValue::GlobalByteSubscript { index, .. }
        | LoweredLValue::GlobalIntSubscript { index, .. }
        | LoweredLValue::GlobalPointerSubscript { index, .. } => expr_uses_call(index),
        LoweredLValue::PointerSubscript { pointer, index, .. } => {
            expr_uses_call(pointer) || expr_uses_call(index)
        }
        LoweredLValue::PointerField { pointer, .. } => expr_uses_call(pointer),
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

fn x86_stack_object_offset(byte_offset: usize, byte_size: usize) -> String {
    format!("-{}", byte_offset + byte_size)
}

fn x86_stack_byte_offset(object_offset: usize, object_size: usize, byte_offset: usize) -> String {
    let index = byte_offset - object_offset;
    format!("-{}", object_offset + object_size - index)
}

fn local_stack_bytes(function: &LoweredFunction) -> usize {
    function
        .local_slots
        .iter()
        .map(|local_slot| local_slot.offset + local_slot.byte_size)
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
    emit_string_literal_data_returning_to(label, value, target, ".text\n", assembly)
}

fn emit_string_literal_data_returning_to(
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

const fn memory_scale_shift_for_byte_size(byte_size: usize) -> Option<u8> {
    match byte_size {
        1 => Some(0),
        2 => Some(1),
        4 => Some(2),
        8 => Some(3),
        _ => None,
    }
}

const fn memory_scale_bytes_for_byte_size(byte_size: usize) -> Option<u8> {
    match byte_size {
        1 => Some(1),
        2 => Some(2),
        4 => Some(4),
        8 => Some(8),
        _ => None,
    }
}

fn label_name(name: &str, target: Target) -> String {
    match target {
        Target::Aarch64AppleDarwin | Target::X86_64AppleDarwin => format!("_{name}"),
        Target::X86_64UnknownLinuxGnu => name.to_string(),
    }
}

fn global_string_label(name: &str, index: usize, target: Target) -> String {
    match target {
        Target::Aarch64AppleDarwin | Target::X86_64AppleDarwin => {
            format!("L{name}_str{index}")
        }
        Target::X86_64UnknownLinuxGnu => format!(".L{name}_str{index}"),
    }
}

fn branch_label(function: &str, label: usize, target: Target) -> String {
    match target {
        Target::Aarch64AppleDarwin | Target::X86_64AppleDarwin => format!("L{function}_{label}"),
        Target::X86_64UnknownLinuxGnu => format!(".L{function}_{label}"),
    }
}
