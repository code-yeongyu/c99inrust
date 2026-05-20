use super::support::{OracleCase, assert_build_run_matches_host, assert_compile_run_matches_host};

#[test]
fn zero_arg_function_call_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "zero_arg_function_call",
        source: "int answer(void) { int value = 40; return value; } int main(void) { return 2 + answer(); }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn nested_zero_arg_function_calls_match_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "nested_zero_arg_function_calls",
        source: "int answer(void) { int value = 40; return value; } int two(void) { int value = 2; return value; } int main(void) { return 100 + (answer() + two()); }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn build_command_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "build_command",
        source: "int tick(void) { return 1; } int main(void) { return 41 + tick(); }\n",
    };
    assert_build_run_matches_host(case);
}

#[test]
fn top_level_declaration_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "top_level_declaration_slice",
        source: "static const char rcsid[] = \"doom\"; int main(void) { return 42; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn void_function_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "void_function_slice",
        source: "void tick(void) { return; } int main(void) { return 42; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn parameter_list_signature_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "parameter_list_signature_slice",
        source: "int main(int argc, char **argv) { return 42; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn typedef_return_signature_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "typedef_return_signature_slice",
        source: "typedef int fixed_t; fixed_t FixedMul(fixed_t a, fixed_t b) { return 42; } int main(void) { return 42; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn unsigned_return_signature_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "unsigned_return_signature_slice",
        source: "unsigned short SwapSHORT(unsigned short x) { return 42; } int main(void) { return 42; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn unsigned_parameter_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "unsigned_parameter_slice",
        source: "int add(unsigned ofs, int count) { return ofs + count; } int main(void) { return add(2, 3) == 5 ? 0 : 1; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn parameter_binding_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "parameter_binding_slice",
        source: "int main(int argc, char **argv) { return argc; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn function_call_argument_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "function_call_argument_slice",
        source: "int add(int left, int right) { return left + right; } int main(int argc, char **argv) { return add(argc, 41); }\n",
    };
    assert_compile_run_matches_host(case);
}
