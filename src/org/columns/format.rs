use crate::config::{AgendaBlock, AgendaKind, AgendaView};
#[derive(Clone, Copy)]
pub(super) enum Summary {
    Number,
    Duration,
}
pub(super) struct Column {
    pub key: String,
    pub summary: Option<Summary>,
}
pub(crate) struct Format {
    pub(super) columns: Vec<Column>,
}
impl Format {
    pub(crate) fn parse(source: &str) -> Result<Self, String> {
        let source = if source.trim().is_empty() {
            "%ITEM %EFFORT{:} %CLOCKSUM %CLOCKOPEN %CLOCKUNRESOLVED"
        } else {
            source
        };
        if source.len() > 4096 || source.chars().any(char::is_control) {
            return Err("Column format requires at most 4096 bytes without controls".into());
        }
        let mut columns = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        for word in source.split_whitespace() {
            let column = Column::parse(word)?;
            if columns.len() == 16 || !seen.insert(column.key.clone()) {
                return Err("Column format requires 1..16 distinct properties".into());
            }
            columns.push(column);
        }
        Ok(Self { columns })
    }
    pub(super) fn view(&self) -> AgendaView {
        AgendaView {
            key: "columns".into(),
            title: "Property columns".into(),
            blocks: vec![AgendaBlock {
                kind: AgendaKind::Columns,
                title: "Columns".into(),
                columns: self.columns.iter().map(|c| c.key.clone()).collect(),
                ..AgendaBlock::default()
            }],
        }
    }
}
impl Column {
    fn parse(word: &str) -> Result<Self, String> {
        let word = word
            .strip_prefix('%')
            .ok_or("Column fields require %PROPERTY")?;
        let (key, summary) = if let Some((key, summary)) = word.split_once('{') {
            let summary = match summary {
                "+}" => Summary::Number,
                ":}" => Summary::Duration,
                _ => return Err("Unsupported column summary; admit {+} or {:}".into()),
            };
            (key, Some(summary))
        } else {
            (word, None)
        };
        let key = property(key)?;
        if summary.is_some()
            && matches!(
                key.as_str(),
                "ITEM"
                    | "TODO"
                    | "TAGS"
                    | "ALLTAGS"
                    | "LEVEL"
                    | "PRIORITY"
                    | "CLOCKSUM"
                    | "CLOCKOPEN"
                    | "CLOCKUNRESOLVED"
            )
        {
            return Err("Summary operators require a stored numeric or duration property".into());
        }
        Ok(Self { key, summary })
    }
}
pub(crate) struct Options {
    pub format: Option<String>,
    pub global: bool,
    pub maxlevel: usize,
}
impl Options {
    pub(crate) fn parse(source: &str) -> Result<Self, String> {
        if source.len() > 4096 {
            return Err("Columnview options exceed 4096 bytes".into());
        }
        let mut value = Self {
            format: None,
            global: false,
            maxlevel: 256,
        };
        let mut seen = std::collections::BTreeSet::new();
        let mut rest = source.trim();
        while !rest.is_empty() {
            let (key, tail) = word(rest)?;
            let (arg, tail) = word(tail.trim_start())?;
            if !seen.insert(key) {
                return Err("Repeated columnview option".into());
            }
            value.option(key, arg)?;
            rest = tail.trim_start();
        }
        Ok(value)
    }
    fn option(&mut self, key: &str, value: &str) -> Result<(), String> {
        match key {
            ":id" => {
                self.global = match value {
                    "global" => true,
                    "local" => false,
                    _ => return Err("Columnview :id requires local or global".into()),
                }
            }
            ":format" => self.format = Some(value.into()),
            ":maxlevel" => {
                self.maxlevel = value
                    .parse()
                    .map_err(|_| "Columnview maxlevel requires 1..256")?;
                if !(1..=256).contains(&self.maxlevel) {
                    return Err("Columnview maxlevel requires 1..256".into());
                }
            }
            _ => return Err(format!("Unsupported columnview option: {key}")),
        }
        Ok(())
    }
}
fn word(source: &str) -> Result<(&str, &str), String> {
    if source.is_empty() {
        return Err("Columnview option requires a value".into());
    }
    if let Some(source) = source.strip_prefix('"') {
        let end = source
            .find('"')
            .ok_or("Unclosed columnview option string")?;
        let tail = &source[end + 1..];
        if !tail.is_empty() && !tail.starts_with(char::is_whitespace) {
            return Err("Columnview options require separators".into());
        }
        return Ok((&source[..end], tail));
    }
    let end = source.find(char::is_whitespace).unwrap_or(source.len());
    Ok((&source[..end], &source[end..]))
}

fn property(key: &str) -> Result<String, String> {
    if key.bytes().next().is_none_or(|b| !b.is_ascii_alphabetic()) {
        return Err("Column widths and numeric property names are unsupported".into());
    }
    if key.len() > 64 || !key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
        return Err("Column property requires 1..64 ASCII letters, digits or underscores".into());
    }
    Ok(key.to_ascii_uppercase())
}
