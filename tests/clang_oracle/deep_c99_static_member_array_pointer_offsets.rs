use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn global_member_array_pointer_offset_initializer_matches_host_stdout_and_exit_code() {
    // given
    let name = "global_member_array_pointer_offset_initializer";
    let source = "int puts(char*); typedef struct { int values[4]; } bag_t; bag_t bag = { { 7, 8, 9, 10 } }; int *ptr = &bag.values[0] + 2; int main(void) { puts(\"global-member-array-pointer-offset\"); return *ptr == 9 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn global_member_array_pointer_array_offsets_match_host_stdout_and_exit_code() {
    // given
    let name = "global_member_array_pointer_array_offsets";
    let source = "int puts(char*); typedef struct { int values[4]; } bag_t; bag_t bag = { { 1, 2, 3, 4 } }; int *items[] = { &bag.values[1] + 2, &bag.values[3] - 1, 0 }; int main(void) { puts(\"global-member-array-pointer-array-offsets\"); return *items[0] == 4 && *items[1] == 3 && items[2] == 0 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn global_member_array_pointer_commuted_offset_matches_host_stdout_and_exit_code() {
    // given
    let name = "global_member_array_pointer_commuted_offset";
    let source = "int puts(char*); typedef struct { int values[4]; } bag_t; bag_t bag = { { 21, 22, 23, 24 } }; int *ptr = 2 + &bag.values[0]; int main(void) { puts(\"global-member-array-pointer-commuted\"); return *ptr == 23 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn nested_global_member_array_pointer_offset_matches_host_stdout_and_exit_code() {
    // given
    let name = "nested_global_member_array_pointer_offset";
    let source = "int puts(char*); typedef struct { int values[3]; } inner_t; typedef struct { inner_t inner; } outer_t; outer_t box = { { { 11, 12, 13 } } }; int *ptr = &box.inner.values[0] + 1; int main(void) { puts(\"nested-global-member-array-pointer-offset\"); return *ptr == 12 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}
