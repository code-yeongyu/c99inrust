use super::multifile_support::{
    OracleMultiFileCase, OracleSourceFile, assert_multifile_compile_run_matches_host,
};
use super::support::{OracleCase, assert_compile_run_matches_host};

#[test]
fn nested_struct_union_array_stride_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "nested_struct_union_array_stride",
        source: "typedef union { char c[3]; int i; } u_t; typedef struct { char tag; u_t u; short tail; } s_t; typedef struct { char lead; s_t items[2]; long long wide; } box_t; int main(void) { return sizeof(u_t) == 4 && sizeof(s_t) == 12 && sizeof(box_t) == 40 && sizeof(s_t[2]) == 24 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn function_pointer_array_two_arg_calls_match_host_exit_code() {
    // given
    let case = OracleCase {
        name: "function_pointer_array_two_arg_calls",
        source: "int add(int a, int b) { return a + b; } int mul(int a, int b) { return a * b; } int main(void) { int (*ops[2])(int, int) = { add, mul }; int pick = 1; return (*ops[pick])(3, 4) == 12 && ops[0](3, 4) == 7 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn kr_style_parameter_declarations_out_of_order_match_host_exit_code() {
    // given
    let case = OracleCase {
        name: "kr_style_parameter_declarations_out_of_order",
        source: "int weighted(a, b, c) int c; int a; int b; { return a * 100 + b * 10 + c; } int main(void) { return weighted(2, 3, 4) == 234 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn anonymous_struct_union_member_array_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "anonymous_struct_union_member_array",
        source: "typedef struct { union { struct { int x; int y; }; int pair[2]; }; } vec_t; vec_t values[2]; int main(void) { values[1].x = 4; values[1].y = 5; return values[1].pair[0] + values[1].pair[1] == 9 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn typeof_conditional_with_attribute_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "typeof_conditional_with_attribute",
        source: "int main(void) { __typeof__(1 ? 2 : 3) value __attribute__((unused, aligned(4))) = 7; return value == 7 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn multi_file_extern_long_long_global_matches_host_exit_code() {
    // given
    let case = OracleMultiFileCase {
        name: "multi_file_extern_long_long_global",
        files: &[
            OracleSourceFile {
                path: "state.c",
                source: "long long shared64 = 0x100000000LL; int read_high(void) { return (int)(shared64 >> 32); }\n",
            },
            OracleSourceFile {
                path: "main.c",
                source: "extern long long shared64; int read_high(void); int main(void) { return read_high() == 1 && (int)shared64 == 0 ? 0 : 1; }\n",
            },
        ],
    };

    // when/then
    assert_multifile_compile_run_matches_host(case);
}

#[test]
fn static_inline_mutates_pointer_argument_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "static_inline_mutates_pointer_argument",
        source: "static inline void add_in_place(int *value, int delta) { *value = *value + delta; } int main(void) { int item = 5; add_in_place(&item, 2); return item == 7 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn restrict_const_source_array_update_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "restrict_const_source_array_update",
        source: "void add_all(int n, int * restrict dst, const int * restrict src) { int i; for (i = 0; i < n; i++) dst[i] = dst[i] + src[i]; } int main(void) { int dst[3] = { 1, 2, 3 }; int src[3] = { 4, 5, 6 }; add_all(3, dst, src); return dst[0] == 5 && dst[1] == 7 && dst[2] == 9 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn constexpr_sizeof_and_ternary_array_size_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "constexpr_sizeof_and_ternary_array_size",
        source: "enum { BYTES = sizeof(int) * 3, COUNT = BYTES > 8 ? BYTES - 4 : 1 }; int main(void) { int values[COUNT]; return sizeof(values) == 8 * sizeof(int) ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn nested_struct_array_initializer_braces_match_host_exit_code() {
    // given
    let case = OracleCase {
        name: "nested_struct_array_initializer_braces",
        source: "typedef struct { int x; int y; } pair_t; pair_t pairs[2] = { { 1, 2 }, { 3, 4 } }; int main(void) { return pairs[0].y == 2 && pairs[1].x == 3 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}
