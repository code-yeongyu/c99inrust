use std::path::{Path, PathBuf};

use crate::diagnostics::{CompileError, CompileResult};

pub(super) fn resolve_include(
    include_paths: &[PathBuf],
    include_path: &str,
    current_dir: Option<&Path>,
) -> CompileResult<PathBuf> {
    if let Some(dir) = current_dir {
        let candidate = dir.join(include_path);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    for dir in include_paths {
        let candidate = dir.join(include_path);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(CompileError::new(format!(
        "include not found: {include_path}"
    )))
}
