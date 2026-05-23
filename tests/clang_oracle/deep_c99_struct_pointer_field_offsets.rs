use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn global_struct_pointer_field_decay_offset_matches_host_stdout_and_exit_code() {
    // given
    let name = "global_struct_pointer_field_decay_offset";
    let source = "int puts(char*); int values[4] = { 5, 6, 7, 8 }; typedef struct { int *ptr; } holder_t; holder_t holder = { values + 2 }; int main(void) { puts(\"struct-pointer-field-decay-offset\"); return *holder.ptr == 7 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn global_struct_pointer_field_subscript_offset_matches_host_stdout_and_exit_code() {
    // given
    let name = "global_struct_pointer_field_subscript_offset";
    let source = "int puts(char*); int values[4] = { 11, 12, 13, 14 }; typedef struct { int *ptr; } holder_t; holder_t holder = { &values[0] + 3 }; int main(void) { puts(\"struct-pointer-field-subscript-offset\"); return *holder.ptr == 14 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn global_struct_array_pointer_field_offsets_match_host_stdout_and_exit_code() {
    // given
    let name = "global_struct_array_pointer_field_offsets";
    let source = "int puts(char*); int values[4] = { 2, 4, 6, 8 }; typedef struct { int *ptr; } holder_t; holder_t holders[2] = { { values + 1 }, { 2 + values } }; int main(void) { puts(\"struct-array-pointer-field-offsets\"); return *holders[0].ptr == 4 && *holders[1].ptr == 6 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn global_struct_pointer_field_cast_offset_matches_host_stdout_and_exit_code() {
    // given
    let name = "global_struct_pointer_field_cast_offset";
    let source = "int puts(char*); int values[4] = { 31, 32, 33, 34 }; typedef struct { int *ptr; } holder_t; holder_t holder = { (int *)(values + 1) }; int main(void) { puts(\"struct-pointer-field-cast-offset\"); return *holder.ptr == 32 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}
