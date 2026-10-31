//! Native document facts feed disposable proposals, never edits or IO.
mod context;
mod names;
mod vocabulary;
use super::{Cancellation, Coverage, Limits, Structure, StructureScan};
use crate::buffer::Buffer;
use crate::completion::{Candidate, MAX_BYTES, MAX_LABEL, MAX_SYMBOLS, Symbol};
use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Todo,
    Tag,
    Property,
    Directive,
    Link,
    Heading,
}
impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Todo => "todo",
            Self::Tag => "org-tag",
            Self::Property => "property-key",
            Self::Directive => "directive",
            Self::Link => "link-type",
            Self::Heading => "heading",
        }
    }
    fn colon(self, name: &str) -> bool {
        match self {
            Self::Tag | Self::Property | Self::Link => true,
            Self::Directive => !name.starts_with("BEGIN_") && !name.starts_with("END_"),
            _ => false,
        }
    }
}
pub(crate) struct Target {
    pub range: Range<usize>,
    pub prefix: String,
    pub original: String,
    pub kind: Kind,
    pub closing: bool,
    pub symbols: Vec<Symbol>,
}
pub(crate) struct Scan {
    structure: StructureScan,
    todo: crate::config::Todo,
}
impl Scan {
    pub fn new(buffer: &Buffer, todo: &crate::config::Todo) -> Result<Self, String> {
        let limits = Limits::new(MAX_BYTES, 200_000, 256).map_err(|e| e.to_string())?;
        Ok(Self {
            structure: StructureScan::new(buffer, limits).map_err(|e| e.to_string())?,
            todo: todo.clone(),
        })
    }
    pub fn run(
        self,
        text: &ropey::Rope,
        byte: usize,
        cancel: &Cancellation,
    ) -> Result<Option<Target>, String> {
        let structure = self.structure.run(cancel);
        check(cancel)?;
        if structure.coverage() != Coverage::Complete {
            return Err("Native Org completion structural limit reached".into());
        }
        let Some(mut target) = context::at(&structure, text, byte, cancel)? else {
            return Ok(None);
        };
        target.symbols = names::collect(&structure, text, &self.todo, target.kind, cancel)?;
        Ok(Some(target))
    }
}
fn check(cancel: &Cancellation) -> Result<(), String> {
    if cancel.is_cancelled() {
        Err("Native Org completion cancelled or work limit reached".into())
    } else {
        Ok(())
    }
}
fn symbol(name: String, kind: Kind, byte: usize) -> Result<Symbol, String> {
    if name.len() + 2 + kind.label().len() > MAX_LABEL {
        return Err("Native completion label limit reached".into());
    }
    Ok(Symbol {
        name,
        kind: kind.label(),
        byte,
        origin: 0,
        scope: None,
    })
}
pub(crate) fn literals(candidates: &mut [Candidate], kind: Kind, closing: bool) {
    for candidate in candidates {
        if kind == Kind::Heading {
            candidate.text = super::links::encode(&candidate.text);
        } else if !closing && kind.colon(&candidate.text) {
            candidate.text.push(':');
        }
    }
}

pub(crate) fn trigger(text: &str) -> bool {
    text.ends_with([' ', ':'])
        || ["#+", "[[", "[*"]
            .iter()
            .any(|prefix| text.ends_with(prefix))
}
