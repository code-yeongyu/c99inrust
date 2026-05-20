use super::multifile_support::{
    OracleMultiFileCase, OracleSourceFile, assert_multifile_compile_run_matches_host,
};
use super::support::{OracleCase, assert_compile_run_matches_host};

#[test]
fn nested_struct_padding_alignment_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "nested_struct_padding_alignment",
        source: "typedef struct { char c; int i; } inner_t; typedef struct { char a; inner_t b; short s; } outer_t; int main(void) { return sizeof(inner_t) == 8 && sizeof(outer_t) == 16 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn function_pointer_array_indirect_calls_match_host_exit_code() {
    // given
    let case = OracleCase {
        name: "function_pointer_array_indirect_calls",
        source: "int inc(int x) { return x + 1; } int dec(int x) { return x - 1; } int main(void) { int (*ops[2])(int) = { inc, dec }; return ops[0](4) == 5 && ops[1](4) == 3 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn kr_style_function_definition_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "kr_style_function_definition",
        source: "int add(a, b) int a; int b; { return a + b; } int main(void) { return add(2, 3) == 5 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn anonymous_union_local_member_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "anonymous_union_local_member",
        source: "int main(void) { union { char s[9]; int x[2]; } name; name.s[0] = 'A'; return name.s[0] == 'A' ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn typeof_and_attribute_declaration_match_host_exit_code() {
    // given
    let case = OracleCase {
        name: "typeof_and_attribute_declaration",
        source: "int tagged(int x) __attribute__((unused)); int tagged(int x) { __typeof__(x) y = x + 2; return y; } int main(void) { return tagged(5) == 7 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn multi_file_extern_linkage_matches_host_exit_code() {
    // given
    let case = OracleMultiFileCase {
        name: "multi_file_extern_linkage",
        files: &[
            OracleSourceFile {
                path: "helper.c",
                source: "int shared = 4; int bump(int value) { return value + shared; }\n",
            },
            OracleSourceFile {
                path: "main.c",
                source: "extern int shared; int bump(int value); int main(void) { return bump(3) == 7 && shared == 4 ? 0 : 1; }\n",
            },
        ],
    };

    // when/then
    assert_multifile_compile_run_matches_host(case);
}

#[test]
fn static_inline_function_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "static_inline_function",
        source: "static inline int add2(int value) { return value + 2; } int main(void) { return add2(5) == 7 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn restrict_pointer_qualifier_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "restrict_pointer_qualifier",
        source: "void add(int * restrict left, int * restrict right) { *left = *left + *right; } int main(void) { int a = 3; int b = 4; add(&a, &b); return a == 7 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn complex_constant_expression_array_size_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "complex_constant_expression_array_size",
        source: "enum { A = 2, B = 3 }; int main(void) { int values[(A + B) * 2 - 4]; return sizeof(values) == 6 * sizeof(int) ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn recursive_macro_expansion_chain_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "recursive_macro_expansion_chain",
        source: "#define A0 7\n#define A1 A0\n#define A2 A1\n#define A3 A2\n#define A4 A3\n#define A5 A4\n#define A6 A5\n#define A7 A6\n#define A8 A7\n#define A9 A8\n#define A10 A9\n#define A11 A10\n#define A12 A11\n#define A13 A12\n#define A14 A13\n#define A15 A14\n#define A16 A15\n#define A17 A16\n#define A18 A17\n#define A19 A18\n#define A20 A19\nint main(void) { return A20 == 7 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn variadic_macro_va_args_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "variadic_macro_va_args",
        source: "#define ADD(first, ...) ((first) + (__VA_ARGS__))\nint main(void) { return ADD(2, 3) == 5 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn predefined_macros_and_func_name_match_host_exit_code() {
    // given
    let case = OracleCase {
        name: "predefined_macros_and_func_name",
        source: "int main(void) { int line = __LINE__; char *file = __FILE__; return line > 0 && file[0] != 0 && __func__[0] == 'm' ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn nested_brace_initializer_list_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "nested_brace_initializer_list",
        source: "int matrix[2][3] = { { 1, 2, 3 }, { 4, 5, 6 } }; int main(void) { return matrix[1][2] == 6 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn ternary_side_effects_match_host_exit_code() {
    // given
    let case = OracleCase {
        name: "ternary_side_effects",
        source: "int main(void) { int x = 0; int y = x++ ? x++ : ++x; return x == 2 && y == 2 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn sizeof_expression_vs_type_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "sizeof_expression_vs_type",
        source: "int bump(void) { return 7; } int main(void) { int x = 3; return sizeof(x + 1) == sizeof(int) && sizeof(bump()) == sizeof(int) ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn incompatible_pointer_cast_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "incompatible_pointer_cast",
        source: "int main(void) { int x = 0x41424344; char *p = (char*)&x; return p[0] != 0 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn plain_char_signedness_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "plain_char_signedness",
        source: "int main(void) { char bytes[1]; bytes[0] = 255; return bytes[0] < 0 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn long_long_shift_arithmetic_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "long_long_shift_arithmetic",
        source: "int main(void) { long long value = 0x100000000LL; return (int)(value >> 32) == 1 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn floating_point_literal_parsing_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "floating_point_literal_parsing",
        source: "int main(void) { double x = 1.25; double y = .75; return (int)((x + y) * 2.0) == 4 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn hex_float_constant_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "hex_float_constant",
        source: "int main(void) { double x = 0x1.8p+2; return (int)x == 6 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}
