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
fn cli_compile_rejects_int_cast_string_for_struct_pointer_field() {
    // given
    let root = fresh_temp_dir("struct-pointer-int-cast");
    let source = root.join("main.c");
    let assembly = root.join("main.s");
    fs::write(
        &source,
        "typedef struct { char *text; } holder_t; holder_t holder = { (int)(\"doom\" + 1) }; int main(void) { return holder.text[0]; }\n",
    )
    .expect("source should be written");

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
    assert!(
        !output.status.success(),
        "compile unexpectedly succeeded with stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        stderr(&output)
            .contains("unsupported global struct pointer initializer string pointer cast")
    );
}

#[test]
fn cli_compile_emits_int_string_relocation_offset_for_struct_field() {
    // given
    let root = fresh_temp_dir("struct-int-string-offset");
    let source = root.join("main.c");
    let assembly = root.join("main.s");
    fs::write(
        &source,
        "typedef struct { int defaultvalue; } default_t; default_t defaults[] = { { (int)(\"microsoft\" + 5) } }; int main(void) { return defaults[0].defaultvalue != 0 ? 0 : 1; }\n",
    )
    .expect("source should be written");

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
    assert!(generated.contains("\t.long .Ldefaults_str0+5\n"));
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
    assert!(stdout.contains("official-source-shape=incomplete expected_c_files=62"));
    assert!(stdout.contains("recorded-compile-smoke=ok compile_ok=62 compile_fail=0"));
    assert!(stdout.contains("recorded-link-smoke=ok link_status=0"));
    assert!(stdout.contains("recorded-movement-smoke=ok movement_status=0"));
    assert!(stdout.contains(
        "status=source audit incomplete; recorded QA evidence is not for this input tree"
    ));
}

#[test]
fn cli_doom_audit_recognizes_official_source_shape() {
    // given
    let root = fresh_temp_dir("doom-audit-official-shape");
    let linuxdoom = root.join("linuxdoom-1.10");
    fs::create_dir_all(&linuxdoom).expect("linuxdoom dir should be created");
    fs::write(linuxdoom.join("Makefile"), "all:\n").expect("Makefile should be written");
    fs::write(linuxdoom.join("doomdef.h"), "#define DOOM 1\n").expect("header should be written");
    for index in 0..62 {
        fs::write(
            linuxdoom.join(format!("unit_{index}.c")),
            "int doom_unit(void) { return 0; }\n",
        )
        .expect("C source should be written");
    }

    // when
    let output = Command::new(compiler())
        .arg("doom-audit")
        .arg(&root)
        .output()
        .expect("doom-audit command should run");

    // then
    assert!(output.status.success(), "stderr={}", stderr(&output));
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(stdout.contains("linuxdoom-c-files=62"));
    assert!(stdout.contains("linuxdoom-h-files=1"));
    assert!(stdout.contains("linuxdoom-makefile=true"));
    assert!(stdout.contains("official-source-shape=ok c_files=62"));
    assert!(
        stdout.contains("status=source audit ok; recorded Doom compile/link/movement QA available")
    );
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
