use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn static_local_double_persists_across_calls_matches_host_stdout_and_exit_code() {
    // given
    let name = "static_local_double_persists_across_calls";
    let source = "int puts(char*); int step(void) { static double value = 1.5; value = value + 2.0; return value == 3.5 ? 1 : 2; } int main(void) { int first = step(); int second = step(); puts(\"static-double\"); return first == 1 && second == 2 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn static_local_complex_real_persists_across_calls_matches_host_stdout_and_exit_code() {
    // given
    let name = "static_local_complex_real_persists_across_calls";
    let source = "int puts(char*); int step(void) { static double _Complex z = 2.0; double *raw = (double *)&z; raw[0] = raw[0] + 1.0; return raw[0] == 3.0 ? 1 : 2; } int main(void) { int first = step(); int second = step(); puts(\"static-complex-real\"); return first == 1 && second == 2 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn static_local_complex_imaginary_lane_persists_matches_host_stdout_and_exit_code() {
    // given
    let name = "static_local_complex_imaginary_lane_persists";
    let source = "int puts(char*); int step(void) { static double _Complex z; double *raw = (double *)&z; raw[1] = raw[1] + 2.0; return raw[1] == 2.0 ? 1 : 2; } int main(void) { int first = step(); int second = step(); puts(\"static-complex-imag\"); return first == 1 && second == 2 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}
