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
