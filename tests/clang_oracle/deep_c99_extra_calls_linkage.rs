use super::multifile_support::{
    OracleMultiFileCase, OracleSourceFile, assert_multifile_compile_run_matches_host,
};
use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn extra_function_pointer_array_rotates_indirect_calls_match_host_stdout_and_exit_code() {
    // given
    let name = "extra_function_pointer_array_rotates_indirect_calls";
    let source = "int puts(char*); int inc(int v) { return v + 1; } int dbl(int v) { return v * 2; } int dec(int v) { return v - 3; } int main(void) { int (*ops[3])(int) = { inc, dbl, dec }; int index = 0; int total = ops[index](4); index = index + 1; total = total + ops[index](5); ops[2] = ops[0]; puts(\"extra-fnptr\"); return total + ops[2](6); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn extra_kr_style_array_parameter_decay_matches_host_stdout_and_exit_code() {
    // given
    let name = "extra_kr_style_array_parameter_decay";
    let source = "int puts(char*); int legacy(n, values) int n; int values[]; { int i; int total; total = 0; for (i = 0; i < n; i++) total = total + values[i] * (i + 1); return total; } int main(void) { int values[4] = { 1, 3, 5, 7 }; puts(\"extra-kr-array\"); return legacy(4, values); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn extra_multifile_extern_pointer_and_array_linkage_match_host_stdout_and_exit_code() {
    // given
    let case = OracleMultiFileCase {
        name: "extra_multifile_extern_pointer_and_array_linkage",
        files: &[
            OracleSourceFile {
                path: "shared.h",
                source: "extern int values[3]; extern int *cursor; void init_cursor(void); int read_cursor(void);\n",
            },
            OracleSourceFile {
                path: "state.c",
                source: "#include \"shared.h\"\nint values[3] = { 4, 6, 8 }; int *cursor; void init_cursor(void) { cursor = values; }\n",
            },
            OracleSourceFile {
                path: "read.c",
                source: "#include \"shared.h\"\nint read_cursor(void) { return *cursor + values[2]; }\n",
            },
            OracleSourceFile {
                path: "main.c",
                source: "#include \"shared.h\"\nint puts(char*); int main(void) { init_cursor(); puts(\"extra-mf\"); return read_cursor(); }\n",
            },
        ],
    };

    // when/then
    assert_multifile_compile_run_matches_host(case);
}

#[test]
fn extra_static_inline_array_accessor_chain_matches_host_stdout_and_exit_code() {
    // given
    let name = "extra_static_inline_array_accessor_chain";
    let source = "int puts(char*); static inline int at(int *base, int index) { return base[index]; } static inline int mix_at(int *base) { return at(base, 0) + at(base, 2) * 3; } int main(void) { int values[3] = { 2, 4, 6 }; puts(\"extra-inline\"); return mix_at(values); }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn extra_restrict_three_way_accumulate_matches_host_stdout_and_exit_code() {
    // given
    let name = "extra_restrict_three_way_accumulate";
    let source = "int puts(char*); void fuse(int n, int * restrict out, const int * restrict a, const int * restrict b) { int i; for (i = 0; i < n; i++) out[i] = a[i] + b[n - i - 1]; } int main(void) { int a[3] = { 1, 2, 3 }; int b[3] = { 10, 20, 30 }; int out[3]; fuse(3, out, a, b); puts(\"extra-restrict\"); return out[0] + out[1] + out[2]; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn extra_double_return_libc_prototype_matches_host_stdout_and_exit_code() {
    // given
    let name = "extra_double_return_libc_prototype";
    let source = "int puts(char*); double fabs(double); int main(void) { double x = fabs(-3.5); puts(\"extra-proto-double\"); return x == 3.5 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn extra_double_parameter_function_definition_matches_host_stdout_and_exit_code() {
    // given
    let name = "extra_double_parameter_function_definition";
    let source = "int puts(char*); double half(double x) { return x / 2.0; } int main(void) { double y = half(7.0); puts(\"extra-double-param\"); return y == 3.5 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn extra_double_return_function_pointer_matches_host_stdout_and_exit_code() {
    // given
    let name = "extra_double_return_function_pointer";
    let source = "int puts(char*); double bump(double x) { return x + 0.25; } int main(void) { double (*fp)(double) = bump; double y = fp(1.25); puts(\"extra-double-fnptr\"); return y == 1.5 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn extra_double_return_function_pointer_array_matches_host_stdout_and_exit_code() {
    // given
    let name = "extra_double_return_function_pointer_array";
    let source = "int puts(char*); double bump(double x) { return x + 0.25; } double twice(double x) { return x * 2.0; } int main(void) { double (*ops[2])(double) = { bump, twice }; double a = ops[0](1.25); double b = ops[1](2.0); puts(\"extra-double-fnptr-array\"); return a == 1.5 && b == 4.0 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn extra_global_double_return_function_pointer_initializer_matches_host_stdout_and_exit_code() {
    // given
    let name = "extra_global_double_return_function_pointer_initializer";
    let source = "int puts(char*); double bump(double x) { return x + 0.75; } double (*hook)(double) = bump; int main(void) { double y = hook(1.25); puts(\"extra-global-double-fnptr\"); return y == 2.0 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}
