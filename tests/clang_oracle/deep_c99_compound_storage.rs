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

#[test]
fn bool_array_compound_literal_pointer_assignment_normalizes_values_matches_host_stdout_and_exit_code()
 {
    // given
    let source = "int puts(char*); int main(void) { _Bool *p; p = (_Bool[]){ 2, 0, -7 }; puts(\"compound-assign-bool-array\"); return p[0] == 1 && p[1] == 0 && p[2] == 1 ? 0 : 1; }\n";

    // when/then
    assert_case(
        "bool_array_compound_literal_pointer_assignment_normalizes_values",
        source,
    );
}

#[test]
fn short_array_compound_literal_pointer_assignment_sign_extends_matches_host_stdout_and_exit_code()
{
    // given
    let source = "int puts(char*); int main(void) { short *p; p = (short[]){ 65535, 2 }; puts(\"compound-assign-short-array\"); return p[0] == -1 && p[1] == 2 ? 0 : 1; }\n";

    // when/then
    assert_case(
        "short_array_compound_literal_pointer_assignment_sign_extends",
        source,
    );
}

#[test]
fn unsigned_short_array_compound_literal_pointer_assignment_promotes_matches_host_stdout_and_exit_code()
 {
    // given
    let source = "int puts(char*); int main(void) { unsigned short *p; p = (unsigned short[]){ 65535, 1 }; puts(\"compound-assign-ushort-array\"); return p[0] == 65535 && p[1] == 1 ? 0 : 1; }\n";

    // when/then
    assert_case(
        "unsigned_short_array_compound_literal_pointer_assignment_promotes",
        source,
    );
}

#[test]
fn long_long_array_compound_literal_pointer_assignment_preserves_width_matches_host_stdout_and_exit_code()
 {
    // given
    let source = "int puts(char*); int main(void) { long long *p; p = (long long[]){ 4294967296LL, -9LL }; puts(\"compound-assign-ll-array\"); return p[0] == 4294967296LL && p[1] == -9LL ? 0 : 1; }\n";

    // when/then
    assert_case(
        "long_long_array_compound_literal_pointer_assignment_preserves_width",
        source,
    );
}

#[test]
fn string_pointer_array_compound_literal_assignment_keeps_offsets_matches_host_stdout_and_exit_code()
 {
    // given
    let source = "int puts(char*); int main(void) { char **p; p = (char *[]){ \"alpha\" + 2, \"doom\" + 1, 0 }; puts(\"compound-assign-string-pointers\"); return p[0][0] == 'p' && p[1][0] == 'o' && p[2] == 0 ? 0 : 1; }\n";

    // when/then
    assert_case(
        "string_pointer_array_compound_literal_assignment_keeps_offsets",
        source,
    );
}

#[test]
fn short_array_compound_literal_element_address_assignment_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { short *p; p = &(short[]){ 4, 5, 6 }[1]; puts(\"compound-short-element-address\"); return p[0] == 5 && p[1] == 6 ? 0 : 1; }\n";

    // when/then
    assert_case(
        "short_array_compound_literal_element_address_assignment",
        source,
    );
}
