use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn local_long_double_array_initializer_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { long double values[3] = { 1.0L, 2.0L }; values[2] = 3.0L; puts(\"local-long-double-array\"); return (int)(values[0] + values[1] + values[2]); }\n";

    // when/then
    assert_case("local_long_double_array_initializer", source);
}

#[test]
fn local_complex_double_array_assignment_preserves_lanes_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex values[2]; double _Complex z = 3.0; double *zp = (double *)&z; zp[1] = 4.0; values[1] = z; double *raw = (double *)values; puts(\"local-complex-array-assign\"); return raw[2] == 3.0 && raw[3] == 4.0 ? 0 : 1; }\n";

    // when/then
    assert_case(
        "local_complex_double_array_assignment_preserves_lanes",
        source,
    );
}

#[test]
fn local_complex_double_array_initializer_preserves_lanes_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex z = 5.0; double *zp = (double *)&z; zp[1] = 6.0; double _Complex values[2] = { 1.0, z }; double *raw = (double *)values; puts(\"local-complex-array-init\"); return raw[0] == 1.0 && raw[1] == 0.0 && raw[2] == 5.0 && raw[3] == 6.0 ? 0 : 1; }\n";

    // when/then
    assert_case(
        "local_complex_double_array_initializer_preserves_lanes",
        source,
    );
}
