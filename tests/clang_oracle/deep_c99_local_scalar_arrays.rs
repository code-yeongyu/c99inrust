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

#[test]
fn local_long_long_array_store_and_load_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { long long values[2]; values[0] = 4000000000LL; values[1] = 5000000000LL; puts(\"local-long-long-array\"); return values[0] + values[1] == 9000000000LL ? 0 : 1; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "local_long_long_array_store_and_load",
        source,
    });
}

#[test]
fn local_double_array_initializer_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { double values[3] = { 1.5, 2.5 }; puts(\"local-double-array-init\"); return values[0] + values[1] == 4.0 && values[2] == 0.0 ? 0 : 1; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "local_double_array_initializer",
        source,
    });
}

#[test]
fn local_bool_array_initializer_uses_one_byte_elements_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { _Bool values[3] = { 2, 0, -5 }; puts(\"local-bool-array-init\"); return sizeof(values) == 3 && values[0] == 1 && values[1] == 0 && values[2] == 1 ? 0 : 1; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "local_bool_array_initializer_one_byte_elements",
        source,
    });
}

#[test]
fn local_bool_array_decays_to_one_byte_pointer_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int main(void) { _Bool values[3] = { 1, 0, 1 }; _Bool *p = values; puts(\"local-bool-array-decay\"); return p[2] == 1 && ((char *)&p[1] - (char *)&p[0]) == 1 ? 0 : 1; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "local_bool_array_decay_one_byte_pointer",
        source,
    });
}
