use super::aarch64_analysis::{instruction_depth, next_available_label};
use super::call_usage::{function_uses_aarch64_preserved_temp, function_uses_call};
use super::data_literals::branch_label;
use super::stack_helpers::{align_to, local_stack_bytes};
use super::target::Target;
use super::widths::TEMPORARY_BYTES;
use crate::ir::LoweredFunction;

pub(in crate::codegen) struct LabelAllocator<'a> {
    pub(in crate::codegen) function: &'a str,
    pub(in crate::codegen) target: Target,
    pub(in crate::codegen) next_label: usize,
    pub(in crate::codegen) x86_64_variadic: Option<X86_64VariadicFrame>,
    pub(in crate::codegen) aarch64_variadic: Option<Aarch64VariadicFrame>,
}

#[derive(Clone, Copy)]
pub(in crate::codegen) struct X86_64VariadicFrame {
    pub(in crate::codegen) gp_offset: usize,
    pub(in crate::codegen) overflow_arg_offset: usize,
    pub(in crate::codegen) register_save_offset: usize,
    pub(in crate::codegen) register_save_size: usize,
}

impl<'a> LabelAllocator<'a> {
    pub(in crate::codegen) fn new(function: &'a LoweredFunction, target: Target) -> Self {
        Self {
            function: &function.name,
            target,
            next_label: next_available_label(function),
            x86_64_variadic: None,
            aarch64_variadic: None,
        }
    }

    pub(in crate::codegen) fn fresh(&mut self) -> String {
        let label = self.next_label;
        self.next_label += 1;
        branch_label(self.function, label, self.target)
    }
}
pub(in crate::codegen) struct Aarch64Frame {
    pub(in crate::codegen) temporary_base: usize,
    pub(in crate::codegen) stack_bytes: usize,
    pub(in crate::codegen) link_register_offset: Option<usize>,
    pub(in crate::codegen) preserved_temp_offset: Option<usize>,
}

#[derive(Clone, Copy)]
pub(in crate::codegen) struct Aarch64VariadicFrame {
    pub(in crate::codegen) gp_offset: usize,
    pub(in crate::codegen) overflow_arg_offset: usize,
    pub(in crate::codegen) register_save_offset: usize,
    pub(in crate::codegen) register_save_size: usize,
}

impl Aarch64Frame {
    pub(in crate::codegen) fn new(function: &LoweredFunction) -> Self {
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
pub(in crate::codegen) struct Aarch64Epilogue<'a> {
    pub(in crate::codegen) preserved_temp_offset: Option<usize>,
    pub(in crate::codegen) link_register_offset: Option<usize>,
    pub(in crate::codegen) stack_bytes: usize,
    pub(in crate::codegen) shared_label: Option<&'a str>,
}
