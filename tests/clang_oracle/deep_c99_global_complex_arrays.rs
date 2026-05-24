use super::support::{OracleCase, assert_compile_run_matches_host};

#[test]
fn global_complex_double_array_assignment_preserves_lanes_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); double _Complex values[2]; int main(void) { double _Complex z = 3.0; double *zp = (double *)&z; zp[1] = 4.0; values[1] = z; double *raw = (double *)values; puts(\"global-complex-array-assign\"); return raw[0] == 0.0 && raw[1] == 0.0 && raw[2] == 3.0 && raw[3] == 4.0 ? 0 : 1; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "global_complex_double_array_assignment_preserves_lanes",
        source,
    });
}

#[test]
fn sizeof_global_complex_double_array_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); double _Complex values[3]; int main(void) { puts(\"global-complex-array-sizeof\"); return sizeof(values) == 3 * sizeof(double _Complex) ? 0 : 1; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "sizeof_global_complex_double_array",
        source,
    });
}
