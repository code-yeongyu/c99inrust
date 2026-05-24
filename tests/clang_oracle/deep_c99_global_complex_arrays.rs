use super::multifile_support::{
    OracleMultiFileCase, OracleSourceFile, assert_multifile_compile_run_matches_host,
};
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

#[test]
fn extern_global_complex_double_array_matches_host_stdout_and_exit_code() {
    // given
    let case = OracleMultiFileCase {
        name: "extern_global_complex_double_array",
        files: &[
            OracleSourceFile {
                path: "state.c",
                source: "double _Complex values[2]; void seed(void) { double _Complex z = 5.0; double *zp = (double *)&z; zp[1] = 6.0; values[1] = z; }\n",
            },
            OracleSourceFile {
                path: "main.c",
                source: "int puts(char*); extern double _Complex values[2]; void seed(void); int main(void) { seed(); double *raw = (double *)values; puts(\"extern-global-complex-array\"); return raw[0] == 0.0 && raw[1] == 0.0 && raw[2] == 5.0 && raw[3] == 6.0 ? 0 : 1; }\n",
            },
        ],
    };

    // when/then
    assert_multifile_compile_run_matches_host(case);
}

#[test]
fn initialized_global_complex_double_array_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); double _Complex values[2] = { 1.0, 2.0 }; int main(void) { double *raw = (double *)values; puts(\"initialized-global-complex-array\"); return raw[0] == 1.0 && raw[1] == 0.0 && raw[2] == 2.0 && raw[3] == 0.0 ? 0 : 1; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "initialized_global_complex_double_array",
        source,
    });
}

#[test]
fn initialized_global_double_array_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); double values[2] = { 1.0, 2.0 }; int main(void) { puts(\"initialized-global-double-array\"); return values[0] == 1.0 && values[1] == 2.0 ? 0 : 1; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "initialized_global_double_array",
        source,
    });
}

#[test]
fn unsized_initialized_global_complex_double_array_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); double _Complex values[] = { 1.0, 2.0, 3.0 }; int main(void) { double *raw = (double *)values; puts(\"unsized-initialized-global-complex-array\"); return sizeof(values) == 3 * sizeof(double _Complex) && raw[0] == 1.0 && raw[1] == 0.0 && raw[4] == 3.0 && raw[5] == 0.0 ? 0 : 1; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "unsized_initialized_global_complex_double_array",
        source,
    });
}
