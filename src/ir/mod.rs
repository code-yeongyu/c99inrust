const POINTER_REFERENT: &str = "*";
const VARIADIC_GP_SAVE_BYTES: usize = 64;

mod bindings;
mod builtin_calls;
mod call_args;
mod complex_arithmetic_store;
mod complex_scalar_arithmetic_expr;
mod complex_scalar_expr;
mod complex_scalar_parts;
mod complex_scalar_real;
mod complex_scalar_store;
mod compound_literal_storage;
mod compound_literals;
mod compound_struct_literals;
mod const_eval;
mod constant_inlining;
mod constant_tables;
mod context;
mod context_address;
mod context_array_resolution;
mod context_assignment;
mod context_calls;
mod context_control_flow;
mod context_declarations;
mod context_expr_core;
mod context_expr_ops;
mod context_identifiers;
mod context_int_matrices;
mod context_labels;
mod context_members;
mod context_post_increment;
mod context_slots;
mod context_statements;
mod context_store;
mod context_struct_address;
mod context_subscript_expr;
mod context_subscript_lvalue;
mod doom_alloc;
mod function_lowering;
mod global_bindings;
mod global_initializers;
mod global_lowering;
mod local_array;
mod local_initializer_values;
mod local_struct_initializer;
mod local_struct_initializer_array;
mod local_struct_initializer_struct_array;
mod lowered_expr_helpers;
mod pointer_arithmetic;
mod pointer_referent;
mod program_lowering;
mod scalar_global_initializers;
mod sizeof_expr;
mod static_local;
mod struct_initializer;
mod types;

pub use const_eval::const_eval;
pub use function_lowering::lower_function;
pub use program_lowering::lower;
pub use types::{
    Instruction, LocalSlot, LoweredExpr, LoweredFunction, LoweredGlobal, LoweredGlobalInitializer,
    LoweredLValue, LoweredProgram, LoweredStructInitializerScalar, LoweredStructInitializerValue,
};

pub(in crate::ir) use bindings::{
    ArrayFieldSubscript, GlobalBinding, LocalBinding, NestedArrayFieldSubscript, ResolvedMember,
    StructAddress,
};
pub(in crate::ir) use complex_scalar_arithmetic_expr::{
    ComplexBinaryLanes, complex_arithmetic_lane_expr,
};
pub(in crate::ir) use complex_scalar_expr::{
    complex_equality_expr, complex_expr_scalar_type, complex_lane_value_expr, complex_truth_expr,
};
pub(in crate::ir) use complex_scalar_parts::{
    complex_binary_operands, complex_indirect_target, complex_lane_byte_size, complex_lane_expr,
    complex_object_pointer, complex_unary_operand, is_complex_scalar,
};
pub(in crate::ir) use complex_scalar_real::{real_scalar_expr_type, real_scalar_lane_expr};
pub(in crate::ir) use const_eval::{cast_const_value, eval_binary};
pub(in crate::ir) use constant_inlining::{constant_return_functions, inline_constant_calls};
pub(in crate::ir) use constant_tables::{
    lower_constants, lower_function_names, lower_pointer_return_functions,
};
pub(in crate::ir) use context::LoweringContext;
pub(in crate::ir) use function_lowering::lower_function_with_globals;
pub(in crate::ir) use global_bindings::{insert_builtin_libc_bindings, insert_global_binding};
pub(in crate::ir) use global_initializers::lower_defined_global_initializer;
pub(in crate::ir) use global_lowering::{lower_extern_global_binding, lower_globals};
pub(in crate::ir) use local_initializer_values::{
    align_to, local_char_array_initializer_values, local_char_matrix_byte_size,
    local_char_matrix_initializer_values, local_int_array_byte_size, local_int_matrix_byte_size,
    local_pointer_array_byte_size, local_short_array_byte_size, scalar_size, struct_alignment,
    zero_expr_for,
};
pub(in crate::ir) use local_struct_initializer_array::LocalStructArrayField;
pub(in crate::ir) use lowered_expr_helpers::{
    ensure_post_increment_scalar, lowered_expr_scalar_type, lowered_lvalue_scalar_type,
    lowered_lvalue_to_expr, pointer_field_address,
};
pub(in crate::ir) use scalar_global_initializers::lower_scalar_global_initializer;
