use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn extra_ternary_side_effects_with_comma_match_host_stdout_and_exit_code() {
    // given
    let name = "extra_ternary_side_effects_with_comma";
    let source = "int puts(char*); int main(void) { int x = 1; int y = 2; int z = 3; int a = x ? (y += 5, y) : (z += 50, z); int b = (x--, x) ? (y += 100, y) : (z += a, z); puts(\"extra-ternary\"); return x + y + z + a + b; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn extra_sizeof_unevaluated_post_increment_matches_host_stdout_and_exit_code() {
    // given
    let name = "extra_sizeof_unevaluated_post_increment";
    let source = "int puts(char*); int main(void) { int x = 7; int values[5]; int total = sizeof(x++) + sizeof values + sizeof(int[2]); puts(\"extra-sizeof\"); return x + total; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn extra_char_signedness_shift_chain_matches_host_stdout_and_exit_code() {
    // given
    let name = "extra_char_signedness_shift_chain";
    let source = "int puts(char*); int main(void) { char c = (char)128; unsigned char u = c; signed char s = u; int total = (c < 0 ? 1 : 2) + (u >> 6) + (s < 0 ? 10 : 20); puts(\"extra-char\"); return total; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn extra_long_long_division_remainder_mix_matches_host_stdout_and_exit_code() {
    // given
    let name = "extra_long_long_division_remainder_mix";
    let source = "int puts(char*); int main(void) { long long value = 1234567890123LL; long long q = value / 1000000LL; long long r = value % 1000LL; puts(\"extra-ll\"); return (int)((q % 1000LL) + r); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn extra_decimal_float_literal_precision_mix_matches_host_stdout_and_exit_code() {
    // given
    let name = "extra_decimal_float_literal_precision_mix";
    let source = "int puts(char*); int main(void) { double a = 3.75e1; double b = 0.25e2; double c = 6.; puts(\"extra-float\"); return (int)(a + b + c); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn extra_hex_float_literal_negative_exponent_mix_matches_host_stdout_and_exit_code() {
    // given
    let name = "extra_hex_float_literal_negative_exponent_mix";
    let source = "int puts(char*); int main(void) { double a = 0x1.4p+4; double b = 0x1.8p-1; double c = 0x1p+2; puts(\"extra-hexfloat\"); return (int)(a + b + c); }\n";

    // when/then
    assert_case(name, source);
}
