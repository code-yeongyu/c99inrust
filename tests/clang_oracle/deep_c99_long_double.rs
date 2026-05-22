use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn long_double_local_arithmetic_matches_host_stdout_and_exit_code() {
    // given
    let name = "long_double_local_arithmetic";
    let source = "int puts(char*); int main(void) { long double a = 1.25L; long double b = 2.75L; long double c = a + b; puts(\"ld-arith\"); return (int)(c * 10.0L) == 40 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn long_double_cast_from_int_matches_host_stdout_and_exit_code() {
    // given
    let name = "long_double_cast_from_int";
    let source = "int puts(char*); int main(void) { int x = 7; long double y = (long double)x + .5L; puts(\"ld-cast\"); return (int)(y * 2.0L) == 15 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn long_double_sizeof_matches_host_stdout_and_exit_code() {
    // given
    let name = "long_double_sizeof";
    let source = "int puts(char*); int main(void) { puts(\"ld-size\"); return sizeof(long double) == sizeof(double) ? 0 : sizeof(long double); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn long_double_function_return_matches_host_stdout_and_exit_code() {
    // given
    let name = "long_double_function_return";
    let source = "int puts(char*); long double widen(long double x) { return x + 0.5L; } int main(void) { long double y = widen(1.5L); puts(\"ld-return\"); return (int)(y * 10.0L) == 20 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}
