use super::support::{OracleCase, assert_compile_run_matches_host};

fn assert_case(name: &'static str, source: &'static str) {
    assert_compile_run_matches_host(OracleCase { name, source });
}

#[test]
fn variadic_body_sums_int_va_args_matches_host_stdout_and_exit_code() {
    // given
    let name = "variadic_body_sums_int_va_args";
    let source = "#include <stdarg.h>\nint puts(char*); int sum(int count, ...) { va_list ap; va_start(ap, count); int total = 0; for (int i = 0; i < count; i++) { total = total + va_arg(ap, int); } va_end(ap); return total; } int main(void) { puts(\"va-int\"); return sum(5, 3, 4, 5, 6, 7) == 25 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn variadic_body_reads_pointer_va_args_matches_host_stdout_and_exit_code() {
    // given
    let name = "variadic_body_reads_pointer_va_args";
    let source = "#include <stdarg.h>\nint puts(char*); char* pick(int index, ...) { va_list ap; va_start(ap, index); char* selected = 0; for (int i = 0; i <= index; i++) { selected = va_arg(ap, char*); } va_end(ap); return selected; } int main(void) { char* value = pick(2, \"zero\", \"one\", \"two\"); puts(value); return value[0] == 't' && value[1] == 'w' && value[2] == 'o' ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn variadic_body_reads_overflow_va_args_matches_host_stdout_and_exit_code() {
    // given
    let name = "variadic_body_reads_overflow_va_args";
    let source = "#include <stdarg.h>\nint puts(char*); int sum_many(int count, ...) { va_list ap; va_start(ap, count); int total = 0; for (int i = 0; i < count; i++) { total = total + va_arg(ap, int); } va_end(ap); return total; } int main(void) { puts(\"va-overflow\"); return sum_many(10, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10) == 55 ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}

#[test]
fn variadic_body_reads_long_long_va_args_matches_host_stdout_and_exit_code() {
    // given
    let name = "variadic_body_reads_long_long_va_args";
    let source = "#include <stdarg.h>\nint puts(char*); long long pick64(int index, ...) { va_list ap; va_start(ap, index); long long selected = 0; for (int i = 0; i <= index; i++) { selected = va_arg(ap, long long); } va_end(ap); return selected; } int main(void) { long long value = pick64(2, 11LL, 22LL, 33LL); puts(\"va-i64\"); return value == 33LL ? 0 : 1; }\n";

    // when/then
    assert_case(name, source);
}
