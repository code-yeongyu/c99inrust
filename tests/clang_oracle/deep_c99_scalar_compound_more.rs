use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn scalar_int_compound_literal_value_matches_host_stdout_and_exit_code() {
    // given
    let source =
        "int puts(char*); int main(void) { puts(\"scalar-compound-int\"); return (int){ 17 }; }\n";

    // when/then
    assert_case("scalar_int_compound_literal_value", source);
}

#[test]
fn scalar_bool_compound_literal_conversion_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { _Bool b = (_Bool){ 7 }; puts(\"scalar-compound-bool\"); return b ? 0 : 1; }\n";

    // when/then
    assert_case("scalar_bool_compound_literal_conversion", source);
}

#[test]
fn scalar_double_compound_literal_value_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double d = (double){ 2.5 }; puts(\"scalar-compound-double\"); return d == 2.5 ? 0 : 1; }\n";

    // when/then
    assert_case("scalar_double_compound_literal_value", source);
}

#[test]
fn scalar_int_compound_literal_address_initializer_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { int *p = &(int){ 7 }; puts(\"scalar-compound-int-address-init\"); return *p == 7 ? 0 : 1; }\n";

    // when/then
    assert_case("scalar_int_compound_literal_address_initializer", source);
}

#[test]
fn scalar_int_compound_literal_address_assignment_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { int *p; p = &(int){ 11 }; puts(\"scalar-compound-int-address-assign\"); return *p == 11 ? 0 : 1; }\n";

    // when/then
    assert_case("scalar_int_compound_literal_address_assignment", source);
}

#[test]
fn scalar_bool_compound_literal_address_conversion_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { _Bool *p = &(_Bool){ 9 }; puts(\"scalar-compound-bool-address\"); return *p ? 0 : 1; }\n";

    // when/then
    assert_case("scalar_bool_compound_literal_address_conversion", source);
}

#[test]
fn scalar_short_compound_literal_address_conversion_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { short *p = &(short){ 65535 }; puts(\"scalar-compound-short-address\"); return *p; }\n";

    // when/then
    assert_case("scalar_short_compound_literal_address_conversion", source);
}

#[test]
fn scalar_char_compound_literal_sizeof_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { puts(\"scalar-compound-char-sizeof\"); return sizeof((char){ 1 }); }\n";

    // when/then
    assert_case("scalar_char_compound_literal_sizeof", source);
}

#[test]
fn scalar_unsigned_char_compound_literal_sizeof_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { puts(\"scalar-compound-uchar-sizeof\"); return sizeof((unsigned char){ 1 }); }\n";

    // when/then
    assert_case("scalar_unsigned_char_compound_literal_sizeof", source);
}

#[test]
fn scalar_short_compound_literal_sizeof_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { puts(\"scalar-compound-short-sizeof\"); return sizeof((short){ 1 }); }\n";

    // when/then
    assert_case("scalar_short_compound_literal_sizeof", source);
}

#[test]
fn scalar_int_compound_literal_assignment_expression_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { int saved = ((int){ 1 } = 9); puts(\"scalar-compound-int-assign-expr\"); return saved == 9 ? 0 : 1; }\n";

    // when/then
    assert_case("scalar_int_compound_literal_assignment_expression", source);
}

#[test]
fn scalar_bool_compound_literal_assignment_converts_rhs_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { _Bool saved = ((_Bool){ 0 } = 42); puts(\"scalar-compound-bool-assign\"); return saved ? 0 : 1; }\n";

    // when/then
    assert_case(
        "scalar_bool_compound_literal_assignment_converts_rhs",
        source,
    );
}

#[test]
fn scalar_compound_literal_assignment_evaluates_initializer_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int hits = 0; int bump(void) { hits = hits + 1; return 4; } int main(void) { (int){ bump() } = 7; puts(\"scalar-compound-assign-init\"); return hits == 1 ? 0 : 1; }\n";

    // when/then
    assert_case(
        "scalar_compound_literal_assignment_evaluates_initializer",
        source,
    );
}

#[test]
fn scalar_unsigned_char_compound_literal_assignment_narrows_rhs_matches_host_stdout_and_exit_code()
{
    // given
    let source = "int puts(char*); int main(void) { int saved = ((unsigned char){ 1 } = 300); puts(\"scalar-compound-uchar-assign\"); return saved == 44 ? 0 : 1; }\n";

    // when/then
    assert_case(
        "scalar_unsigned_char_compound_literal_assignment_narrows_rhs",
        source,
    );
}

#[test]
fn scalar_short_compound_literal_assignment_narrows_rhs_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { int saved = ((short){ 0 } = 65535); puts(\"scalar-compound-short-assign\"); return saved == -1 ? 0 : 1; }\n";

    // when/then
    assert_case(
        "scalar_short_compound_literal_assignment_narrows_rhs",
        source,
    );
}
