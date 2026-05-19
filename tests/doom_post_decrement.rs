use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[test]
fn compiler_checks_original_pointer_value_for_doom_post_decrement_scan() {
    // given
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    let root = fresh_temp_dir("doom_post_decrement_scan");
    let source = root.join("case.c");
    let assembly = root.join("case.s");
    let executable = root.join("case");
    fs::write(
        &source,
        "int values[2]; int *base; int scan(void) { int *p; p = base + 2; while (p-- != base) { if (*p == 7) return p - base; } return -1; } int main(void) { values[0] = 7; values[1] = 9; base = values; return scan() == 0 ? 0 : 1; }\n",
    )
    .expect("oracle source should be written");

    // when
    compile_with_c99inrust(&source, &assembly)
        .and_then(|()| assemble(&assembly, &executable))
        .expect("c99inrust path should compile and link");
    let status = run_exit_code(&executable).expect("c99inrust executable should run");

    // then
    assert_eq!(
        status, 0,
        "post-decrement scan skipped the base element that Doom uses for PLAYPAL"
    );
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
    status
        .success()
        .then_some(())
        .ok_or_else(|| format!("c99inrust exited with {status}"))
}

fn assemble(source: &Path, output: &Path) -> Result<(), String> {
    let status = Command::new("cc")
        .arg(source)
        .arg("-o")
        .arg(output)
        .status()
        .map_err(|error| format!("failed to assemble c99inrust output: {error}"))?;
    status
        .success()
        .then_some(())
        .ok_or_else(|| format!("assembler exited with {status}"))
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
    let root = std::env::temp_dir().join(format!("c99inrust-{name}-{}", std::process::id()));
    fs::create_dir_all(&root).expect("temp dir should be created");
    root
}
