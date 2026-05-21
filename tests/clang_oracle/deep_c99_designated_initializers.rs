use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn local_int_array_designators_match_host_stdout_and_exit_code() {
    // given
    let name = "local_int_array_designators";
    let source = "int puts(char*); int main(void) { int values[5] = { [3] = 9, [1] = 4, 7 }; puts(\"des-local-int\"); return values[0] == 0 && values[1] == 4 && values[2] == 7 && values[3] == 9 && values[4] == 0 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn global_int_array_designator_then_positional_matches_host_stdout_and_exit_code() {
    // given
    let name = "global_int_array_designator_then_positional";
    let source = "int puts(char*); int values[5] = { [4] = 10, [1] = 3, 6 }; int main(void) { puts(\"des-global-int\"); return values[0] == 0 && values[1] == 3 && values[2] == 6 && values[3] == 0 && values[4] == 10 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn local_char_array_designators_match_host_stdout_and_exit_code() {
    // given
    let name = "local_char_array_designators";
    let source = "int puts(char*); int main(void) { unsigned char bytes[4] = { [2] = 255, [0] = 7, 9 }; puts(\"des-local-char\"); return bytes[0] == 7 && bytes[1] == 9 && bytes[2] == 255 && bytes[3] == 0 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn local_struct_field_designators_match_host_stdout_and_exit_code() {
    // given
    let name = "local_struct_field_designators";
    let source = "int puts(char*); typedef struct { int x; int y; int z; } triple_t; int main(void) { triple_t t = { .z = 9, .x = 3 }; puts(\"des-local-struct\"); return t.x == 3 && t.y == 0 && t.z == 9 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn local_struct_designator_then_positional_matches_host_stdout_and_exit_code() {
    // given
    let name = "local_struct_designator_then_positional";
    let source = "int puts(char*); typedef struct { int x; int y; int z; } triple_t; int main(void) { triple_t t = { .y = 5, 8 }; puts(\"des-local-next\"); return t.x == 0 && t.y == 5 && t.z == 8 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn global_struct_field_designators_match_host_stdout_and_exit_code() {
    // given
    let name = "global_struct_field_designators";
    let source = "int puts(char*); typedef struct { int x; int y; int z; } triple_t; triple_t t = { .z = 11, .x = 6 }; int main(void) { puts(\"des-global-struct\"); return t.x == 6 && t.y == 0 && t.z == 11 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn global_struct_designator_then_positional_matches_host_stdout_and_exit_code() {
    // given
    let name = "global_struct_designator_then_positional";
    let source = "int puts(char*); typedef struct { int x; int y; int z; } triple_t; triple_t t = { .y = 12, 14 }; int main(void) { puts(\"des-global-next\"); return t.x == 0 && t.y == 12 && t.z == 14 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}
