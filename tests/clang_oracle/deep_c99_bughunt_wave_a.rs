use super::multifile_support::{
    OracleMultiFileCase, OracleSourceFile, assert_multifile_compile_run_matches_host,
};
use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn bughunt_global_pointer_initializer_plus_offset_matches_host_stdout_and_exit_code() {
    // given
    let name = "bughunt_global_pointer_initializer_plus_offset";
    let source = "int puts(char*); int values[4] = { 3, 5, 7, 11 }; int *cursor = values + 2; int main(void) { puts(\"bug-gptr-plus\"); return *cursor == 7 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn bughunt_global_pointer_initializer_array_decay_matches_host_stdout_and_exit_code() {
    // given
    let name = "bughunt_global_pointer_initializer_array_decay";
    let source = "int puts(char*); int values[3] = { 13, 17, 19 }; int *cursor = values; int main(void) { puts(\"bug-gptr-decay\"); return cursor[1] == 17 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn bughunt_multifile_global_pointer_initializer_plus_offset_matches_host_stdout_and_exit_code() {
    // given
    let case = OracleMultiFileCase {
        name: "bughunt_multifile_global_pointer_initializer_plus_offset",
        files: &[
            OracleSourceFile {
                path: "shared.h",
                source: "extern int values[4]; extern int *cursor; int read_cursor(void);\n",
            },
            OracleSourceFile {
                path: "state.c",
                source: "#include \"shared.h\"\nint values[4] = { 2, 4, 6, 8 }; int *cursor = values + 3;\n",
            },
            OracleSourceFile {
                path: "read.c",
                source: "#include \"shared.h\"\nint read_cursor(void) { return *cursor + values[1]; }\n",
            },
            OracleSourceFile {
                path: "main.c",
                source: "#include \"shared.h\"\nint puts(char*); int main(void) { puts(\"bug-mf-gptr\"); return read_cursor() == 12 ? 0 : 1; }\n",
            },
        ],
    };

    // when/then
    assert_multifile_compile_run_matches_host(case);
}

