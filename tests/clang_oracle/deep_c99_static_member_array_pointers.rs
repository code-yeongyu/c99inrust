use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn global_struct_array_member_element_pointer_initializer_matches_host_stdout_and_exit_code() {
    // given
    let name = "global_struct_array_member_element_pointer_initializer";
    let source = "int puts(char*); typedef struct { int values[3]; } bag_t; bag_t bag = { { 4, 5, 6 } }; int *ptr = &bag.values[1]; int main(void) { puts(\"global-member-array-pointer\"); return *ptr == 5 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn global_struct_array_member_element_pointer_array_initializer_matches_host_stdout_and_exit_code()
{
    // given
    let name = "global_struct_array_member_element_pointer_array_initializer";
    let source = "int puts(char*); typedef struct { int values[3]; } bag_t; bag_t bags[2] = { { { 1, 2, 3 } }, { { 4, 5, 6 } } }; int *items[] = { &bags[0].values[2], &bags[1].values[0], 0 }; int main(void) { puts(\"global-member-array-pointer-array\"); return *items[0] == 3 && *items[1] == 4 && items[2] == 0 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn static_local_struct_array_member_element_pointer_initializer_matches_host_stdout_and_exit_code()
{
    // given
    let name = "static_local_struct_array_member_element_pointer_initializer";
    let source = "int puts(char*); typedef struct { int values[3]; } bag_t; bag_t bag = { { 8, 9, 10 } }; int main(void) { static int *ptr = &bag.values[2]; puts(\"static-member-array-pointer\"); return *ptr == 10 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}
