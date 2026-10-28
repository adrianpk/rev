use std::ops::Range;

use super::elements::Frame;
use super::tokens::Token;
use super::{Parsed, Parser, Raw, StructureKind};

impl Parser<'_> {
    pub(super) fn list(
        &mut self,
        frame: &mut Frame,
        start: usize,
        row: usize,
    ) -> Parsed<Vec<Frame>> {
        let items = self.item_ranges(row, frame.rows.end)?;
        let end = items.last().expect("starting item").end;
        let tail = self.postblank(end, frame.rows.end);
        let ordered = matches!(self.rows[row].token, Token::Bullet { ordered: true, .. });
        let mut node = Raw::new(
            StructureKind::List { ordered },
            self.range(row..tail),
            Some(frame.parent),
        );
        self.attach(&mut node, start, row);
        let id = self.push(node)?;
        self.attributes(id, start..row)?;
        let mut children = Vec::new();
        for item in items {
            let mut child = Frame::new(item, id, None);
            child.pending_item = true;
            children.push(child);
        }
        frame.position = tail;
        Ok(children)
    }

    pub(super) fn start_item(&mut self, frame: &mut Frame) -> Parsed<()> {
        let rows = frame.rows.clone();
        let row = rows.start;
        let indent = self.rows[row].indent;
        let mut node = Raw::new(
            StructureKind::Item { indent },
            self.range(rows.clone()),
            Some(frame.parent),
        );
        if let Token::Bullet { content, .. } = self.rows[row].token {
            node.content.start = self.rows[row].source.start + content;
        }
        node.opening = Some(node.source.start..node.content.start);
        let content = node.content.start;
        let id = self.push(node)?;
        frame.parent = id;
        frame.first_content = Some(content);
        frame.pending_item = false;
        Ok(())
    }

    fn item_ranges(&mut self, start: usize, end: usize) -> Parsed<Vec<Range<usize>>> {
        let indent = self.rows[start].indent;
        let mut items = Vec::new();
        let mut first = start;
        let mut row = start + 1;
        let mut blank = false;
        while row < end {
            self.work.tick()?;
            if self.list_boundary(row, indent, blank) {
                break;
            }
            if self.rows[row].indent == indent
                && matches!(self.rows[row].token, Token::Bullet { .. })
            {
                items.push(first..row);
                first = row;
            }
            blank = matches!(self.rows[row].token, Token::Blank);
            row = self.owned_end(row, end);
        }
        items.push(first..row);
        Ok(items)
    }

    fn list_boundary(&self, row: usize, indent: usize, blank: bool) -> bool {
        if matches!(self.rows[row].token, Token::Blank) {
            return blank;
        }
        if self.rows[row].indent > indent {
            return false;
        }
        !matches!(self.rows[row].token, Token::Bullet { .. }) || self.rows[row].indent < indent
    }

    // Delimited element bodies do not participate in list indentation tests.
    fn owned_end(&self, row: usize, end: usize) -> usize {
        self.rows[row]
            .close
            .filter(|close| *close < end)
            .map_or(row + 1, |close| close + 1)
    }
}
