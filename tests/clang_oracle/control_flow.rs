use super::support::{OracleCase, assert_compile_run_matches_host};

#[test]
fn post_decrement_condition_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "post_decrement_condition_slice",
        source: "void run(int ticks) { while (ticks--) { ticks = ticks; } } int main(void) { return 0; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn post_increment_value_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "post_increment_value_slice",
        source: "int main(void) { int x = 4; return (x++ == 4 && x == 5) ? 0 : 1; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn empty_while_post_increment_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "empty_while_post_increment_slice",
        source: "int main(void) { int x = 0; while (x++ < 1); return x == 2 ? 0 : 1; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn if_else_comparison_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "if_else_comparison",
        source: "int main(void) { int x = 7; if (x >= 7) { x = x + 30; } else { x = 1; } if (x != 37) { return 2; } return x; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn while_loop_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "while_loop",
        source: "int main(void) { int x = 0; int total = 0; while (x < 5) { total = total + x; x = x + 1; } return total; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn do_while_loop_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "do_while_loop",
        source: "int main(void) { int x = 0; int total = 0; do { total = total + x; x = x + 1; } while (x < 5); return total; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn logical_short_circuit_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "logical_short_circuit",
        source: "int main(void) { int x = 0; if (x != 0 && 10 / x > 1) { return 1; } if (x == 0 || 10 / x > 1) { return 42; } return 2; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn for_loop_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "for_loop",
        source: "int main(void) { int total = 0; for (int i = 0; i < 5; i = i + 1) { total = total + i; } return total; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn post_increment_for_loop_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "post_increment_for_loop",
        source: "int main(void) { int i; int total = 0; for (i = 0; i < 4; i++) { total = total + i; } return total == 6 ? 0 : 1; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn prefix_increment_condition_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "prefix_increment_condition_slice",
        source: "int fuzzpos; int main(void) { fuzzpos = 49; return (++fuzzpos == 50 && fuzzpos == 50) ? 0 : 1; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn conditional_expression_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "conditional_expression_slice",
        source: "int main(int argc, char **argv) { return argc < 0 ? 2 : 42; }\n",
    };
    assert_compile_run_matches_host(case);
}
