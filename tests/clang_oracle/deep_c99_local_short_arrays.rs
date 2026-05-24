use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn local_short_array_initializer_sign_extends_like_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { short values[3] = { 65535, 2, -3 }; puts(\"local-short-init\"); return values[0] == -1 && values[1] == 2 && values[2] == -3 ? 0 : 1; }\n";

    // when/then
    assert_case("local_short_array_initializer_sign_extends", source);
}

#[test]
fn local_unsigned_short_array_initializer_wraps_like_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { unsigned short values[3] = { 65535, 65536, -2 }; puts(\"local-ushort-init\"); return values[0] == 65535 && values[1] == 0 && values[2] == 65534 ? 0 : 1; }\n";

    // when/then
    assert_case("local_unsigned_short_array_initializer_wraps", source);
}

#[test]
fn local_short_array_partial_initializer_zero_fills_like_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { short values[4] = { 7, -8 }; puts(\"local-short-zero-fill\"); return values[0] == 7 && values[1] == -8 && values[2] == 0 && values[3] == 0 ? 0 : 1; }\n";

    // when/then
    assert_case("local_short_array_partial_initializer_zero_fills", source);
}
