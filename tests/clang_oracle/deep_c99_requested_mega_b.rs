use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn variadic_macro_token_paste_dispatch_matches_host_stdout_and_exit_code() {
    // given
    let name = "variadic_macro_token_paste_dispatch";
    let source = "#define JOIN(a, b) a ## b\n#define DISPATCH(name, ...) JOIN(run_, name)(__VA_ARGS__)\nint puts(char*); int run_add(int a, int b, int c) { return a + b + c; } int main(void) { puts(\"mega-vaargs\"); return DISPATCH(add, 5, 7, 9); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn predefined_line_file_func_exact_value_matches_host_stdout_and_exit_code() {
    // given
    let name = "predefined_line_file_func_exact_value";
    let source = "#define LINE_VALUE __LINE__\nint puts(char*); int helper(void) { puts(__func__); return LINE_VALUE; }\nint main(void) { char *file = __FILE__; puts(file); return helper() + __LINE__; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn nested_local_initializer_braces_exact_sum_matches_host_stdout_and_exit_code() {
    // given
    let name = "nested_local_initializer_braces_exact_sum";
    let source = "int puts(char*); typedef struct { int matrix[2][3]; int tail; } grid_t; int main(void) { grid_t grid = { { { 1, 2 }, { 3, 4, 5 } }, 6 }; puts(\"mega-init\"); return grid.matrix[0][0] + grid.matrix[0][2] + grid.matrix[1][2] + grid.tail; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn ternary_nested_side_effect_exact_value_matches_host_stdout_and_exit_code() {
    // given
    let name = "ternary_nested_side_effect_exact_value";
    let source = "int puts(char*); int main(void) { int x = 0; int y = 10; int a = x++ ? ++y : y++; int b = x ? (x += 3) : (y += 5); puts(\"mega-ternary\"); return x * 10 + y + a + b; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn sizeof_expression_side_effect_suppression_matches_host_stdout_and_exit_code() {
    // given
    let name = "sizeof_expression_side_effect_suppression";
    let source = "int puts(char*); int bump(int *p) { *p = *p + 100; return *p; } int main(void) { int x = 3; int bytes = sizeof(bump(&x)) + sizeof x; puts(\"mega-sizeof\"); return bytes + x; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn incompatible_pointer_cast_short_overlay_matches_host_stdout_and_exit_code() {
    // given
    let name = "incompatible_pointer_cast_short_overlay";
    let source = "int puts(char*); int main(void) { int value = 0; unsigned short *parts = (unsigned short *)(void *)&value; parts[0] = 0x34; parts[1] = 0x12; puts(\"mega-ptrcast\"); return value == 0 ? 1 : ((int)parts[0] + (int)parts[1]); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn char_signedness_exact_host_result_matches_stdout_and_exit_code() {
    // given
    let name = "char_signedness_exact_host_result";
    let source = "int puts(char*); int main(void) { char c = -1; unsigned char u = c; signed char s = u; puts(\"mega-char\"); return (c < 0 ? 10 : 20) + (u == 255 ? 3 : 5) + (s == -1 ? 1 : 2); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn long_long_shift_mask_exact_value_matches_host_stdout_and_exit_code() {
    // given
    let name = "long_long_shift_mask_exact_value";
    let source = "int puts(char*); int main(void) { long long value = (1LL << 36) + (5LL << 8) + 7LL; long long high = value >> 32; long long low = value & 0xfffLL; puts(\"mega-longlong\"); return (int)(high + low); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn floating_literal_suffixes_exact_truncation_matches_host_stdout_and_exit_code() {
    // given
    let name = "floating_literal_suffixes_exact_truncation";
    let source = "int puts(char*); int main(void) { double a = 1e2; double b = 2.5e1f; double c = .5; puts(\"mega-float\"); return (int)(a + b + c); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn hex_float_mixed_exponents_exact_truncation_matches_host_stdout_and_exit_code() {
    // given
    let name = "hex_float_mixed_exponents_exact_truncation";
    let source = "int puts(char*); int main(void) { double a = 0x1.2p+4; double b = 0x1.8p-1; puts(\"mega-hexfloat\"); return (int)(a + b); }\n";

    // when/then
    assert_case(name, source);
}
