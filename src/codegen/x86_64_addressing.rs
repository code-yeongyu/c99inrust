use super::frames::LabelAllocator;
use super::sized_fields::emit_x86_64_load as emit_x86_64_load_sized_field;
use super::stack_helpers::memory_scale_bytes_for_byte_size;
use super::target::Target;
use super::widths::{
    PointerFieldExpr, PointerOffsetExpr, TEMPORARY_BYTES, ValueWidth, scalar_width,
};
use super::x86_64_expr::emit_x86_64_expr_with_width;
use super::x86_64_temporaries::{
    emit_x86_64_load_temporary_to_register, emit_x86_64_store_temporary,
};
use crate::diagnostics::{CompileError, CompileResult};
use crate::ir::LoweredExpr;

pub(in crate::codegen) fn emit_x86_64_address_expr(
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

pub(in crate::codegen) fn emit_x86_64_pointer_offset(
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

pub(in crate::codegen) fn emit_x86_64_load_pointer_field(
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
    emit_x86_64_load_sized_field(
        field.byte_size,
        width,
        field.is_unsigned,
        "%rax",
        field.offset,
        assembly,
    )
}
pub(in crate::codegen) fn x86_64_argument_register(
    index: usize,
    width: ValueWidth,
) -> CompileResult<&'static str> {
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

pub(in crate::codegen) const fn x86_64_stack_argument_scratch_register(
    width: ValueWidth,
) -> &'static str {
    match width {
        ValueWidth::I32 => "%r10d",
        ValueWidth::I64 => "%r10",
        ValueWidth::F64 => "%xmm8",
    }
}

pub(in crate::codegen) const fn x86_64_instruction_suffix(width: ValueWidth) -> &'static str {
    match width {
        ValueWidth::I32 => "l",
        ValueWidth::I64 => "q",
        ValueWidth::F64 => "sd",
    }
}

pub(in crate::codegen) const fn x86_64_result_register(width: ValueWidth) -> &'static str {
    match width {
        ValueWidth::I32 => "%eax",
        ValueWidth::I64 => "%rax",
        ValueWidth::F64 => "%xmm0",
    }
}
