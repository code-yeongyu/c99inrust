use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileError {
    message: String,
    path: Option<PathBuf>,
    line: Option<usize>,
    column: Option<usize>,
}

impl CompileError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            path: None,
            line: None,
            column: None,
        }
    }

    #[must_use]
    pub const fn at(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    #[must_use]
    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.path = Some(path);
        self
    }
}

impl Display for CompileError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(path) = &self.path {
            write!(formatter, "{}: ", path.display())?;
        }
        if let (Some(line), Some(column)) = (self.line, self.column) {
            write!(formatter, "{line}:{column}: ")?;
        }
        write!(formatter, "{}", self.message)
    }
}

impl Error for CompileError {}

pub type CompileResult<T> = Result<T, CompileError>;
