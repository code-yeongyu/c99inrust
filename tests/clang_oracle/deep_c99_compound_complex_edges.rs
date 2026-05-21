use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn struct_compound_literal_array_field_subscript_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { int values[3]; } bucket_t; int main(void) { puts(\"compound-array-field\"); return ((bucket_t){ { 2, 4, 6 } }).values[1] == 4 ? 0 : 1; }\n";

    // when/then
    assert_case("struct_compound_literal_array_field_subscript", source);
}

#[test]
fn struct_compound_literal_array_field_designator_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { int values[4]; } bucket_t; int main(void) { puts(\"compound-array-designator-field\"); return ((bucket_t){ .values = { [2] = 9 } }).values[2] == 9 ? 0 : 1; }\n";

    // when/then
    assert_case("struct_compound_literal_array_field_designator", source);
}

#[test]
fn nested_struct_compound_literal_member_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { int x; int y; } pair_t; typedef struct { pair_t pair; int z; } box_t; int main(void) { puts(\"compound-nested-member\"); return ((box_t){ { 3, 8 }, 5 }).pair.y == 8 ? 0 : 1; }\n";

    // when/then
    assert_case("nested_struct_compound_literal_member", source);
}

#[test]
fn complex_divided_by_real_scales_both_lanes_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex z = 6.0; double *zp = (double *)&z; zp[1] = 8.0; double _Complex q = z / 2.0; double *qp = (double *)&q; puts(\"complex-div-real\"); return qp[0] == 3.0 && qp[1] == 4.0 ? 0 : 1; }\n";

    // when/then
    assert_case("complex_divided_by_real_scales_both_lanes", source);
}

#[test]
fn real_divided_by_complex_uses_both_lanes_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex z = 2.0; double *zp = (double *)&z; zp[1] = 4.0; double _Complex q = 20.0 / z; double *qp = (double *)&q; puts(\"real-div-complex\"); return qp[0] == 2.0 && qp[1] == -4.0 ? 0 : 1; }\n";

    // when/then
    assert_case("real_divided_by_complex_uses_both_lanes", source);
}

#[test]
fn complex_add_assign_real_preserves_imaginary_lane_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex z = 1.0; double *zp = (double *)&z; zp[1] = 5.0; z += 6.0; puts(\"complex-add-assign-real\"); return zp[0] == 7.0 && zp[1] == 5.0 ? 0 : 1; }\n";

    // when/then
    assert_case("complex_add_assign_real_preserves_imaginary_lane", source);
}

#[test]
fn flexible_array_member_double_alignment_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); void *malloc(int); typedef struct { char tag; double values[]; } bag_t; int main(void) { bag_t *bag = (bag_t*)malloc(sizeof(bag_t) + 2 * sizeof(double)); bag->values[0] = 3.0; bag->values[1] = 4.0; puts(\"flex-double-align\"); return (int)bag->values[0] + (int)bag->values[1]; }\n";

    // when/then
    assert_case("flexible_array_member_double_alignment", source);
}

#[test]
fn flexible_array_member_complex_plus_real_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); void *malloc(int); typedef struct { int length; double _Complex values[]; } bag_t; int main(void) { bag_t *bag = (bag_t*)malloc(sizeof(bag_t) + sizeof(double _Complex)); double _Complex z = 4.0; double *zp = (double *)&z; zp[1] = 6.0; bag->values[0] = z + 3.0; double *raw = (double *)bag->values; puts(\"flex-complex-real\"); return raw[0] == 7.0 && raw[1] == 6.0 ? 0 : 1; }\n";

    // when/then
    assert_case("flexible_array_member_complex_plus_real", source);
}
