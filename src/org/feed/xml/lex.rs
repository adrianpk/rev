use super::*;
pub(super) struct Tag {
    pub name: String,
    pub attrs: BTreeMap<String, String>,
    pub empty: bool,
}
pub(super) enum Token {
    Open(Tag),
    Close(String),
    Text(String),
    Declaration,
}
pub(super) struct Reader<'a> {
    value: &'a str,
    at: usize,
    cancel: &'a Cancellation,
}
impl<'a> Reader<'a> {
    pub fn new(value: &'a str, cancel: &'a Cancellation) -> Self {
        Self {
            value,
            at: 0,
            cancel,
        }
    }
    pub fn next(&mut self) -> Result<Option<Token>, String> {
        while self.at < self.value.len() {
            check(self.cancel)?;
            if self.take("<!--") {
                self.comment()?;
                continue;
            }
            return self.token().map(Some);
        }
        Ok(None)
    }
    fn token(&mut self) -> Result<Token, String> {
        if self.take("<![CDATA[") {
            return self.until("]]>").and_then(|s| plain(&s)).map(Token::Text);
        }
        if self.take("<?xml") {
            return self.declaration();
        }
        if self.value[self.at..].starts_with("<!") {
            return Err("Feed XML forbids DTD and entity declarations".into());
        }
        if self.take("</") {
            let name = self.name()?;
            self.space();
            self.expect(">")?;
            return Ok(Token::Close(name));
        }
        if self.take("<") {
            return self.open().map(Token::Open);
        }
        let count = self.value[self.at..]
            .find('<')
            .unwrap_or(self.value.len() - self.at);
        let raw = &self.value[self.at..self.at + count];
        if raw.contains("]]>") {
            return Err("CDATA closing marker outside CDATA".into());
        }
        let value = decode(raw, self.cancel)?;
        self.at += count;
        Ok(Token::Text(value))
    }
    fn open(&mut self) -> Result<Tag, String> {
        let name = self.name()?;
        let attrs = self.attributes()?;
        let empty = self.take("/");
        self.expect(">")?;
        Ok(Tag { name, attrs, empty })
    }
    fn attributes(&mut self) -> Result<BTreeMap<String, String>, String> {
        let mut attrs = BTreeMap::new();
        loop {
            let space = self.space();
            if self.value[self.at..].starts_with(['>', '/', '?']) {
                return Ok(attrs);
            }
            if space == 0 || attrs.len() >= 64 {
                return Err("XML attribute separation/count limit".into());
            }
            let key = self.name()?;
            self.space();
            self.expect("=")?;
            self.space();
            let value = self.quoted()?;
            if attrs.insert(key, value).is_some() {
                return Err("Duplicate XML attribute".into());
            }
        }
    }
    fn quoted(&mut self) -> Result<String, String> {
        let quote = self
            .value
            .as_bytes()
            .get(self.at)
            .copied()
            .ok_or("Unclosed XML attribute")?;
        if ![b'\'', b'"'].contains(&quote) {
            return Err("XML attributes require quotes".into());
        }
        self.at += 1;
        let end = self.value[self.at..]
            .find(char::from(quote))
            .ok_or("Unclosed XML attribute")?;
        let raw = &self.value[self.at..self.at + end];
        if raw.contains('<') {
            return Err("Unescaped XML attribute delimiter".into());
        }
        let value = decode(raw, self.cancel)?;
        self.at += end + 1;
        Ok(value)
    }
    fn declaration(&mut self) -> Result<Token, String> {
        let attrs = self.attributes()?;
        self.expect("?>")?;
        if attrs.get("version").map(String::as_str) != Some("1.0") {
            return Err("Feed XML requires version 1.0".into());
        }
        if attrs
            .get("encoding")
            .is_some_and(|s| !s.eq_ignore_ascii_case("utf-8"))
        {
            return Err("Feed XML requires UTF-8".into());
        }
        Ok(Token::Declaration)
    }
    fn name(&mut self) -> Result<String, String> {
        let mut size = 0;
        for c in self.value[self.at..].chars() {
            if !c.is_alphanumeric() && !"_:-.".contains(c) {
                break;
            }
            size += c.len_utf8();
            if size > 128 {
                return Err("XML name exceeds 128 bytes".into());
            }
        }
        let name = &self.value[self.at..self.at + size];
        if !name
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_')
        {
            return Err("Invalid XML name".into());
        }
        self.at += size;
        Ok(name.into())
    }
    fn comment(&mut self) -> Result<(), String> {
        let value = self.until("-->")?;
        if value.contains("--") {
            return Err("Invalid XML comment".into());
        }
        Ok(())
    }
    fn until(&mut self, closing: &str) -> Result<String, String> {
        let count = self.value[self.at..]
            .find(closing)
            .ok_or("Unclosed XML token")?;
        if count > 65536 {
            return Err("Feed XML token exceeds 64 KiB".into());
        }
        let value = self.value[self.at..self.at + count].into();
        self.at += count + closing.len();
        Ok(value)
    }
    fn take(&mut self, value: &str) -> bool {
        if self.value[self.at..].starts_with(value) {
            self.at += value.len();
            true
        } else {
            false
        }
    }
    fn expect(&mut self, value: &str) -> Result<(), String> {
        if self.take(value) {
            Ok(())
        } else {
            Err(format!("Expected XML delimiter {value}"))
        }
    }
    fn space(&mut self) -> usize {
        let start = self.at;
        while self
            .value
            .as_bytes()
            .get(self.at)
            .is_some_and(|b| b" \t\r\n".contains(b))
        {
            self.at += 1;
        }
        self.at - start
    }
}
fn plain(value: &str) -> Result<String, String> {
    if value.len() > 65536 || value.chars().any(|c| !character(c)) {
        return Err("Invalid or oversized XML text".into());
    }
    Ok(value.replace("\r\n", "\n").replace('\r', "\n"))
}
fn decode(value: &str, cancel: &Cancellation) -> Result<String, String> {
    if value.len() > 65536 {
        return Err("Feed XML text exceeds 64 KiB".into());
    }
    let mut rest = value;
    let mut result = String::new();
    while let Some(start) = rest.find('&') {
        check(cancel)?;
        result.push_str(&plain(&rest[..start])?);
        let tail = &rest[start + 1..];
        let end = tail.find(';').ok_or("Unclosed XML entity reference")?;
        result.push(entity(&tail[..end])?);
        rest = &tail[end + 1..];
    }
    result.push_str(&plain(rest)?);
    Ok(result)
}
fn entity(value: &str) -> Result<char, String> {
    let c = match value {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "apos" => '\'',
        "quot" => '"',
        _ => numeric(value)?,
    };
    if !character(c) {
        return Err("Invalid XML character reference".into());
    }
    Ok(c)
}
fn numeric(value: &str) -> Result<char, String> {
    let (value, radix) = if let Some(s) = value.strip_prefix("#x") {
        (s, 16)
    } else {
        (value.strip_prefix('#').ok_or("Unknown XML entity")?, 10)
    };
    u32::from_str_radix(value, radix)
        .ok()
        .and_then(char::from_u32)
        .ok_or("Invalid XML numeric entity".into())
}
fn character(c: char) -> bool {
    matches!(c, '\t' | '\r' | '\n') || c >= ' ' && c != '\u{fffe}' && c != '\u{ffff}'
}
