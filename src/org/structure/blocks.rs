use super::elements::Frame;
use super::tokens::Token;
use super::{BlockKind, InlineContext, Parsed, Parser, Raw, StructureKind};

impl Parser<'_> {
    pub(super) fn block(
        &mut self,
        frame: &mut Frame,
        start: usize,
        row: usize,
    ) -> Parsed<Vec<Frame>> {
        let Some(close) = self.rows[row].close.filter(|end| *end < frame.rows.end) else {
            self.malformed(frame, start, row)?;
            return Ok(Vec::new());
        };
        let Token::Block(kind, _, name) = self.rows[row].token.clone() else {
            unreachable!("block candidate");
        };
        let end = self.postblank(close + 1, frame.rows.end)?;
        let mut node = Raw::new(
            StructureKind::Block(kind),
            self.range(row..end),
            Some(frame.parent),
        );
        node.opening = Some(self.rows[row].source.clone());
        node.closing = Some(self.rows[close].source.clone());
        node.name = Some(self.local(row, name));
        node.content = self.rows[row].source.end..self.rows[close].source.start;
        node.context = block_context(kind);
        self.attach(&mut node, start, row);
        let id = self.push(node)?;
        self.attributes(id, start..row)?;
        frame.position = end;
        if kind == BlockKind::Greater {
            let mut child = Frame::new(row + 1..close, id, None);
            child.drawers = frame.drawers;
            return Ok(vec![child]);
        }
        Ok(Vec::new())
    }
}

fn block_context(kind: BlockKind) -> Option<InlineContext> {
    match kind {
        BlockKind::Greater => None,
        BlockKind::Verse => Some(InlineContext::Standard),
        _ => Some(InlineContext::Literal),
    }
}
