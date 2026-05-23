use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn global_struct_string_pointer_field_offset_matches_host_stdout_and_exit_code() {
    // given
    let name = "global_struct_string_pointer_field_offset";
    let source = "int puts(char*); typedef struct { char *text; } holder_t; holder_t holder = { \"doom\" + 1 }; int main(void) { puts(\"struct-string-pointer-offset\"); return holder.text[0] == 'o' && holder.text[2] == 'm' ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn global_struct_string_pointer_field_subscript_offset_matches_host_stdout_and_exit_code() {
    // given
    let name = "global_struct_string_pointer_field_subscript_offset";
    let source = "int puts(char*); typedef struct { char *text; } holder_t; holder_t holder = { &\"alpha\"[1] + 2 }; int main(void) { puts(\"struct-string-pointer-subscript-offset\"); return holder.text[0] == 'h' && holder.text[1] == 'a' ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn global_struct_array_string_pointer_field_offsets_match_host_stdout_and_exit_code() {
    // given
    let name = "global_struct_array_string_pointer_field_offsets";
    let source = "int puts(char*); typedef struct { char *text; } holder_t; holder_t holders[2] = { { \"left\" + 2 }, { 1 + \"right\" } }; int main(void) { puts(\"struct-array-string-pointer-offsets\"); return holders[0].text[0] == 'f' && holders[1].text[0] == 'i' ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn global_struct_string_pointer_field_cast_offset_matches_host_stdout_and_exit_code() {
    // given
    let name = "global_struct_string_pointer_field_cast_offset";
    let source = "int puts(char*); typedef struct { char *text; } holder_t; holder_t holder = { (char *)(\"cast\" + 2) }; int main(void) { puts(\"struct-string-pointer-cast-offset\"); return holder.text[0] == 's' && holder.text[1] == 't' ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}
