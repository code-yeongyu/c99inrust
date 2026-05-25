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

#[test]
fn struct_compound_literal_assignment_zero_fills_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { int x; int y; } pair_t; int main(void) { pair_t p = { 9, 8 }; p = (pair_t){ .y = 4 }; puts(\"compound-assign-zero\"); return p.x == 0 && p.y == 4 ? 0 : 1; }\n";

    // when/then
    assert_case("struct_compound_literal_assignment_zero_fills", source);
}

#[test]
fn nested_struct_member_compound_literal_assignment_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { int x; int y; } pair_t; typedef struct { int tag; pair_t pair; } box_t; int main(void) { box_t box = { 11, { 9, 8 } }; box.pair = (pair_t){ .y = 5 }; puts(\"compound-member-assign\"); return box.tag == 11 && box.pair.x == 0 && box.pair.y == 5 ? 0 : 1; }\n";

    // when/then
    assert_case("nested_struct_member_compound_literal_assignment", source);
}

#[test]
fn struct_compound_literal_assignment_array_field_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { int values[3]; } bucket_t; int main(void) { bucket_t bucket = { { 1, 2, 3 } }; bucket = (bucket_t){ .values = { [1] = 7 } }; puts(\"compound-assign-array\"); return bucket.values[0] == 0 && bucket.values[1] == 7 && bucket.values[2] == 0 ? 0 : 1; }\n";

    // when/then
    assert_case("struct_compound_literal_assignment_array_field", source);
}

#[test]
fn struct_compound_literal_assignment_complex_field_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { double _Complex z; int tag; } box_t; int main(void) { box_t box = { 0.0, 99 }; double _Complex z = 3.0; double *zp = (double *)&z; zp[1] = 4.0; box = (box_t){ z, 6 }; double *raw = (double *)&box.z; puts(\"compound-assign-complex\"); return raw[0] == 3.0 && raw[1] == 4.0 && box.tag == 6 ? 0 : 1; }\n";

    // when/then
    assert_case("struct_compound_literal_assignment_complex_field", source);
}

#[test]
fn global_struct_compound_literal_assignment_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { int x; int y; } pair_t; pair_t global = { 9, 8 }; int main(void) { global = (pair_t){ .y = 6 }; puts(\"compound-global-assign\"); return global.x == 0 && global.y == 6 ? 0 : 1; }\n";

    // when/then
    assert_case("global_struct_compound_literal_assignment", source);
}

#[test]
fn pointer_struct_compound_literal_assignment_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { int x; int y; } pair_t; int main(void) { pair_t value = { 9, 8 }; pair_t *p = &value; p[0] = (pair_t){ .x = 7 }; puts(\"compound-pointer-assign\"); return value.x == 7 && value.y == 0 ? 0 : 1; }\n";

    // when/then
    assert_case("pointer_struct_compound_literal_assignment", source);
}

#[test]
fn pointer_member_struct_compound_literal_assignment_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { int x; int y; } pair_t; typedef struct { int tag; pair_t pair; } box_t; int main(void) { box_t box = { 3, { 9, 8 } }; box_t *p = &box; p->pair = (pair_t){ .x = 2 }; puts(\"compound-pointer-member\"); return box.tag == 3 && box.pair.x == 2 && box.pair.y == 0 ? 0 : 1; }\n";

    // when/then
    assert_case("pointer_member_struct_compound_literal_assignment", source);
}

#[test]
fn struct_array_field_compound_literal_assignment_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { int x; int y; } pair_t; typedef struct { pair_t items[2]; } box_t; int main(void) { box_t box = { { { 1, 2 }, { 3, 4 } } }; box.items[1] = (pair_t){ .y = 9 }; puts(\"compound-struct-array-field\"); return box.items[0].x == 1 && box.items[1].x == 0 && box.items[1].y == 9 ? 0 : 1; }\n";

    // when/then
    assert_case("struct_array_field_compound_literal_assignment", source);
}

#[test]
fn array_compound_literal_element_address_initializer_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { int *p = &(int[3]){ 4, 5, 6 }[1]; puts(\"compound-element-address-init\"); return *p == 5 ? 0 : 1; }\n";

    // when/then
    assert_case("array_compound_literal_element_address_initializer", source);
}

#[test]
fn array_compound_literal_element_address_add_negative_initializer_matches_host_stdout_and_exit_code()
 {
    // given
    let source = "int puts(char*); int main(void) { int *p = &(int[3]){ 4, 5, 6 }[1] + -1; puts(\"compound-element-address-add-negative-init\"); return p[0] == 4 && p[1] == 5 ? 0 : 1; }\n";

    // when/then
    assert_case(
        "array_compound_literal_element_address_add_negative_initializer",
        source,
    );
}

#[test]
fn array_compound_literal_element_address_assignment_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { unsigned char *p; p = &(unsigned char[3]){ 250, 4, 6 }[0]; puts(\"compound-element-address-assign\"); return p[0] + p[1] == 254 ? 0 : 1; }\n";

    // when/then
    assert_case("array_compound_literal_element_address_assignment", source);
}

#[test]
fn array_compound_literal_element_address_subtract_assignment_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { unsigned char *p; p = &(unsigned char[3]){ 250, 4, 6 }[2] - 1; puts(\"compound-element-address-subtract-assign\"); return p[0] == 4 && p[1] == 6 ? 0 : 1; }\n";

    // when/then
    assert_case(
        "array_compound_literal_element_address_subtract_assignment",
        source,
    );
}

#[test]
fn complex_array_compound_literal_element_address_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double _Complex z = 3.0; double *zp = (double *)&z; zp[1] = 4.0; double _Complex *p = &(double _Complex[2]){ 0.0, z }[1]; double *raw = (double *)p; puts(\"compound-complex-element-address\"); return raw[0] == 3.0 && raw[1] == 4.0 ? 0 : 1; }\n";

    // when/then
    assert_case("complex_array_compound_literal_element_address", source);
}

#[test]
fn pointer_array_compound_literal_element_address_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { char **p = &(char *[2]){ \"a\", \"bc\" }[1]; puts(\"compound-pointer-element-address\"); return (*p)[1] == 'c' ? 0 : 1; }\n";

    // when/then
    assert_case("pointer_array_compound_literal_element_address", source);
}

#[test]
fn struct_compound_literal_member_address_initializer_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { int x; int y; } pair_t; int main(void) { int *p = &((pair_t){ 3, 4 }).y; puts(\"compound-member-address-init\"); return *p == 4 ? 0 : 1; }\n";

    // when/then
    assert_case("struct_compound_literal_member_address_initializer", source);
}

#[test]
fn struct_compound_literal_member_address_assignment_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { int x; int y; } pair_t; int main(void) { int *p; p = &((pair_t){ .x = 8, .y = 9 }).x; puts(\"compound-member-address-assign\"); return *p == 8 ? 0 : 1; }\n";

    // when/then
    assert_case("struct_compound_literal_member_address_assignment", source);
}

#[test]
fn complex_struct_compound_literal_member_address_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { double _Complex z; int tag; } box_t; int main(void) { double _Complex z = 5.0; double *zp = (double *)&z; zp[1] = 6.0; double _Complex *p = &((box_t){ z, 2 }).z; double *raw = (double *)p; puts(\"compound-complex-member-address\"); return raw[0] == 5.0 && raw[1] == 6.0 ? 0 : 1; }\n";

    // when/then
    assert_case("complex_struct_compound_literal_member_address", source);
}

#[test]
fn pointer_struct_compound_literal_member_address_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { char *text; int tag; } box_t; int main(void) { char **p = &((box_t){ \"az\", 7 }).text; puts(\"compound-pointer-member-address\"); return (*p)[1] == 'z' ? 0 : 1; }\n";

    // when/then
    assert_case("pointer_struct_compound_literal_member_address", source);
}
