use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn file_scope_array_compound_literal_pointer_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int *items = (int[]){ 7, 8, 9 }; int main(void) { puts(\"global-compound-array-ptr\"); return items[0] == 7 && items[2] == 9 ? 0 : 1; }\n";

    // when/then
    assert_case("file_scope_array_compound_literal_pointer", source);
}

#[test]
fn file_scope_struct_compound_literal_address_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { int x; int y; } pair_t; pair_t *pair = &(pair_t){ .x = 11, .y = 13 }; int main(void) { puts(\"global-compound-struct-ptr\"); return pair->x == 11 && pair->y == 13 ? 0 : 1; }\n";

    // when/then
    assert_case("file_scope_struct_compound_literal_address", source);
}

#[test]
fn file_scope_char_array_compound_literal_pointer_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); char *text = (char[]){ 'o', 'k', 0 }; int main(void) { puts(text); return text[0] == 'o' && text[1] == 'k' && text[2] == 0 ? 0 : 1; }\n";

    // when/then
    assert_case("file_scope_char_array_compound_literal_pointer", source);
}

#[test]
fn file_scope_double_array_compound_literal_pointer_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); double *values = (double[]){ 1.5, 2.5 }; int main(void) { puts(\"global-compound-double-ptr\"); return values[0] == 1.5 && values[1] == 2.5 ? 0 : 1; }\n";

    // when/then
    assert_case("file_scope_double_array_compound_literal_pointer", source);
}
