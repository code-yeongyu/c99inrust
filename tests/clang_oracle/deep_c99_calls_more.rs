use super::multifile_support::{
    OracleMultiFileCase, OracleSourceFile, assert_multifile_compile_run_matches_host,
};
use super::support::{OracleCase, assert_compile_run_matches_host};

#[test]
fn function_pointer_array_conditional_index_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "function_pointer_array_conditional_index",
        source: "int inc(int x) { return x + 1; } int dec(int x) { return x - 1; } int main(void) { int (*ops[2])(int) = { inc, dec }; int pick = 1; return ops[pick ? 0 : 1](10) == 11 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn function_pointer_array_assigned_later_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "function_pointer_array_assigned_later",
        source: "int square(int x) { return x * x; } int cube(int x) { return x * x * x; } int main(void) { int (*ops[2])(int); ops[0] = square; ops[1] = cube; return ops[0](4) == 16 && ops[1](3) == 27 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn function_pointer_cast_direct_call_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "function_pointer_cast_direct_call",
        source: "int add3(int x) { return x + 3; } int main(void) { return ((int (*)(int))add3)(4) == 7 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn function_pointer_parameter_indirect_call_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "function_pointer_parameter_indirect_call",
        source: "int inc(int x) { return x + 1; } int apply(int (*op)(int), int value) { return (*op)(value); } int main(void) { return apply(inc, 6) == 7 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn function_pointer_array_post_increment_index_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "function_pointer_array_post_increment_index",
        source: "int inc(int x) { return x + 1; } int dec(int x) { return x - 1; } int main(void) { int (*ops[2])(int) = { inc, dec }; int index = 0; int first = ops[index++](3); int second = ops[index](3); return first == 4 && second == 2 && index == 1 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn kr_style_three_parameter_sum_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "kr_style_three_parameter_sum",
        source: "int sum3(a, b, c) int a; int b; int c; { return a + b + c; } int main(void) { return sum3(2, 3, 4) == 9 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn kr_style_parameter_declaration_order_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "kr_style_parameter_declaration_order",
        source: "int choose(flag, a, b) int b; int flag; int a; { return flag ? a : b; } int main(void) { return choose(0, 11, 13) == 13 && choose(1, 11, 13) == 11 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn static_inline_calls_static_inline_matches_host_exit_code() {
    // given
    let case = OracleCase {
        name: "static_inline_calls_static_inline",
        source: "static inline int add1(int x) { return x + 1; } static inline int add2(int x) { return add1(add1(x)); } int main(void) { return add2(5) == 7 ? 0 : 1; }\n",
    };

    // when/then
    assert_compile_run_matches_host(case);
}

#[test]
fn multi_file_extern_array_linkage_matches_host_exit_code() {
    // given
    let case = OracleMultiFileCase {
        name: "multi_file_extern_array_linkage",
        files: &[
            OracleSourceFile {
                path: "data.c",
                source: "int values[3] = { 2, 4, 6 }; int get_value(int index) { return values[index]; }\n",
            },
            OracleSourceFile {
                path: "main.c",
                source: "extern int values[3]; int get_value(int index); int main(void) { return get_value(2) == 6 && values[1] == 4 ? 0 : 1; }\n",
            },
        ],
    };

    // when/then
    assert_multifile_compile_run_matches_host(case);
}

#[test]
fn multi_file_static_internal_linkage_matches_host_exit_code() {
    // given
    let case = OracleMultiFileCase {
        name: "multi_file_static_internal_linkage",
        files: &[
            OracleSourceFile {
                path: "hidden.c",
                source: "static int hidden = 9; int read_hidden(void) { return hidden; }\n",
            },
            OracleSourceFile {
                path: "main.c",
                source: "int read_hidden(void); int main(void) { return read_hidden() == 9 ? 0 : 1; }\n",
            },
        ],
    };

    // when/then
    assert_multifile_compile_run_matches_host(case);
}
