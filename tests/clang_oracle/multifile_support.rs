use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Clone, Copy)]
pub struct OracleSourceFile {
    pub path: &'static str,
    pub source: &'static str,
}

#[derive(Clone, Copy)]
pub struct OracleMultiFileCase {
    pub name: &'static str,
    pub files: &'static [OracleSourceFile],
}

pub fn assert_multifile_compile_run_matches_host(case: OracleMultiFileCase) {
    if cfg!(windows) || !command_exists("cc") {
        return;
    }
    assert_multifile_case_matches_host(case);
}

fn assert_multifile_case_matches_host(case: OracleMultiFileCase) {
    // given
    let root = fresh_temp_dir(case.name);
    let c99_exe = executable_path(&root, "c99inrust");
    let clang_exe = executable_path(&root, "clang");
    write_oracle_files(&root, case.files);
    let units = case
        .files
        .iter()
        .filter(|file| is_c_source(file.path))
        .collect::<Vec<_>>();

    // when
    let c99_output = units
        .iter()
        .enumerate()
        .map(|(index, file)| compile_c99_unit(&root, file.path, index))
        .collect::<Result<Vec<_>, _>>()
        .and_then(|objects| link_objects(&objects, &c99_exe).map(|()| objects))
        .and_then(|_objects| run_program(&c99_exe))
        .expect("c99inrust multi-file path should compile, link, and run");
    let clang_sources = units
        .iter()
        .map(|file| root.join(file.path))
        .collect::<Vec<_>>();
    let clang_output = compile_with_host_c_files(&clang_sources, &clang_exe)
        .and_then(|()| run_program(&clang_exe))
        .expect("host C compiler multi-file path should compile and run");

    // then
    assert_eq!(c99_output, clang_output);
}

fn compile_c99_unit(root: &Path, source_path: &str, index: usize) -> Result<PathBuf, String> {
    let source = root.join(source_path);
    let asm = root.join(format!("unit{index}.s"));
    let object = root.join(format!("unit{index}.o"));
    compile_with_c99inrust(&source, &asm)
        .and_then(|()| assemble_object(&asm, &object))
        .map(|()| object)
}

fn is_c_source(path: &str) -> bool {
    Path::new(path)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("c"))
}

fn write_oracle_files(root: &Path, files: &[OracleSourceFile]) {
    for file in files {
        let path = root.join(file.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("oracle source parent should be created");
        }
        fs::write(path, file.source).expect("oracle source should be written");
    }
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

fn compile_with_host_c_files(sources: &[PathBuf], output: &Path) -> Result<(), String> {
    let mut command = Command::new("cc");
    command.arg("-std=c99").arg("-O0");
    for source in sources {
        command.arg(source);
    }
    let status = command
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

fn assemble_object(source: &Path, output: &Path) -> Result<(), String> {
    let status = Command::new("cc")
        .arg("-c")
        .arg(source)
        .arg("-o")
        .arg(output)
        .status()
        .map_err(|error| format!("failed to assemble c99inrust object: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("object assembler exited with {status}"))
    }
}

fn link_objects(objects: &[PathBuf], output: &Path) -> Result<(), String> {
    let mut command = Command::new("cc");
    for object in objects {
        command.arg(object);
    }
    let status = command
        .arg("-o")
        .arg(output)
        .status()
        .map_err(|error| format!("failed to link c99inrust objects: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("object linker exited with {status}"))
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ProgramOutput {
    exit_code: i32,
    stdout: Vec<u8>,
}

fn run_program(executable: &Path) -> Result<ProgramOutput, String> {
    let output = Command::new(executable)
        .output()
        .map_err(|error| format!("failed to run executable: {error}"))?;
    Ok(ProgramOutput {
        exit_code: output.status.code().unwrap_or(255),
        stdout: output.stdout,
    })
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
