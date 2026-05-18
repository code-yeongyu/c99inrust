use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[test]
fn constant_return_matches_host_c_compiler_exit_code() {
    // given
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    let case = OracleCase {
        name: "constant_return",
        source: "#define ANSWER 42\nint main(void) { return (ANSWER * 2) - 42; }\n",
    };
    let root = fresh_temp_dir(case.name);
    let source = root.join("case.c");
    let c99_asm = root.join("c99inrust.s");
    let c99_exe = executable_path(&root, "c99inrust");
    let clang_exe = executable_path(&root, "clang");
    fs::write(&source, case.source).expect("oracle source should be written");

    // when
    let c99_status = compile_with_c99inrust(&source, &c99_asm)
        .and_then(|()| assemble(&c99_asm, &c99_exe))
        .and_then(|()| run_exit_code(&c99_exe))
        .expect("c99inrust path should compile, link, and run");
    let clang_status = compile_with_host_c(&source, &clang_exe)
        .and_then(|()| run_exit_code(&clang_exe))
        .expect("host C compiler path should compile and run");

    // then
    assert_eq!(c99_status, clang_status);
}

#[test]
fn arithmetic_precedence_matches_host_c_compiler_exit_code() {
    // given
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    let case = OracleCase {
        name: "arithmetic_precedence",
        source: "int main(void) { return 3 + 5 * 8 - (9 >> 1); }\n",
    };
    let root = fresh_temp_dir(case.name);
    let source = root.join("case.c");
    let c99_asm = root.join("c99inrust.s");
    let c99_exe = executable_path(&root, "c99inrust");
    let clang_exe = executable_path(&root, "clang");
    fs::write(&source, case.source).expect("oracle source should be written");

    // when
    let c99_status = compile_with_c99inrust(&source, &c99_asm)
        .and_then(|()| assemble(&c99_asm, &c99_exe))
        .and_then(|()| run_exit_code(&c99_exe))
        .expect("c99inrust path should compile, link, and run");
    let clang_status = compile_with_host_c(&source, &clang_exe)
        .and_then(|()| run_exit_code(&clang_exe))
        .expect("host C compiler path should compile and run");

    // then
    assert_eq!(c99_status, clang_status);
}

#[test]
fn local_int_assignment_matches_host_c_compiler_exit_code() {
    // given
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    let case = OracleCase {
        name: "local_int_assignment",
        source: "int main(void) { int x = 40; int y = x + 1; x = y + 1; return x; }\n",
    };
    let root = fresh_temp_dir(case.name);
    let source = root.join("case.c");
    let c99_asm = root.join("c99inrust.s");
    let c99_exe = executable_path(&root, "c99inrust");
    let clang_exe = executable_path(&root, "clang");
    fs::write(&source, case.source).expect("oracle source should be written");

    // when
    let c99_status = compile_with_c99inrust(&source, &c99_asm)
        .and_then(|()| assemble(&c99_asm, &c99_exe))
        .and_then(|()| run_exit_code(&c99_exe))
        .expect("c99inrust path should compile, link, and run");
    let clang_status = compile_with_host_c(&source, &clang_exe)
        .and_then(|()| run_exit_code(&clang_exe))
        .expect("host C compiler path should compile and run");

    // then
    assert_eq!(c99_status, clang_status);
}

#[test]
fn if_else_comparison_matches_host_c_compiler_exit_code() {
    // given
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    let case = OracleCase {
        name: "if_else_comparison",
        source: "int main(void) { int x = 7; if (x >= 7) { x = x + 30; } else { x = 1; } if (x != 37) { return 2; } return x; }\n",
    };
    let root = fresh_temp_dir(case.name);
    let source = root.join("case.c");
    let c99_asm = root.join("c99inrust.s");
    let c99_exe = executable_path(&root, "c99inrust");
    let clang_exe = executable_path(&root, "clang");
    fs::write(&source, case.source).expect("oracle source should be written");

    // when
    let c99_status = compile_with_c99inrust(&source, &c99_asm)
        .and_then(|()| assemble(&c99_asm, &c99_exe))
        .and_then(|()| run_exit_code(&c99_exe))
        .expect("c99inrust path should compile, link, and run");
    let clang_status = compile_with_host_c(&source, &clang_exe)
        .and_then(|()| run_exit_code(&clang_exe))
        .expect("host C compiler path should compile and run");

    // then
    assert_eq!(c99_status, clang_status);
}

