use super::support::{OracleCase, assert_compile_run_matches_host};

#[test]
fn sequence_points_short_circuit_side_effects_match_host_c_compiler_exit_code() {
    // given
    let case = OracleCase {
        name: "sequence_points_short_circuit_side_effects",
        source: "int bump(int *p) { *p = *p + 1; return *p; } int main(void) { int x = 0; if (bump(&x) == 1 && bump(&x) == 2) { return x == 2 ? 0 : 1; } return 2; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn signed_integer_overflow_wraps_like_host_c_compiler_exit_code() {
    // given
    let case = OracleCase {
        name: "signed_integer_overflow_wraps_like_host",
        source: "int main(void) { int x = 2147483647; x = x + 1; return x < 0 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn strict_aliasing_short_pointer_pun_matches_host_c_compiler_exit_code() {
    // given
    let case = OracleCase {
        name: "strict_aliasing_short_pointer_pun",
        source: "int main(void) { int x = 0x12345678; short *p; p = (short*)&x; *p = 0x7fff; return (x & 0xffff) == 0x7fff ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn union_type_punning_matches_host_c_compiler_exit_code() {
    // given
    let case = OracleCase {
        name: "union_type_punning",
        source: "typedef union { int i; short s[2]; } pun_t; int main(void) { pun_t u; u.i = 0x12345678; return u.s[0] == 0x5678 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn pointer_one_past_difference_matches_host_c_compiler_exit_code() {
    // given
    let case = OracleCase {
        name: "pointer_one_past_difference",
        source: "int main(void) { int values[3]; int *begin; int *end; begin = &values[0]; end = &values[3]; return end - begin == 3 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn integer_field_conversion_matches_host_c_compiler_exit_code() {
    // given
    let case = OracleCase {
        name: "integer_field_conversion",
        source: "typedef struct { unsigned char c; unsigned short s; } pack_t; int main(void) { pack_t p; p.c = 300; p.s = 65537; return p.c == 44 && p.s == 1 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn implicit_pointer_conversion_matches_host_c_compiler_exit_code() {
    // given
    let case = OracleCase {
        name: "implicit_pointer_conversion",
        source: "void *id(void *p) { return p; } int main(void) { int x = 7; int *p; p = id(&x); return *p == 7 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn comma_operator_for_clause_order_matches_host_c_compiler_exit_code() {
    // given
    let case = OracleCase {
        name: "comma_operator_for_clause_order",
        source: "int main(void) { int i; int j; for (i = 0, j = 1; i < 3; i = i + 1, j = j * 2) { } return j == 8 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn short_circuit_skips_undefined_division_matches_host_c_compiler_exit_code() {
    // given
    let case = OracleCase {
        name: "short_circuit_skips_undefined_division",
        source: "int main(void) { int x = 0; int y = 0; if (x && 10 / y) { return 1; } if (!x || 10 / y) { return 0; } return 2; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn setjmp_longjmp_flow_matches_host_c_compiler_exit_code() {
    // given
    let case = OracleCase {
        name: "setjmp_longjmp_flow",
        source: "int setjmp(void **env); void longjmp(void **env, int value); void *env[64]; int jump_once(void) { if (setjmp(env) == 0) { longjmp(env, 9); return 1; } return 0; } int main(void) { return jump_once(); }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn variadic_ignored_arguments_match_host_c_compiler_exit_code() {
    // given
    let case = OracleCase {
        name: "variadic_ignored_arguments",
        source: "int first(int value, ...) { return value; } int main(void) { return first(0, 1, 2, 3); }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn flexible_array_member_size_matches_host_c_compiler_exit_code() {
    // given
    let case = OracleCase {
        name: "flexible_array_member_size",
        source: "typedef struct { int length; int data[]; } packet_t; int main(void) { return sizeof(packet_t) == 4 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn scalar_compound_literal_matches_host_c_compiler_exit_code() {
    // given
    let case = OracleCase {
        name: "scalar_compound_literal",
        source: "int main(void) { return ((int){7}) == 7 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn array_designated_initializer_matches_host_c_compiler_exit_code() {
    // given
    let case = OracleCase {
        name: "array_designated_initializer",
        source: "int values[4] = { [2] = 9, [0] = 3 }; int main(void) { return values[0] == 3 && values[1] == 0 && values[2] == 9 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}
