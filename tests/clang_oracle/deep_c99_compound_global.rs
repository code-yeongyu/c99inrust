use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn file_scope_array_compound_literal_pointer_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int *items = (int[]){ 7, 8, 9 }; int main(void) { puts(\"global-compound-array-ptr\"); return items[0] == 7 && items[2] == 9 ? 0 : 1; }\n";

    // when/then
    assert_case("file_scope_array_compound_literal_pointer", source);
}

#[test]
fn file_scope_array_compound_literal_pointer_offset_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int *items = (int[]){ 7, 8, 9 } + 1; int main(void) { puts(\"global-compound-array-offset\"); return items[0] == 8 && items[1] == 9 ? 0 : 1; }\n";

    // when/then
    assert_case("file_scope_array_compound_literal_pointer_offset", source);
}

#[test]
fn file_scope_array_compound_literal_element_address_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int *items = &(int[]){ 7, 8, 9 }[1]; int main(void) { puts(\"global-compound-array-element-address\"); return items[0] == 8 && items[1] == 9 ? 0 : 1; }\n";

    // when/then
    assert_case("file_scope_array_compound_literal_element_address", source);
}

#[test]
fn file_scope_array_compound_literal_element_address_offset_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int *items = &(int[]){ 7, 8, 9 }[0] + 2; int main(void) { puts(\"global-compound-array-element-address-offset\"); return items[0] == 9 ? 0 : 1; }\n";

    // when/then
    assert_case(
        "file_scope_array_compound_literal_element_address_offset",
        source,
    );
}

#[test]
fn file_scope_struct_compound_literal_address_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); typedef struct { int x; int y; } pair_t; pair_t *pair = &(pair_t){ .x = 11, .y = 13 }; int main(void) { puts(\"global-compound-struct-ptr\"); return pair->x == 11 && pair->y == 13 ? 0 : 1; }\n";

    // when/then
    assert_case("file_scope_struct_compound_literal_address", source);
}

#[test]
fn file_scope_char_array_compound_literal_pointer_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); char *text = (char[]){ 'o', 'k', 0 }; int main(void) { puts(text); return text[0] == 'o' && text[1] == 'k' && text[2] == 0 ? 0 : 1; }\n";

    // when/then
    assert_case("file_scope_char_array_compound_literal_pointer", source);
}

#[test]
fn file_scope_double_array_compound_literal_pointer_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); double *values = (double[]){ 1.5, 2.5 }; int main(void) { puts(\"global-compound-double-ptr\"); return values[0] == 1.5 && values[1] == 2.5 ? 0 : 1; }\n";

    // when/then
    assert_case("file_scope_double_array_compound_literal_pointer", source);
}

#[test]
fn file_scope_complex_array_compound_literal_pointer_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); double _Complex *values = (double _Complex[]){ 1.5, 2.5 }; int main(void) { double *raw = (double *)values; puts(\"global-compound-complex-ptr\"); return raw[0] == 1.5 && raw[1] == 0.0 && raw[2] == 2.5 && raw[3] == 0.0 ? 0 : 1; }\n";

    // when/then
    assert_case("file_scope_complex_array_compound_literal_pointer", source);
}

#[test]
fn file_scope_short_array_compound_literal_pointer_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); short *values = (short[]){ 1, 258, 513 }; int main(void) { puts(\"global-compound-short-ptr\"); return values[0] == 1 && values[1] == 258 && values[2] == 513 ? 0 : 1; }\n";

    // when/then
    assert_case("file_scope_short_array_compound_literal_pointer", source);
}

#[test]
fn file_scope_unsigned_short_array_compound_literal_pointer_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); unsigned short *values = (unsigned short[]){ 65535, 2, 65534 }; int main(void) { puts(\"global-compound-ushort-ptr\"); return values[0] == 65535 && values[1] == 2 && values[2] == 65534 ? 0 : 1; }\n";

    // when/then
    assert_case(
        "file_scope_unsigned_short_array_compound_literal_pointer",
        source,
    );
}

#[test]
fn file_scope_long_long_array_compound_literal_pointer_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); long long *values = (long long[]){ 4294967296LL, -7LL }; int main(void) { puts(\"global-compound-long-long-ptr\"); return values[0] == 4294967296LL && values[1] == -7LL ? 0 : 1; }\n";

    // when/then
    assert_case(
        "file_scope_long_long_array_compound_literal_pointer",
        source,
    );
}

#[test]
fn file_scope_bool_array_compound_literal_pointer_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); _Bool *flags = (_Bool[]){ 2, 0, -3 }; int main(void) { puts(\"global-compound-bool-ptr\"); return flags[0] == 1 && flags[1] == 0 && flags[2] == 1 ? 0 : 1; }\n";

    // when/then
    assert_case("file_scope_bool_array_compound_literal_pointer", source);
}

#[test]
fn file_scope_pointer_array_compound_literal_pointer_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); char **items = (char *[]){ \"alpha\" + 2, \"doom\" + 1, 0 }; int main(void) { puts(\"global-compound-pointer-array\"); return items[0][0] == 'p' && items[1][0] == 'o' && items[2] == 0 ? 0 : 1; }\n";

    // when/then
    assert_case("file_scope_pointer_array_compound_literal_pointer", source);
}

#[test]
fn file_scope_int_scalar_compound_literal_address_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); int *item = &(int){ 41 }; int main(void) { puts(\"global-compound-int-scalar-ptr\"); return *item == 41 ? 0 : 1; }\n";

    // when/then
    assert_case("file_scope_int_scalar_compound_literal_address", source);
}

#[test]
fn file_scope_bool_scalar_compound_literal_address_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); _Bool *flag = &(_Bool){ 7 }; int main(void) { puts(\"global-compound-bool-scalar-ptr\"); return *flag == 1 ? 0 : 1; }\n";

    // when/then
    assert_case("file_scope_bool_scalar_compound_literal_address", source);
}

#[test]
fn file_scope_long_long_scalar_compound_literal_address_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); long long *value = &(long long){ 4294967296LL }; int main(void) { puts(\"global-compound-ll-scalar-ptr\"); return *value == 4294967296LL ? 0 : 1; }\n";

    // when/then
    assert_case(
        "file_scope_long_long_scalar_compound_literal_address",
        source,
    );
}

#[test]
fn file_scope_double_scalar_compound_literal_address_matches_host_stdout_and_exit_code() {
    // given
    let source = "int puts(char*); double *value = &(double){ 2.25 }; int main(void) { puts(\"global-compound-double-scalar-ptr\"); return *value == 2.25 ? 0 : 1; }\n";

    // when/then
    assert_case("file_scope_double_scalar_compound_literal_address", source);
}
