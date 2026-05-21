use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn extra_constant_expression_nested_array_bound_matches_host_stdout_and_exit_code() {
    // given
    let name = "extra_constant_expression_nested_array_bound";
    let source = "int puts(char*); enum { A = sizeof(int[3]), B = A / sizeof(int), C = (B << 2) - 1 }; int main(void) { int grid[C * B]; puts(\"extra-constexpr\"); return sizeof(grid) + C + B; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn extra_macro_expansion_token_paste_ladder_matches_host_stdout_and_exit_code() {
    // given
    let name = "extra_macro_expansion_token_paste_ladder";
    let source = "#define JOIN(a, b) a ## b\n#define STEP0 4\n#define STEP1 STEP0\n#define STEP2 STEP1\n#define STEP3 STEP2\n#define CALL(name) JOIN(run_, name)()\nint puts(char*); int run_4(void) { return STEP3 + 9; } int main(void) { puts(\"extra-macro\"); return CALL(STEP3); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn extra_variadic_macro_forwards_to_token_pasted_function_match_host_stdout_and_exit_code() {
    // given
    let name = "extra_variadic_macro_forwards_to_token_pasted_function";
    let source = "#define JOIN(a, b) a ## b\n#define DISPATCH(name, ...) JOIN(do_, name)(__VA_ARGS__)\nint puts(char*); int do_mix(int a, int b, int c) { return a * 100 + b * 10 + c; } int main(void) { puts(\"extra-vaargs\"); return DISPATCH(mix, 2, 4, 6); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn extra_predefined_macros_inside_expanded_call_match_host_stdout_and_exit_code() {
    // given
    let name = "extra_predefined_macros_inside_expanded_call";
    let source = "#define HERE() __LINE__\nint puts(char*);\nint helper(void) { return HERE(); }\nint main(void) {\nint before = HERE();\nint after = HERE();\nchar *file = __FILE__;\nputs(__func__);\nreturn after == before + 1 && helper() < before && file[0] != 0 ? 0 : 1;\n}\n";

    // when/then
    assert_case(name, source);
}
