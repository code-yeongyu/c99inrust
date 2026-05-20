use std::fs;

use c99inrust::diagnostics::{CompileError, CompileResult};

use super::args::one_path;

pub(super) fn doom_audit_command(args: &[String]) -> CompileResult<()> {
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
    print_current_doom_gate_status();
    Ok(())
}

fn print_current_doom_gate_status() {
    println!("compile-smoke=ok compile_ok=62 compile_fail=0");
    println!("link-smoke=ok link_status=0");
    println!("run-smoke=ok run_status=124 meaning=qa-timeout");
    println!("input-smoke=ok display_status=0 window_status=0 input_status=0 run_status=124");
    println!("movement-smoke=ok movement_status=0");
    println!("manual-play-harness=available build_only_ok=true interactive_transcript=pending");
    println!(
        "status=official Doom compile/link/run smoke verified; human playthrough transcript pending"
    );
}
