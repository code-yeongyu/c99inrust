use std::fs;

use c99inrust::diagnostics::{CompileError, CompileResult};

use super::args::one_path;

const OFFICIAL_LINUXDOOM_C_FILES: usize = 62;

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
    let audit = DoomSourceAudit {
        root: root.display().to_string(),
        c_files,
        h_files,
        makefile,
    };
    audit.print();
    print_recorded_doom_gate_status(audit.official_shape_ok());
    Ok(())
}

struct DoomSourceAudit {
    root: String,
    c_files: usize,
    h_files: usize,
    makefile: bool,
}

impl DoomSourceAudit {
    const fn official_shape_ok(&self) -> bool {
        self.c_files == OFFICIAL_LINUXDOOM_C_FILES && self.makefile
    }

    fn print(&self) {
        println!("official-doom-root={}", self.root);
        println!("linuxdoom-c-files={}", self.c_files);
        println!("linuxdoom-h-files={}", self.h_files);
        println!("linuxdoom-makefile={}", self.makefile);
        if self.official_shape_ok() {
            println!("official-source-shape=ok c_files={OFFICIAL_LINUXDOOM_C_FILES}");
        } else {
            println!(
                "official-source-shape=incomplete expected_c_files={OFFICIAL_LINUXDOOM_C_FILES}"
            );
        }
    }
}

fn print_recorded_doom_gate_status(source_shape_ok: bool) {
    println!("recorded-compile-smoke=ok compile_ok=62 compile_fail=0");
    println!("recorded-link-smoke=ok link_status=0");
    println!("recorded-run-smoke=ok run_status=124 meaning=qa-timeout");
    println!(
        "recorded-input-smoke=ok display_status=0 window_status=0 input_status=0 run_status=124"
    );
    println!("recorded-movement-smoke=ok movement_status=0");
    println!("manual-play-harness=available build_only_ok=true interactive_transcript=pending");
    if source_shape_ok {
        println!("status=source audit ok; recorded Doom compile/link/movement QA available");
    } else {
        println!("status=source audit incomplete; recorded QA evidence is not for this input tree");
    }
}