#[test]
fn bughunt_local_int_matrix_initializer_matches_host_stdout_and_exit_code() {
    // given
    let name = "bughunt_local_int_matrix_initializer";
    let source = "int puts(char*); int main(void) { int matrix[2][3] = { { 1, 2, 3 }, { 4, 5, 6 } }; puts(\"bug-local-matrix\"); return matrix[1][2] == 6 && sizeof(matrix) == 24 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn bughunt_local_int_matrix_row_decay_matches_host_stdout_and_exit_code() {
    // given
    let name = "bughunt_local_int_matrix_row_decay";
    let source = "int puts(char*); int main(void) { int matrix[2][2] = { { 3, 5 }, { 7, 11 } }; int *row = matrix[1]; puts(\"bug-row-decay\"); return row[0] + row[1] == 18 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn bughunt_local_int_matrix_assignment_matches_host_stdout_and_exit_code() {
    // given
    let name = "bughunt_local_int_matrix_assignment";
    let source = "int puts(char*); int main(void) { int matrix[2][2]; matrix[0][0] = 9; matrix[1][1] = 14; puts(\"bug-matrix-assign\"); return matrix[0][0] + matrix[1][1] == 23 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn bughunt_switch_default_middle_preserves_fallthrough_matches_host_stdout_and_exit_code() {
    // given
    let name = "bughunt_switch_default_middle_preserves_fallthrough";
    let source = "int puts(char*); int main(void) { int x = 0; switch (2) { case 1: x = x + 1; default: x = x + 10; case 2: x = x + 100; } puts(\"bug-switch-mid\"); return x == 100 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn bughunt_switch_default_first_preserves_fallthrough_matches_host_stdout_and_exit_code() {
    // given
    let name = "bughunt_switch_default_first_preserves_fallthrough";
    let source = "int puts(char*); int main(void) { int x = 0; switch (9) { default: x = x + 3; case 4: x = x * 5; break; case 9: x = 99; } puts(\"bug-switch-first\"); return x == 15 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn bughunt_commuted_local_array_subscript_matches_host_stdout_and_exit_code() {
    // given
    let name = "bughunt_commuted_local_array_subscript";
    let source = "int puts(char*); int main(void) { int values[4] = { 1, 3, 5, 7 }; puts(\"bug-commute-local\"); return 2[values] == 5 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn bughunt_commuted_global_array_subscript_matches_host_stdout_and_exit_code() {
    // given
    let name = "bughunt_commuted_global_array_subscript";
    let source = "int puts(char*); int values[4] = { 2, 4, 6, 8 }; int main(void) { puts(\"bug-commute-global\"); return 3[values] == 8 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn bughunt_function_pointer_pointer_cursor_matches_host_stdout_and_exit_code() {
    // given
    let name = "bughunt_function_pointer_pointer_cursor";
    let source = "int puts(char*); int inc(int x) { return x + 1; } int triple(int x) { return x * 3; } int main(void) { int (*ops[2])(int) = { inc, triple }; int (**cursor)(int) = ops; puts(\"bug-fnptr-cursor\"); return cursor[1](7) == 21 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn bughunt_nested_function_pointer_ternary_matches_host_stdout_and_exit_code() {
    // given
    let name = "bughunt_nested_function_pointer_ternary";
    let source = "int puts(char*); int add2(int x) { return x + 2; } int mul4(int x) { return x * 4; } int (*choose(int flag))(int) { return flag ? mul4 : add2; } int main(void) { int (*fn)(int) = choose(1); puts(\"bug-fnptr-ternary\"); return fn(6) == 24 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn bughunt_token_paste_field_name_matches_host_stdout_and_exit_code() {
    // given
    let name = "bughunt_token_paste_field_name";
    let source = "#define CAT(a, b) a ## b\n#define FIELD(n) CAT(field_, n)\nint puts(char*); typedef struct { int field_left; int field_right; } pair_t; int main(void) { pair_t p; p.FIELD(left) = 12; p.FIELD(right) = 30; puts(\"bug-paste-field\"); return p.field_left + p.field_right == 42 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn bughunt_variadic_token_paste_dispatch_matches_host_stdout_and_exit_code() {
    // given
    let name = "bughunt_variadic_token_paste_dispatch";
    let source = "#define CAT(a, b) a ## b\n#define CALL(name, ...) CAT(run_, name)(__VA_ARGS__)\nint puts(char*); int run_mix(int a, int b) { return a * 10 + b; } int main(void) { puts(\"bug-va-paste\"); return CALL(mix, 4, 2) == 42 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn bughunt_nested_initializer_scalar_braces_matches_host_stdout_and_exit_code() {
    // given
    let name = "bughunt_nested_initializer_scalar_braces";
    let source = "int puts(char*); int main(void) { int values[4] = { { 1 }, 2, { 3 }, 4 }; puts(\"bug-init-braces\"); return values[0] + values[2] == 4 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn bughunt_struct_pointer_cast_array_overlay_matches_host_stdout_and_exit_code() {
    // given
    let name = "bughunt_struct_pointer_cast_array_overlay";
    let source = "int puts(char*); typedef struct { short lo; short hi; } pair_t; int main(void) { int words[2]; pair_t *pairs = (pair_t *)(void *)words; pairs[0].lo = 5; pairs[1].hi = 9; puts(\"bug-overlay\"); return pairs[0].lo + pairs[1].hi == 14 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn bughunt_chained_ternary_side_effects_matches_host_stdout_and_exit_code() {
    // given
    let name = "bughunt_chained_ternary_side_effects";
    let source = "int puts(char*); int main(void) { int x = 0; int y = 1; int z = 2; int r = x ? ++y : y ? (z += 5) : (x += 9); puts(\"bug-ternary-chain\"); return x + y + z + r == 15 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn bughunt_pointer_arithmetic_compound_lvalue_matches_host_stdout_and_exit_code() {
    // given
    let name = "bughunt_pointer_arithmetic_compound_lvalue";
    let source = "int puts(char*); int main(void) { int values[4] = { 1, 2, 3, 4 }; int *cursor = values; *(cursor + 2) += 10; puts(\"bug-ptr-lvalue\"); return values[2] == 13 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn bughunt_sizeof_matrix_row_expression_matches_host_stdout_and_exit_code() {
    // given
    let name = "bughunt_sizeof_matrix_row_expression";
    let source = "int puts(char*); int main(void) { int matrix[3][4]; int *row = matrix[2]; puts(\"bug-sizeof-row\"); return sizeof(matrix[0]) == 16 && sizeof(row) == sizeof(int*) ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn bughunt_comma_operator_in_subscript_matches_host_stdout_and_exit_code() {
    // given
    let name = "bughunt_comma_operator_in_subscript";
    let source = "int puts(char*); int main(void) { int i = 0; int values[3] = { 4, 8, 12 }; puts(\"bug-comma-sub\"); return values[(i += 1, i + 1)] == 12 && i == 1 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}
