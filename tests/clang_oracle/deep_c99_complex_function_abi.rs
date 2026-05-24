use super::multifile_support::{
    OracleMultiFileCase, OracleSourceFile, assert_multifile_compile_run_matches_host,
};
use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn complex_double_function_return_preserves_imaginary_lane_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); double _Complex make_value(void) { double _Complex z = 4.0; double *raw = (double *)&z; raw[1] = 5.0; return z; } int main(void) { double _Complex z = make_value(); double *raw = (double *)&z; puts(\"complex-return-double\"); return raw[0] == 4.0 && raw[1] == 5.0 ? 0 : 1; }\n";

    // when/then
    assert_case(
        "complex_double_function_return_preserves_imaginary_lane",
        source,
    );
}

#[test]
fn complex_double_parameter_preserves_imaginary_lane_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int consume(double _Complex z) { double *raw = (double *)&z; return raw[0] == 6.0 && raw[1] == 7.0; } int main(void) { double _Complex z = 6.0; double *raw = (double *)&z; raw[1] = 7.0; puts(\"complex-parameter-double\"); return consume(z) ? 0 : 1; }\n";

    // when/then
    assert_case("complex_double_parameter_preserves_imaginary_lane", source);
}

#[test]
fn complex_double_parameter_return_arithmetic_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); double _Complex combine(double _Complex a, double _Complex b) { return a + b; } int main(void) { double _Complex a = 1.0; double _Complex b = 3.0; double *ap = (double *)&a; double *bp = (double *)&b; ap[1] = 2.0; bp[1] = 4.0; double _Complex z = combine(a, b); double *raw = (double *)&z; puts(\"complex-param-return-arith\"); return raw[0] == 4.0 && raw[1] == 6.0 ? 0 : 1; }\n";

    // when/then
    assert_case("complex_double_parameter_return_arithmetic", source);
}

#[test]
fn extern_complex_double_function_return_matches_host_stdout_and_exit_code() {
    // given
    let case = OracleMultiFileCase {
        name: "extern_complex_double_function_return",
        files: &[
            OracleSourceFile {
                path: "maker.c",
                source: "double _Complex make_shared(void) { double _Complex z = 8.0; double *raw = (double *)&z; raw[1] = 9.0; return z; }\n",
            },
            OracleSourceFile {
                path: "main.c",
                source: "int puts(char*); extern double _Complex make_shared(void); int main(void) { double _Complex z = make_shared(); double *raw = (double *)&z; puts(\"extern-complex-return\"); return raw[0] == 8.0 && raw[1] == 9.0 ? 0 : 1; }\n",
            },
        ],
    };

    // when/then
    assert_multifile_compile_run_matches_host(case);
}
