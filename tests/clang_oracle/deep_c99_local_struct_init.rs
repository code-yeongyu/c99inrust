use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn flat_local_struct_initializer_matches_host_exit_code() {
    // given
    let source = "typedef struct { int x; int y; } pair_t; int main(void) { pair_t p = { 2, 3 }; return p.x + p.y == 5 ? 0 : 1; }\n";

    // when/then
    assert_case("flat_local_struct_initializer", source);
}

#[test]
fn partial_local_struct_initializer_zero_fills_tail_matches_host_exit_code() {
    // given
    let source = "typedef struct { int x; int y; int z; } triple_t; int main(void) { triple_t t = { 7 }; return t.x == 7 && t.y == 0 && t.z == 0 ? 0 : 1; }\n";

    // when/then
    assert_case("partial_local_struct_initializer_zero_fills_tail", source);
}

#[test]
fn nested_local_struct_initializer_respects_braces_matches_host_exit_code() {
    // given
    let source = "typedef struct { int x; int y; } inner_t; typedef struct { inner_t inner; int tail; } outer_t; int main(void) { outer_t o = { { 4 }, 5 }; return o.inner.x == 4 && o.inner.y == 0 && o.tail == 5 ? 0 : 1; }\n";

    // when/then
    assert_case("nested_local_struct_initializer_respects_braces", source);
}

#[test]
fn unbraced_nested_local_struct_initializer_spills_matches_host_exit_code() {
    // given
    let source = "typedef struct { int x; int y; } inner_t; typedef struct { inner_t inner; int tail; } outer_t; int main(void) { outer_t o = { 1, 2, 3 }; return o.inner.x == 1 && o.inner.y == 2 && o.tail == 3 ? 0 : 1; }\n";

    // when/then
    assert_case("unbraced_nested_local_struct_initializer_spills", source);
}

#[test]
fn local_struct_copy_initializer_matches_host_exit_code() {
    // given
    let source = "typedef struct { int x; int y; } pair_t; int main(void) { pair_t a = { 2, 5 }; pair_t b = a; return b.x == 2 && b.y == 5 ? 0 : 1; }\n";

    // when/then
    assert_case("local_struct_copy_initializer", source);
}

#[test]
fn multiple_local_struct_initializers_match_host_exit_code() {
    // given
    let source = "typedef struct { int x; int y; } pair_t; int main(void) { pair_t a = { 1, 2 }, b = { 3, 4 }; return a.y == 2 && b.x == 3 && b.y == 4 ? 0 : 1; }\n";

    // when/then
    assert_case("multiple_local_struct_initializers", source);
}

#[test]
fn local_struct_long_long_and_pointer_initializer_matches_host_exit_code() {
    // given
    let source = "typedef struct { long long wide; int *ptr; } cell_t; int main(void) { cell_t cell = { 0x100000002LL, 0 }; return cell.wide == 0x100000002LL && cell.ptr == 0 ? 0 : 1; }\n";

    // when/then
    assert_case("local_struct_long_long_and_pointer_initializer", source);
}

#[test]
fn local_struct_array_field_braced_initializer_matches_host_exit_code() {
    // given
    let source = "typedef struct { int values[3]; int tail; } bag_t; int main(void) { bag_t bag = { { 1, 2 }, 5 }; return bag.values[0] == 1 && bag.values[1] == 2 && bag.values[2] == 0 && bag.tail == 5 ? 0 : 1; }\n";

    // when/then
    assert_case("local_struct_array_field_braced_initializer", source);
}

#[test]
fn local_struct_array_field_unbraced_spill_matches_host_exit_code() {
    // given
    let source = "typedef struct { int values[3]; int tail; } bag_t; int main(void) { bag_t bag = { 1, 2, 3, 9 }; return bag.values[0] == 1 && bag.values[2] == 3 && bag.tail == 9 ? 0 : 1; }\n";

    // when/then
    assert_case("local_struct_array_field_unbraced_spill", source);
}

#[test]
fn local_struct_char_array_string_field_matches_host_exit_code() {
    // given
    let source = "typedef struct { char name[4]; int value; } item_t; int main(void) { item_t item = { \"abc\", 7 }; return item.name[0] == 'a' && item.name[2] == 'c' && item.name[3] == 0 && item.value == 7 ? 0 : 1; }\n";

    // when/then
    assert_case("local_struct_char_array_string_field", source);
}

#[test]
fn local_struct_matrix_field_nested_initializer_matches_host_exit_code() {
    // given
    let source = "typedef struct { int matrix[2][2]; int tail; } grid_t; int main(void) { grid_t grid = { { { 1, 2 }, { 3 } }, 8 }; return grid.matrix[0][1] == 2 && grid.matrix[1][0] == 3 && grid.matrix[1][1] == 0 && grid.tail == 8 ? 0 : 1; }\n";

    // when/then
    assert_case("local_struct_matrix_field_nested_initializer", source);
}
