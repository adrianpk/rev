//! Invocation authority comes exclusively from a qualified native source span.
use std::path::PathBuf;

use crate::buffer::{Buffer, Span, edit::Point};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Explain,
    Prompt,
    Implement,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Explain => "ai-explain",
            Self::Prompt => "ai-prompt",
            Self::Implement => "ai-implement",
        }
    }
}

pub(crate) use crate::config::AiOutput as Output;

pub(crate) struct Target {
    pub source: Span,
    pub point: Point,
    pub path: Option<PathBuf>,
    pub output: Output,
    pub dirty: bool,
    pub bom: bool,
}

impl Target {
    pub fn capture(buffer: &Buffer, point: Point, kind: Kind) -> Result<Self, String> {
        buffer
            .slice(&Span {
                stamp: point.stamp,
                range: point.position.0..point.position.0,
            })
            .map_err(|e| e.to_string())?;
        let selection = if kind == Kind::Implement {
            None
        } else {
            selection(
                buffer,
                point,
                &format!("{} requires a linear selection or point", kind.name()),
            )?
        };
        let (source, output) = match (kind, selection) {
            (Kind::Explain, Some(source)) => (source, Output::Explanation),
            (Kind::Explain, None) => return Err("ai-explain requires an active selection".into()),
            (Kind::Prompt, Some(source)) => (source, Output::Replace),
            _ => (
                buffer
                    .span(point.position.0..point.position.0)
                    .map_err(|e| e.to_string())?,
                Output::Insert,
            ),
        };
        let target = Self {
            source,
            point,
            output,
            path: buffer.file().map(|f| f.path().to_owned()),
            dirty: buffer.is_modified(),
            bom: buffer.has_bom(),
        };
        target.current(buffer)?;
        Ok(target)
    }

    pub fn defined(
        buffer: &Buffer,
        point: Point,
        definition: &crate::config::AiCommand,
    ) -> Result<Self, String> {
        use crate::config::AiInput;
        buffer
            .slice(&Span {
                stamp: point.stamp,
                range: point.position.0..point.position.0,
            })
            .map_err(|e| e.to_string())?;
        let source = match definition.input {
            AiInput::Point => buffer
                .span(point.position.0..point.position.0)
                .map_err(|e| e.to_string())?,
            AiInput::Buffer => buffer
                .span(0..buffer.len_chars())
                .map_err(|e| e.to_string())?,
            AiInput::Selection => {
                selection(buffer, point, "AI selection requires a linear region")?
                    .ok_or("AI command requires an active selection")?
            }
        };
        let target = Self {
            source,
            point,
            path: buffer.file().map(|f| f.path().to_owned()),
            output: definition.output,
            dirty: buffer.is_modified(),
            bom: buffer.has_bom(),
        };
        target.current(buffer)?;
        Ok(target)
    }

    /// Revision and association are checked again before submission/application.
    /// Native preparation/publication independently enforce read-only admission.
    pub fn current(&self, buffer: &Buffer) -> Result<(), String> {
        buffer
            .slice(&self.source)
            .map_err(|e| format!("AI target is stale: {e}"))?;
        if buffer.file().map(|f| f.path()) != self.path.as_deref() {
            return Err("AI target file association changed; result retained".into());
        }
        if self.output.edits() && buffer.is_readonly() {
            return Err("AI target is read-only; source unchanged".into());
        }
        Ok(())
    }
}

fn selection(buffer: &Buffer, point: Point, error: &str) -> Result<Option<Span>, String> {
    if buffer.visual() == Some(crate::buffer::Visual::Block) || buffer.rectangle_active() {
        return Err(error.into());
    }
    Ok(buffer
        .region(point)
        .map_err(|e| e.to_string())?
        .filter(|s| !s.range.is_empty()))
}
