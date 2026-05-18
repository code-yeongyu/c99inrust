use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use c99inrust::codegen::{Target, emit_assembly};
use c99inrust::diagnostics::{CompileError, CompileResult};
use c99inrust::front_end::lexer::lex;
use c99inrust::front_end::preprocessor::Preprocessor;
use c99inrust::ir::lower;
use c99inrust::parser::{parse, parse_translation_unit};

fn main() -> ExitCode {
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
    let unit = preprocessor_from(&common).preprocess_file(&common.input)?;
    let tokens = lex(&unit.source)?;
    let program = parse(&tokens)?;
    let lowered = lower(&program)?;
    let assembly = emit_assembly(&lowered, common.target)?;
    fs::write(&output, assembly)
        .map_err(|error| CompileError::new(format!("failed to write assembly: {error}")))?;
    Ok(())
}

fn doom_audit_command(args: &[String]) -> CompileResult<()> {
    let root = one_path(args, "doom-audit <official-doom-checkout>")?;
    let linuxdoom = root.join("linuxdoom-1.10");
    if !linuxdoom.is_dir() {
        return Err(CompileError::new(
            "expected official id-Software/DOOM checkout with linuxdoom-1.10",
        ));
    }
    let mut c_files = 0usize;
    let mut h_files = 0usize;
    let mut makefile = false;
    for entry in fs::read_dir(&linuxdoom)
        .map_err(|error| CompileError::new(format!("failed to read Doom source: {error}")))?
    {
        let entry = entry
            .map_err(|error| CompileError::new(format!("failed to read Doom entry: {error}")))?;
        let path = entry.path();
        if path.file_name().is_some_and(|name| name == "Makefile") {
            makefile = true;
        }
        match path.extension().and_then(std::ffi::OsStr::to_str) {
            Some("c") => c_files += 1,
            Some("h") => h_files += 1,
            _ => {}
        }
    }
    println!("official-doom-root={}", root.display());
    println!("linuxdoom-c-files={c_files}");
    println!("linuxdoom-h-files={h_files}");
    println!("linuxdoom-makefile={makefile}");
    println!("status=audited language surface only; full Doom compilation is a future milestone");
    Ok(())
}

fn one_path(args: &[String], usage_text: &str) -> CompileResult<PathBuf> {
    if args.len() != 1 {
        return Err(CompileError::new(format!("usage: c99inrust {usage_text}")));
    }
    Ok(PathBuf::from(&args[0]))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CommonArgs {
    include_paths: Vec<PathBuf>,
    defines: Vec<(String, String)>,
    input: PathBuf,
    output: Option<PathBuf>,
    target: Target,
}

fn preprocessor_from(common: &CommonArgs) -> Preprocessor {
    let mut preprocessor = Preprocessor::new();
    for path in &common.include_paths {
        preprocessor = preprocessor.with_include_path(path);
    }
    for (name, value) in &common.defines {
        preprocessor = preprocessor.with_define(name, value);
    }
    preprocessor
}

fn parse_common_args(args: &[String], command: &str) -> CompileResult<CommonArgs> {
    let mut include_paths = Vec::new();
    let mut defines = Vec::new();
    let mut input = None;
    let mut output = None;
    let mut target = Target::native();
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "-I" => {
                let Some(path) = args.get(index + 1) else {
                    return Err(CompileError::new("-I requires a path"));
                };
                include_paths.push(PathBuf::from(path));
                index += 2;
            }
            "-D" => {
                let Some(value) = args.get(index + 1) else {
                    return Err(CompileError::new("-D requires a macro name"));
                };
                defines.push(parse_define_arg(value)?);
                index += 2;
            }
            value if value.starts_with("-D") => {
                defines.push(parse_define_arg(&value[2..])?);
                index += 1;
            }
            "-o" => {
                let Some(path) = args.get(index + 1) else {
                    return Err(CompileError::new("-o requires a path"));
                };
                output = Some(PathBuf::from(path));
                index += 2;
            }
            "-S" => {
                index += 1;
            }
            "--target" => {
                let Some(value) = args.get(index + 1) else {
                    return Err(CompileError::new("--target requires a target triple"));
                };
                target = Target::parse(value)?;
                index += 2;
            }
            value if value.starts_with('-') => {
                return Err(CompileError::new(format!(
                    "unsupported option for {command}: {value}"
                )));
            }
            value => {
                if input.is_some() {
                    return Err(CompileError::new(format!("multiple inputs for {command}")));
                }
                input = Some(PathBuf::from(value));
                index += 1;
            }
        }
    }
    let Some(input) = input else {
        return Err(CompileError::new(format!(
            "usage: c99inrust {command} [opts] <input.c>"
        )));
    };
    Ok(CommonArgs {
        include_paths,
        defines,
        input,
        output,
        target,
    })
}

fn parse_define_arg(value: &str) -> CompileResult<(String, String)> {
    if value.is_empty() {
        return Err(CompileError::new("-D requires a macro name"));
    }
    let (name, replacement) = value
        .split_once('=')
        .map_or((value, "1"), |(name, replacement)| (name, replacement));
    if name.is_empty() {
        return Err(CompileError::new("-D requires a macro name"));
    }
    Ok((name.to_string(), replacement.to_string()))
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
    println!("  c99inrust doom-audit <official-doom-checkout>");
}
