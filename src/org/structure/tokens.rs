use std::ops::Range;

use super::BlockKind;

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
        cells: Vec<Range<usize>>,
    },
    Comment,
    Fixed,
    Text,
}

pub(super) fn classify(line: &str) -> Token {
    let trimmed = line.trim_matches([' ', '\t', '\r', '\n']);
    if trimmed.is_empty() {
        return Token::Blank;
    }
    let start = line.len() - line.trim_start_matches([' ', '\t']).len();
    let body = line[start..].trim_end_matches(['\r', '\n']);
    heading(line)
        .or_else(|| block(body, start))
        .or_else(|| drawer(body, start))
        .or_else(|| property(body, start))
        .or_else(|| keyword(body, start))
        .or_else(|| simple(body, start))
        .or_else(|| bullet(body, start))
        .unwrap_or(Token::Text)
}

fn heading(line: &str) -> Option<Token> {
    let level = line.bytes().take_while(|byte| *byte == b'*').count();
    if level == 0 || line.as_bytes().get(level) != Some(&b' ') {
        return None;
    }
    let title = line[level..].trim_matches([' ', '\t', '\r', '\n']);
    let start = line.len() - line[level..].trim_start_matches([' ', '\t']).len();
    Some(Token::Heading(level, start..start + title.len()))
}

fn block(body: &str, offset: usize) -> Option<Token> {
    let word = body.split_whitespace().next()?;
    if word
        .get(..6)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("#+end_"))
    {
        return body[word.len()..]
            .trim()
            .is_empty()
            .then(|| Token::EndBlock(word[6..].to_ascii_lowercase()));
    }
    let prefix = word.get(..8)?;
    let name = word.get(8..)?;
    if !prefix.eq_ignore_ascii_case("#+begin_") || name.is_empty() {
        return None;
    }
    Some(Token::Block(
        block_kind(name),
        name.to_ascii_lowercase(),
        offset + 8..offset + word.len(),
    ))
}

fn block_kind(name: &str) -> BlockKind {
    match name.to_ascii_lowercase().as_str() {
        "src" => BlockKind::Source,
        "example" => BlockKind::Example,
        "export" => BlockKind::Export,
        "comment" => BlockKind::Comment,
        "verse" => BlockKind::Verse,
        _ => BlockKind::Greater,
    }
}

fn drawer(body: &str, offset: usize) -> Option<Token> {
    let trimmed = body.trim_end_matches([' ', '\t']);
    let name = trimmed.strip_prefix(':')?.strip_suffix(':')?;
    if name.is_empty() || !name.chars().all(word_character) {
        return property(body, offset);
    }
    if name.eq_ignore_ascii_case("END") {
        return Some(Token::EndDrawer);
    }
    Some(Token::Drawer(offset + 1..offset + 1 + name.len()))
}

fn property(body: &str, offset: usize) -> Option<Token> {
    let tail = body.strip_prefix(':')?;
    let end = tail.find(':')? + 1;
    let key = &body[1..end];
    if key.is_empty() || key.chars().any(char::is_whitespace) {
        return None;
    }
    let rest = &body[end + 1..];
    if !rest.is_empty() && !rest.starts_with([' ', '\t']) {
        return None;
    }
    let value = rest.trim_matches([' ', '\t']);
    let start = body.len() - rest.trim_start_matches([' ', '\t']).len();
    Some(Token::Property {
        key: offset + 1..offset + end,
        value: offset + start..offset + start + value.len(),
        additive: key.ends_with('+'),
    })
}

fn word_character(character: char) -> bool {
    character.is_alphanumeric() || "_-".contains(character)
}

fn keyword(body: &str, offset: usize) -> Option<Token> {
    let tail = body.strip_prefix("#+")?;
    let end = keyword_end(tail)? + 2;
    let key = &body[2..end];
    let value = body[end + 1..].trim_matches([' ', '\t']);
    let start = body.len() - body[end + 1..].trim_start_matches([' ', '\t']).len();
    Some(Token::Keyword {
        key: offset + 2..offset + end,
        value: offset + start..offset + start + value.len(),
        affiliated: affiliated(key),
    })
}

fn keyword_end(tail: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (index, character) in tail.char_indices() {
        match character {
            '[' => depth += 1,
            ']' => depth = depth.saturating_sub(1),
            ':' if depth == 0 => return (index > 0).then_some(index),
            character if character.is_whitespace() && depth == 0 => return None,
            _ => (),
        }
    }
    None
}

fn affiliated(key: &str) -> bool {
    let key = key.split('[').next().unwrap_or(key).to_ascii_uppercase();
    [
        "CAPTION", "DATA", "HEADER", "HEADERS", "LABEL", "NAME", "PLOT", "RESNAME", "RESULT",
        "RESULTS", "SOURCE", "SRCNAME",
    ]
    .contains(&key.as_str())
        || key
            .strip_prefix("ATTR_")
            .is_some_and(|name| !name.is_empty())
}

fn simple(body: &str, offset: usize) -> Option<Token> {
    if body.starts_with('|') {
        return Some(table(body, offset));
    }
    if ["SCHEDULED:", "DEADLINE:", "CLOSED:"]
        .iter()
        .any(|key| body.starts_with(key))
    {
        return Some(Token::Planning);
    }
    if marker(body, '#') {
        return Some(Token::Comment);
    }
    marker(body, ':').then_some(Token::Fixed)
}

fn marker(body: &str, marker: char) -> bool {
    body.strip_prefix(marker)
        .is_some_and(|tail| tail.is_empty() || tail.starts_with([' ', '\t']))
}

fn bullet(body: &str, offset: usize) -> Option<Token> {
    let (word, tail) = body.split_once([' ', '\t']).unwrap_or((body, ""));
    let ordered = ordered(word);
    if !ordered && !["-", "+", "*"].contains(&word) {
        return None;
    }
    let content = body.len() - tail.trim_start_matches([' ', '\t']).len();
    Some(Token::Bullet {
        ordered,
        content: offset + content,
    })
}

fn ordered(word: &str) -> bool {
    let Some(counter) = word.strip_suffix(['.', ')']) else {
        return false;
    };
    !counter.is_empty() && counter.bytes().all(|byte| byte.is_ascii_digit())
}

fn table(body: &str, offset: usize) -> Token {
    let separator = body.starts_with("|-");
    let mut cells = Vec::new();
    let mut start = 1;
    for (index, _) in body.match_indices('|').skip(1) {
        cells.push(trim_cell(body, start..index, offset));
        start = index + 1;
    }
    if !body[start..].trim().is_empty() {
        cells.push(trim_cell(body, start..body.len(), offset));
    }
    Token::Table { separator, cells }
}

fn trim_cell(body: &str, range: Range<usize>, offset: usize) -> Range<usize> {
    let value = body[range.clone()].trim_matches([' ', '\t']);
    let part = &body[range.clone()];
    let start = offset + range.start + part.len() - part.trim_start_matches([' ', '\t']).len();
    start..start + value.len()
}
