use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn complex_equality_compares_imaginary_lane_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex a = 1.0; double _Complex b = 1.0; double *ap = (double *)&a; double *bp = (double *)&b; ap[1] = 2.0; bp[1] = 3.0; puts(\"complex-eq-imag\"); return (a == b) ? 1 : 0; }\n";

    // when/then
    assert_case("complex_equality_compares_imaginary_lane", source);
}

#[test]
fn complex_inequality_compares_imaginary_lane_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex a = 4.0; double _Complex b = 4.0; double *ap = (double *)&a; double *bp = (double *)&b; ap[1] = 5.0; bp[1] = 6.0; puts(\"complex-ne-imag\"); return (a != b) ? 0 : 1; }\n";

    // when/then
    assert_case("complex_inequality_compares_imaginary_lane", source);
}

#[test]
fn complex_logical_and_uses_imaginary_lane_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex z = 0.0; double *zp = (double *)&z; zp[1] = 9.0; puts(\"complex-and-imag\"); return (z && 1) ? 0 : 1; }\n";

    // when/then
    assert_case("complex_logical_and_uses_imaginary_lane", source);
}

#[test]
fn complex_logical_or_uses_imaginary_lane_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex z = 0.0; double *zp = (double *)&z; zp[1] = 7.0; puts(\"complex-or-imag\"); return (z || 0) ? 0 : 1; }\n";

    // when/then
    assert_case("complex_logical_or_uses_imaginary_lane", source);
}

#[test]
fn scalar_int_compound_literal_value_matches_host_stdout_and_exit_code() {
    // given
    let source =
        "int puts(char*); int main(void) { puts(\"scalar-compound-int\"); return (int){ 17 }; }\n";

    // when/then
    assert_case("scalar_int_compound_literal_value", source);
}

#[test]
fn scalar_bool_compound_literal_conversion_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { _Bool b = (_Bool){ 7 }; puts(\"scalar-compound-bool\"); return b ? 0 : 1; }\n";

    // when/then
    assert_case("scalar_bool_compound_literal_conversion", source);
}

#[test]
fn scalar_double_compound_literal_value_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double d = (double){ 2.5 }; puts(\"scalar-compound-double\"); return d == 2.5 ? 0 : 1; }\n";

    // when/then
    assert_case("scalar_double_compound_literal_value", source);
}

#[test]
fn scalar_complex_compound_literal_sizeof_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { puts(\"scalar-compound-complex-sizeof\"); return sizeof((double _Complex){ 3.0 }); }\n";

    // when/then
    assert_case("scalar_complex_compound_literal_sizeof", source);
}

#[test]
fn scalar_complex_compound_literal_zeroes_imaginary_lane_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex z = (double _Complex){ 3.0 }; double *zp = (double *)&z; puts(\"scalar-compound-complex-zero\"); return (int)zp[0] + (int)zp[1]; }\n";

    // when/then
    assert_case(
        "scalar_complex_compound_literal_zeroes_imaginary_lane",
        source,
    );
}

#[test]
fn complex_array_compound_literal_designator_preserves_imaginary_lane_matches_host_stdout_and_exit_code()
 {
    // given
    let source = "int puts(char*); int main(void) { double _Complex a = 1.0; double *ap = (double *)&a; ap[1] = 8.0; double _Complex *items = (double _Complex[3]){ [2] = a }; double *raw = (double *)&items[2]; puts(\"complex-array-designator\"); return (int)raw[0] + (int)raw[1]; }\n";

    // when/then
    assert_case(
        "complex_array_compound_literal_designator_preserves_imaginary_lane",
        source,
    );
}

#[test]
fn complex_flexible_array_member_assignment_preserves_imaginary_lane_matches_host_stdout_and_exit_code()
 {
    // given
    let source = "int puts(char*); void *malloc(int); typedef struct { int length; double _Complex values[]; } bag_t; int main(void) { bag_t *bag = (bag_t*)malloc(sizeof(bag_t) + 2 * sizeof(double _Complex)); double _Complex a = 2.0; double *ap = (double *)&a; ap[1] = 5.0; bag->values[1] = a; double *raw = (double *)bag->values; puts(\"flex-complex-assign\"); return (int)raw[2] + (int)raw[3]; }\n";

    // when/then
    assert_case(
        "complex_flexible_array_member_assignment_preserves_imaginary_lane",
        source,
    );
}

#[test]
fn complex_flexible_array_member_arithmetic_preserves_imaginary_lane_matches_host_stdout_and_exit_code()
 {
    // given
    let source = "int puts(char*); void *malloc(int); typedef struct { int length; double _Complex values[]; } bag_t; int main(void) { bag_t *bag = (bag_t*)malloc(sizeof(bag_t) + sizeof(double _Complex)); double _Complex a = 1.0; double _Complex b = 3.0; double *ap = (double *)&a; double *bp = (double *)&b; ap[1] = 2.0; bp[1] = 4.0; bag->values[0] = a + b; double *raw = (double *)bag->values; puts(\"flex-complex-add\"); return (int)raw[0] + (int)raw[1]; }\n";

    // when/then
    assert_case(
        "complex_flexible_array_member_arithmetic_preserves_imaginary_lane",
        source,
    );
}
