use super::support::{OracleCase, assert_compile_run_matches_host};

#[test]
fn typedef_function_pointer_array_compound_pointer_matches_host_stdout_and_exit_code() {
    // given
    let case = OracleCase {
        name: "typedef_function_pointer_array_compound_pointer",
        source: "int puts(char *); typedef int (*op_t)(int); int inc(int x) { return x + 1; } int dec(int x) { return x - 1; } int main(void) { op_t *ops = (op_t[2]){ inc, dec }; puts(\"fnptr-typedef-compound\"); return ops[0](4) == 5 && ops[1](4) == 3 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn typedef_function_pointer_array_compound_direct_call_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "typedef_function_pointer_array_compound_direct_call",
        source: "typedef int (*op_t)(int); int inc(int x) { return x + 1; } int dec(int x) { return x - 1; } int main(void) { return ((op_t[2]){ inc, dec })[1](9) == 8 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn typedef_double_function_pointer_compound_pointer_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "typedef_double_function_pointer_compound_pointer",
        source: "typedef double (*op_t)(double); double half(double x) { return x / 2.0; } double twice(double x) { return x * 2.0; } int main(void) { op_t *ops = (op_t[2]){ half, twice }; return ops[0](9.0) == 4.5 && ops[1](3.0) == 6.0 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn global_typedef_function_pointer_array_initializer_matches_host_stdout_and_exit_code() {
    // given
    let case = OracleCase {
        name: "global_typedef_function_pointer_array_initializer",
        source: "int puts(char *); typedef int (*op_t)(int); int add1(int x) { return x + 1; } int add2(int x) { return x + 2; } op_t ops[2] = { add1, add2 }; int main(void) { puts(\"global-fnptr-typedef-array\"); return ops[0](4) == 5 && ops[1](4) == 6 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}
