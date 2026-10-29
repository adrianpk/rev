use super::*;
pub(super) fn tokens(text: &str) -> Result<Vec<Token>, String> {
    let mut parser = Parser {
        text,
        position: 0,
        tokens: Vec::new(),
        literal: String::new(),
    };
    while parser.position < text.len() {
        parser.next()?;
    }
    parser.flush()?;
    Ok(parser.tokens)
}
struct Parser<'a> {
    text: &'a str,
    position: usize,
    tokens: Vec<Token>,
    literal: String,
}
impl Parser<'_> {
    fn next(&mut self) -> Result<(), String> {
        let tail = &self.text[self.position..];
        if tail.starts_with("\\%") {
            self.literal.push('%');
            self.position += 2;
            return Ok(());
        }
        let c = tail.chars().next().expect("remaining template");
        self.position += c.len_utf8();
        if c != '%' {
            self.literal.push(c);
            return Ok(());
        }
        self.flush()?;
        let c = self.text[self.position..]
            .chars()
            .next()
            .ok_or("Trailing % in capture template")?;
        self.position += c.len_utf8();
        let token = self.escape(c)?;
        self.push(token)
    }
    fn escape(&mut self, c: char) -> Result<Token, String> {
        if "ialtTuUncx".contains(c) {
            return Ok(Token::Field(c));
        }
        let token = match c {
            '?' => Token::Cursor,
            '%' => Token::Literal("%".into()),
            '^' => Token::Prompt(self.prompt()?),
            '<' => self.format()?,
            '(' | '[' => {
                return Err(
                    "Capture function/Lisp/file escapes are unsupported and never evaluated".into(),
                );
            }
            _ => return Err(format!("Unsupported capture escape %{c}")),
        };
        Ok(token)
    }
    fn format(&mut self) -> Result<Token, String> {
        let format = self.delimited('>', 256)?;
        time::validate(&format)?;
        Ok(Token::Format(format))
    }
    fn delimited(&mut self, end: char, maximum: usize) -> Result<String, String> {
        let tail = &self.text[self.position..];
        let size = tail.find(end).ok_or("Unterminated capture escape")?;
        let value = &tail[..size];
        bounded(value, maximum)?;
        self.position += size + end.len_utf8();
        Ok(value.into())
    }
    fn prompt(&mut self) -> Result<Prompt, String> {
        if !self.text[self.position..].starts_with('{') {
            return Err("Capture string prompt requires %^{label}".into());
        }
        self.position += 1;
        let value = self.delimited('}', 16384)?;
        let mut parts = value.split('|');
        let label = parts.next().expect("prompt label");
        if label.trim().is_empty() || label.len() > 256 || label.chars().any(char::is_control) {
            return Err(
                "Capture prompt label requires 1..256 bytes without control characters".into(),
            );
        }
        let default = parts.next().unwrap_or("").to_owned();
        bounded(&default, 4096)?;
        let mut completions = Vec::new();
        for part in parts {
            if completions.len() == 64 {
                return Err("Capture prompt exceeds 64 completions".into());
            }
            bounded(part, 4096)?;
            completions.push(part.into());
        }
        Ok(Prompt {
            label: format!("{label}: "),
            default,
            completions,
        })
    }
    fn push(&mut self, token: Token) -> Result<(), String> {
        if self.tokens.len() == 4096 {
            return Err("Capture template exceeds 4096 tokens".into());
        }
        self.tokens.push(token);
        Ok(())
    }
    fn flush(&mut self) -> Result<(), String> {
        if !self.literal.is_empty() {
            let literal = std::mem::take(&mut self.literal);
            self.push(Token::Literal(literal))?;
        }
        Ok(())
    }
}
