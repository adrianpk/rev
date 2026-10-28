use std::collections::HashMap;
use std::ops::Range;

use super::tokens::{self, Token};
use super::{Coverage, Document, Limit, Limits, Parsed, Stop, Work};

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
    result: &mut Document,
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
    let mut allowance = limits.nodes;
    for character in text.chars() {
        work.tick()?;
        if position + character.len_utf8() > limits.bytes {
            return Err(Stop(Coverage::Incomplete(Limit::AnalysisBytes)));
        }
        line.push(character);
        position += character.len_utf8();
        if character == '\n' {
            push(rows, &mut line, position, &mut allowance, work)?;
        }
    }
    if !line.is_empty() {
        push(rows, &mut line, position, &mut allowance, work)?;
    }
    Ok(())
}

fn push(
    rows: &mut Vec<Row>,
    line: &mut String,
    end: usize,
    allowance: &mut usize,
    work: &mut Work<'_>,
) -> Parsed<()> {
    if *allowance == 0 {
        return Err(Stop(Coverage::Incomplete(Limit::SourceNodes)));
    }
    let facts = tokens::classify(line, work)?;
    *allowance -= 1;
    rows.push(Row {
        source: end - line.len()..end,
        token: facts.token,
        indent: facts.indent,
        close: None,
        properties: facts.properties,
        valid_properties: false,
        formula: facts.formula,
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
        if !matches!(
            row.token,
            Token::Property { .. } | Token::Drawer(_) | Token::Blank
        ) {
            invalid += 1;
        }
    }
    Ok(())
}
