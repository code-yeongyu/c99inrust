use super::support::{
    OracleCase, assert_compile_run_matches_host, assert_macos_compile_run_matches_host,
};

#[test]
fn constant_return_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "constant_return",
        source: "#define ANSWER 42\nint main(void) { return (ANSWER * 2) - 42; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn arithmetic_precedence_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "arithmetic_precedence",
        source: "int main(void) { return 3 + 5 * 8 - (9 >> 1); }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn local_int_assignment_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "local_int_assignment",
        source: "int main(void) { int x = 40; int y = x + 1; x = y + 1; return x; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn compound_assignment_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "compound_assignment",
        source: "int main(void) { int x = 40; int y = 8; x += y / 2; x -= 1; return x; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn unsigned_cast_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "unsigned_cast_slice",
        source: "int main(void) { int x = 7; return ((unsigned)x >= 0 && (unsigned char)x == x) ? 0 : 1; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn signed_long_long_cast_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "signed_long_long_cast_slice",
        source: "int main(int argc, char **argv) { int a = argc << 30; return ((long long) a * 4) >> 30; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn fixeddiv2_double_slice_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "fixeddiv2_double_slice",
        source: r#"typedef int fixed_t;
void I_Error(char *message) { return; }
fixed_t FixedDiv2(fixed_t a, fixed_t b) {
    double c;
    c = ((double)a) / ((double)b) * (1<<16);
    if (c >= 2147483648.0 || c < -2147483648.0)
        I_Error("FixedDiv: divide by zero");
    return (fixed_t)c;
}
int main(void) { return FixedDiv2(3, 2) == 98304 ? 0 : 1; }
"#,
    };
    assert_macos_compile_run_matches_host(case);
}

#[test]
fn fixed_point_global_initializer_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "fixed_point_global_initializer",
        source: "static int scale_mtof = (.2*(1<<16)); int main(void) { return scale_mtof == 13107 ? 0 : 1; }\n",
    };
    assert_compile_run_matches_host(case);
}

#[test]
fn unparenthesized_global_initializer_matches_host_c_compiler_exit_code() {
    let case = OracleCase {
        name: "unparenthesized_global_initializer",
        source: "static int finit_height = 200 - 32; int main(void) { return finit_height == 168 ? 0 : 1; }\n",
    };
    assert_compile_run_matches_host(case);
}
