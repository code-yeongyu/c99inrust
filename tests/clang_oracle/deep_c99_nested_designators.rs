use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn local_struct_array_field_nested_designators_match_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { int values[4]; int tail; } bucket_t; int main(void) { bucket_t b = { .values[2] = 9, .values[0] = 1, .tail = 5 }; puts(\"nested-array-designator\"); return b.values[0] == 1 && b.values[1] == 0 && b.values[2] == 9 && b.values[3] == 0 && b.tail == 5 ? 0 : 1; }\n";

    // when/then
    assert_case("local_struct_array_field_nested_designators", source);
}
