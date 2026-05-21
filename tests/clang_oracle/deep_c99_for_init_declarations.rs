use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn for_init_scalar_shadowing_matches_host_stdout_and_exit_code() {
    // given
    let name = "for_init_scalar_shadowing";
    let source = "int puts(char*); int main(void) { int i = 40; int total = 0; for (int i = 0; i < 4; i++) total = total + i; puts(\"for-shadow\"); return i == 40 && total == 6 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn for_init_multi_declarator_matches_host_stdout_and_exit_code() {
    // given
    let name = "for_init_multi_declarator";
    let source = "int puts(char*); int main(void) { int total = 0; for (int i = 0, j = 5; i < 3; i++, j--) total = total + i * 10 + j; puts(\"for-multi\"); return total == 48 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn for_init_array_declaration_matches_host_stdout_and_exit_code() {
    // given
    let name = "for_init_array_declaration";
    let source = "int puts(char*); int main(void) { int total = 0; for (int values[3] = { 2, 4, 6 }; values[0] < 5; values[0]++) total = total + values[0] + values[1] + values[2]; puts(\"for-array\"); return total == 39 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn for_init_struct_declaration_matches_host_stdout_and_exit_code() {
    // given
    let name = "for_init_struct_declaration";
    let source = "int puts(char*); typedef struct { int x; int y; } pair_t; int main(void) { int total = 0; for (pair_t p = { .x = 1, .y = 4 }; p.x < 4; p.x++) total = total + p.x + p.y; puts(\"for-struct\"); return total == 18 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn for_init_function_pointer_declaration_matches_host_stdout_and_exit_code() {
    // given
    let name = "for_init_function_pointer_declaration";
    let source = "int puts(char*); int inc(int x) { return x + 1; } int main(void) { int total = 0; for (int (*step)(int) = inc, i = 0; i < 4; i = step(i)) total = total + i; puts(\"for-fptr\"); return total == 6 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}
