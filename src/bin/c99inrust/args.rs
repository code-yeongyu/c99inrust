use std::path::PathBuf;

use c99inrust::codegen::Target;
use c99inrust::diagnostics::{CompileError, CompileResult};
use c99inrust::front_end::preprocessor::Preprocessor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CommonArgs {
    pub(super) include_paths: Vec<PathBuf>,
    pub(super) defines: Vec<(String, String)>,
    pub(super) input: PathBuf,
    pub(super) output: Option<PathBuf>,
    pub(super) target: Target,
}

pub(super) fn one_path(args: &[String], usage_text: &str) -> CompileResult<PathBuf> {
    if args.len() != 1 {
        return Err(CompileError::new(format!("usage: c99inrust {usage_text}")));
    }
    Ok(PathBuf::from(&args[0]))
}

pub(super) fn preprocessor_from(common: &CommonArgs) -> Preprocessor {
    let mut preprocessor = Preprocessor::new();
    for path in &common.include_paths {
        preprocessor = preprocessor.with_include_path(path);
    }
    for (name, value) in &common.defines {
        preprocessor = preprocessor.with_define(name, value);
    }
    preprocessor
}

pub(super) fn parse_common_args(args: &[String], command: &str) -> CompileResult<CommonArgs> {
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
