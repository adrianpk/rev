use std::fmt;
use std::ops::Range;
use std::path::{Path, PathBuf};

/// One-based source position; columns count Unicode scalars, not UTF-8 bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

/// One rejected source, with a decoded key path and original UTF-8 byte span
/// where available. Line and column are one-based; columns count Unicode scalars.
/// Messages omit source contents and unrelated environment values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub source: PathBuf,
    pub key: Vec<String>,
    pub span: Option<Range<usize>>,
    pub position: Option<Position>,
    pub message: String,
}

impl Diagnostic {
    pub(super) fn new(source: &Path, message: String) -> Self {
        Self {
            source: source.to_owned(),
            key: Vec::new(),
            span: None,
            position: None,
            message,
        }
    }

    pub(super) fn locate(mut self, text: &str, span: Option<Range<usize>>) -> Self {
        if let Some(prefix) = span.as_ref().and_then(|span| text.get(..span.start)) {
            self.position = Some(Position {
                line: prefix.bytes().filter(|byte| *byte == b'\n').count() + 1,
                column: prefix.rsplit('\n').next().unwrap_or("").chars().count() + 1,
            });
        }
        self.span = span;
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(output, "{}", self.source.display())?;
        if let Some(position) = self.position {
            write!(output, ":{}:{}", position.line, position.column)?;
        }
        if !self.key.is_empty() {
            write!(output, " [{}]", self.key.join("."))?;
        }
        write!(output, ": {}", self.message)
    }
}

impl std::error::Error for Diagnostic {}
