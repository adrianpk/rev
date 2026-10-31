use std::path::PathBuf;

use crate::buffer::{Buffer, Buffers, Stamp};
use crate::config::Grammar;
use crate::syntax::Syntax;

pub(crate) const MAX_BUFFERS: usize = 32;
pub(crate) const MAX_SNAPSHOTS: usize = 8 * 1024 * 1024;
pub(crate) const MAX_RETAINED: usize = 16384;
pub(crate) const MAX_PAYLOAD: usize = 8 * 1024 * 1024;

/// Pins both text and source eligibility. Rename can change a path or language
/// without changing canonical text; neither may validate an older observation.
#[derive(Clone, Debug)]
pub(crate) struct Observation {
    pub stamp: Stamp,
    generation: u64,
    org: bool,
    language: String,
    grammar: Grammar,
    path: Option<PathBuf>,
}

impl Observation {
    pub fn capture(buffer: &Buffer) -> Self {
        Self {
            stamp: buffer.stamp(),
            generation: buffer.syntax_generation(),
            org: buffer.is_org(),
            language: buffer.syntax_language().to_owned(),
            grammar: buffer.completion_grammar(),
            path: buffer.file().map(|file| file.path().to_owned()),
        }
    }

    pub fn current(&self, buffers: &Buffers) -> bool {
        buffers.get(self.stamp.buffer).is_ok_and(|buffer| {
            self.org == buffer.is_org()
                && self.stamp == buffer.stamp()
                && self.generation == buffer.syntax_generation()
                && self.language == buffer.syntax_language()
                && self.grammar == buffer.completion_grammar()
                && self.path.as_deref() == buffer.file().map(|file| file.path())
                && buffer.completion_available()
        })
    }
}

/// A shared immutable rope and the buffer's sole retained syntax owner.
pub(crate) struct Source {
    pub observation: Observation,
    pub text: ropey::Rope,
    pub syntax: Syntax,
    pub org: Option<crate::org::completion::Scan>,
}
