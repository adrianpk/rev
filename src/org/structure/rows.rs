use std::collections::HashMap;
use std::ops::Range;

use super::tokens::{self, Token};
use super::{Coverage, Limit, Limits, Parsed, Stop, Structure, Work};

pub(super) struct Row {
    pub source: Range<usize>,
    pub token: Token,
    pub indent: usize,
    pub close: Option<usize>,
    pub properties: bool,
    pub valid_properties: bool,
    pub formula: bool,
}

pub(super) fn collect(
    text: &ropey::Rope,
    limits: Limits,
    work: &mut Work<'_>,
    result: &mut Structure,
) -> Vec<Row> {
    let mut rows = Vec::new();
    if let Err(Stop(coverage)) = read_rows(text, limits, work, &mut rows) {
        result.coverage = coverage;
    }
    result.covered = rows.last().map_or(0, |row| row.source.end);
    if let Err(Stop(coverage)) = pair(&mut rows, work) {
        result.coverage = coverage;
    }
    rows
}

fn read_rows(
    text: &ropey::Rope,
    limits: Limits,
    work: &mut Work<'_>,
    rows: &mut Vec<Row>,
) -> Parsed<()> {
    let mut line = String::new();
    let mut position = 0;
    for character in text.chars() {
        work.tick()?;
        if position + character.len_utf8() > limits.bytes {
            return Err(Stop(Coverage::Incomplete(Limit::AnalysisBytes)));
        }
        line.push(character);
        position += character.len_utf8();
        if character == '\n' {
            push(rows, &mut line, position, limits.nodes)?;
        }
    }
    if !line.is_empty() {
        push(rows, &mut line, position, limits.nodes)?;
    }
    Ok(())
}

fn push(rows: &mut Vec<Row>, line: &mut String, end: usize, limit: usize) -> Parsed<()> {
    if rows.len() == limit {
        return Err(Stop(Coverage::Incomplete(Limit::SourceNodes)));
    }
    let token = tokens::classify(line);
    let trimmed = line.trim_start_matches([' ', '\t']);
    let indent = line[..line.len() - trimmed.len()]
        .chars()
        .fold(0, |column, character| {
            if character == '\t' {
                column + 8 - column % 8
            } else {
                column + 1
            }
        });
    rows.push(Row {
        source: end - line.len()..end,
        token,
        indent,
        close: None,
        properties: trimmed.trim_end().eq_ignore_ascii_case(":PROPERTIES:"),
        valid_properties: false,
        formula: trimmed
            .get(..8)
            .is_some_and(|key| key.eq_ignore_ascii_case("#+TBLFM:")),
    });
    line.clear();
    Ok(())
}

// A single reverse pass pairs candidates within context-free heading boundaries.
// Repeated malformed openers never rescan the remaining document suffix.
fn pair(rows: &mut [Row], work: &mut Work<'_>) -> Parsed<()> {
    let mut blocks = HashMap::new();
    let mut drawer = None;
    let mut invalid = 0usize;
    for index in (0..rows.len()).rev() {
        work.tick()?;
        let row = &mut rows[index];
        match &row.token {
            Token::Heading(..) => {
                blocks.clear();
                drawer = None;
            }
            Token::EndBlock(name) => {
                blocks.insert(name.clone(), index);
            }
            Token::Block(_, name, _) => row.close = blocks.get(name).copied(),
            Token::EndDrawer => drawer = Some((index, invalid + 1)),
            Token::Drawer(_) => {
                row.close = drawer.map(|(end, _)| end);
                row.valid_properties = drawer.is_some_and(|(_, count)| count == invalid);
            }
            _ => (),
        }
        if !matches!(row.token, Token::Property { .. } | Token::Blank) {
            invalid += 1;
        }
    }
    Ok(())
}
