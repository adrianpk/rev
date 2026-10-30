//! FEEDSTATUS is a literal list of (id handled checksum) records, never Lisp.
use super::*;
use std::collections::BTreeMap;
#[derive(Clone, Debug)]
struct Record {
    handled: bool,
    checksum: Option<String>,
}
#[derive(Default)]
pub(super) struct Status {
    values: BTreeMap<String, Record>,
}
impl Status {
    pub fn parse(value: &str, cancel: &Cancellation) -> Result<Self, String> {
        let mut read = Read {
            value,
            at: 0,
            cancel,
        };
        if read.symbol("nil") {
            read.finish()?;
            return Ok(Self::default());
        }
        read.expect('(')?;
        let mut values = BTreeMap::new();
        while !read.take(')') {
            xml::check(cancel)?;
            if values.len() >= 10000 {
                return Err("FEEDSTATUS exceeds 10000 retained identities".into());
            }
            let (id, record) = read.record()?;
            if values.insert(id, record).is_some() {
                return Err("Duplicate FEEDSTATUS identity".into());
            }
        }
        read.finish()?;
        Ok(Self { values })
    }
    pub fn handled(&self, id: &str) -> bool {
        self.values.get(id).is_some_and(|r| r.handled)
    }
    pub fn add(&mut self, id: &str) -> Result<(), String> {
        if self.values.len() >= 10000 && !self.values.contains_key(id) {
            return Err("FEEDSTATUS retained identity limit reached".into());
        }
        self.values
            .entry(id.into())
            .and_modify(|r| r.handled = true)
            .or_insert(Record {
                handled: true,
                checksum: None,
            });
        Ok(())
    }
    pub fn render(&self) -> String {
        if self.values.is_empty() {
            return "nil\n".into();
        }
        let mut result = String::from("(");
        for (i, (id, record)) in self.values.iter().enumerate() {
            if i > 0 {
                result.push('\n');
            }
            let checksum = record
                .checksum
                .as_deref()
                .map(quote)
                .unwrap_or("nil".into());
            result.push_str(&format!(
                "({} {} {checksum})",
                quote(id),
                if record.handled { "t" } else { "nil" }
            ));
        }
        result.push_str(")\n");
        result
    }
}
fn quote(value: &str) -> String {
    let text = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{text}\"")
}
struct Read<'a> {
    value: &'a str,
    at: usize,
    cancel: &'a Cancellation,
}
impl Read<'_> {
    fn record(&mut self) -> Result<(String, Record), String> {
        self.expect('(')?;
        let id = self.string()?;
        let handled = if self.symbol("t") {
            true
        } else {
            self.required_symbol("nil")?;
            false
        };
        let checksum = if self.symbol("nil") {
            None
        } else {
            Some(self.string()?)
        };
        self.expect(')')?;
        identity(&id)?;
        Ok((id, Record { handled, checksum }))
    }
    fn string(&mut self) -> Result<String, String> {
        self.expect('"')?;
        let mut value = String::new();
        loop {
            xml::check(self.cancel)?;
            let c = self.character()?;
            if c == '"' {
                return Ok(value);
            }
            let c = if c == '\\' { self.escape()? } else { c };
            value.push(c);
            if value.len() > 4096 {
                return Err("FEEDSTATUS string exceeds 4096 bytes".into());
            }
        }
    }
    fn escape(&mut self) -> Result<char, String> {
        match self.character()? {
            'n' => Ok('\n'),
            'r' => Ok('\r'),
            't' => Ok('\t'),
            '"' => Ok('"'),
            '\\' => Ok('\\'),
            _ => Err("Unsupported FEEDSTATUS string escape".into()),
        }
    }
    fn character(&mut self) -> Result<char, String> {
        let c = self.value[self.at..]
            .chars()
            .next()
            .ok_or("Unclosed FEEDSTATUS literal")?;
        self.at += c.len_utf8();
        Ok(c)
    }
    fn space(&mut self) {
        while self.value[self.at..]
            .chars()
            .next()
            .is_some_and(char::is_whitespace)
        {
            self.at += self.value[self.at..]
                .chars()
                .next()
                .expect("checked character")
                .len_utf8();
        }
    }
    fn take(&mut self, c: char) -> bool {
        self.space();
        if self.value[self.at..].starts_with(c) {
            self.at += c.len_utf8();
            true
        } else {
            false
        }
    }
    fn expect(&mut self, c: char) -> Result<(), String> {
        if self.take(c) {
            Ok(())
        } else {
            Err(format!("FEEDSTATUS requires literal delimiter {c}"))
        }
    }
    fn symbol(&mut self, symbol: &str) -> bool {
        self.space();
        let tail = &self.value[self.at..];
        if tail.starts_with(symbol)
            && tail[symbol.len()..]
                .chars()
                .next()
                .is_none_or(|c| c.is_whitespace() || c == ')')
        {
            self.at += symbol.len();
            true
        } else {
            false
        }
    }
    fn required_symbol(&mut self, symbol: &str) -> Result<(), String> {
        if self.symbol(symbol) {
            Ok(())
        } else {
            Err("FEEDSTATUS accepts handled t/nil only".into())
        }
    }
    fn finish(&mut self) -> Result<(), String> {
        self.space();
        if self.at == self.value.len() {
            Ok(())
        } else {
            Err("Executable or trailing FEEDSTATUS data rejected".into())
        }
    }
}

fn identity(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > 4096 || id.chars().any(char::is_control) {
        return Err("Invalid FEEDSTATUS identity".into());
    }
    Ok(())
}
