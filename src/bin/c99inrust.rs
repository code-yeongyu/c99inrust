use std::process::ExitCode;

#[path = "c99inrust/mod.rs"]
mod cli;

fn main() -> ExitCode {
    cli::run_from_env()
}
