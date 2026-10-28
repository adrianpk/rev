use super::elements::Frame;
use super::tokens::Token;
use super::{InlineContext, Limit, Parsed, Parser, Raw, Stop, StructureKind};

impl Parser<'_> {
    pub(super) fn outline(&mut self) -> Parsed<()> {
        let mut ancestors = Vec::new();
        let mut row = 0;
        while row < self.rows.len() {
            self.work.tick()?;
            if let Token::Heading(level, title) = self.rows[row].token.clone() {
                self.heading(row, level, title, &mut ancestors)?;
                row += 1;
            } else {
                row = self.section(row, ancestors.last().map(|(_, id)| *id))?;
            }
        }
        self.close_headings(&mut ancestors, 0, self.result.covered);
        Ok(())
    }

    fn heading(
        &mut self,
        row: usize,
        level: usize,
        title: std::ops::Range<usize>,
        ancestors: &mut Vec<(usize, usize)>,
    ) -> Parsed<()> {
        if level > self.limits.depth {
            return Err(Stop(super::Coverage::Incomplete(Limit::SourceDepth)));
        }
        let source = self.rows[row].source.clone();
        self.close_headings(ancestors, level, source.start);
        let mut node = Raw::new(
            StructureKind::Heading { level },
            source.start..self.result.covered,
            ancestors.last().map(|(_, id)| *id),
        );
        node.opening = Some(source.clone());
        node.content = source.start + title.start..source.start + title.end;
        node.context = Some(InlineContext::Standard);
        let id = self.push(node)?;
        ancestors.push((level, id));
        Ok(())
    }

    fn close_headings(&mut self, ancestors: &mut Vec<(usize, usize)>, level: usize, end: usize) {
        while ancestors.last().is_some_and(|(depth, _)| *depth >= level) {
            let (_, id) = ancestors.pop().expect("observed ancestor");
            self.result.nodes[id].source.end = end;
        }
    }

    fn section(&mut self, start: usize, heading: Option<usize>) -> Parsed<usize> {
        let mut end = start;
        while end < self.rows.len() && !matches!(self.rows[end].token, Token::Heading(..)) {
            self.work.tick()?;
            end += 1;
        }
        let first = self.postblank(start, end)?;
        if first < end {
            let section = self.push(Raw::new(
                StructureKind::Section,
                self.range(first..end),
                heading,
            ))?;
            let properties = self.property_position(start, heading, end)?;
            let mut frame = Frame::new(first..end, section, properties);
            frame.planning = heading
                .filter(|_| matches!(self.rows[start].token, Token::Planning))
                .map(|_| start);
            self.elements(frame)?;
        }
        Ok(end)
    }

    fn property_position(
        &mut self,
        start: usize,
        heading: Option<usize>,
        end: usize,
    ) -> Parsed<Option<usize>> {
        if heading.is_some() {
            let planning = matches!(self.rows[start].token, Token::Planning);
            return Ok(Some(start + usize::from(planning)));
        }
        let mut row = start;
        while row < end {
            self.work.tick()?;
            if !matches!(self.rows[row].token, Token::Blank | Token::Comment) {
                return Ok(Some(row));
            }
            row += 1;
        }
        Ok(None)
    }
}
