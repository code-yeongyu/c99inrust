use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn global_string_pointer_array_designators_zero_fill_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); char *names[3] = { [2] = \"zz\", [0] = \"aa\" }; int main(void) { puts(\"global-string-designator\"); return names[0][1] == 'a' && names[2][1] == 'z' && names[1] == 0 ? 0 : 1; }\n";

    // when/then
    assert_case("global_string_pointer_array_designators_zero_fill", source);
}

#[test]
fn global_string_pointer_array_designator_offsets_match_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); char *names[4] = { [3] = \"alpha\" + 2, [1] = \"doom\" + 1 }; int main(void) { puts(\"global-string-designator-offset\"); return names[1][0] == 'o' && names[3][0] == 'p' && names[0] == 0 && names[2] == 0 ? 0 : 1; }\n";

    // when/then
    assert_case("global_string_pointer_array_designator_offsets", source);
}

#[test]
fn global_name_pointer_array_designators_zero_fill_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int values[4] = { 3, 5, 7, 11 }; int *ptrs[3] = { [1] = values + 2, [2] = &values[3] }; int main(void) { puts(\"global-name-designator\"); return *ptrs[1] == 7 && *ptrs[2] == 11 && ptrs[0] == 0 ? 0 : 1; }\n";

    // when/then
    assert_case("global_name_pointer_array_designators_zero_fill", source);
}

#[test]
fn global_name_pointer_array_designator_subscript_offsets_match_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int values[5] = { 2, 4, 6, 8, 10 }; int *ptrs[4] = { [0] = &values[1] + 2, [3] = values + 4 }; int main(void) { puts(\"global-name-designator-offset\"); return *ptrs[0] == 8 && *ptrs[3] == 10 && ptrs[1] == 0 && ptrs[2] == 0 ? 0 : 1; }\n";

    // when/then
    assert_case("global_name_pointer_array_designator_offsets", source);
}
