use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn int_array_compound_literal_pointer_assignment_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { int *p; p = (int[]){ 11, 13, 17 }; puts(\"compound-assign-int\"); return p[0] + p[2]; }\n";

    // when/then
    assert_case("int_array_compound_literal_pointer_assignment", source);
}

#[test]
fn sized_array_compound_literal_pointer_assignment_zero_fills_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { int *p; p = (int[5]){ 2, 4, 6 }; puts(\"compound-assign-sized\"); return p[0] + p[2] + p[4]; }\n";

    // when/then
    assert_case(
        "sized_array_compound_literal_pointer_assignment_zero_fills",
        source,
    );
}

#[test]
fn char_array_compound_literal_pointer_assignment_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { char *p; p = (char[]){ 'q', 'r', 0 }; puts(\"compound-assign-char\"); return p[0] + p[1]; }\n";

    // when/then
    assert_case("char_array_compound_literal_pointer_assignment", source);
}

#[test]
fn unsigned_char_compound_literal_assignment_promotes_like_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { unsigned char *p; p = (unsigned char[]){ 250, 7 }; puts(\"compound-assign-uchar\"); return p[0] + p[1] == 257 ? 0 : 1; }\n";

    // when/then
    assert_case(
        "unsigned_char_compound_literal_assignment_promotes_like_host",
        source,
    );
}
