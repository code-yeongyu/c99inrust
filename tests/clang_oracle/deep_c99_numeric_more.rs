use super::support::{OracleCase, assert_compile_run_matches_host};

#[test]
fn bool_from_integer_edges_match_host_exit_code() {
    // given
    let case = OracleCase {
        name: "bool_from_integer_edges",
        source: "int main(void) { _Bool a = 0; _Bool b = -42; _Bool c = 300; return a == 0 && b == 1 && c == 1 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn bool_from_pointer_edges_match_host_exit_code() {
    // given
    let case = OracleCase {
        name: "bool_from_pointer_edges",
        source: "int main(void) { int value = 0; void *nullp = (void*)0; void *live = &value; _Bool a = nullp; _Bool b = live; return a == 0 && b == 1 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn enum_negative_large_chain_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "enum_negative_large_chain",
        source: "enum { LOW = -7, HIGH = 0x7fffffff, WRAPPED = LOW + 3 }; int main(void) { return LOW == -7 && WRAPPED == -4 && HIGH > 0 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn local_enum_depends_on_previous_values_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "local_enum_depends_on_previous_values",
        source: "int main(void) { enum { A = 3, B = A * 2, C = B - 5 }; return A == 3 && B == 6 && C == 1 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn plain_char_promotion_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "plain_char_promotion",
        source: "int main(void) { char c[1]; c[0] = 255; int promoted = c[0]; return promoted == -1 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn unsigned_char_conversion_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "unsigned_char_conversion",
        source: "int main(void) { unsigned char c[1]; c[0] = 300; int promoted = c[0]; return promoted == 44 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn long_long_high_bits_survive_addition_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "long_long_high_bits_survive_addition",
        source: "int main(void) { long long base = 0x100000000LL; long long sum = base + 5LL; return (int)(sum >> 32) == 1 && (int)sum == 5 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn signed_long_long_right_shift_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "signed_long_long_right_shift",
        source: "int main(void) { long long value = -8LL; return (int)(value >> 1) == -4 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn fractional_float_literals_match_host_exit_code() {
    // given
    let case = OracleCase {
        name: "fractional_float_literals",
        source: "int main(void) { double a = 2.5; double b = .125; return (int)((a + b) * 8.0) == 21 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn uppercase_hex_float_literal_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "uppercase_hex_float_literal",
        source: "int main(void) { double a = 0X1.0P+4; double b = 0x1p-1; return (int)(a + b + 0.5) == 17 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}
