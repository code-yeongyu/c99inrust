use super::multifile_support::{
    OracleMultiFileCase, OracleSourceFile, assert_multifile_compile_run_matches_host,
};
use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

fn assert_multifile_case(name: &'static str, files: &'static [OracleSourceFile]) {
    assert_multifile_compile_run_matches_host(OracleMultiFileCase { name, files });
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
fn local_complex_float_real_initializer_zeroes_imaginary_part_matches_host_stdout_and_exit_code() {
    // given
    let name = "local_complex_float_real_initializer_zeroes_imaginary_part";
    let source = "int puts(char*); int main(void) { float _Complex z = 3.0; int *parts = (int *)&z; puts(\"complex-float-layout\"); return parts[0] == 0x40400000 && parts[1] == 0 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn local_complex_float_real_initializer_cast_reads_real_part_matches_host_stdout_and_exit_code() {
    // given
    let name = "local_complex_float_real_initializer_cast_reads_real_part";
    let source = "int puts(char*); int main(void) { float _Complex z = 6.0; puts(\"complex-float-cast\"); return (int)z; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn local_complex_float_real_assignment_zeroes_imaginary_part_matches_host_stdout_and_exit_code() {
    // given
    let name = "local_complex_float_real_assignment_zeroes_imaginary_part";
    let source = "int puts(char*); int main(void) { float _Complex z = 1.0; int *parts = (int *)&z; parts[1] = -1; z = 4.0; puts(\"complex-float-assign\"); return parts[0] == 0x40800000 && parts[1] == 0 ? 0 : 1; }\n";

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

#[test]
fn global_complex_double_real_initializer_matches_host_stdout_and_exit_code() {
    // given
    let name = "global_complex_double_real_initializer";
    let source = "int puts(char*); double _Complex g = 5.0; int main(void) { puts(\"complex-global-real\"); return (int)g; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn global_complex_double_layout_zeroes_imaginary_part_stdout_and_exit_code() {
    // given
    let name = "global_complex_double_layout_zeroes_imaginary_part";
    let source = "int puts(char*); double _Complex g = 5.0; int main(void) { double *parts = (double *)&g; puts(\"complex-global-layout\"); return (int)parts[0] + (int)parts[1]; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn global_complex_float_layout_zeroes_imaginary_part_stdout_and_exit_code() {
    // given
    let name = "global_complex_float_layout_zeroes_imaginary_part";
    let source = "int puts(char*); float _Complex g = 3.0; int main(void) { int *parts = (int *)&g; puts(\"complex-global-float-layout\"); return parts[0] == 0x40400000 && parts[1] == 0 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn extern_complex_double_global_matches_host_stdout_and_exit_code() {
    // given
    static FILES: &[OracleSourceFile] = &[
        OracleSourceFile {
            path: "defs.c",
            source: "double _Complex shared = 7.0;\n",
        },
        OracleSourceFile {
            path: "main.c",
            source: "int puts(char*); extern double _Complex shared; int main(void) { puts(\"complex-extern\"); return (int)shared; }\n",
        },
    ];

    // when/then
    assert_multifile_case("extern_complex_double_global", FILES);
}

#[test]
fn local_complex_double_real_initializer_zeroes_imaginary_part_matches_host_stdout_and_exit_code() {
    // given
    let name = "local_complex_double_real_initializer_zeroes_imaginary_part";
    let source = "int puts(char*); int main(void) { double _Complex z = 5.0; double *parts = (double *)&z; puts(\"complex-local-init-zero-imag\"); return (int)parts[0] + (int)parts[1]; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn local_complex_double_real_initializer_cast_reads_real_part_matches_host_stdout_and_exit_code() {
    // given
    let name = "local_complex_double_real_initializer_cast_reads_real_part";
    let source = "int puts(char*); int main(void) { double _Complex z = 6.0; puts(\"complex-local-init-cast-real\"); return (int)z; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn local_complex_double_real_assignment_zeroes_imaginary_part_matches_host_stdout_and_exit_code() {
    // given
    let name = "local_complex_double_real_assignment_zeroes_imaginary_part";
    let source = "int puts(char*); int main(void) { double _Complex z = 1.0; double *parts = (double *)&z; parts[1] = 9.0; z = 5.0; puts(\"complex-local-assign-zero-imag\"); return (int)parts[0] + (int)parts[1]; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn global_complex_double_real_assignment_zeroes_imaginary_part_matches_host_stdout_and_exit_code() {
    // given
    let name = "global_complex_double_real_assignment_zeroes_imaginary_part";
    let source = "int puts(char*); double _Complex g = 1.0; int main(void) { double *parts = (double *)&g; parts[1] = 8.0; g = 4.0; puts(\"complex-global-assign-zero-imag\"); return (int)parts[0] + (int)parts[1]; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn local_complex_double_copy_preserves_imaginary_lane_matches_host_stdout_and_exit_code() {
    // given
    let name = "local_complex_double_copy_preserves_imaginary_lane";
    let source = "int puts(char*); int main(void) { double _Complex a = 1.0; double *ap = (double *)&a; ap[1] = 7.0; double _Complex b = a; double *bp = (double *)&b; puts(\"complex-local-copy\"); return (int)bp[0] + (int)bp[1]; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn local_complex_float_copy_preserves_imaginary_lane_matches_host_stdout_and_exit_code() {
    // given
    let name = "local_complex_float_copy_preserves_imaginary_lane";
    let source = "int puts(char*); int main(void) { float _Complex a = 2.0; float *ap = (float *)&a; ap[1] = 5.0; float _Complex b = a; float *bp = (float *)&b; puts(\"complex-float-copy\"); return (int)bp[0] + (int)bp[1]; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn local_complex_double_assignment_preserves_imaginary_lane_matches_host_stdout_and_exit_code() {
    // given
    let name = "local_complex_double_assignment_preserves_imaginary_lane";
    let source = "int puts(char*); int main(void) { double _Complex a = 3.0; double *ap = (double *)&a; ap[1] = 4.0; double _Complex b = 0.0; b = a; double *bp = (double *)&b; puts(\"complex-local-assign-copy\"); return (int)bp[0] + (int)bp[1]; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn local_complex_float_assignment_preserves_imaginary_lane_matches_host_stdout_and_exit_code() {
    // given
    let name = "local_complex_float_assignment_preserves_imaginary_lane";
    let source = "int puts(char*); int main(void) { float _Complex a = 4.0; float *ap = (float *)&a; ap[1] = 6.0; float _Complex b = 0.0; b = a; float *bp = (float *)&b; puts(\"complex-float-assign-copy\"); return (int)bp[0] + (int)bp[1]; }\n";

    // when/then
    assert_case(name, source);
}
