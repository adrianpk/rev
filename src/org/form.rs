use crate::buffer::Error;

use super::{Source, SourceRange};

/// Source classifications for the native recognizer. Unknown and malformed
/// observations retain their original ranges instead of inventing valid syntax.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormKind {
    Text,
    Heading,
    Directive,
    Block,
    Unknown,
    Malformed,
}

/// Derived classification and location only: no copied body, editable document,
/// evaluated value or directive execution capability is stored here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Form {
    kind: FormKind,
    source: SourceRange,
}

impl Source<'_> {
    /// Attach a recognizer-supplied classification to a current checked range.
    /// This validates source identity and revision; it does not parse or execute.
    pub fn form(&self, kind: FormKind, source: SourceRange) -> Result<Form, Error> {
        self.text(&source)?;
        Ok(Form { kind, source })
    }
}

impl Form {
    pub fn kind(&self) -> FormKind {
        self.kind
    }

    /// Every later text access or edit must revalidate this observed location.
    pub fn source(&self) -> &SourceRange {
        &self.source
    }
}
