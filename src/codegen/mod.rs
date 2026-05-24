use std::fmt::{self, Write as _};

use crate::diagnostics::{CompileError, CompileResult};

macro_rules! write_assembly {
    ($assembly:expr, $($argument:tt)*) => {
        $crate::codegen::write_assembly($assembly, format_args!($($argument)*))
    };
}

mod aarch64_addressing;
mod aarch64_analysis;
mod aarch64_assign;
mod aarch64_binary;
mod aarch64_calls;
mod aarch64_complex_abi;
mod aarch64_conditionals;
mod aarch64_control;
mod aarch64_expr;
mod aarch64_function;
mod aarch64_function_params;
mod aarch64_global_stores;
mod aarch64_loads;
mod aarch64_logical;
mod aarch64_memory_expr;
mod aarch64_pointer_subscript;
mod aarch64_post_increment;
mod aarch64_temporaries;
mod aarch64_unary;
mod aarch64_variadic;
mod call_usage;
mod complex_abi;
mod data_literals;
mod emit_program;
mod frames;
mod global_pointer_arrays;
mod global_real_scalars;
mod globals;
mod pointer_cast;
mod sized_fields;
mod stack_helpers;
mod struct_globals;
mod target;
mod widths;
mod x86_64_addressing;
mod x86_64_assign;
mod x86_64_binary;
mod x86_64_builtin_calls;
mod x86_64_calls;
mod x86_64_complex_abi;
mod x86_64_complex_expr_args;
mod x86_64_complex_params;
mod x86_64_conditionals;
mod x86_64_expr;
mod x86_64_expr_special;
mod x86_64_function;
mod x86_64_global_pointer_stores;
mod x86_64_loads;
mod x86_64_pointer_stores;
mod x86_64_post_increment;
mod x86_64_temporaries;
mod x86_64_unary;
mod x86_64_variadic;

pub use emit_program::emit_assembly;
pub use target::Target;

pub(in crate::codegen) fn write_assembly(
    assembly: &mut String,
    arguments: fmt::Arguments<'_>,
) -> CompileResult<()> {
    assembly
        .write_fmt(arguments)
        .map_err(|_| CompileError::new("failed to format assembly"))
}
