use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn extra_nested_struct_alignment_offsets_match_host_stdout_and_exit_code() {
    // given
    let name = "extra_nested_struct_alignment_offsets";
    let source = "int puts(char*); typedef struct { char c; long long q; } leaf_t; typedef struct { short s; leaf_t leaves[2]; double d; char tail; } outer_t; int main(void) { outer_t item; int off0 = (int)((char *)&item.leaves[0].q - (char *)&item); int off1 = (int)((char *)&item.leaves[1].q - (char *)&item); puts(\"extra-align\"); return sizeof(outer_t) + off0 + off1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn extra_anonymous_union_in_struct_array_matches_host_stdout_and_exit_code() {
    // given
    let name = "extra_anonymous_union_in_struct_array";
    let source = "int puts(char*); typedef struct { union { struct { short lo; short hi; }; int word; }; int tag; } cell_t; int main(void) { cell_t cells[2]; cells[0].word = 0; cells[0].lo = 5; cells[0].hi = 8; cells[1].tag = 13; puts(\"extra-anon\"); return cells[0].lo + cells[0].hi + cells[1].tag; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn extra_typeof_attribute_array_lvalue_matches_host_stdout_and_exit_code() {
    // given
    let name = "extra_typeof_attribute_array_lvalue";
    let source = "int puts(char*); int main(void) { int values[3] = { 9, 11, 13 }; int *cursor = values + 2; __typeof__(*cursor) copy __attribute__((unused, aligned(8))) = *cursor + values[0]; puts(\"extra-typeof\"); return copy + sizeof(copy); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn extra_nested_initializer_partial_braces_match_host_stdout_and_exit_code() {
    // given
    let name = "extra_nested_initializer_partial_braces";
    let source = "int puts(char*); typedef struct { int row[3]; } row_t; typedef struct { row_t rows[2]; int tail; } table_t; int main(void) { table_t table = { { { { 1, 2 } }, { { 3, 4, 5 } } }, 6 }; puts(\"extra-init\"); return table.rows[0].row[2] + table.rows[1].row[2] + table.tail; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn extra_incompatible_pointer_cast_struct_overlay_matches_host_stdout_and_exit_code() {
    // given
    let name = "extra_incompatible_pointer_cast_struct_overlay";
    let source = "int puts(char*); typedef struct { int x; int y; } point_t; typedef struct { int a; int b; } pair_t; int main(void) { point_t point; pair_t *pair = (pair_t *)(void *)&point; pair->a = 14; pair->b = 15; puts(\"extra-ptrcast\"); return point.x + point.y; }\n";

    // when/then
    assert_case(name, source);
}
