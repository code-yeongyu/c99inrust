use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn cli_preprocesses_define_arguments() {
    // given
    let root = fresh_temp_dir("preprocess-defines");
    let source = root.join("main.c");
    fs::write(&source, "int main(void) { return ANSWER; }\n").expect("source should be written");

    // when
    let output = Command::new(compiler())
        .arg("preprocess")
        .arg("-D")
        .arg("ANSWER=42")
        .arg(&source)
        .output()
        .expect("preprocess command should run");

    // then
    assert!(output.status.success(), "stderr={}", stderr(&output));
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("return 42;"));
}

#[test]
fn cli_parse_check_reports_translation_unit_counts() {
    // given
    let root = fresh_temp_dir("parse-check");
    let source = root.join("main.c");
    fs::write(&source, "int helper(void); int main(void) { return 0; }\n")
        .expect("source should be written");

    // when
    let output = Command::new(compiler())
        .arg("parse-check")
        .arg(&source)
        .output()
        .expect("parse-check command should run");

    // then
    assert!(output.status.success(), "stderr={}", stderr(&output));
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("items=2"));
    assert!(stdout.contains("prototypes=1"));
    assert!(stdout.contains("function-definitions=1"));
}

#[test]
fn cli_compile_writes_target_assembly() {
    // given
    let root = fresh_temp_dir("compile-assembly");
    let source = root.join("main.c");
    let assembly = root.join("main.s");
    fs::write(&source, "int main(void) { return 42; }\n").expect("source should be written");

    // when
    let output = Command::new(compiler())
        .arg("compile")
        .arg("-S")
        .arg("--target")
        .arg("x86_64-unknown-linux-gnu")
        .arg(&source)
        .arg("-o")
        .arg(&assembly)
        .output()
        .expect("compile command should run");

    // then
    assert!(output.status.success(), "stderr={}", stderr(&output));
    let generated = fs::read_to_string(&assembly).expect("assembly should be written");
    assert!(generated.contains(".globl main"));
    assert!(generated.contains("movl $42, %eax"));
}

#[test]
fn cli_doom_audit_reports_current_doom_gate_status() {
    // given
    let root = fresh_temp_dir("doom-audit");
    let linuxdoom = root.join("linuxdoom-1.10");
    fs::create_dir_all(&linuxdoom).expect("linuxdoom dir should be created");
    fs::write(linuxdoom.join("Makefile"), "all:\n").expect("Makefile should be written");
    fs::write(linuxdoom.join("d_main.c"), "int main(void) { return 0; }\n")
        .expect("C source should be written");
    fs::write(linuxdoom.join("doomdef.h"), "#define DOOM 1\n").expect("header should be written");

    // when
    let output = Command::new(compiler())
        .arg("doom-audit")
        .arg(&root)
        .output()
        .expect("doom-audit command should run");

    // then
    assert!(output.status.success(), "stderr={}", stderr(&output));
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("linuxdoom-c-files=1"));
    assert!(stdout.contains("linuxdoom-h-files=1"));
    assert!(stdout.contains("linuxdoom-makefile=true"));
    assert!(stdout.contains("compile-smoke=ok compile_ok=62 compile_fail=0"));
    assert!(stdout.contains("link-smoke=ok link_status=0"));
    assert!(stdout.contains("movement-smoke=ok movement_status=0"));
    assert!(stdout.contains(
        "status=official Doom compile/link/run smoke verified; human playthrough transcript pending"
    ));
}

const fn compiler() -> &'static str {
    env!("CARGO_BIN_EXE_c99inrust")
}

fn fresh_temp_dir(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("c99inrust-cli-{name}-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("stale temp dir should be removed");
    }
    fs::create_dir_all(&root).expect("temp dir should be created");
    root
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}
