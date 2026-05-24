use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn local_struct_array_element_field_designator_continues_inside_array_matches_host_stdout_and_exit_code()
 {
    // given
    let source = "int puts(char*); typedef struct { int x; int y; } pair_t; typedef struct { pair_t items[2]; int tail; } box_t; int main(void) { box_t box = { .items[0].y = 5, 7 }; puts(\"mixed-local-array-field\"); return box.items[0].x == 0 && box.items[0].y == 5 && box.items[1].x == 7 && box.items[1].y == 0 && box.tail == 0 ? 0 : 1; }\n";

    // when/then
    assert_case(
        "local_struct_array_element_field_designator_continues_inside_array",
        source,
    );
}

#[test]
fn local_struct_array_last_element_field_designator_continues_to_tail_matches_host_stdout_and_exit_code()
 {
    // given
    let source = "int puts(char*); typedef struct { int x; int y; } pair_t; typedef struct { pair_t items[2]; int tail; } box_t; int main(void) { box_t box = { .items[1].y = 9, 10 }; puts(\"mixed-local-array-tail\"); return box.items[0].x == 0 && box.items[1].x == 0 && box.items[1].y == 9 && box.tail == 10 ? 0 : 1; }\n";

    // when/then
    assert_case(
        "local_struct_array_last_element_field_designator_continues_to_tail",
        source,
    );
}

#[test]
fn global_struct_array_element_field_designator_continues_like_clang_matches_host_stdout_and_exit_code()
 {
    // given
    let source = "int puts(char*); typedef struct { int x; int y; } pair_t; typedef struct { pair_t items[2]; int tail; } box_t; box_t box = { .items[0].y = 5, 7 }; int main(void) { puts(\"mixed-global-array-field\"); return box.items[0].x == 0 && box.items[0].y == 5 && box.items[1].x == 7 && box.items[1].y == 0 && box.tail == 0 ? 0 : 1; }\n";

    // when/then
    assert_case(
        "global_struct_array_element_field_designator_continues_like_clang",
        source,
    );
}

#[test]
fn compound_literal_struct_array_element_field_designator_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { int x; int y; } pair_t; typedef struct { pair_t items[2]; int tail; } box_t; int main(void) { box_t box = (box_t){ .items[1].y = 9, 10 }; puts(\"mixed-compound-array-field\"); return box.items[0].x == 0 && box.items[1].x == 0 && box.items[1].y == 9 && box.tail == 10 ? 0 : 1; }\n";

    // when/then
    assert_case(
        "compound_literal_struct_array_element_field_designator",
        source,
    );
}