#[test]
fn while_loop_matches_host_c_compiler_exit_code() {
    // given
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    let case = OracleCase {
        name: "while_loop",
        source: "int main(void) { int x = 0; int total = 0; while (x < 5) { total = total + x; x = x + 1; } return total; }\n",
    };
    let root = fresh_temp_dir(case.name);
    let source = root.join("case.c");
    let c99_asm = root.join("c99inrust.s");
    let c99_exe = executable_path(&root, "c99inrust");
    let clang_exe = executable_path(&root, "clang");
    fs::write(&source, case.source).expect("oracle source should be written");

    // when
    let c99_status = compile_with_c99inrust(&source, &c99_asm)
        .and_then(|()| assemble(&c99_asm, &c99_exe))
        .and_then(|()| run_exit_code(&c99_exe))
        .expect("c99inrust path should compile, link, and run");
    let clang_status = compile_with_host_c(&source, &clang_exe)
        .and_then(|()| run_exit_code(&clang_exe))
        .expect("host C compiler path should compile and run");

    // then
    assert_eq!(c99_status, clang_status);
}

#[test]
fn logical_short_circuit_matches_host_c_compiler_exit_code() {
    // given
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    let case = OracleCase {
        name: "logical_short_circuit",
        source: "int main(void) { int x = 0; if (x != 0 && 10 / x > 1) { return 1; } if (x == 0 || 10 / x > 1) { return 42; } return 2; }\n",
    };
    let root = fresh_temp_dir(case.name);
    let source = root.join("case.c");
    let c99_asm = root.join("c99inrust.s");
    let c99_exe = executable_path(&root, "c99inrust");
    let clang_exe = executable_path(&root, "clang");
    fs::write(&source, case.source).expect("oracle source should be written");

    // when
    let c99_status = compile_with_c99inrust(&source, &c99_asm)
        .and_then(|()| assemble(&c99_asm, &c99_exe))
        .and_then(|()| run_exit_code(&c99_exe))
        .expect("c99inrust path should compile, link, and run");
    let clang_status = compile_with_host_c(&source, &clang_exe)
        .and_then(|()| run_exit_code(&clang_exe))
        .expect("host C compiler path should compile and run");

    // then
    assert_eq!(c99_status, clang_status);
}

#[test]
fn for_loop_matches_host_c_compiler_exit_code() {
    // given
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    let case = OracleCase {
        name: "for_loop",
        source: "int main(void) { int total = 0; for (int i = 0; i < 5; i = i + 1) { total = total + i; } return total; }\n",
    };
    let root = fresh_temp_dir(case.name);
    let source = root.join("case.c");
    let c99_asm = root.join("c99inrust.s");
    let c99_exe = executable_path(&root, "c99inrust");
    let clang_exe = executable_path(&root, "clang");
    fs::write(&source, case.source).expect("oracle source should be written");

    // when
    let c99_status = compile_with_c99inrust(&source, &c99_asm)
        .and_then(|()| assemble(&c99_asm, &c99_exe))
        .and_then(|()| run_exit_code(&c99_exe))
        .expect("c99inrust path should compile, link, and run");
    let clang_status = compile_with_host_c(&source, &clang_exe)
        .and_then(|()| run_exit_code(&clang_exe))
        .expect("host C compiler path should compile and run");

    // then
    assert_eq!(c99_status, clang_status);
}

#[test]
fn post_increment_for_loop_matches_host_c_compiler_exit_code() {
    // given
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    let case = OracleCase {
        name: "post_increment_for_loop",
        source: "int main(void) { int i; int total = 0; for (i = 0; i < 4; i++) { total = total + i; } return total == 6 ? 0 : 1; }\n",
    };
    let root = fresh_temp_dir(case.name);
    let source = root.join("case.c");
    let c99_asm = root.join("c99inrust.s");
    let c99_exe = executable_path(&root, "c99inrust");
    let clang_exe = executable_path(&root, "clang");
    fs::write(&source, case.source).expect("oracle source should be written");

    // when
    let c99_status = compile_with_c99inrust(&source, &c99_asm)
        .and_then(|()| assemble(&c99_asm, &c99_exe))
        .and_then(|()| run_exit_code(&c99_exe))
        .expect("c99inrust path should compile, link, and run");
    let clang_status = compile_with_host_c(&source, &clang_exe)
        .and_then(|()| run_exit_code(&clang_exe))
        .expect("host C compiler path should compile and run");

    // then
    assert_eq!(c99_status, clang_status);
}

