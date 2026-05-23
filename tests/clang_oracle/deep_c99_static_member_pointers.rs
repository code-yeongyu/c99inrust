use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn global_struct_member_pointer_initializer_matches_host_stdout_and_exit_code() {
    // given
    let name = "global_struct_member_pointer_initializer";
    let source = "int puts(char*); typedef struct { int lo; int hi; } pair_t; pair_t pair = { 3, 7 }; int *ptr = &pair.hi; int main(void) { puts(\"global-member-pointer\"); return *ptr == 7 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn global_struct_array_member_pointer_array_initializer_matches_host_stdout_and_exit_code() {
    // given
    let name = "global_struct_array_member_pointer_array_initializer";
    let source = "int puts(char*); typedef struct { int lo; int hi; } pair_t; pair_t pairs[2] = { { 1, 2 }, { 3, 4 } }; int *items[] = { &pairs[0].hi, &pairs[1].lo, 0 }; int main(void) { puts(\"global-member-pointer-array\"); return *items[0] == 2 && *items[1] == 3 && items[2] == 0 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn static_local_struct_member_pointer_initializer_matches_host_stdout_and_exit_code() {
    // given
    let name = "static_local_struct_member_pointer_initializer";
    let source = "int puts(char*); typedef struct { int lo; int hi; } pair_t; pair_t pair = { 5, 11 }; int main(void) { static int *ptr = &pair.hi; puts(\"static-member-pointer\"); return *ptr == 11 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}
