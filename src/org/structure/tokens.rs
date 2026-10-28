use std::ops::Range;

use super::{BlockKind, Coverage, Limit, Parsed, Stop, Work};

#[derive(Clone)]
pub(super) enum Token {
    Blank,
    Heading(usize, Range<usize>),
    Bullet {
        ordered: bool,
        content: usize,
    },
    Drawer(Range<usize>),
    EndDrawer,
    Block(BlockKind, String, Range<usize>),
    EndBlock(String),
    Keyword {
        key: Range<usize>,
        value: Range<usize>,
        affiliated: bool,
    },
    Planning,
    Property {
        key: Range<usize>,
        value: Range<usize>,
        additive: bool,
    },
    Table {
        separator: bool,
    },
    Comment,
    Fixed,
    Text,
}

pub(super) struct Facts {
    pub token: Token,
    pub indent: usize,
    pub properties: bool,
    pub formula: bool,
}

// Line scratch text is released after classification. Every potentially long
// character search observes cancellation; cells share the finite node budget.
pub(super) fn classify(line: &str, work: &mut Work<'_>) -> Parsed<Facts> {
    let mut lex = Lex {
        line,
        work,
        body: 0..line.len(),
        allowance: 0,
    };
    let (start, indent) = lex.indentation()?;
    let end = lex.trim_end(0..line.len(), b"\r\n")?;
    lex.body = start..end.max(start);
    let token = lex.classify()?;
    let trimmed = lex.trim(lex.body.clone())?;
    let body = &line[lex.body.clone()];
    Ok(Facts {
        token,
        indent,
        properties: line[trimmed].eq_ignore_ascii_case(":PROPERTIES:"),
        formula: body
            .get(..8)
            .is_some_and(|key| key.eq_ignore_ascii_case("#+TBLFM:")),
    })
}

pub(super) fn cells(
    line: &str,
    allowance: usize,
    work: &mut Work<'_>,
) -> Parsed<Vec<Range<usize>>> {
    let mut lex = Lex {
        line,
        work,
        body: 0..line.len(),
        allowance,
    };
    let (start, _) = lex.indentation()?;
    let end = lex.trim_end(start..line.len(), b"\r\n")?;
    lex.body = start..end;
    lex.cells()
}

struct Lex<'a, 'b, 'c> {
    line: &'a str,
    work: &'b mut Work<'c>,
    body: Range<usize>,
    allowance: usize,
}