#[test]
fn zero_arg_function_call_matches_host_c_compiler_exit_code() {
    // given
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    let case = OracleCase {
        name: "zero_arg_function_call",
        source: "int answer(void) { int value = 40; return value; } int main(void) { return 2 + answer(); }\n",
    };
    let root = fresh_temp_dir(case.name);
    let source = root.join("case.c");
    let c99_asm = root.join("c99inrust.s");
    let c99_exe = executable_path(&root, "c99inrust");
    let clang_exe = executable_path(&root, "clang");
    fs::write(&source, case.source).expect("oracle source should be written");

    // when
    let c99_status = compile_with_c99inrust(&source, &c99_asm)
        .and_then(|()| assemble(&c99_asm, &c99_exe))
        .and_then(|()| run_exit_code(&c99_exe))
        .expect("c99inrust path should compile, link, and run");
    let clang_status = compile_with_host_c(&source, &clang_exe)
        .and_then(|()| run_exit_code(&clang_exe))
        .expect("host C compiler path should compile and run");

    // then
    assert_eq!(c99_status, clang_status);
}

#[test]
fn nested_zero_arg_function_calls_match_host_c_compiler_exit_code() {
    // given
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    let case = OracleCase {
        name: "nested_zero_arg_function_calls",
        source: "int answer(void) { int value = 40; return value; } int two(void) { int value = 2; return value; } int main(void) { return 100 + (answer() + two()); }\n",
    };
    let root = fresh_temp_dir(case.name);
    let source = root.join("case.c");
    let c99_asm = root.join("c99inrust.s");
    let c99_exe = executable_path(&root, "c99inrust");
    let clang_exe = executable_path(&root, "clang");
    fs::write(&source, case.source).expect("oracle source should be written");

    // when
    let c99_status = compile_with_c99inrust(&source, &c99_asm)
        .and_then(|()| assemble(&c99_asm, &c99_exe))
        .and_then(|()| run_exit_code(&c99_exe))
        .expect("c99inrust path should compile, link, and run");
    let clang_status = compile_with_host_c(&source, &clang_exe)
        .and_then(|()| run_exit_code(&clang_exe))
        .expect("host C compiler path should compile and run");

    // then
    assert_eq!(c99_status, clang_status);
}

#[test]
fn build_command_matches_host_c_compiler_exit_code() {
    // given
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    let case = OracleCase {
        name: "build_command",
        source: "int tick(void) { return 1; } int main(void) { return 41 + tick(); }\n",
    };
    let root = fresh_temp_dir(case.name);
    let source = root.join("case.c");
    let c99_exe = executable_path(&root, "c99inrust-build");
    let clang_exe = executable_path(&root, "clang");
    fs::write(&source, case.source).expect("oracle source should be written");

    // when
    let c99_status = build_with_c99inrust(&source, &c99_exe)
        .and_then(|()| run_exit_code(&c99_exe))
        .expect("c99inrust build path should compile, link, and run");
    let clang_status = compile_with_host_c(&source, &clang_exe)
        .and_then(|()| run_exit_code(&clang_exe))
        .expect("host C compiler path should compile and run");

    // then
    assert_eq!(c99_status, clang_status);
}

#[test]
fn top_level_declaration_slice_matches_host_c_compiler_exit_code() {
    // given
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    let case = OracleCase {
        name: "top_level_declaration_slice",
        source: "static const char rcsid[] = \"doom\"; int main(void) { return 42; }\n",
    };
    let root = fresh_temp_dir(case.name);
    let source = root.join("case.c");
    let c99_asm = root.join("c99inrust.s");
    let c99_exe = executable_path(&root, "c99inrust");
    let clang_exe = executable_path(&root, "clang");
    fs::write(&source, case.source).expect("oracle source should be written");

    // when
    let c99_status = compile_with_c99inrust(&source, &c99_asm)
        .and_then(|()| assemble(&c99_asm, &c99_exe))
        .and_then(|()| run_exit_code(&c99_exe))
        .expect("c99inrust path should compile, link, and run");
    let clang_status = compile_with_host_c(&source, &clang_exe)
        .and_then(|()| run_exit_code(&clang_exe))
        .expect("host C compiler path should compile and run");

    // then
    assert_eq!(c99_status, clang_status);
}

