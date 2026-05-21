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
fn scalar_int_compound_literal_address_initializer_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { int *p = &(int){ 7 }; puts(\"scalar-compound-int-address-init\"); return *p == 7 ? 0 : 1; }\n";

    // when/then
    assert_case("scalar_int_compound_literal_address_initializer", source);
}

#[test]
fn scalar_int_compound_literal_address_assignment_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { int *p; p = &(int){ 11 }; puts(\"scalar-compound-int-address-assign\"); return *p == 11 ? 0 : 1; }\n";

    // when/then
    assert_case("scalar_int_compound_literal_address_assignment", source);
}

#[test]
fn scalar_bool_compound_literal_address_conversion_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { _Bool *p = &(_Bool){ 9 }; puts(\"scalar-compound-bool-address\"); return *p ? 0 : 1; }\n";

    // when/then
    assert_case("scalar_bool_compound_literal_address_conversion", source);
}

#[test]
fn scalar_short_compound_literal_address_conversion_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { short *p = &(short){ 65535 }; puts(\"scalar-compound-short-address\"); return *p; }\n";

    // when/then
    assert_case("scalar_short_compound_literal_address_conversion", source);
}

#[test]
fn scalar_char_compound_literal_sizeof_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { puts(\"scalar-compound-char-sizeof\"); return sizeof((char){ 1 }); }\n";

    // when/then
    assert_case("scalar_char_compound_literal_sizeof", source);
}

#[test]
fn scalar_unsigned_char_compound_literal_sizeof_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { puts(\"scalar-compound-uchar-sizeof\"); return sizeof((unsigned char){ 1 }); }\n";

    // when/then
    assert_case("scalar_unsigned_char_compound_literal_sizeof", source);
}

#[test]
fn scalar_short_compound_literal_sizeof_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { puts(\"scalar-compound-short-sizeof\"); return sizeof((short){ 1 }); }\n";

    // when/then
    assert_case("scalar_short_compound_literal_sizeof", source);
}

#[test]
fn scalar_complex_compound_literal_address_preserves_lanes_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex z = 2.0; double *zp = (double *)&z; zp[1] = 3.0; double _Complex *p = &(double _Complex){ z }; double *raw = (double *)p; puts(\"scalar-compound-complex-address\"); return raw[0] == 2.0 && raw[1] == 3.0 ? 0 : 1; }\n";

    // when/then
    assert_case(
        "scalar_complex_compound_literal_address_preserves_lanes",
        source,
    );
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

#[test]
fn complex_addition_equality_compares_imaginary_lane_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex a = 1.0; double _Complex b = 2.0; double _Complex c = 3.0; double *ap = (double *)&a; double *bp = (double *)&b; double *cp = (double *)&c; ap[1] = 4.0; bp[1] = 5.0; cp[1] = 8.0; puts(\"complex-add-eq\"); return (a + b) == c ? 1 : 0; }\n";

    // when/then
    assert_case("complex_addition_equality_compares_imaginary_lane", source);
}

#[test]
fn complex_addition_inequality_compares_imaginary_lane_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex a = 2.0; double _Complex b = 3.0; double _Complex c = 5.0; double *ap = (double *)&a; double *bp = (double *)&b; double *cp = (double *)&c; ap[1] = 7.0; bp[1] = 11.0; cp[1] = 18.0; puts(\"complex-add-ne\"); return (a + b) != c ? 1 : 0; }\n";

    // when/then
    assert_case(
        "complex_addition_inequality_compares_imaginary_lane",
        source,
    );
}

#[test]
fn complex_multiplication_equality_compares_both_lanes_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex a = 1.0; double _Complex b = 3.0; double _Complex c = -5.0; double *ap = (double *)&a; double *bp = (double *)&b; double *cp = (double *)&c; ap[1] = 2.0; bp[1] = 4.0; cp[1] = 10.0; puts(\"complex-mul-eq\"); return (a * b) == c ? 0 : 1; }\n";

    // when/then
    assert_case(
        "complex_multiplication_equality_compares_both_lanes",
        source,
    );
}

#[test]
fn complex_arithmetic_condition_uses_imaginary_lane_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex a = 1.0; double _Complex b = -1.0; double *ap = (double *)&a; double *bp = (double *)&b; ap[1] = 6.0; bp[1] = -2.0; puts(\"complex-arith-if\"); if (a + b) return 0; return 1; }\n";

    // when/then
    assert_case("complex_arithmetic_condition_uses_imaginary_lane", source);
}

#[test]
fn complex_arithmetic_bool_cast_uses_imaginary_lane_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex a = 2.0; double _Complex b = -2.0; double *ap = (double *)&a; double *bp = (double *)&b; ap[1] = 9.0; bp[1] = -3.0; _Bool truth = a + b; puts(\"complex-arith-bool\"); return truth ? 0 : 1; }\n";

    // when/then
    assert_case("complex_arithmetic_bool_cast_uses_imaginary_lane", source);
}

#[test]
fn complex_unary_minus_equality_compares_imaginary_lane_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex a = 4.0; double _Complex b = -4.0; double *ap = (double *)&a; double *bp = (double *)&b; ap[1] = -7.0; bp[1] = 6.0; puts(\"complex-unary-eq\"); return (-a) == b ? 1 : 0; }\n";

    // when/then
    assert_case(
        "complex_unary_minus_equality_compares_imaginary_lane",
        source,
    );
}

#[test]
fn complex_division_equality_compares_both_lanes_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex a = 4.0; double _Complex b = 1.0; double _Complex c = 3.0; double *ap = (double *)&a; double *bp = (double *)&b; double *cp = (double *)&c; ap[1] = 2.0; bp[1] = 1.0; cp[1] = -1.0; puts(\"complex-div-eq\"); return (a / b) == c ? 0 : 1; }\n";

    // when/then
    assert_case("complex_division_equality_compares_both_lanes", source);
}

#[test]
fn complex_expression_to_expression_equality_uses_all_lanes_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex a = 1.0; double _Complex b = 2.0; double _Complex c = 4.0; double _Complex d = -1.0; double *ap = (double *)&a; double *bp = (double *)&b; double *cp = (double *)&c; double *dp = (double *)&d; ap[1] = 8.0; bp[1] = -3.0; cp[1] = 2.0; dp[1] = 3.0; puts(\"complex-expr-expr-eq\"); return (a + b) == (c + d) ? 0 : 1; }\n";

    // when/then
    assert_case(
        "complex_expression_to_expression_equality_uses_all_lanes",
        source,
    );
}

#[test]
fn complex_cast_equality_zeroes_imaginary_lane_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex z = 7.0; double *zp = (double *)&z; zp[1] = 1.0; puts(\"complex-cast-eq\"); return ((double _Complex)7.0) == z ? 1 : 0; }\n";

    // when/then
    assert_case("complex_cast_equality_zeroes_imaginary_lane", source);
}

#[test]
fn complex_arithmetic_logical_not_uses_imaginary_lane_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex a = 5.0; double _Complex b = -5.0; double *ap = (double *)&a; double *bp = (double *)&b; ap[1] = 4.0; bp[1] = -1.0; puts(\"complex-arith-not\"); return !(a + b) ? 1 : 0; }\n";

    // when/then
    assert_case("complex_arithmetic_logical_not_uses_imaginary_lane", source);
}

#[test]
fn complex_plus_real_preserves_imaginary_lane_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex z = 2.0; double *zp = (double *)&z; zp[1] = 7.0; double _Complex sum = z + 5.0; double *sp = (double *)&sum; puts(\"complex-plus-real\"); return sp[0] == 7.0 && sp[1] == 7.0 ? 0 : 1; }\n";

    // when/then
    assert_case("complex_plus_real_preserves_imaginary_lane", source);
}

#[test]
fn real_plus_complex_preserves_imaginary_lane_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex z = 4.0; double *zp = (double *)&z; zp[1] = -6.0; double _Complex sum = 3.0 + z; double *sp = (double *)&sum; puts(\"real-plus-complex\"); return sp[0] == 7.0 && sp[1] == -6.0 ? 0 : 1; }\n";

    // when/then
    assert_case("real_plus_complex_preserves_imaginary_lane", source);
}

#[test]
fn complex_times_real_scales_imaginary_lane_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex z = 3.0; double *zp = (double *)&z; zp[1] = 4.0; double _Complex product = z * 2.0; double *pp = (double *)&product; puts(\"complex-times-real\"); return pp[0] == 6.0 && pp[1] == 8.0 ? 0 : 1; }\n";

    // when/then
    assert_case("complex_times_real_scales_imaginary_lane", source);
}

#[test]
fn real_minus_complex_negates_imaginary_lane_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex z = 2.0; double *zp = (double *)&z; zp[1] = 5.0; double _Complex diff = 9.0 - z; double *dp = (double *)&diff; puts(\"real-minus-complex\"); return dp[0] == 7.0 && dp[1] == -5.0 ? 0 : 1; }\n";

    // when/then
    assert_case("real_minus_complex_negates_imaginary_lane", source);
}

#[test]
fn complex_equals_real_requires_zero_imaginary_lane_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex z = 3.0; double *zp = (double *)&z; zp[1] = 1.0; puts(\"complex-eq-real\"); return z == 3.0 ? 1 : 0; }\n";

    // when/then
    assert_case("complex_equals_real_requires_zero_imaginary_lane", source);
}
