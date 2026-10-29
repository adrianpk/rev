//! Bounded source block observations. Names and header values remain data;
//! recognizing, inspecting or editing a block never grants process authority.
mod headers;
mod metadata;
mod text;
use super::{BlockKind, Cancellation, Coverage, Limits, Source, StructureKind, structure};
use crate::{
    buffer::{Buffer, Stamp, edit::Replacement},
    coordinates::Byte,
};
pub use headers::{Header, Headers};
use ropey::Rope;
use std::ops::Range;
#[derive(Clone, Debug)]
pub struct Definition {
    pub stamp: Stamp,
    pub language: String,
    pub name: Option<String>,
    pub switches: String,
    pub headers: Headers,
    pub body: Range<usize>,
    opening: Range<usize>,
    original: String,
    pub draft: String,
    indent: String,
    newline: String,
}
pub struct Scan {
    text: Rope,
    stamp: Stamp,
    at: usize,
}
impl Scan {
    pub fn new(buffer: &Buffer, at: Byte) -> Result<Self, String> {
        buffer.byte_span(at.0..at.0).map_err(|e| e.to_string())?;
        if buffer.len_bytes() > Limits::default().bytes {
            return Err("Source block analysis byte limit reached".into());
        }
        Ok(Self {
            text: Source::new(buffer).snapshot().map_err(|e| e.to_string())?,
            stamp: buffer.stamp(),
            at: at.0,
        })
    }
    pub fn run(self, cancel: &Cancellation) -> Result<Definition, String> {
        let document = structure::observe(self.text.clone(), Limits::default(), cancel);
        if document.coverage != Coverage::Complete {
            return Err("Source block structure is incomplete or cancelled".into());
        }
        let node = document
            .nodes
            .iter()
            .find(|n| selected(n, self.at, self.text.len_bytes()))
            .ok_or("No complete source block at point")?;
        let opening = node.opening.clone().ok_or("Source block has no opening")?;
        let metadata = metadata::observe(&self.text, &document, node, cancel)?;
        let original = self.text.byte_slice(node.content.clone()).to_string();
        let (draft, indent) = text::draft(&original, cancel)?;
        let newline = if self
            .text
            .byte_slice(opening.clone())
            .to_string()
            .ends_with("\r\n")
        {
            "\r\n"
        } else {
            "\n"
        }
        .into();
        Ok(Definition {
            stamp: self.stamp,
            language: metadata.language,
            name: metadata.name,
            switches: metadata.switches,
            headers: metadata.headers,
            body: node.content.clone(),
            opening,
            original,
            draft,
            indent,
            newline,
        })
    }
}
fn selected(node: &structure::Raw, at: usize, end: usize) -> bool {
    if node.kind != StructureKind::Block(BlockKind::Source) {
        return false;
    }
    let Some(closing) = &node.closing else {
        return false;
    };
    let start = node
        .affiliated
        .as_ref()
        .map_or(node.source.start, |r| r.start);
    at >= start && (at < closing.end || at == end && at == closing.end)
}
impl Definition {
    pub fn current(&self, buffer: &Buffer) -> Result<(), String> {
        super::source::current(buffer, self.stamp)
            .map_err(|e| format!("Source block changed; draft retained: {e}"))
    }
    pub fn replacement(&self, buffer: &Buffer, draft: &str) -> Result<Replacement, String> {
        self.current(buffer)?;
        let value = if draft == self.draft {
            self.original.clone()
        } else {
            text::body(draft, &self.indent, &self.newline)?
        };
        let source = buffer
            .byte_span(self.body.clone())
            .map_err(|e| e.to_string())?;
        Ok(Replacement {
            source,
            text: value,
        })
    }
    pub fn header_replacement(
        &self,
        buffer: &Buffer,
        argument: &str,
    ) -> Result<Replacement, String> {
        self.current(buffer)?;
        if self.language.is_empty() {
            return Err("Declare a source language before inserting header arguments".into());
        }
        let argument = headers::single(argument)?;
        let source = buffer
            .byte_span(self.opening.clone())
            .map_err(|e| e.to_string())?;
        let line = buffer
            .slice(&source)
            .map_err(|e| e.to_string())?
            .to_string();
        let body = line.trim_end_matches(['\r', '\n']);
        let text = format!("{body} {argument}{}", &line[body.len()..]);
        Ok(Replacement { source, text })
    }
    pub fn info(&self) -> String {
        format!(
            "Source language {}; name {}; headers {}",
            if self.language.is_empty() {
                "plain"
            } else {
                &self.language
            },
            self.name.as_deref().unwrap_or("unnamed"),
            self.headers.display()
        )
    }
}
