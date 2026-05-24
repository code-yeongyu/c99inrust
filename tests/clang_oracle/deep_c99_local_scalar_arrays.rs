use super::support::{OracleCase, assert_compile_run_matches_host};

#[test]
fn local_double_array_store_and_load_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double values[2]; values[0] = 1.5; values[1] = 2.5; puts(\"local-double-array-store\"); return values[0] + values[1] == 4.0 ? 0 : 1; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "local_double_array_store_and_load",
        source,
    });
}

#[test]
fn local_double_array_sizeof_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double values[2]; puts(\"local-double-array-sizeof\"); return sizeof(values) == 2 * sizeof(double) ? 0 : 1; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "local_double_array_sizeof",
        source,
    });
}

#[test]
fn local_double_array_pointer_decay_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double values[2]; double *p = values; values[0] = 1.25; values[1] = 3.75; puts(\"local-double-array-decay\"); return p[0] + p[1] == 5.0 ? 0 : 1; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "local_double_array_pointer_decay",
        source,
    });
}