#[test]
fn void_function_slice_matches_host_c_compiler_exit_code() {
    // given
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    let case = OracleCase {
        name: "void_function_slice",
        source: "void tick(void) { return; } int main(void) { return 42; }\n",
    };
    let root = fresh_temp_dir(case.name);
    let source = root.join("case.c");
    let c99_asm = root.join("c99inrust.s");
    let c99_exe = executable_path(&root, "c99inrust");
    let clang_exe = executable_path(&root, "clang");
    fs::write(&source, case.source).expect("oracle source should be written");

    // when
    let c99_status = compile_with_c99inrust(&source, &c99_asm)
        .and_then(|()| assemble(&c99_asm, &c99_exe))
        .and_then(|()| run_exit_code(&c99_exe))
        .expect("c99inrust path should compile, link, and run");
    let clang_status = compile_with_host_c(&source, &clang_exe)
        .and_then(|()| run_exit_code(&clang_exe))
        .expect("host C compiler path should compile and run");

    // then
    assert_eq!(c99_status, clang_status);
}

#[test]
fn parameter_list_signature_slice_matches_host_c_compiler_exit_code() {
    // given
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    let case = OracleCase {
        name: "parameter_list_signature_slice",
        source: "int main(int argc, char **argv) { return 42; }\n",
    };
    let root = fresh_temp_dir(case.name);
    let source = root.join("case.c");
    let c99_asm = root.join("c99inrust.s");
    let c99_exe = executable_path(&root, "c99inrust");
    let clang_exe = executable_path(&root, "clang");
    fs::write(&source, case.source).expect("oracle source should be written");

    // when
    let c99_status = compile_with_c99inrust(&source, &c99_asm)
        .and_then(|()| assemble(&c99_asm, &c99_exe))
        .and_then(|()| run_exit_code(&c99_exe))
        .expect("c99inrust path should compile, link, and run");
    let clang_status = compile_with_host_c(&source, &clang_exe)
        .and_then(|()| run_exit_code(&clang_exe))
        .expect("host C compiler path should compile and run");

    // then
    assert_eq!(c99_status, clang_status);
}

#[test]
fn typedef_return_signature_slice_matches_host_c_compiler_exit_code() {
    // given
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    let case = OracleCase {
        name: "typedef_return_signature_slice",
        source: "typedef int fixed_t; fixed_t FixedMul(fixed_t a, fixed_t b) { return 42; } int main(void) { return 42; }\n",
    };
    let root = fresh_temp_dir(case.name);
    let source = root.join("case.c");
    let c99_asm = root.join("c99inrust.s");
    let c99_exe = executable_path(&root, "c99inrust");
    let clang_exe = executable_path(&root, "clang");
    fs::write(&source, case.source).expect("oracle source should be written");

    // when
    let c99_status = compile_with_c99inrust(&source, &c99_asm)
        .and_then(|()| assemble(&c99_asm, &c99_exe))
        .and_then(|()| run_exit_code(&c99_exe))
        .expect("c99inrust path should compile, link, and run");
    let clang_status = compile_with_host_c(&source, &clang_exe)
        .and_then(|()| run_exit_code(&clang_exe))
        .expect("host C compiler path should compile and run");

    // then
    assert_eq!(c99_status, clang_status);
}

#[test]
fn unsigned_return_signature_slice_matches_host_c_compiler_exit_code() {
    // given
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    let case = OracleCase {
        name: "unsigned_return_signature_slice",
        source: "unsigned short SwapSHORT(unsigned short x) { return 42; } int main(void) { return 42; }\n",
    };
    let root = fresh_temp_dir(case.name);
    let source = root.join("case.c");
    let c99_asm = root.join("c99inrust.s");
    let c99_exe = executable_path(&root, "c99inrust");
    let clang_exe = executable_path(&root, "clang");
    fs::write(&source, case.source).expect("oracle source should be written");

    // when
    let c99_status = compile_with_c99inrust(&source, &c99_asm)
        .and_then(|()| assemble(&c99_asm, &c99_exe))
        .and_then(|()| run_exit_code(&c99_exe))
        .expect("c99inrust path should compile, link, and run");
    let clang_status = compile_with_host_c(&source, &clang_exe)
        .and_then(|()| run_exit_code(&clang_exe))
        .expect("host C compiler path should compile and run");

    // then
    assert_eq!(c99_status, clang_status);
}

