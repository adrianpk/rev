use super::super::{CELLS, COLUMNS, Cancellation, Limits, ROWS, check};

#[derive(Clone, Copy)]
pub(super) struct Format(pub char);
impl Format {
    pub fn path(path: &std::path::Path) -> Result<Self, String> {
        match path.extension().and_then(|s| s.to_str()) {
            Some(s) if s.eq_ignore_ascii_case("csv") => Ok(Self(',')),
            Some(s) if s.eq_ignore_ascii_case("tsv") => Ok(Self('\t')),
            _ => Err("Table interchange requires a .csv or .tsv file".into()),
        }
    }
}
#[derive(Clone, Copy, PartialEq)]
enum State {
    Start,
    Bare,
    Quoted,
    Closed,
}
struct Reader {
    rows: Vec<Vec<String>>,
    row: Vec<String>,
    field: String,
    state: State,
    format: Format,
}
pub(super) fn decode(
    source: &str,
    format: Format,
    cancel: &Cancellation,
) -> Result<Vec<Vec<String>>, String> {
    if source.is_empty() || source.len() > Limits::default().bytes {
        return Err("Table data is empty or exceeds the native 8 MiB limit".into());
    }
    let mut reader = Reader {
        rows: Vec::new(),
        row: Vec::new(),
        field: String::new(),
        state: State::Start,
        format,
    };
    let mut chars = source.chars().peekable();
    while let Some(c) = chars.next() {
        check(cancel)?;
        if c == '\r' && reader.state != State::Quoted {
            if chars.next() != Some('\n') {
                return Err("Table records require LF or CRLF endings".into());
            }
            reader.character('\n')?;
        } else {
            reader.character(c)?;
        }
    }
    reader.finish()
}
impl Reader {
    fn character(&mut self, c: char) -> Result<(), String> {
        if self.state == State::Quoted {
            return self.quoted(c);
        }
        if c == self.format.0 {
            return self.field();
        }
        if c == '\n' {
            return self.row();
        }
        if c == '"' && self.state == State::Start {
            self.state = State::Quoted;
            return Ok(());
        }
        if c == '"' && self.state == State::Closed {
            self.field.push('"');
            self.state = State::Quoted;
            return Ok(());
        }
        if c == '"' || self.state == State::Closed {
            return Err("Malformed table quoting; no data imported".into());
        }
        self.field.push(c);
        self.state = State::Bare;
        Ok(())
    }
    fn quoted(&mut self, c: char) -> Result<(), String> {
        if matches!(c, '\r' | '\n') {
            return Err("Embedded cell line breaks cannot be represented in an Org table".into());
        }
        if c == '"' {
            self.state = State::Closed;
        } else {
            self.field.push(c);
        }
        Ok(())
    }
    fn field(&mut self) -> Result<(), String> {
        representable(&self.field)?;
        self.row.push(std::mem::take(&mut self.field));
        self.state = State::Start;
        if self.row.len() > COLUMNS {
            return Err("Table data exceeds 256 columns".into());
        }
        Ok(())
    }
    fn row(&mut self) -> Result<(), String> {
        self.field()?;
        if self.rows.first().is_some_and(|r| r.len() != self.row.len()) {
            return Err("Ragged table data cannot be converted without inventing fields".into());
        }
        self.rows.push(std::mem::take(&mut self.row));
        if self.rows.len() > ROWS || self.rows.len() * self.rows[0].len() > CELLS {
            return Err("Table data exceeds 10000 rows or 100000 cells".into());
        }
        Ok(())
    }
    fn finish(mut self) -> Result<Vec<Vec<String>>, String> {
        if self.state == State::Quoted {
            return Err("Unterminated quoted table field".into());
        }
        if self.state != State::Start || !self.row.is_empty() || self.rows.is_empty() {
            self.row()?;
        }
        Ok(self.rows)
    }
}
pub(super) fn representable(value: &str) -> Result<(), String> {
    if value.contains(['\u{2028}', '\u{2029}']) {
        return Err("Embedded cell line breaks cannot be represented in an Org table".into());
    }
    if value.trim_matches([' ', '\t']) != value {
        return Err(
            "Leading or trailing cell whitespace cannot be retained as an Org value".into(),
        );
    }
    if value.contains('|') || value.chars().any(|c| c.is_control() && c != '\t') {
        return Err(
            "Cell delimiters or control characters cannot be represented losslessly".into(),
        );
    }
    Ok(())
}
pub(super) fn encode(
    rows: &[Vec<String>],
    format: Format,
    cancel: &Cancellation,
) -> Result<String, String> {
    let mut out = String::new();
    for row in rows {
        check(cancel)?;
        record(&mut out, row, format, cancel)?;
    }
    Ok(out)
}
fn record(
    out: &mut String,
    row: &[String],
    format: Format,
    cancel: &Cancellation,
) -> Result<(), String> {
    for (index, value) in row.iter().enumerate() {
        check(cancel)?;
        representable(value)?;
        if index != 0 {
            out.push(format.0);
        }
        out.push('"');
        out.push_str(&value.replace('"', "\"\""));
        out.push('"');
        if out.len() >= Limits::default().bytes {
            return Err("Table export exceeds the native 8 MiB output limit".into());
        }
    }
    out.push('\n');
    Ok(())
}
