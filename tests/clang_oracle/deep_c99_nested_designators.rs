use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn local_struct_array_field_nested_designators_match_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { int values[4]; int tail; } bucket_t; int main(void) { bucket_t b = { .values[2] = 9, .values[0] = 1, .tail = 5 }; puts(\"nested-array-designator\"); return b.values[0] == 1 && b.values[1] == 0 && b.values[2] == 9 && b.values[3] == 0 && b.tail == 5 ? 0 : 1; }\n";

    // when/then
    assert_case("local_struct_array_field_nested_designators", source);
}

#[test]
fn global_struct_array_field_nested_designators_match_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { int values[4]; int tail; } bucket_t; bucket_t b = { .values[2] = 9, .values[0] = 1, .tail = 5 }; int main(void) { puts(\"global-nested-array-designator\"); return b.values[0] == 1 && b.values[1] == 0 && b.values[2] == 9 && b.values[3] == 0 && b.tail == 5 ? 0 : 1; }\n";

    // when/then
    assert_case("global_struct_array_field_nested_designators", source);
}

#[test]
fn local_struct_field_nested_designators_match_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { int x; int y; } inner_t; typedef struct { int tag; inner_t inner; int tail; } box_t; int main(void) { box_t b = { .inner.y = 9, .inner.x = 4, .tag = 1, .tail = 2 }; puts(\"local-nested-struct-field\"); return b.tag == 1 && b.inner.x == 4 && b.inner.y == 9 && b.tail == 2 ? 0 : 1; }\n";

    // when/then
    assert_case("local_struct_field_nested_designators", source);
}

#[test]
fn global_struct_field_nested_designators_match_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { int x; int y; } inner_t; typedef struct { int tag; inner_t inner; int tail; } box_t; box_t b = { .inner.y = 9, .inner.x = 4, .tag = 1, .tail = 2 }; int main(void) { puts(\"global-nested-struct-field\"); return b.tag == 1 && b.inner.x == 4 && b.inner.y == 9 && b.tail == 2 ? 0 : 1; }\n";

    // when/then
    assert_case("global_struct_field_nested_designators", source);
}

#[test]
fn local_struct_field_nested_designator_continues_inside_nested_struct() {
    // given
    let source = "int puts(char*); typedef struct { int x; int y; int z; } inner_t; typedef struct { int tag; inner_t inner; int tail; } box_t; int main(void) { box_t b = { .inner.y = 9, 10, .tail = 7 }; puts(\"local-nested-field-continuation\"); return b.tag == 0 && b.inner.x == 0 && b.inner.y == 9 && b.inner.z == 10 && b.tail == 7 ? 0 : 1; }\n";

    // when/then
    assert_case("local_struct_field_nested_designator_continuation", source);
}

#[test]
fn global_struct_field_nested_designator_continues_inside_nested_struct() {
    // given
    let source = "int puts(char*); typedef struct { int x; int y; int z; } inner_t; typedef struct { int tag; inner_t inner; int tail; } box_t; box_t b = { .inner.y = 9, 10, .tail = 7 }; int main(void) { puts(\"global-nested-field-continuation\"); return b.tag == 0 && b.inner.x == 0 && b.inner.y == 9 && b.inner.z == 10 && b.tail == 7 ? 0 : 1; }\n";

    // when/then
    assert_case("global_struct_field_nested_designator_continuation", source);
}

#[test]
fn local_struct_array_field_designator_continues_inside_array_field() {
    // given
    let source = "int puts(char*); typedef struct { int values[4]; int tail; } bucket_t; int main(void) { bucket_t b = { .values[2] = 9, 10, .tail = 7 }; puts(\"local-array-field-continuation\"); return b.values[0] == 0 && b.values[1] == 0 && b.values[2] == 9 && b.values[3] == 10 && b.tail == 7 ? 0 : 1; }\n";

    // when/then
    assert_case("local_struct_array_field_designator_continuation", source);
}

#[test]
fn global_struct_array_field_designator_continues_inside_array_field() {
    // given
    let source = "int puts(char*); typedef struct { int values[4]; int tail; } bucket_t; bucket_t b = { .values[2] = 9, 10, .tail = 7 }; int main(void) { puts(\"global-array-field-continuation\"); return b.values[0] == 0 && b.values[1] == 0 && b.values[2] == 9 && b.values[3] == 10 && b.tail == 7 ? 0 : 1; }\n";

    // when/then
    assert_case("global_struct_array_field_designator_continuation", source);
}

#[test]
fn local_nested_struct_array_field_path_designator_continues_inside_array_field() {
    // given
    let source = "int puts(char*); typedef struct { int values[4]; int done; } inner_t; typedef struct { int tag; inner_t inner; int tail; } box_t; int main(void) { box_t b = { .inner.values[2] = 9, 10, .tail = 7 }; puts(\"local-nested-array-path-continuation\"); return b.tag == 0 && b.inner.values[0] == 0 && b.inner.values[1] == 0 && b.inner.values[2] == 9 && b.inner.values[3] == 10 && b.inner.done == 0 && b.tail == 7 ? 0 : 1; }\n";

    // when/then
    assert_case("local_nested_struct_array_field_path_continuation", source);
}

#[test]
fn global_nested_struct_array_field_path_designator_continues_inside_array_field() {
    // given
    let source = "int puts(char*); typedef struct { int values[4]; int done; } inner_t; typedef struct { int tag; inner_t inner; int tail; } box_t; box_t b = { .inner.values[2] = 9, 10, .tail = 7 }; int main(void) { puts(\"global-nested-array-path-continuation\"); return b.tag == 0 && b.inner.values[0] == 0 && b.inner.values[1] == 0 && b.inner.values[2] == 9 && b.inner.values[3] == 10 && b.inner.done == 0 && b.tail == 7 ? 0 : 1; }\n";

    // when/then
    assert_case("global_nested_struct_array_field_path_continuation", source);
}

#[test]
fn local_nested_struct_array_field_path_designator_continues_to_next_nested_field() {
    // given
    let source = "int puts(char*); typedef struct { int values[4]; int done; } inner_t; typedef struct { int tag; inner_t inner; int tail; } box_t; int main(void) { box_t b = { .inner.values[3] = 9, 10, .tail = 7 }; puts(\"local-nested-array-path-next-field\"); return b.tag == 0 && b.inner.values[0] == 0 && b.inner.values[1] == 0 && b.inner.values[2] == 0 && b.inner.values[3] == 9 && b.inner.done == 10 && b.tail == 7 ? 0 : 1; }\n";

    // when/then
    assert_case("local_nested_struct_array_field_path_next_field", source);
}

#[test]
fn global_nested_struct_array_field_path_designator_continues_to_next_nested_field() {
    // given
    let source = "int puts(char*); typedef struct { int values[4]; int done; } inner_t; typedef struct { int tag; inner_t inner; int tail; } box_t; box_t b = { .inner.values[3] = 9, 10, .tail = 7 }; int main(void) { puts(\"global-nested-array-path-next-field\"); return b.tag == 0 && b.inner.values[0] == 0 && b.inner.values[1] == 0 && b.inner.values[2] == 0 && b.inner.values[3] == 9 && b.inner.done == 10 && b.tail == 7 ? 0 : 1; }\n";

    // when/then
    assert_case("global_nested_struct_array_field_path_next_field", source);
}
