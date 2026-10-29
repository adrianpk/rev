//! XML 1.0 text admission and explicit ODF whitespace, bounded before append.
use super::*;
pub(super) struct Xml<'a> {
    pub output: String,
    limit: usize,
    cancel: &'a Cancellation,
}
impl<'a> Xml<'a> {
    pub fn new(limit: usize, cancel: &'a Cancellation) -> Self {
        Self {
            output: String::new(),
            limit,
            cancel,
        }
    }
    pub fn emit(&mut self, value: &str) -> Result<(), String> {
        document::check(self.cancel)?;
        if value.len() > self.limit.saturating_sub(self.output.len()) {
            return Err("ODT export output byte limit reached".into());
        }
        self.output.push_str(value);
        Ok(())
    }
    pub fn start(&mut self, name: &str) -> Result<(), String> {
        self.emit(&format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?><office:{name} {NAMESPACES} office:version=\"1.3\">"))
    }
    pub fn text(&mut self, value: &str) -> Result<(), String> {
        for c in value.chars() {
            self.scalar(c)?;
        }
        Ok(())
    }
    pub fn prose(&mut self, value: &str) -> Result<(), String> {
        for c in value.replace("\r\n", "\n").replace('\r', "\n").chars() {
            match c {
                ' ' => self.emit("<text:s/>")?,
                '\t' => self.emit("<text:tab/>")?,
                '\n' => self.emit("<text:line-break/>")?,
                _ => self.scalar(c)?,
            }
        }
        Ok(())
    }
    fn scalar(&mut self, c: char) -> Result<(), String> {
        if !valid(c) {
            return Err(format!("ODT XML cannot represent U+{:04X}", c as u32));
        }
        let mut bytes = [0; 4];
        self.emit(match c {
            '&' => "&amp;",
            '<' => "&lt;",
            '>' => "&gt;",
            '"' => "&quot;",
            '\'' => "&apos;",
            _ => c.encode_utf8(&mut bytes),
        })
    }
}
fn valid(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\r' | '\u{20}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}')
}
const NAMESPACES: &str = "xmlns:office=\"urn:oasis:names:tc:opendocument:xmlns:office:1.0\" xmlns:text=\"urn:oasis:names:tc:opendocument:xmlns:text:1.0\" xmlns:style=\"urn:oasis:names:tc:opendocument:xmlns:style:1.0\" xmlns:fo=\"urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0\" xmlns:table=\"urn:oasis:names:tc:opendocument:xmlns:table:1.0\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" xmlns:dc=\"http://purl.org/dc/elements/1.1/\"";
