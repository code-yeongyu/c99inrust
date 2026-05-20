use crate::diagnostics::{CompileError, CompileResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Aarch64AppleDarwin,
    X86_64AppleDarwin,
    X86_64UnknownLinuxGnu,
}
impl Target {
    #[must_use]
    pub const fn native() -> Self {
        if cfg!(all(target_arch = "aarch64", target_os = "macos")) {
            Self::Aarch64AppleDarwin
        } else if cfg!(all(target_arch = "x86_64", target_os = "macos")) {
            Self::X86_64AppleDarwin
        } else {
            Self::X86_64UnknownLinuxGnu
        }
    }

    /// Parses a supported backend target triple.
    ///
    /// # Errors
    ///
    /// Returns an error when `value` is not `native` or one of the explicit
    /// target triples supported by the current backend.
    pub fn parse(value: &str) -> CompileResult<Self> {
        match value {
            "aarch64-apple-darwin" => Ok(Self::Aarch64AppleDarwin),
            "x86_64-apple-darwin" => Ok(Self::X86_64AppleDarwin),
            "x86_64-unknown-linux-gnu" => Ok(Self::X86_64UnknownLinuxGnu),
            "native" => Ok(Self::native()),
            _ => Err(CompileError::new(format!("unsupported target: {value}"))),
        }
    }
}
