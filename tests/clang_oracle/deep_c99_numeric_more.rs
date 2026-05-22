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
fn unsigned_char_pointer_promotion_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "unsigned_char_pointer_promotion",
        source: "int main(void) { unsigned char data[1]; unsigned char *p; data[0] = 255; p = data; return p[0] == 255 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn global_plain_char_promotion_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "global_plain_char_promotion",
        source: "char data[1] = {255}; int main(void) { return data[0] < 0 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn global_unsigned_char_matrix_promotion_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "global_unsigned_char_matrix_promotion",
        source: "unsigned char grid[1][1] = {{255}}; int main(void) { return grid[0][0] == 255 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn local_plain_char_assignment_narrows_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "local_plain_char_assignment_narrows",
        source: "int main(void) { char c; c = 255; return c == -1 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn local_unsigned_char_assignment_narrows_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "local_unsigned_char_assignment_narrows",
        source: "int main(void) { unsigned char c; c = 300; return c == 44 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn local_short_assignment_narrows_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "local_short_assignment_narrows",
        source: "int main(void) { short s; s = 65535; return s == -1 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn local_unsigned_char_post_increment_wraps_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "local_unsigned_char_post_increment_wraps",
        source: "int main(void) { unsigned char c = 255; int old = c++; return old == 255 && c == 0 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn local_short_pre_increment_wraps_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "local_short_pre_increment_wraps",
        source: "int main(void) { short s = 32767; int saved = ++s; return saved == -32768 && s == -32768 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn local_narrow_scalar_sizeof_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "local_narrow_scalar_sizeof",
        source: "int main(void) { char c; unsigned short s; return sizeof(c) == 1 && sizeof(s) == 2 ? 0 : 1; }\n",
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
