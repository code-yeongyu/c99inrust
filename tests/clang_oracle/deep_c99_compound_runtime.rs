use super::support::{OracleCase, assert_compile_run_matches_host};

#[test]
fn array_compound_literal_runtime_subscript_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { int idx = 1; puts(\"compound-runtime-subscript\"); return (int[]){ 4, 5, 6 }[idx] == 5 ? 0 : 1; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "array_compound_literal_runtime_subscript",
        source,
    });
}

#[test]
fn array_compound_literal_runtime_index_side_effect_runs_once() {
    // given
    let source = "int puts(char*); int calls = 0; int next(void) { calls = calls + 1; return 1; } int main(void) { int selected = (int[]){ 4, 5, 6 }[next()]; puts(\"compound-runtime-index-once\"); return selected == 5 && calls == 1 ? 0 : 1; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "array_compound_literal_runtime_index_side_effect_once",
        source,
    });
}
