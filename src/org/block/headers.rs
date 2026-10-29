use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub key: String,
    pub value: String,
}
#[derive(Clone, Debug, Default)]
pub struct Headers {
    values: Vec<Header>,
    bytes: usize,
}
impl Headers {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values
            .iter()
            .rev()
            .find(|h| h.key == key)
            .map(|h| h.value.as_str())
    }
    /// Repeated variable declarations retain ordered data for later consumers.
    pub fn iter(&self) -> impl Iterator<Item = &Header> {
        self.values.iter()
    }
    pub(super) fn defaults() -> Self {
        let mut headers = Self::default();
        for (key, value) in [
            ("session", "none"),
            ("results", "replace"),
            ("exports", "code"),
            ("cache", "no"),
            ("noweb", "no"),
            ("hlines", "no"),
            ("tangle", "no"),
        ] {
            headers.values.push(Header {
                key: key.into(),
                value: value.into(),
            });
        }
        headers
    }
    pub(super) fn apply(&mut self, value: &str, cancel: &Cancellation) -> Result<(), String> {
        self.bytes = self
            .bytes
            .checked_add(value.len())
            .filter(|n| *n <= 65536)
            .ok_or("Source headers exceed 64 KiB")?;
        for header in parse(value, cancel)? {
            if header.key != "var" {
                self.values.retain(|h| h.key != header.key);
            }
            if self.values.len() >= 256 {
                return Err("Source headers exceed 256 arguments".into());
            }
            self.values.push(header);
        }
        Ok(())
    }
    pub(super) fn display(&self) -> String {
        self.values
            .iter()
            .map(|h| format!(":{} {}", h.key, h.value))
            .collect::<Vec<_>>()
            .join(" ")
    }
}
pub(super) fn opening(line: &str) -> Result<(String, String, String), String> {
    let line = line.trim();
    let after = line.get(11..).ok_or("Malformed BEGIN_SRC opening")?.trim();
    let end = after.find(char::is_whitespace).unwrap_or(after.len());
    let language = &after[..end];
    if language.len() > 128
        || !language
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_+.-".contains(&b))
    {
        return Err("Source language requires at most 128 ASCII identifier bytes".into());
    }
    let tail = after[end..].trim();
    let start = header_start(tail).unwrap_or(tail.len());
    Ok((
        language.into(),
        tail[..start].trim().into(),
        tail[start..].into(),
    ))
}
fn header_start(value: &str) -> Option<usize> {
    value
        .char_indices()
        .find(|(i, c)| *c == ':' && (*i == 0 || value.as_bytes()[i - 1].is_ascii_whitespace()))
        .map(|(i, _)| i)
}
pub(super) fn single(value: &str) -> Result<&str, String> {
    if value.contains(['\r', '\n']) {
        return Err("Header argument must stay on one line".into());
    }
    let headers = parse(value, &Cancellation::default())?;
    if headers.len() != 1 {
        return Err("Enter one :key value header argument".into());
    }
    Ok(value.trim())
}
pub(super) fn affiliated(
    text: &Rope,
    node: &structure::Raw,
    headers: &mut Headers,
    cancel: &Cancellation,
) -> Result<Option<String>, String> {
    let Some(range) = &node.affiliated else {
        return Ok(None);
    };
    if range.len() > 65536 {
        return Err("Source affiliations exceed 64 KiB".into());
    }
    let mut name = None;
    for line in text.byte_slice(range.clone()).lines() {
        check(cancel)?;
        let line = line.to_string();
        let Some((key, value)) = line
            .trim()
            .strip_prefix("#+")
            .and_then(|l| l.split_once(':'))
        else {
            continue;
        };
        match key.to_ascii_uppercase().as_str() {
            "NAME" => {
                if name.is_some() {
                    return Err("Source block has duplicate names".into());
                }
                name = Some(block_name(value)?);
            }
            "HEADER" | "HEADERS" => headers.apply(value.trim(), cancel)?,
            _ => {}
        }
    }
    Ok(name)
}
fn block_name(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > 128
        || value.chars().any(|c| c.is_whitespace() || c.is_control())
    {
        return Err("Block names require 1..128 bytes without whitespace or controls".into());
    }
    Ok(value.into())
}
pub(super) fn check(cancel: &Cancellation) -> Result<(), String> {
    if cancel.is_cancelled() {
        Err("Source block observation cancelled or expired".into())
    } else {
        Ok(())
    }
}
#[derive(Default)]
struct Lex {
    quote: bool,
    escape: bool,
    delimiters: Vec<char>,
}
impl Lex {
    fn boundary(&self) -> bool {
        !self.quote && !self.escape && self.delimiters.is_empty()
    }
    fn step(&mut self, c: char) -> Result<(), String> {
        if self.escape {
            self.escape = false;
            return Ok(());
        }
        if c == '\\' {
            self.escape = true;
            return Ok(());
        }
        if c == '"' {
            self.quote = !self.quote;
            return Ok(());
        }
        if !self.quote {
            self.delimiter(c)?;
        }
        Ok(())
    }
    fn delimiter(&mut self, c: char) -> Result<(), String> {
        match c {
            '(' | '[' | '{' => {
                if self.delimiters.len() >= 64 {
                    return Err("Source header nesting exceeds 64".into());
                }
                self.delimiters.push(c);
            }
            ')' | ']' | '}' => {
                let expected = "([{"
                    .chars()
                    .zip(")]}".chars())
                    .find_map(|(open, close)| (c == close).then_some(open));
                if self.delimiters.pop() != expected {
                    return Err("Unbalanced source header value".into());
                }
            }
            _ => {}
        }
        Ok(())
    }
}
fn parse(value: &str, cancel: &Cancellation) -> Result<Vec<Header>, String> {
    let value = header_line(value)?;
    if value.is_empty() {
        return Ok(Vec::new());
    }
    let starts = argument_starts(value, cancel)?;
    starts
        .windows(2)
        .map(|r| argument(&value[r[0]..r[1]]))
        .collect()
}
fn header_line(value: &str) -> Result<&str, String> {
    if value.len() > 4096 || value.chars().any(|c| c.is_control() && !c.is_whitespace()) {
        return Err("Source header line exceeds 4096 bytes or contains controls".into());
    }
    let value = value.trim();
    if !value.is_empty() && !value.starts_with(':') {
        return Err("Source header arguments require :key value".into());
    }
    Ok(value)
}
fn argument_starts(value: &str, cancel: &Cancellation) -> Result<Vec<usize>, String> {
    let mut lex = Lex::default();
    let mut starts = Vec::new();
    for (i, c) in value.char_indices() {
        check(cancel)?;
        if c == ':' && lex.boundary() && (i == 0 || value.as_bytes()[i - 1].is_ascii_whitespace()) {
            starts.push(i);
        }
        lex.step(c)?;
    }
    if !lex.boundary() {
        return Err("Unbalanced source header literal".into());
    }
    starts.push(value.len());
    Ok(starts)
}
fn argument(value: &str) -> Result<Header, String> {
    let value = &value[1..];
    let end = value.find(char::is_whitespace).unwrap_or(value.len());
    let key = &value[..end];
    if key.is_empty()
        || !key
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b == b'-' || b == b'_')
    {
        return Err("Source header names require lowercase ASCII letters, - or _".into());
    }
    Ok(Header {
        key: key.into(),
        value: value[end..].trim().into(),
    })
}
