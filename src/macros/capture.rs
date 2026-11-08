use crate::command::PrefixArgument;
use crate::search::Direction;

use super::{DEFINITION_BYTES, Step};

/// A native prompt role, independent of labels and presentation rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Role {
    Path,
    Buffer,
    Line,
    Write,
    Search,
    SearchRepeat(Direction),
    ReplaceQuery,
    Replacement,
}

#[derive(Clone)]
pub(crate) struct Argument {
    pub role: Role,
    pub value: String,
}

#[derive(Clone)]
pub(crate) struct CommandStep {
    pub name: String,
    pub argument: PrefixArgument,
    // None is an accepted M-x/colon name. Key entry retains native lookup;
    // the canonical name never substitutes for a changed physical binding.
    pub keys: Option<Vec<Step>>,
    pub arguments: Vec<Argument>,
}

impl CommandStep {
    pub fn bytes(&self) -> usize {
        self.arguments
            .iter()
            .map(|argument| text_bytes(&argument.value).saturating_add(64))
            .chain(self.keys.iter().flatten().map(Step::bytes))
            .fold(
                text_bytes(&self.name).saturating_add(128),
                usize::saturating_add,
            )
    }
}

#[derive(Clone)]
pub(crate) struct CompletionEdit {
    pub start: i64,
    pub end: i64,
    pub expected: String,
    pub text: String,
}

/// Coordinates are scalar offsets relative to the current invocation point.
/// No provider, source identity or candidate ordinal survives acceptance.
#[derive(Clone)]
pub(crate) struct LiteralCompletion {
    pub edits: Vec<CompletionEdit>,
    pub cursor: i64,
}

impl LiteralCompletion {
    pub fn bytes(&self) -> usize {
        self.edits.iter().fold(128, |bytes, edit| {
            bytes
                .saturating_add(text_bytes(&edit.expected))
                .saturating_add(text_bytes(&edit.text))
                .saturating_add(128)
        })
    }
}

pub(super) fn text_bytes(text: &str) -> usize {
    if text.len() > DEFINITION_BYTES {
        return usize::MAX;
    }
    toml_edit::Value::from(text)
        .to_string()
        .len()
        .saturating_add(64)
}
