use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn complex_double_multiplication_preserves_both_lanes_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex a = 2.0; double _Complex b = 4.0; double *ap = (double *)&a; double *bp = (double *)&b; ap[1] = 3.0; bp[1] = 5.0; double _Complex z = a * b; double *zp = (double *)&z; puts(\"complex-mul-double\"); return zp[0] == -7.0 && zp[1] == 22.0 ? 0 : 1; }\n";

    // when/then
    assert_case("complex_double_multiplication_preserves_both_lanes", source);
}

#[test]
fn complex_float_multiplication_preserves_both_lanes_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { float _Complex a = 3.0; float _Complex b = 2.0; float *ap = (float *)&a; float *bp = (float *)&b; ap[1] = 4.0; bp[1] = 5.0; float _Complex z = a * b; float *zp = (float *)&z; puts(\"complex-mul-float\"); return zp[0] == -14.0f && zp[1] == 23.0f ? 0 : 1; }\n";

    // when/then
    assert_case("complex_float_multiplication_preserves_both_lanes", source);
}

#[test]
fn complex_double_division_preserves_both_lanes_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex a = 4.0; double _Complex b = 1.0; double *ap = (double *)&a; double *bp = (double *)&b; ap[1] = 2.0; bp[1] = 1.0; double _Complex z = a / b; double *zp = (double *)&z; puts(\"complex-div-double\"); return zp[0] == 3.0 && zp[1] == -1.0 ? 0 : 1; }\n";

    // when/then
    assert_case("complex_double_division_preserves_both_lanes", source);
}

#[test]
fn complex_multiply_assignment_updates_both_lanes_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex a = 1.0; double _Complex b = 3.0; double *ap = (double *)&a; double *bp = (double *)&b; ap[1] = 2.0; bp[1] = 4.0; a *= b; puts(\"complex-mul-assign\"); return ap[0] == -5.0 && ap[1] == 10.0 ? 0 : 1; }\n";

    // when/then
    assert_case("complex_multiply_assignment_updates_both_lanes", source);
}

#[test]
fn complex_division_assignment_updates_both_lanes_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex a = 4.0; double _Complex b = 1.0; double *ap = (double *)&a; double *bp = (double *)&b; ap[1] = 2.0; bp[1] = 1.0; a /= b; puts(\"complex-div-assign\"); return ap[0] == 3.0 && ap[1] == -1.0 ? 0 : 1; }\n";

    // when/then
    assert_case("complex_division_assignment_updates_both_lanes", source);
}

#[test]
fn complex_conditional_initializer_preserves_imaginary_lane_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { int choose = 0; double _Complex a = 1.0; double _Complex b = 5.0; double *ap = (double *)&a; double *bp = (double *)&b; ap[1] = 2.0; bp[1] = 7.0; double _Complex z = choose ? a : b; double *zp = (double *)&z; puts(\"complex-conditional\"); return zp[0] == 5.0 && zp[1] == 7.0 ? 0 : 1; }\n";

    // when/then
    assert_case(
        "complex_conditional_initializer_preserves_imaginary_lane",
        source,
    );
}

#[test]
fn complex_comma_initializer_evaluates_once_and_preserves_imaginary_lane_matches_host_stdout_and_exit_code()
 {
    // given
    let source = "int puts(char*); int main(void) { int hits = 0; double _Complex a = 8.0; double *ap = (double *)&a; ap[1] = 9.0; double _Complex z = (hits = hits + 1, a); double *zp = (double *)&z; puts(\"complex-comma\"); return hits == 1 && zp[0] == 8.0 && zp[1] == 9.0 ? 0 : 1; }\n";

    // when/then
    assert_case(
        "complex_comma_initializer_evaluates_once_and_preserves_imaginary_lane",
        source,
    );
}

#[test]
fn complex_compound_literal_multiplication_preserves_imaginary_lane_matches_host_stdout_and_exit_code()
 {
    // given
    let source = "int puts(char*); int main(void) { double _Complex a = 2.0; double _Complex b = 4.0; double *ap = (double *)&a; double *bp = (double *)&b; ap[1] = 3.0; bp[1] = 5.0; double _Complex *items = (double _Complex[]){ 0.0, a * b }; double *raw = (double *)&items[1]; puts(\"complex-compound-mul\"); return raw[0] == -7.0 && raw[1] == 22.0 ? 0 : 1; }\n";

    // when/then
    assert_case(
        "complex_compound_literal_multiplication_preserves_imaginary_lane",
        source,
    );
}

#[test]
fn complex_unary_minus_negates_both_lanes_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex a = 2.0; double *ap = (double *)&a; ap[1] = -5.0; double _Complex z = -a; double *zp = (double *)&z; puts(\"complex-unary-minus\"); return zp[0] == -2.0 && zp[1] == 5.0 ? 0 : 1; }\n";

    // when/then
    assert_case("complex_unary_minus_negates_both_lanes", source);
}

#[test]
fn complex_bool_conversion_uses_imaginary_lane_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex z = 0.0; double *zp = (double *)&z; zp[1] = 4.0; _Bool truth = z; puts(\"complex-bool-imag\"); return truth ? 0 : 1; }\n";

    // when/then
    assert_case("complex_bool_conversion_uses_imaginary_lane", source);
}

#[test]
fn complex_logical_not_uses_imaginary_lane_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex z = 0.0; double *zp = (double *)&z; zp[1] = 3.0; puts(\"complex-not-imag\"); return !z ? 1 : 0; }\n";

    // when/then
    assert_case("complex_logical_not_uses_imaginary_lane", source);
}

#[test]
fn complex_if_condition_uses_imaginary_lane_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex z = 0.0; double *zp = (double *)&z; zp[1] = -2.0; puts(\"complex-if-imag\"); if (z) return 0; return 1; }\n";

    // when/then
    assert_case("complex_if_condition_uses_imaginary_lane", source);
}