#[test]
fn parameter_binding_slice_matches_host_c_compiler_exit_code() {
    // given
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    let case = OracleCase {
        name: "parameter_binding_slice",
        source: "int main(int argc, char **argv) { return argc; }\n",
    };
    let root = fresh_temp_dir(case.name);
    let source = root.join("case.c");
    let c99_asm = root.join("c99inrust.s");
    let c99_exe = executable_path(&root, "c99inrust");
    let clang_exe = executable_path(&root, "clang");
    fs::write(&source, case.source).expect("oracle source should be written");

    // when
    let c99_status = compile_with_c99inrust(&source, &c99_asm)
        .and_then(|()| assemble(&c99_asm, &c99_exe))
        .and_then(|()| run_exit_code(&c99_exe))
        .expect("c99inrust path should compile, link, and run");
    let clang_status = compile_with_host_c(&source, &clang_exe)
        .and_then(|()| run_exit_code(&clang_exe))
        .expect("host C compiler path should compile and run");

    // then
    assert_eq!(c99_status, clang_status);
}

#[test]
fn signed_long_long_cast_slice_matches_host_c_compiler_exit_code() {
    // given
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    let case = OracleCase {
        name: "signed_long_long_cast_slice",
        source: "int main(int argc, char **argv) { int a = argc << 30; return ((long long) a * 4) >> 30; }\n",
    };
    let root = fresh_temp_dir(case.name);
    let source = root.join("case.c");
    let c99_asm = root.join("c99inrust.s");
    let c99_exe = executable_path(&root, "c99inrust");
    let clang_exe = executable_path(&root, "clang");
    fs::write(&source, case.source).expect("oracle source should be written");

    // when
    let c99_status = compile_with_c99inrust(&source, &c99_asm)
        .and_then(|()| assemble(&c99_asm, &c99_exe))
        .and_then(|()| run_exit_code(&c99_exe))
        .expect("c99inrust path should compile, link, and run");
    let clang_status = compile_with_host_c(&source, &clang_exe)
        .and_then(|()| run_exit_code(&clang_exe))
        .expect("host C compiler path should compile and run");

    // then
    assert_eq!(c99_status, clang_status);
}

#[test]
fn function_call_argument_slice_matches_host_c_compiler_exit_code() {
    // given
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    let case = OracleCase {
        name: "function_call_argument_slice",
        source: "int add(int left, int right) { return left + right; } int main(int argc, char **argv) { return add(argc, 41); }\n",
    };
    let root = fresh_temp_dir(case.name);
    let source = root.join("case.c");
    let c99_asm = root.join("c99inrust.s");
    let c99_exe = executable_path(&root, "c99inrust");
    let clang_exe = executable_path(&root, "clang");
    fs::write(&source, case.source).expect("oracle source should be written");

    // when
    let c99_status = compile_with_c99inrust(&source, &c99_asm)
        .and_then(|()| assemble(&c99_asm, &c99_exe))
        .and_then(|()| run_exit_code(&c99_exe))
        .expect("c99inrust path should compile, link, and run");
    let clang_status = compile_with_host_c(&source, &clang_exe)
        .and_then(|()| run_exit_code(&clang_exe))
        .expect("host C compiler path should compile and run");

    // then
    assert_eq!(c99_status, clang_status);
}

#[test]
fn conditional_expression_slice_matches_host_c_compiler_exit_code() {
    // given
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    let case = OracleCase {
        name: "conditional_expression_slice",
        source: "int main(int argc, char **argv) { return argc < 0 ? 2 : 42; }\n",
    };
    let root = fresh_temp_dir(case.name);
    let source = root.join("case.c");
    let c99_asm = root.join("c99inrust.s");
    let c99_exe = executable_path(&root, "c99inrust");
    let clang_exe = executable_path(&root, "clang");
    fs::write(&source, case.source).expect("oracle source should be written");

    // when
    let c99_status = compile_with_c99inrust(&source, &c99_asm)
        .and_then(|()| assemble(&c99_asm, &c99_exe))
        .and_then(|()| run_exit_code(&c99_exe))
        .expect("c99inrust path should compile, link, and run");
    let clang_status = compile_with_host_c(&source, &clang_exe)
        .and_then(|()| run_exit_code(&clang_exe))
        .expect("host C compiler path should compile and run");

    // then
    assert_eq!(c99_status, clang_status);
}

