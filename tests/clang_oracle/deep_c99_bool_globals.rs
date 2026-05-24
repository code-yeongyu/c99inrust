use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn global_bool_scalar_initializer_converts_to_one_byte_object_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); _Bool truth = 7; _Bool zero = 0; int main(void) { puts(\"global-bool-scalar\"); return sizeof(truth) == 1 && truth == 1 && zero == 0 ? 0 : 1; }\n";

    // when/then
    assert_case("global_bool_scalar_initializer_one_byte_object", source);
}

#[test]
fn global_bool_array_initializer_and_pointer_stride_match_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); _Bool values[4] = { 2, 0, -5 }; int main(void) { _Bool *p = values; puts(\"global-bool-array\"); return sizeof(values) == 4 && values[0] == 1 && values[1] == 0 && values[2] == 1 && values[3] == 0 && p[2] == 1 && ((char *)&p[1] - (char *)&p[0]) == 1 ? 0 : 1; }\n";

    // when/then
    assert_case("global_bool_array_initializer_pointer_stride", source);
}