impl Lex<'_, '_, '_> {
    fn classify(&mut self) -> Parsed<Token> {
        if self.trim(self.body.clone())?.is_empty() {
            return Ok(Token::Blank);
        }
        if let Some(heading) = self.heading()? {
            return Ok(heading);
        }
        Ok(self.candidate()?.unwrap_or(Token::Text))
    }

    fn candidate(&mut self) -> Parsed<Option<Token>> {
        match self.line.as_bytes().get(self.body.start) {
            Some(b'#') => self.hash(),
            Some(b':') => self.colon(),
            Some(b'|') => Ok(Some(Token::Table {
                separator: self.line[self.body.clone()].starts_with("|-"),
            })),
            _ => self.other(),
        }
    }

    fn indentation(&mut self) -> Parsed<(usize, usize)> {
        let mut start = 0;
        let mut column = 0;
        while let Some(byte) = self.line.as_bytes().get(start).copied() {
            self.work.tick()?;
            match byte {
                b' ' => column += 1,
                b'\t' => column += 8 - column % 8,
                _ => break,
            }
            start += 1;
        }
        Ok((start, column))
    }

    fn trim_end(&mut self, range: Range<usize>, pattern: &[u8]) -> Parsed<usize> {
        let mut end = range.end;
        while end > range.start && pattern.contains(&self.line.as_bytes()[end - 1]) {
            self.work.tick()?;
            end -= 1;
        }
        Ok(end)
    }

    fn trim(&mut self, range: Range<usize>) -> Parsed<Range<usize>> {
        let mut start = range.start;
        while start < range.end && b" \t".contains(&self.line.as_bytes()[start]) {
            self.work.tick()?;
            start += 1;
        }
        Ok(start..self.trim_end(start..range.end, b" \t")?)
    }

    fn heading(&mut self) -> Parsed<Option<Token>> {
        let mut level = 0;
        while self.line.as_bytes().get(level) == Some(&b'*') {
            self.work.tick()?;
            level += 1;
        }
        if level == 0 || self.line.as_bytes().get(level) != Some(&b' ') {
            return Ok(None);
        }
        let title = self.trim(level..self.body.end)?;
        Ok(Some(Token::Heading(level, title)))
    }

    fn hash(&mut self) -> Parsed<Option<Token>> {
        if let Some(block) = self.dynamic()? {
            return Ok(Some(block));
        }
        if let Some(block) = self.block()? {
            return Ok(Some(block));
        }
        if let Some(keyword) = self.keyword()? {
            return Ok(Some(keyword));
        }
        Ok(self.marker('#').then_some(Token::Comment))
    }

    fn word_end(&mut self, start: usize) -> Parsed<usize> {
        for (index, character) in self.line[start..self.body.end].char_indices() {
            self.work.tick()?;
            if character.is_whitespace() {
                return Ok(start + index);
            }
        }
        Ok(self.body.end)
    }

    fn lower(&mut self, range: Range<usize>) -> Parsed<String> {
        let mut name = String::new();
        for character in self.line[range].chars() {
            self.work.tick()?;
            name.push(character.to_ascii_lowercase());
        }
        Ok(name)
    }

    fn dynamic(&mut self) -> Parsed<Option<Token>> {
        let range = self.trim(self.body.clone())?;
        let body = &self.line[range];
        if body.eq_ignore_ascii_case("#+end:") {
            return Ok(Some(Token::EndBlock("dynamic".into())));
        }
        if !body
            .get(..8)
            .is_some_and(|v| v.eq_ignore_ascii_case("#+begin:"))
        {
            return Ok(None);
        }
        let value = self.trim(self.body.start + 8..self.body.end)?;
        let end = self.word_end(value.start)?;
        if value.is_empty() {
            return Ok(None);
        }
        Ok(Some(Token::Block(
            BlockKind::Dynamic,
            "dynamic".into(),
            value.start..end,
        )))
    }
    fn block(&mut self) -> Parsed<Option<Token>> {
        let body = &self.line[self.body.clone()];
        let closing = body
            .get(..6)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("#+end_"));
        let opening = body
            .get(..8)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("#+begin_"));
        if !closing && !opening {
            return Ok(None);
        }
        let end = self.word_end(self.body.start)?;
        if closing {
            return self.end_block(end);
        }
        let name = self.body.start + 8..end;
        if name.is_empty() {
            return Ok(None);
        }
        let lower = self.lower(name.clone())?;
        Ok(Some(Token::Block(block_kind(&lower), lower, name)))
    }

    fn end_block(&mut self, end: usize) -> Parsed<Option<Token>> {
        if !self.trim(end..self.body.end)?.is_empty() {
            return Ok(None);
        }
        Ok(Some(Token::EndBlock(self.lower(self.body.start + 6..end)?)))
    }

    fn colon(&mut self) -> Parsed<Option<Token>> {
        if let Some(drawer) = self.drawer()? {
            return Ok(Some(drawer));
        }
        if let Some(property) = self.property()? {
            return Ok(Some(property));
        }
        Ok(self.marker(':').then_some(Token::Fixed))
    }

    fn drawer(&mut self) -> Parsed<Option<Token>> {
        let end = self.trim_end(self.body.clone(), b" \t")?;
        if end <= self.body.start + 2 || self.line.as_bytes()[end - 1] != b':' {
            return Ok(None);
        }
        let name = self.body.start + 1..end - 1;
        for character in self.line[name.clone()].chars() {
            self.work.tick()?;
            if !word_character(character) {
                return Ok(None);
            }
        }
        if self.line[name.clone()].eq_ignore_ascii_case("END") {
            return Ok(Some(Token::EndDrawer));
        }
        Ok(Some(Token::Drawer(name)))
    }

    fn property(&mut self) -> Parsed<Option<Token>> {
        let mut end = self.body.start + 1;
        while end < self.body.end && self.line.as_bytes()[end] != b':' {
            self.work.tick()?;
            end += 1;
        }
        if end == self.body.end || end == self.body.start + 1 {
            return Ok(None);
        }
        let key = self.body.start + 1..end;
        if !self.property_key(key.clone())? {
            return Ok(None);
        }
        let Some(value) = self.property_value(end + 1)? else {
            return Ok(None);
        };
        Ok(Some(Token::Property {
            additive: self.line[key.clone()].ends_with('+'),
            key,
            value,
        }))
    }

    fn property_value(&mut self, start: usize) -> Parsed<Option<Range<usize>>> {
        if start < self.body.end && !b" \t".contains(&self.line.as_bytes()[start]) {
            return Ok(None);
        }
        Ok(Some(self.trim(start..self.body.end)?))
    }

    fn property_key(&mut self, key: Range<usize>) -> Parsed<bool> {
        for character in self.line[key].chars() {
            self.work.tick()?;
            if character.is_whitespace() {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn keyword(&mut self) -> Parsed<Option<Token>> {
        if !self.line[self.body.clone()].starts_with("#+") {
            return Ok(None);
        }
        let Some(end) = self.keyword_end()? else {
            return Ok(None);
        };
        let key = self.body.start + 2..end;
        let value = self.trim(end + 1..self.body.end)?;
        let affiliated = self.affiliated(key.clone())?;
        Ok(Some(Token::Keyword {
            key,
            value,
            affiliated,
        }))
    }

    fn keyword_end(&mut self) -> Parsed<Option<usize>> {
        let start = self.body.start + 2;
        let mut depth = 0usize;
        for (index, character) in self.line[start..self.body.end].char_indices() {
            self.work.tick()?;
            match character {
                '[' => depth += 1,
                ']' => depth = depth.saturating_sub(1),
                ':' if depth == 0 => return Ok((index > 0).then_some(start + index)),
                character if character.is_whitespace() && depth == 0 => return Ok(None),
                _ => (),
            }
        }
        Ok(None)
    }

    fn affiliated(&mut self, key: Range<usize>) -> Parsed<bool> {
        let mut end = key.end;
        for (index, character) in self.line[key.clone()].char_indices() {
            self.work.tick()?;
            if character == '[' {
                end = key.start + index;
                break;
            }
        }
        let key = &self.line[key.start..end];
        let known = [
            "CAPTION", "DATA", "HEADER", "HEADERS", "LABEL", "NAME", "PLOT", "RESNAME", "RESULT",
            "RESULTS", "SOURCE", "SRCNAME",
        ];
        Ok(known.iter().any(|name| key.eq_ignore_ascii_case(name))
            || key
                .get(..5)
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case("ATTR_"))
                && key.len() > 5)
    }

    fn marker(&self, marker: char) -> bool {
        self.line[self.body.clone()]
            .strip_prefix(marker)
            .is_some_and(|tail| tail.is_empty() || tail.starts_with([' ', '\t']))
    }

    fn other(&mut self) -> Parsed<Option<Token>> {
        let body = &self.line[self.body.clone()];
        if ["SCHEDULED:", "DEADLINE:", "CLOSED:"]
            .iter()
            .any(|key| body.starts_with(key))
        {
            return Ok(Some(Token::Planning));
        }
        let end = self.word_end(self.body.start)?;
        let word = &self.line[self.body.start..end];
        if word == "*" && self.body.start == 0 {
            return Ok(None);
        }
        let ordered = self.ordered(self.body.start..end)?;
        if !ordered && !["-", "+", "*"].contains(&word) {
            return Ok(None);
        }
        let content = self.trim(end..self.body.end)?.start;
        Ok(Some(Token::Bullet { ordered, content }))
    }

    fn ordered(&mut self, range: Range<usize>) -> Parsed<bool> {
        let Some(counter) = self.line[range].strip_suffix(['.', ')']) else {
            return Ok(false);
        };
        if counter.is_empty() {
            return Ok(false);
        }
        for byte in counter.bytes() {
            self.work.tick()?;
            if !byte.is_ascii_digit() {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn cells(&mut self) -> Parsed<Vec<Range<usize>>> {
        let mut cells = Vec::new();
        let scan_start = self.body.start + 1;
        let mut start = scan_start;
        for index in scan_start..self.body.end {
            self.work.tick()?;
            if self.line.as_bytes()[index] == b'|' {
                self.cell(&mut cells, start..index)?;
                start = index + 1;
            }
        }
        let tail = self.trim(start..self.body.end)?;
        if !tail.is_empty() || start == self.body.start + 1 {
            self.cell(&mut cells, tail)?;
        }
        Ok(cells)
    }

    fn cell(&mut self, cells: &mut Vec<Range<usize>>, range: Range<usize>) -> Parsed<()> {
        if cells.len() == self.allowance {
            return Err(Stop(Coverage::Incomplete(Limit::SourceNodes)));
        }
        cells.push(self.trim(range)?);
        Ok(())
    }
}

fn block_kind(name: &str) -> BlockKind {
    match name {
        "src" => BlockKind::Source,
        "example" => BlockKind::Example,
        "export" => BlockKind::Export,
        "comment" => BlockKind::Comment,
        "verse" => BlockKind::Verse,
        _ => BlockKind::Greater,
    }
}

fn word_character(character: char) -> bool {
    character.is_alphanumeric() || "_-".contains(character)
}
