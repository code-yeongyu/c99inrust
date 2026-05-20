use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::diagnostics::{CompileError, CompileResult};
use c99inrust::front_end::lexer::lex;
use c99inrust::ir::lower;
use c99inrust::parser::{parse_supported_translation_unit, parse_translation_unit};

use self::args::{CommonArgs, one_path, parse_common_args, preprocessor_from};
use self::doom_audit::doom_audit_command;

mod args;
mod doom_audit;

pub fn run_from_env() -> ExitCode {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(2)
        }
    }
}

fn run(args: &[String]) -> CompileResult<()> {
    let Some(command) = args.first() else {
        usage();
        return Ok(());
    };
    match command.as_str() {
        "lex" => lex_command(&args[1..]),
        "preprocess" => preprocess_command(&args[1..]),
        "parse-check" => parse_check_command(&args[1..]),
        "compile" => compile_command(&args[1..]),
        "build" => build_command(&args[1..]),
        "doom-audit" => doom_audit_command(&args[1..]),
        "help" | "--help" | "-h" => {
            usage();
            Ok(())
        }
        _ => Err(CompileError::new(format!("unknown command: {command}"))),
    }
}

fn lex_command(args: &[String]) -> CompileResult<()> {
    let input = one_path(args, "lex <input.c>")?;
    let source = fs::read_to_string(&input)
        .map_err(|error| CompileError::new(format!("failed to read input: {error}")))?;
    for token in lex(&source)? {
        println!("{:?}", token.kind);
    }
    Ok(())
}

fn preprocess_command(args: &[String]) -> CompileResult<()> {
    let common = parse_common_args(args, "preprocess")?;
    let unit = preprocessor_from(&common).preprocess_file(&common.input)?;
    print!("{}", unit.source);
    Ok(())
}

fn parse_check_command(args: &[String]) -> CompileResult<()> {
    let common = parse_common_args(args, "parse-check")?;
    let unit = preprocessor_from(&common).preprocess_file(&common.input)?;
    let tokens = lex(&unit.source)?;
    let surface = parse_translation_unit(&tokens)?;
    println!("items={}", surface.items.len());
    println!("typedefs={}", surface.typedef_count());
    println!("prototypes={}", surface.prototype_count());
    println!("declarations={}", surface.declaration_count());
    println!(
        "function-definitions={}",
        surface.function_definition_count()
    );
    println!("struct-forwards={}", surface.struct_forward_count());
    Ok(())
}

fn compile_command(args: &[String]) -> CompileResult<()> {
    let common = parse_common_args(args, "compile")?;
    let output = common
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from("a.s"));
    let assembly = compile_to_assembly(&common)?;
    fs::write(&output, assembly)
        .map_err(|error| CompileError::new(format!("failed to write assembly: {error}")))?;
    Ok(())
}

fn build_command(args: &[String]) -> CompileResult<()> {
    let common = parse_common_args(args, "build")?;
    if common.target != Target::native() {
        return Err(CompileError::new(
            "build currently supports native target assembly only",
        ));
    }
    let output = common
        .output
        .clone()
        .unwrap_or_else(default_executable_path);
    let assembly = compile_to_assembly(&common)?;
    link_assembly(&assembly, &output)
}

fn compile_to_assembly(common: &CommonArgs) -> CompileResult<String> {
    let unit = preprocessor_from(common).preprocess_file(&common.input)?;
    let tokens = lex(&unit.source)?;
    let program = parse_supported_translation_unit(&tokens)?;
    let lowered = lower(&program)?;
    emit_assembly(&lowered, common.target)
}

fn link_assembly(assembly: &str, output: &Path) -> CompileResult<()> {
    let mut child = Command::new("cc")
        .arg("-x")
        .arg("assembler")
        .arg("-")
        .arg("-o")
        .arg(output)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|error| CompileError::new(format!("failed to run host assembler: {error}")))?;
    let Some(mut stdin) = child.stdin.take() else {
        return Err(CompileError::new("failed to open host assembler stdin"));
    };
    stdin
        .write_all(assembly.as_bytes())
        .map_err(|error| CompileError::new(format!("failed to write assembly: {error}")))?;
    drop(stdin);
    let status = child.wait().map_err(|error| {
        CompileError::new(format!("failed to wait for host assembler: {error}"))
    })?;
    if status.success() {
        Ok(())
    } else {
        Err(CompileError::new(format!(
            "host assembler exited with {status}"
        )))
    }
}

fn default_executable_path() -> PathBuf {
    if cfg!(windows) {
        PathBuf::from("a.exe")
    } else {
        PathBuf::from("a.out")
    }
}

fn usage() {
    println!("c99inrust");
    println!("usage:");
    println!("  c99inrust lex <input.c>");
    println!("  c99inrust preprocess [-D NAME[=VALUE]] [-I include] <input.c>");
    println!("  c99inrust parse-check [-D NAME[=VALUE]] [-I include] <input.c>");
    println!(
        "  c99inrust compile [-S] [--target native] [-D NAME[=VALUE]] [-I include] <input.c> -o <out.s>"
    );
    println!(
        "  c99inrust build [--target native] [-D NAME[=VALUE]] [-I include] <input.c> -o <out>"
    );
    println!("  c99inrust doom-audit <official-doom-checkout>");
}
