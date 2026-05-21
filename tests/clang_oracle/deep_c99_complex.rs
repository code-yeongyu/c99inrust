use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn complex_float_sizeof_type_matches_host_stdout_and_exit_code() {
    // given
    let name = "complex_float_sizeof_type";
    let source = "int puts(char*); int main(void) { puts(\"complex-float-size\"); return sizeof(float _Complex); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn complex_double_sizeof_type_matches_host_stdout_and_exit_code() {
    // given
    let name = "complex_double_sizeof_type";
    let source = "int puts(char*); int main(void) { puts(\"complex-double-size\"); return sizeof(double _Complex); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn complex_long_double_sizeof_type_matches_host_stdout_and_exit_code() {
    // given
    let name = "complex_long_double_sizeof_type";
    let source = "int puts(char*); int main(void) { puts(\"complex-long-double-size\"); return sizeof(long double _Complex); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn complex_pointer_sizeof_type_matches_host_stdout_and_exit_code() {
    // given
    let name = "complex_pointer_sizeof_type";
    let source = "int puts(char*); int main(void) { puts(\"complex-pointer-size\"); return sizeof(double _Complex *) == sizeof(void *) ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn complex_array_sizeof_type_matches_host_stdout_and_exit_code() {
    // given
    let name = "complex_array_sizeof_type";
    let source = "int puts(char*); int main(void) { puts(\"complex-array-size\"); return sizeof(double _Complex[3]); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn local_complex_double_declaration_size_matches_host_stdout_and_exit_code() {
    // given
    let name = "local_complex_double_declaration_size";
    let source = "int puts(char*); int main(void) { double _Complex z; puts(\"complex-local-double\"); return sizeof(z); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn local_complex_float_declaration_size_matches_host_stdout_and_exit_code() {
    // given
    let name = "local_complex_float_declaration_size";
    let source = "int puts(char*); int main(void) { float _Complex z; puts(\"complex-local-float\"); return sizeof(z); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn local_complex_pointer_declaration_matches_host_stdout_and_exit_code() {
    // given
    let name = "local_complex_pointer_declaration";
    let source = "int puts(char*); int main(void) { double _Complex z; double _Complex *p = &z; puts(\"complex-local-pointer\"); return sizeof(p) == sizeof(void *) ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn complex_struct_field_size_matches_host_stdout_and_exit_code() {
    // given
    let name = "complex_struct_field_size";
    let source = "int puts(char*); typedef struct { char tag; double _Complex z; int tail; } packet_t; int main(void) { puts(\"complex-struct-size\"); return sizeof(packet_t); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn complex_struct_field_offset_matches_host_stdout_and_exit_code() {
    // given
    let name = "complex_struct_field_offset";
    let source = "int puts(char*); typedef struct { char tag; double _Complex z; int tail; } packet_t; int main(void) { packet_t packet; puts(\"complex-struct-offset\"); return (int)((char *)&packet.tail - (char *)&packet); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn complex_struct_array_field_size_matches_host_stdout_and_exit_code() {
    // given
    let name = "complex_struct_array_field_size";
    let source = "int puts(char*); typedef struct { double _Complex cells[2]; int tail; } grid_t; int main(void) { puts(\"complex-struct-array-size\"); return sizeof(grid_t); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn complex_const_qualified_sizeof_type_matches_host_stdout_and_exit_code() {
    // given
    let name = "complex_const_qualified_sizeof_type";
    let source = "int puts(char*); int main(void) { puts(\"complex-const-size\"); return sizeof(const double _Complex); }\n";

    // when/then
    assert_case(name, source);
}