#[test]
fn enum_global_initializer_matches_host_c_compiler_exit_code() {
    // given
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    let case = OracleCase {
        name: "enum_global_initializer",
        source: "typedef enum { shareware, registered, indetermined } GameMode_t; GameMode_t gamemode = indetermined; int main(void) { return gamemode == 2 ? 0 : 1; }\n",
    };
    let root = fresh_temp_dir(case.name);
    let source = root.join("case.c");
    let c99_asm = root.join("c99inrust.s");
    let c99_exe = executable_path(&root, "c99inrust");
    let clang_exe = executable_path(&root, "clang");
    fs::write(&source, case.source).expect("oracle source should be written");

    // when
    let c99_status = compile_with_c99inrust(&source, &c99_asm)
        .and_then(|()| assemble(&c99_asm, &c99_exe))
        .and_then(|()| run_exit_code(&c99_exe))
        .expect("c99inrust path should compile, link, and run");
    let clang_status = compile_with_host_c(&source, &clang_exe)
        .and_then(|()| run_exit_code(&clang_exe))
        .expect("host C compiler path should compile and run");

    // then
    assert_eq!(c99_status, clang_status);
}

#[test]
fn enum_arithmetic_initializer_matches_host_c_compiler_exit_code() {
    // given
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    let case = OracleCase {
        name: "enum_arithmetic_initializer",
        source: "typedef enum { INVULNTICS = (30*35) } powerduration_t; int main(void) { return INVULNTICS == 1050 ? 0 : 1; }\n",
    };
    let root = fresh_temp_dir(case.name);
    let source = root.join("case.c");
    let c99_asm = root.join("c99inrust.s");
    let c99_exe = executable_path(&root, "c99inrust");
    let clang_exe = executable_path(&root, "clang");
    fs::write(&source, case.source).expect("oracle source should be written");

    // when
    let c99_status = compile_with_c99inrust(&source, &c99_asm)
        .and_then(|()| assemble(&c99_asm, &c99_exe))
        .and_then(|()| run_exit_code(&c99_exe))
        .expect("c99inrust path should compile, link, and run");
    let clang_status = compile_with_host_c(&source, &clang_exe)
        .and_then(|()| run_exit_code(&clang_exe))
        .expect("host C compiler path should compile and run");

    // then
    assert_eq!(c99_status, clang_status);
}

#[test]
fn enum_additive_chain_initializer_matches_host_c_compiler_exit_code() {
    // given
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    let case = OracleCase {
        name: "enum_additive_chain_initializer",
        source: "typedef enum { BT_WEAPONMASK = (8+16+32) } buttoncode_t; int main(void) { return BT_WEAPONMASK == 56 ? 0 : 1; }\n",
    };
    let root = fresh_temp_dir(case.name);
    let source = root.join("case.c");
    let c99_asm = root.join("c99inrust.s");
    let c99_exe = executable_path(&root, "c99inrust");
    let clang_exe = executable_path(&root, "clang");
    fs::write(&source, case.source).expect("oracle source should be written");

    // when
    let c99_status = compile_with_c99inrust(&source, &c99_asm)
        .and_then(|()| assemble(&c99_asm, &c99_exe))
        .and_then(|()| run_exit_code(&c99_exe))
        .expect("c99inrust path should compile, link, and run");
    let clang_status = compile_with_host_c(&source, &clang_exe)
        .and_then(|()| run_exit_code(&clang_exe))
        .expect("host C compiler path should compile and run");

    // then
    assert_eq!(c99_status, clang_status);
}

#[test]
fn enum_mixed_precedence_initializer_matches_host_c_compiler_exit_code() {
    // given
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    let case = OracleCase {
        name: "enum_mixed_precedence_initializer",
        source: "typedef enum { MIXED = (8+16*32-4/2) } buttoncode_t; int main(void) { return MIXED == 518 ? 0 : 1; }\n",
    };
    let root = fresh_temp_dir(case.name);
    let source = root.join("case.c");
    let c99_asm = root.join("c99inrust.s");
    let c99_exe = executable_path(&root, "c99inrust");
    let clang_exe = executable_path(&root, "clang");
    fs::write(&source, case.source).expect("oracle source should be written");

    // when
    let c99_status = compile_with_c99inrust(&source, &c99_asm)
        .and_then(|()| assemble(&c99_asm, &c99_exe))
        .and_then(|()| run_exit_code(&c99_exe))
        .expect("c99inrust path should compile, link, and run");
    let clang_status = compile_with_host_c(&source, &clang_exe)
        .and_then(|()| run_exit_code(&clang_exe))
        .expect("host C compiler path should compile and run");

    // then
    assert_eq!(c99_status, clang_status);
}

