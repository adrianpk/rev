use super::regexp::Regexp;
use crate::org::Cancellation;
use ropey::RopeSlice;
#[derive(Debug, Clone)]
pub struct SearchQuery {
    terms: Vec<Term>,
}
#[derive(Debug, Clone)]
struct Term {
    pattern: Regexp,
    negative: bool,
}
impl SearchQuery {
    pub fn parse(source: &str, bytes: usize, depth: usize) -> Result<Self, String> {
        if source.trim().is_empty()
            || source.len() > bytes
            || bytes > 65536
            || source.chars().any(char::is_control)
        {
            return Err("Search requires nonempty bounded text without control characters".into());
        }
        let source = source.to_lowercase();
        let mut terms = Vec::new();
        if source.starts_with('+') {
            boolean(&source, depth, &mut terms)?;
        } else {
            terms.push(Term {
                pattern: Regexp::parse(&literal(&source), depth)?,
                negative: false,
            });
        }
        Ok(Self { terms })
    }
    pub fn matches(&self, text: RopeSlice<'_>, cancel: &Cancellation) -> Result<bool, String> {
        for term in &self.terms {
            if term.pattern.matches_folded(text, cancel)? == term.negative {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
fn boolean(mut source: &str, depth: usize, terms: &mut Vec<Term>) -> Result<(), String> {
    while !source.trim().is_empty() {
        if terms.len() == 64 {
            return Err("Search exceeds 64 Boolean terms".into());
        }
        source = source.trim_start();
        let negative = source.starts_with('-');
        if source.starts_with(['+', '-']) {
            source = &source[1..];
        }
        let (pattern, tail) = term(source)?;
        source = tail;
        terms.push(Term {
            pattern: Regexp::parse(&pattern, depth)?,
            negative,
        });
    }
    Ok(())
}
fn term(source: &str) -> Result<(String, &str), String> {
    if let Some(source) = source.strip_prefix('{') {
        return braced(source);
    }
    if let Some(source) = source.strip_prefix('"') {
        let (value, tail) = quoted(source)?;
        return Ok((literal(&value), tail));
    }
    let end = source.find(char::is_whitespace).unwrap_or(source.len());
    if end == 0 {
        return Err("Search has an empty Boolean term".into());
    }
    let value = &source[..end];
    Ok((format!("\\<{}\\>", literal(value)), &source[end..]))
}
fn braced(source: &str) -> Result<(String, &str), String> {
    let mut escaped = false;
    for (i, c) in source.char_indices() {
        if c == '}' && !escaped {
            return Ok((source[..i].into(), &source[i + 1..]));
        }
        escaped = c == '\\' && !escaped;
    }
    Err("Unclosed search regexp".into())
}
fn quoted(source: &str) -> Result<(String, &str), String> {
    let mut value = String::new();
    let mut escaped = false;
    for (i, c) in source.char_indices() {
        match (escaped, c) {
            (true, c) => {
                value.push(c);
                escaped = false;
            }
            (false, '\\') => escaped = true,
            (false, '"') => return Ok((value, &source[i + 1..])),
            (false, c) => value.push(c),
        }
    }
    Err("Unclosed search phrase".into())
}
fn literal(source: &str) -> String {
    source
        .split_whitespace()
        .map(|word| {
            let mut escaped = String::new();
            for c in word.chars() {
                if "()|".contains(c) {
                    escaped.extend(['[', c, ']']);
                } else {
                    if "\\.[]{}*+?^$".contains(c) {
                        escaped.push('\\');
                    }
                    escaped.push(c);
                }
            }
            escaped
        })
        .collect::<Vec<_>>()
        .join("\\s+")
}
