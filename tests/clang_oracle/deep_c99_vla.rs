use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn local_int_vla_runtime_length_matches_host_stdout_and_exit_code() {
    // given
    let name = "local_int_vla_runtime_length";
    let source = "int puts(char*); int fill(int n) { int values[n]; int i; int total = 0; for (i = 0; i < n; i++) { values[i] = i * 3; total = total + values[i]; } return total; } int main(void) { puts(\"vla-int\"); return fill(5) == 30 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn local_int_vla_decays_to_pointer_matches_host_stdout_and_exit_code() {
    // given
    let name = "local_int_vla_decays_to_pointer";
    let source = "int puts(char*); int sum(int n, int *values) { int i; int total = 0; for (i = 0; i < n; i++) total = total + values[i]; return total; } int main(void) { int n = 4; int values[n]; int i; for (i = 0; i < n; i++) values[i] = i + 2; puts(\"vla-decay\"); return sum(n, values) == 14 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn local_unsigned_char_vla_matches_host_stdout_and_exit_code() {
    // given
    let name = "local_unsigned_char_vla";
    let source = "int puts(char*); int main(void) { int n = 6; unsigned char bytes[n]; int i; int total = 0; for (i = 0; i < n; i++) { bytes[i] = 250 + i; total = total + bytes[i]; } puts(\"vla-byte\"); return total == 1515 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}