#[test]
fn fixeddiv2_double_slice_matches_host_c_compiler_exit_code() {
    // given
    if !cfg!(target_os = "macos") || !command_exists("cc") {
        return;
    }
    let case = OracleCase {
        name: "fixeddiv2_double_slice",
        source: r#"typedef int fixed_t;
void I_Error(char *message) { return; }
fixed_t FixedDiv2(fixed_t a, fixed_t b) {
    double c;
    c = ((double)a) / ((double)b) * (1<<16);
    if (c >= 2147483648.0 || c < -2147483648.0)
        I_Error("FixedDiv: divide by zero");
    return (fixed_t)c;
}
int main(void) { return FixedDiv2(3, 2) == 98304 ? 0 : 1; }
"#,
    };
    let root = fresh_temp_dir(case.name);
    let source = root.join("case.c");
    let c99_asm = root.join("c99inrust.s");
    let c99_exe = executable_path(&root, "c99inrust");
    let clang_exe = executable_path(&root, "clang");
    fs::write(&source, case.source).expect("oracle source should be written");

    // when
    let c99_status = compile_with_c99inrust(&source, &c99_asm)
        .and_then(|()| assemble(&c99_asm, &c99_exe))
        .and_then(|()| run_exit_code(&c99_exe))
        .expect("c99inrust path should compile, link, and run");
    let clang_status = compile_with_host_c(&source, &clang_exe)
        .and_then(|()| run_exit_code(&clang_exe))
        .expect("host C compiler path should compile and run");

    // then
    assert_eq!(c99_status, clang_status);
}

#[test]
fn m_random_global_array_slice_matches_host_c_compiler_exit_code() {
    // given
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    let case = OracleCase {
        name: "m_random_global_array_slice",
        source: r"unsigned char rndtable[4] = { 3, 5, 7, 11 };
int rndindex = 0;
int prndindex = 0;
int P_Random(void) {
    prndindex = (prndindex + 1) & 0x3;
    return rndtable[prndindex];
}
int M_Random(void) {
    rndindex = (rndindex + 1) & 0x3;
    return rndtable[rndindex];
}
void M_ClearRandom(void) {
    rndindex = prndindex = 0;
}
int main(void) {
    int a = P_Random();
    int b = M_Random();
    M_ClearRandom();
    return a == 5 && b == 5 ? 0 : 1;
}
",
    };
    let root = fresh_temp_dir(case.name);
    let source = root.join("case.c");
    let c99_asm = root.join("c99inrust.s");
    let c99_exe = executable_path(&root, "c99inrust");
    let clang_exe = executable_path(&root, "clang");
    fs::write(&source, case.source).expect("oracle source should be written");

    // when
    let c99_status = compile_with_c99inrust(&source, &c99_asm)
        .and_then(|()| assemble(&c99_asm, &c99_exe))
        .and_then(|()| run_exit_code(&c99_exe))
        .expect("c99inrust path should compile, link, and run");
    let clang_status = compile_with_host_c(&source, &clang_exe)
        .and_then(|()| run_exit_code(&clang_exe))
        .expect("host C compiler path should compile and run");

    // then
    assert_eq!(c99_status, clang_status);
}

#[test]
fn m_bbox_pointer_subscript_slice_matches_host_c_compiler_exit_code() {
    // given
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    let case = OracleCase {
        name: "m_bbox_pointer_subscript_slice",
        source: r"enum { BOXTOP, BOXBOTTOM, BOXLEFT, BOXRIGHT };
void M_ClearBox(int *box) {
    box[BOXTOP] = box[BOXRIGHT] = -1;
    box[BOXBOTTOM] = box[BOXLEFT] = 10;
}
void M_AddToBox(int *box, int x, int y) {
    if (x < box[BOXLEFT])
        box[BOXLEFT] = x;
    else if (x > box[BOXRIGHT])
        box[BOXRIGHT] = x;
    if (y < box[BOXBOTTOM])
        box[BOXBOTTOM] = y;
    else if (y > box[BOXTOP])
        box[BOXTOP] = y;
}
int main(void) { return 0; }
",
    };
    let root = fresh_temp_dir(case.name);
    let source = root.join("case.c");
    let c99_asm = root.join("c99inrust.s");
    let c99_exe = executable_path(&root, "c99inrust");
    let clang_exe = executable_path(&root, "clang");
    fs::write(&source, case.source).expect("oracle source should be written");

    // when
    let c99_status = compile_with_c99inrust(&source, &c99_asm)
        .and_then(|()| assemble(&c99_asm, &c99_exe))
        .and_then(|()| run_exit_code(&c99_exe))
        .expect("c99inrust path should compile, link, and run");
    let clang_status = compile_with_host_c(&source, &clang_exe)
        .and_then(|()| run_exit_code(&clang_exe))
        .expect("host C compiler path should compile and run");

    // then
    assert_eq!(c99_status, clang_status);
}

