mod builder;
mod continuation;
mod shift;
mod source;

use crate::buffer::{
    Buffer,
    edit::{Replacement, Retained},
};
use crate::command::MarkdownCommand;
use crate::coordinates::Scalar;

use super::{Kind, Observation};
use source::Source;

/// A plan is source-qualified and owned. Only the canonical buffer publication
/// boundary may admit its replacements, retained spans and initiating cursor.
pub(crate) struct Edit {
    pub(crate) edits: Vec<Replacement>,
    pub(crate) retained: Vec<Retained>,
    pub(crate) cursor: Option<Scalar>,
}

impl Observation {
    pub(crate) fn list_edit(
        &self,
        buffer: &Buffer,
        command: MarkdownCommand,
        point: Scalar,
    ) -> Result<Option<Edit>, String> {
        if self.stamp != buffer.stamp() {
            return Err("Markdown structure is not current".into());
        }
        let source = Source::new(buffer)?;
        let byte = source
            .coordinates
            .to_byte(point)
            .map_err(|e| e.to_string())?
            .0;
        let Some(index) = self.item_at(byte, buffer) else {
            return Ok(None);
        };
        let edit = match command {
            MarkdownCommand::ContinueItem
            | MarkdownCommand::Return
            | MarkdownCommand::MetaReturn => continuation::continue_item(self, &source, index),
            MarkdownCommand::IndentItem => shift::indent(self, &source, index),
            MarkdownCommand::OutdentItem => shift::outdent(self, &source, index),
            _ => unreachable!("presentation controls never prepare source edits"),
        }?;
        Ok(Some(edit))
    }

    fn item_at(&self, byte: usize, buffer: &Buffer) -> Option<usize> {
        if self
            .nodes
            .iter()
            .any(|node| matches!(node.kind, Kind::LiteralBlock) && node.bytes.contains(&byte))
        {
            return None;
        }
        // A final empty physical line is outside the preceding list. Only an
        // unterminated source line can own its otherwise exclusive EOF boundary.
        let eof = buffer
            .line(buffer.len_lines() - 1, 0..1)
            .is_ok()
            .then_some(buffer.len_bytes());
        self.items.iter().rposition(|item| {
            item.bytes.contains(&byte)
                || eof == Some(byte) && item.bytes.end == byte
                || self.indentation_owns(item, byte)
        })
    }

    fn indentation_owns(&self, item: &super::Item, byte: usize) -> bool {
        byte >= item.line_start
            && byte < item.marker.start
            && item
                .parent
                .is_none_or(|parent| self.items[parent].line_start != item.line_start)
    }
}
