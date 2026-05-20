use super::multifile_support::{
    OracleMultiFileCase, OracleSourceFile, assert_multifile_compile_run_matches_host,
};
use super::support::{OracleCase, assert_compile_run_matches_host};

#[test]
fn nested_struct_padding_with_array_member_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "nested_struct_padding_with_array_member",
        source: "typedef struct { char tag; long long wide; } inner_t; typedef struct { char head; inner_t items[2]; int tail; } outer_t; int main(void) { return sizeof(inner_t) == 16 && sizeof(outer_t) == 48 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn function_pointer_typedef_array_cursor_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "function_pointer_typedef_array_cursor",
        source: "typedef int (*op_t)(int); int add1(int x) { return x + 1; } int add2(int x) { return x + 2; } int add3(int x) { return x + 3; } int main(void) { op_t ops[3]; ops[0] = add1; ops[1] = add2; ops[2] = add3; return ops[0](4) == 5 && ops[2](4) == 7 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn kr_style_promoted_char_and_short_parameters_match_host_exit_code() {
    // given
    let case = OracleCase {
        name: "kr_style_promoted_char_and_short_parameters",
        source: "int mix(a, b, c) char a; short b; int c; { return a + b + c; } int main(void) { return mix(1, 2, 3) == 6 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn nested_anonymous_struct_inside_union_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "nested_anonymous_struct_inside_union",
        source: "typedef struct { union { struct { int x; int y; }; int pair[2]; }; int z; } vec_t; int main(void) { vec_t value; value.x = 2; value.y = 3; value.z = 4; return value.pair[0] + value.pair[1] + value.z == 9 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn typeof_array_element_with_local_attribute_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "typeof_array_element_with_local_attribute",
        source: "int main(void) { int values[3] = { 1, 2, 3 }; __typeof__(values[0]) item __attribute__((unused)) = values[2]; return item == 3 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn multi_file_extern_mutated_global_matches_host_exit_code() {
    // given
    let case = OracleMultiFileCase {
        name: "multi_file_extern_mutated_global",
        files: &[
            OracleSourceFile {
                path: "state.c",
                source: "int shared = 3; void set_shared(int value) { shared = value; }\n",
            },
            OracleSourceFile {
                path: "main.c",
                source: "extern int shared; void set_shared(int value); int main(void) { set_shared(7); return shared == 7 ? 0 : 1; }\n",
            },
        ],
    };

    // when/then
    assert_multifile_compile_run_matches_host(case);
}

#[test]
fn static_inline_reads_static_file_scope_value_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "static_inline_reads_static_file_scope_value",
        source: "static int bias = 3; static inline int add_bias(int value) { return value + bias; } int main(void) { return add_bias(4) == 7 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn restrict_pointer_stride_update_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "restrict_pointer_stride_update",
        source: "void bump_even(int n, int * restrict values) { int i; for (i = 0; i < n; i += 2) values[i] = values[i] + 5; } int main(void) { int values[4] = { 1, 2, 3, 4 }; bump_even(4, values); return values[0] == 6 && values[1] == 2 && values[2] == 8 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn conditional_bitwise_constant_expression_array_size_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "conditional_bitwise_constant_expression_array_size",
        source: "enum { WIDTH = (1 << 3), EXTRA = (WIDTH & 3) ? 1 : 5 }; int main(void) { int values[WIDTH + EXTRA - 4]; return sizeof(values) == 9 * sizeof(int) ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn long_recursive_macro_expansion_chain_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "long_recursive_macro_expansion_chain",
        source: "#define M0 11\n#define M1 M0\n#define M2 M1\n#define M3 M2\n#define M4 M3\n#define M5 M4\n#define M6 M5\n#define M7 M6\n#define M8 M7\n#define M9 M8\n#define M10 M9\n#define M11 M10\n#define M12 M11\n#define M13 M12\n#define M14 M13\n#define M15 M14\n#define M16 M15\n#define M17 M16\n#define M18 M17\n#define M19 M18\n#define M20 M19\n#define M21 M20\n#define M22 M21\n#define M23 M22\n#define M24 M23\n#define M25 M24\n#define M26 M25\n#define M27 M26\n#define M28 M27\n#define M29 M28\n#define M30 M29\n#define M31 M30\n#define M32 M31\nint main(void) { return M32 == 11 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn variadic_macro_forwards_three_arguments_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "variadic_macro_forwards_three_arguments",
        source: "#define CALL(fn, ...) fn(__VA_ARGS__)\nint sum3(int a, int b, int c) { return a + b + c; } int main(void) { return CALL(sum3, 2, 3, 4) == 9 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn predefined_macros_inside_helper_match_host_exit_code() {
    // given
    let case = OracleCase {
        name: "predefined_macros_inside_helper",
        source: "int helper(void) { int line = __LINE__; char *file = __FILE__; return line > 0 && file[0] != 0 && __func__[0] == 'h'; } int main(void) { return helper() ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn nested_matrix_initializer_braces_match_host_exit_code() {
    // given
    let case = OracleCase {
        name: "nested_matrix_initializer_braces",
        source: "int matrix[2][2] = { { 1, 2 }, { 3, 4 } }; int main(void) { return matrix[0][1] == 2 && matrix[1][0] == 3 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn ternary_post_increment_side_effect_branch_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "ternary_post_increment_side_effect_branch",
        source: "int main(void) { int flag = 0; int x = 1; int y = flag ? ++x : x++; return x == 2 && y == 1 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn sizeof_parenthesized_type_and_expression_match_host_exit_code() {
    // given
    let case = OracleCase {
        name: "sizeof_parenthesized_type_and_expression",
        source: "int main(void) { int values[3]; int *p = values; return sizeof(values) == 3 * sizeof(int) && sizeof(p + 1) == sizeof(int*) ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn incompatible_struct_pointer_cast_member_store_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "incompatible_struct_pointer_cast_member_store",
        source: "typedef struct { int value; } left_t; typedef struct { int value; } right_t; int main(void) { left_t left; right_t *right = (right_t*)&left; right->value = 7; return left.value == 7 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn char_to_unsigned_char_conversion_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "char_to_unsigned_char_conversion",
        source: "int main(void) { char c[1]; unsigned char u[1]; c[0] = -1; u[0] = c[0]; return c[0] < 0 && u[0] == 255 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn long_long_multiply_divide_roundtrip_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "long_long_multiply_divide_roundtrip",
        source: "int main(void) { long long base = 123456789LL; long long product = base * 9LL; return (int)(product / 9LL) == 123456789 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn floating_point_fractional_truncation_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "floating_point_fractional_truncation",
        source: "int main(void) { double a = 12.50; double b = .25; return (int)(a + b) == 12 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn hex_float_fractional_exponent_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "hex_float_fractional_exponent",
        source: "int main(void) { double value = 0x1.4p+3; return (int)value == 10 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}