#[test]
fn i_main_global_pointer_slice_matches_host_c_compiler_exit_code() {
    // given
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    let case = OracleCase {
        name: "i_main_global_pointer_slice",
        source: r"extern int myargc;
extern char **myargv;
int myargc;
char **myargv;
void D_DoomMain(void) { return; }
int main(int argc, char **argv) {
    myargc = argc;
    myargv = argv;
    D_DoomMain();
    if (myargv != argv)
        return 2;
    return myargc == argc ? 0 : 1;
}
",
    };
    let root = fresh_temp_dir(case.name);
    let source = root.join("case.c");
    let c99_asm = root.join("c99inrust.s");
    let c99_exe = executable_path(&root, "c99inrust");
    let clang_exe = executable_path(&root, "clang");
    fs::write(&source, case.source).expect("oracle source should be written");

    // when
    let c99_status = compile_with_c99inrust(&source, &c99_asm)
        .and_then(|()| assemble(&c99_asm, &c99_exe))
        .and_then(|()| run_exit_code(&c99_exe))
        .expect("c99inrust path should compile, link, and run");
    let clang_status = compile_with_host_c(&source, &clang_exe)
        .and_then(|()| run_exit_code(&clang_exe))
        .expect("host C compiler path should compile and run");

    // then
    assert_eq!(c99_status, clang_status);
}

struct OracleCase {
    name: &'static str,
    source: &'static str,
}

fn compile_with_c99inrust(source: &Path, output: &Path) -> Result<(), String> {
    let compiler = env!("CARGO_BIN_EXE_c99inrust");
    let status = Command::new(compiler)
        .arg("compile")
        .arg("-S")
        .arg(source)
        .arg("-o")
        .arg(output)
        .status()
        .map_err(|error| format!("failed to run c99inrust: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("c99inrust exited with {status}"))
    }
}

fn build_with_c99inrust(source: &Path, output: &Path) -> Result<(), String> {
    let compiler = env!("CARGO_BIN_EXE_c99inrust");
    let status = Command::new(compiler)
        .arg("build")
        .arg(source)
        .arg("-o")
        .arg(output)
        .status()
        .map_err(|error| format!("failed to run c99inrust build: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("c99inrust build exited with {status}"))
    }
}

fn compile_with_host_c(source: &Path, output: &Path) -> Result<(), String> {
    let status = Command::new("cc")
        .arg("-std=c99")
        .arg("-O0")
        .arg(source)
        .arg("-o")
        .arg(output)
        .status()
        .map_err(|error| format!("failed to run host C compiler: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("host C compiler exited with {status}"))
    }
}

fn assemble(source: &Path, output: &Path) -> Result<(), String> {
    let status = Command::new("cc")
        .arg(source)
        .arg("-o")
        .arg(output)
        .status()
        .map_err(|error| format!("failed to assemble c99inrust output: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("assembler exited with {status}"))
    }
}

fn run_exit_code(executable: &Path) -> Result<i32, String> {
    Command::new(executable)
        .status()
        .map_err(|error| format!("failed to run executable: {error}"))
        .map(|status| status.code().unwrap_or(255))
}

fn command_exists(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn fresh_temp_dir(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("c99inrust-oracle-{}-{name}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("old oracle temp dir should be removed");
    }
    fs::create_dir_all(&root).expect("oracle temp dir should be created");
    root
}

fn executable_path(root: &Path, name: &str) -> PathBuf {
    if cfg!(windows) {
        root.join(format!("{name}.exe"))
    } else {
        root.join(name)
    }
}
