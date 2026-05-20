use super::support::{OracleCase, assert_compile_run_matches_host};

#[test]
fn ternary_skips_unselected_side_effect_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "ternary_skips_unselected_side_effect",
        source: "int main(void) { int x = 0; int y = 1 ? ++x : x++; return x == 1 && y == 1 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn nested_ternary_right_associativity_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "nested_ternary_right_associativity",
        source: "int main(void) { int x = 0; int y = x ? 1 : x + 1 ? 2 : 3; return y == 2 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn sizeof_expression_does_not_evaluate_side_effect_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "sizeof_expression_does_not_evaluate_side_effect",
        source: "int main(void) { int x = 1; int y = sizeof(x++); return x == 1 && y == sizeof(int) ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn sizeof_pointer_type_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "sizeof_pointer_type",
        source: "int main(void) { return sizeof(int*) == sizeof(void*) && sizeof(char*) == sizeof(int*) ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn incompatible_pointer_cast_store_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "incompatible_pointer_cast_store",
        source: "int main(void) { int value = 0; void *vp = &value; int *ip = (int*)vp; *ip = 9; return value == 9 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn const_pointer_cast_alias_store_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "const_pointer_cast_alias_store",
        source: "int main(void) { int value = 2; const int *readonly = &value; int *writable = (int*)readonly; *writable = 7; return value == 7 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn restrict_pointer_loop_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "restrict_pointer_loop",
        source: "void add_all(int n, int * restrict out, int * restrict in) { int i; for (i = 0; i < n; i++) out[i] = out[i] + in[i]; } int main(void) { int a[3] = { 1, 2, 3 }; int b[3] = { 4, 5, 6 }; add_all(3, a, b); return a[0] == 5 && a[2] == 9 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn pointer_arithmetic_indexing_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "pointer_arithmetic_indexing",
        source: "int main(void) { int values[4] = { 1, 3, 5, 7 }; int *p = values; return *(p + 2) == 5 && p[3] == 7 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn compound_literal_in_ternary_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "compound_literal_in_ternary",
        source: "int main(void) { int choose = 0; int value = choose ? (int){ 3 } : (int){ 8 }; return value == 8 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn comma_operator_for_init_and_post_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "comma_operator_for_init_and_post",
        source: "int main(void) { int x; int y; for (x = 1, y = 2; x < 3; x++, y += x) { } return x == 3 && y == 5 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}
