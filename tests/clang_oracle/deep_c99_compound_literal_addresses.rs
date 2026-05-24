use super::support::{OracleCase, assert_compile_run_matches_host};

#[test]
fn whole_struct_compound_literal_address_initializer_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { int x; int y; } pair_t; int main(void) { pair_t *p = &(pair_t){ .y = 4, .x = 3 }; puts(\"whole-struct-address-init\"); return p->x == 3 && p->y == 4 ? 0 : 1; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "whole_struct_compound_literal_address_initializer",
        source,
    });
}

#[test]
fn whole_struct_compound_literal_address_assignment_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { int x; int y; } pair_t; int main(void) { pair_t *p; p = &(pair_t){ 5, 7 }; puts(\"whole-struct-address-assign\"); return p->x == 5 && p->y == 7 ? 0 : 1; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "whole_struct_compound_literal_address_assignment",
        source,
    });
}

#[test]
fn struct_array_field_compound_literal_address_preserves_nested_designator_matches_host_stdout_and_exit_code()
 {
    // given
    let source = "int puts(char*); typedef struct { int values[3]; } bucket_t; int main(void) { bucket_t *p = &(bucket_t){ .values[1] = 4, 5 }; puts(\"whole-struct-address-nested\"); return p->values[0] == 0 && p->values[1] == 4 && p->values[2] == 5 ? 0 : 1; }\n";

    // when/then
    assert_compile_run_matches_host(OracleCase {
        name: "struct_array_field_compound_literal_address_preserves_nested_designator",
        source,
    });
}
