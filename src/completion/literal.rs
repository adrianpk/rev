//! Bounded key spelling. Display identities decode escapes; edits encode literals.
use crate::config::Grammar;

pub(crate) fn decode(raw: &str, grammar: Grammar) -> Option<String> {
    let quote = raw.chars().next()?;
    if quote == '\'' && grammar == Grammar::Toml {
        return Some(raw.strip_prefix('\'')?.strip_suffix('\'')?.to_owned());
    }
    if quote != '"' {
        return (grammar == Grammar::Toml && bare(raw)).then(|| raw.to_owned());
    }
    content(raw.strip_prefix('"')?.strip_suffix('"')?, grammar)
}

pub(crate) fn content(raw: &str, grammar: Grammar) -> Option<String> {
    let mut result = String::new();
    let mut chars = raw.chars();
    while let Some(ch) = chars.next() {
        let ch = if ch == '\\' {
            escape(&mut chars, grammar)?
        } else {
            if !plain(ch, grammar) {
                return None;
            }
            ch
        };
        result.push(ch);
    }
    Some(result)
}

fn plain(ch: char, grammar: Grammar) -> bool {
    if ch == '"' || (grammar == Grammar::Toml && ch == '\u{7f}') {
        return false;
    }
    ch >= '\u{20}' || (grammar == Grammar::Toml && ch == '\t')
}

fn escape(chars: &mut std::str::Chars<'_>, grammar: Grammar) -> Option<char> {
    let ch = chars.next()?;
    if ch == 'u' {
        return unicode(chars, grammar);
    }
    if ch == 'U' && grammar == Grammar::Toml {
        return char::from_u32(hex(chars, 8)?);
    }
    let index = ['"', '\\', '/', 'b', 'f', 'n', 'r', 't']
        .iter()
        .position(|c| *c == ch)?;
    if ch == '/' && grammar != Grammar::Json {
        return None;
    }
    Some(['"', '\\', '/', '\u{8}', '\u{c}', '\n', '\r', '\t'][index])
}

fn unicode(chars: &mut std::str::Chars<'_>, grammar: Grammar) -> Option<char> {
    let first = hex(chars, 4)?;
    if grammar != Grammar::Json || !(0xd800..=0xdbff).contains(&first) {
        return char::from_u32(first);
    }
    if chars.next()? != '\\' || chars.next()? != 'u' {
        return None;
    }
    let second = hex(chars, 4)?;
    if !(0xdc00..=0xdfff).contains(&second) {
        return None;
    }
    char::from_u32(0x10000 + ((first - 0xd800) << 10) + second - 0xdc00)
}

fn hex(chars: &mut std::str::Chars<'_>, count: usize) -> Option<u32> {
    (0..count).try_fold(0, |value, _| Some(value * 16 + chars.next()?.to_digit(16)?))
}

pub(crate) fn encode(name: &str, grammar: Grammar) -> String {
    if grammar == Grammar::Toml && bare(name) {
        return name.to_owned();
    }
    let mut result = String::from("\"");
    for ch in name.chars() {
        quoted(ch, &mut result);
    }
    result.push('"');
    result
}

fn quoted(ch: char, result: &mut String) {
    use std::fmt::Write;
    match ch {
        '"' => result.push_str("\\\""),
        '\\' => result.push_str("\\\\"),
        ch if ch.is_control() => {
            let _ = write!(result, "\\u{:04x}", ch as u32);
        }
        _ => result.push(ch),
    }
}

fn bare(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ['_', '-'].contains(&ch))
}

pub(crate) fn path(raw: &str, partial: bool) -> Option<String> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut quoted = Quoted::default();
    for (byte, ch) in raw.char_indices() {
        if quoted.outside(ch) && ch == '.' {
            parts.push(path_part(&raw[start..byte], false)?);
            start = byte + 1;
        }
    }
    parts.push(path_part(&raw[start..], partial)?);
    Some(parts.join("."))
}

pub(crate) fn tail(raw: &str) -> usize {
    let mut quoted = Quoted::default();
    let mut start = 0;
    for (byte, ch) in raw.char_indices() {
        if quoted.outside(ch) && ch == '.' {
            start = byte + 1;
        }
    }
    start
}

fn path_part(raw: &str, partial: bool) -> Option<String> {
    let raw = raw.trim();
    if partial && raw.is_empty() {
        return Some(String::new());
    }
    if let Some(name) = decode(raw, Grammar::Toml) {
        return Some(encode(&name, Grammar::Toml));
    }
    if !partial {
        return None;
    }
    let quote = raw.chars().next()?;
    let rest = raw.strip_prefix(quote)?;
    let name = match quote {
        '"' => content(rest, Grammar::Toml)?,
        '\'' => rest.to_owned(),
        _ => return None,
    };
    let encoded = encode(&name, Grammar::Json);
    Some(encoded[..encoded.len() - 1].to_owned())
}

#[derive(Default)]
struct Quoted {
    quote: Option<char>,
    escaped: bool,
}

impl Quoted {
    fn outside(&mut self, ch: char) -> bool {
        if self.escaped {
            self.escaped = false;
            return false;
        }
        if self.quote == Some('"') && ch == '\\' {
            self.escaped = true;
            return false;
        }
        if self.quote == Some(ch) {
            self.quote = None;
            return false;
        }
        if self.quote.is_some() {
            return false;
        }
        if ['"', '\''].contains(&ch) {
            self.quote = Some(ch);
            return false;
        }
        true
    }
}
