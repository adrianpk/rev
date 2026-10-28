use std::ops::Range;

use super::tokens::Token;
use super::{InlineContext, Parsed, Parser, Raw, StructureKind};

pub(super) struct Frame {
    pub rows: Range<usize>,
    pub position: usize,
    pub parent: usize,
    pub properties: Option<usize>,
    pub planning: Option<usize>,
    pub drawers: bool,
    pub first_content: Option<usize>,
    pub pending_item: bool,
}

impl Frame {
    pub fn new(rows: Range<usize>, parent: usize, properties: Option<usize>) -> Self {
        Self {
            position: rows.start,
            rows,
            parent,
            properties,
            planning: None,
            drawers: true,
            first_content: None,
            pending_item: false,
        }
    }
}

impl Parser<'_> {
    pub(super) fn elements(&mut self, frame: Frame) -> Parsed<()> {
        let mut frames = vec![frame];
        while let Some(mut frame) = frames.pop() {
            self.work.tick()?;
            if frame.position >= frame.rows.end {
                continue;
            }
            if frame.pending_item {
                self.start_item(&mut frame)?;
            }
            let children = self.element(&mut frame)?;
            frames.push(frame);
            frames.extend(children.into_iter().rev());
        }
        Ok(())
    }

    fn element(&mut self, frame: &mut Frame) -> Parsed<Vec<Frame>> {
        let start = frame.position;
        if frame.first_content.is_some() && start == frame.rows.start {
            self.paragraph(frame, start, start)?;
            return Ok(Vec::new());
        }
        let row = self.affiliation(start, frame.rows.end)?;
        match self.rows[row].token {
            Token::Bullet { .. } => self.list(frame, start, row),
            Token::Block(..) => self.block(frame, start, row),
            Token::Drawer(_) if frame.drawers => self.drawer(frame, start, row),
            _ => {
                self.leaf(frame, start, row)?;
                Ok(Vec::new())
            }
        }
    }

    fn leaf(&mut self, frame: &mut Frame, start: usize, row: usize) -> Parsed<()> {
        match self.rows[row].token {
            Token::Blank => {
                frame.position += 1;
                Ok(())
            }
            Token::Table { .. } => self.table(frame, start, row),
            Token::Keyword { .. } => self.keyword(frame, start, row),
            Token::Planning if frame.planning == Some(row) => {
                self.single(frame, row, StructureKind::Planning)
            }
            _ => self.other(frame, start, row),
        }
    }

    fn other(&mut self, frame: &mut Frame, start: usize, row: usize) -> Parsed<()> {
        match self.rows[row].token {
            Token::Block(..) => self.malformed(frame, start, row),
            Token::Comment => self.single(frame, row, StructureKind::Comment),
            Token::Fixed => self.single(frame, row, StructureKind::FixedWidth),
            _ => self.paragraph(frame, start, row),
        }
    }

    pub(super) fn malformed(&mut self, frame: &mut Frame, start: usize, row: usize) -> Parsed<()> {
        let mut node = Raw::new(
            StructureKind::Malformed,
            self.rows[row].source.clone(),
            Some(frame.parent),
        );
        if let Token::Block(kind, _, name) = &self.rows[row].token {
            node.name = Some(self.local(row, name.clone()));
            node.context = Some(if *kind == super::BlockKind::Verse {
                InlineContext::Standard
            } else {
                InlineContext::Literal
            });
        }
        self.attach(&mut node, start, row);
        let id = self.push(node)?;
        self.attributes(id, start..row)?;
        frame.position = row + 1;
        Ok(())
    }

    fn affiliation(&mut self, start: usize, end: usize) -> Parsed<usize> {
        let mut row = start;
        while row < end
            && matches!(
                self.rows[row].token,
                Token::Keyword {
                    affiliated: true,
                    ..
                }
            )
        {
            self.work.tick()?;
            row += 1;
        }
        if row == end {
            return Ok(start);
        }
        Ok(match self.rows[row].token {
            Token::Blank | Token::Planning | Token::Comment => start,
            _ => row,
        })
    }

    pub(super) fn attach(&mut self, node: &mut Raw, start: usize, row: usize) {
        if start < row {
            node.affiliated = Some(self.range(start..row));
            node.source.start = self.rows[start].source.start;
        }
    }

    pub(super) fn attributes(&mut self, parent: usize, rows: Range<usize>) -> Parsed<()> {
        for row in rows {
            let node = self.keyword_node(row, parent, StructureKind::AffiliatedKeyword);
            self.push(node)?;
        }
        Ok(())
    }

    fn keyword_node(&self, row: usize, parent: usize, kind: StructureKind) -> Raw {
        let mut node = Raw::new(kind, self.rows[row].source.clone(), Some(parent));
        if let Token::Keyword { key, value, .. } = &self.rows[row].token {
            node.name = Some(self.local(row, key.clone()));
            node.content = self.local(row, value.clone());
        }
        node.context = Some(InlineContext::Literal);
        node
    }

    fn keyword(&mut self, frame: &mut Frame, start: usize, row: usize) -> Parsed<()> {
        let mut node = self.keyword_node(row, frame.parent, StructureKind::Keyword);
        self.attach(&mut node, start, row);
        let id = self.push(node)?;
        self.attributes(id, start..row)?;
        frame.position = row + 1;
        Ok(())
    }

    fn single(&mut self, frame: &mut Frame, row: usize, kind: StructureKind) -> Parsed<()> {
        let mut node = Raw::new(kind, self.rows[row].source.clone(), Some(frame.parent));
        node.context = Some(InlineContext::Literal);
        self.push(node)?;
        frame.position = row + 1;
        Ok(())
    }

    fn paragraph(&mut self, frame: &mut Frame, start: usize, row: usize) -> Parsed<()> {
        let mut end = row + 1;
        while end < frame.rows.end && paragraph_line(&self.rows[end].token, frame.drawers) {
            self.work.tick()?;
            end += 1;
        }
        let content_end = self.rows[end - 1].source.end;
        let tail = self.postblank(end, frame.rows.end)?;
        let mut node = Raw::new(
            StructureKind::Paragraph,
            self.range(row..tail),
            Some(frame.parent),
        );
        if row == frame.rows.start {
            node.source.start = frame.first_content.unwrap_or(node.source.start);
        }
        node.content = node.source.start..content_end;
        node.context = Some(InlineContext::Standard);
        self.attach(&mut node, start, row);
        let id = self.push(node)?;
        self.attributes(id, start..row)?;
        frame.position = tail;
        Ok(())
    }

    pub(super) fn local(&self, row: usize, range: Range<usize>) -> Range<usize> {
        let start = self.rows[row].source.start;
        start + range.start..start + range.end
    }

    pub(super) fn postblank(&mut self, mut end: usize, limit: usize) -> Parsed<usize> {
        while end < limit && matches!(self.rows[end].token, Token::Blank) {
            self.work.tick()?;
            end += 1;
        }
        Ok(end)
    }

    fn drawer(&mut self, frame: &mut Frame, start: usize, row: usize) -> Parsed<Vec<Frame>> {
        let Some(close) = self.rows[row].close.filter(|end| *end < frame.rows.end) else {
            self.malformed(frame, start, row)?;
            return Ok(Vec::new());
        };
        let properties = self.property_drawer(row, frame);
        let kind = if properties {
            StructureKind::PropertyDrawer
        } else {
            StructureKind::Drawer
        };
        let end = self.postblank(close + 1, frame.rows.end)?;
        let mut node = Raw::new(kind, self.range(row..end), Some(frame.parent));
        node.opening = Some(self.rows[row].source.clone());
        node.closing = Some(self.rows[close].source.clone());
        node.content = self.rows[row].source.end..self.rows[close].source.start;
        if let Token::Drawer(name) = &self.rows[row].token {
            node.name = Some(self.local(row, name.clone()));
        }
        self.attach(&mut node, start, row);
        let id = self.push(node)?;
        self.attributes(id, start..row)?;
        frame.position = end;
        if properties {
            self.properties(id, row + 1..close)?;
            return Ok(Vec::new());
        }
        let mut child = Frame::new(row + 1..close, id, None);
        child.drawers = false;
        Ok(vec![child])
    }

    fn property_drawer(&self, row: usize, frame: &Frame) -> bool {
        self.rows[row].properties
            && self.rows[row].valid_properties
            && frame.properties == Some(row)
    }

    fn properties(&mut self, parent: usize, rows: Range<usize>) -> Parsed<()> {
        for row in rows {
            self.work.tick()?;
            let (key, value, additive) = match self.rows[row].token.clone() {
                Token::Property {
                    key,
                    value,
                    additive,
                } => (key, value, additive),
                Token::Drawer(key) => {
                    let end = key.end + 1;
                    (key, end..end, false)
                }
                _ => continue,
            };
            let mut node = Raw::new(
                StructureKind::Property { additive },
                self.rows[row].source.clone(),
                Some(parent),
            );
            node.name = Some(self.local(row, key));
            node.content = self.local(row, value);
            node.context = Some(InlineContext::Literal);
            self.push(node)?;
        }
        Ok(())
    }

    fn table(&mut self, frame: &mut Frame, start: usize, row: usize) -> Parsed<()> {
        let mut end = self.table_end(row, frame.rows.end)?;
        let formulas = end;
        end = self.formula_end(end, frame.rows.end)?;
        let tail = self.postblank(end, frame.rows.end)?;
        let mut node = Raw::new(
            StructureKind::Table,
            self.range(row..tail),
            Some(frame.parent),
        );
        node.content = self.range(row..formulas);
        self.attach(&mut node, start, row);
        let id = self.push(node)?;
        self.attributes(id, start..row)?;
        self.table_rows(id, row..formulas)?;
        for formula in formulas..end {
            self.push(self.keyword_node(formula, id, StructureKind::TableFormula))?;
        }
        frame.position = tail;
        Ok(())
    }

    fn table_end(&mut self, mut row: usize, end: usize) -> Parsed<usize> {
        while row < end && matches!(self.rows[row].token, Token::Table { .. }) {
            self.work.tick()?;
            row += 1;
        }
        Ok(row)
    }

    fn formula_end(&mut self, mut row: usize, end: usize) -> Parsed<usize> {
        while row < end && self.rows[row].formula {
            self.work.tick()?;
            row += 1;
        }
        Ok(row)
    }

    fn table_rows(&mut self, parent: usize, rows: Range<usize>) -> Parsed<()> {
        for row in rows {
            if let Token::Table { separator } = self.rows[row].token {
                let node = Raw::new(
                    StructureKind::TableRow { separator },
                    self.rows[row].source.clone(),
                    Some(parent),
                );
                let id = self.push(node)?;
                if !separator {
                    let cells = self.cell_ranges(row)?;
                    self.cells(id, row, cells)?;
                }
            }
        }
        Ok(())
    }

    fn cell_ranges(&mut self, row: usize) -> Parsed<Vec<Range<usize>>> {
        let mut line = String::new();
        for character in self.text.byte_slice(self.rows[row].source.clone()).chars() {
            self.work.tick()?;
            line.push(character);
        }
        let allowance = self.limits.nodes - self.result.nodes.len();
        super::tokens::cells(&line, allowance, &mut self.work)
    }

    fn cells(&mut self, parent: usize, row: usize, cells: Vec<Range<usize>>) -> Parsed<()> {
        for cell in cells {
            let mut node = Raw::new(
                StructureKind::TableCell,
                self.local(row, cell),
                Some(parent),
            );
            node.context = Some(InlineContext::Standard);
            self.push(node)?;
        }
        Ok(())
    }
}

fn paragraph_line(token: &Token, drawers: bool) -> bool {
    if matches!(token, Token::Drawer(_)) {
        return !drawers;
    }
    matches!(
        token,
        Token::Text
            | Token::Property { .. }
            | Token::Planning
            | Token::EndDrawer
            | Token::EndBlock(_)
    )
}
