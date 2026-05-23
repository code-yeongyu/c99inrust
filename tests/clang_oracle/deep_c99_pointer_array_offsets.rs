use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn global_string_pointer_array_offsets_match_host_stdout_and_exit_code() {
    // given
    let name = "global_string_pointer_array_offsets";
    let source = "int puts(char*); char *items[] = { \"alpha\" + 2, &\"doom\"[3] - 1 }; int main(void) { puts(\"global-string-array-offsets\"); return items[0][0] == 'p' && items[1][0] == 'o' && items[1][1] == 'm' ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn static_global_string_pointer_array_offsets_match_host_stdout_and_exit_code() {
    // given
    let name = "static_global_string_pointer_array_offsets";
    let source = "int puts(char*); static char *items[3] = { \"left\" + 1, 0, &\"right\"[4] - 2 }; int main(void) { puts(\"static-global-string-array-offsets\"); return items[0][0] == 'e' && items[1] == 0 && items[2][0] == 'g' ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn global_string_pointer_array_explicit_length_zero_fills_tail() {
    // given
    let name = "global_string_pointer_array_explicit_length_zero_fills_tail";
    let source = "int puts(char*); char *items[4] = { \"ab\" + 1, 0 }; int main(void) { puts(\"global-string-array-fill\"); return sizeof(items) == 32 && items[0][0] == 'b' && items[1] == 0 && items[2] == 0 && items[3] == 0 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}
