use super::{Value, parse::Parser};
impl Parser<'_> {
    pub(super) fn name(&mut self) -> Result<String, String> {
        self.space();
        let mut name = String::new();
        while let Some(character) = self.source[self.position..].chars().next() {
            if character == '\\' {
                self.position += 1;
                name.push(self.escaped_name()?);
            } else if character.is_alphanumeric() || "_@#%".contains(character) {
                self.position += character.len_utf8();
                name.push(character);
            } else {
                break;
            }
            if name.len() > 128 {
                return self.fail("name exceeds 128 UTF-8 bytes");
            }
        }
        if name.is_empty() {
            return self.fail("expected a tag or property name");
        }
        Ok(name)
    }
    fn escaped_name(&mut self) -> Result<char, String> {
        let Some(character) = self.source[self.position..].chars().next() else {
            return self.fail("unterminated name escape");
        };
        self.position += character.len_utf8();
        Ok(character)
    }
    pub(super) fn value(&mut self) -> Result<Value, String> {
        if self.eat('"') {
            return self.quoted().map(Value::Text);
        }
        self.space();
        let start = self.position;
        // The scalar scanner distinguishes a leading/exponent sign from the
        // following implicit Boolean AND/NOT operators.
        self.numeric_sign();
        self.digits();
        if self.source[self.position..].starts_with('.') {
            self.position += 1;
            self.digits();
        }
        if self.source[self.position..].starts_with(['e', 'E']) {
            self.position += 1;
            self.numeric_sign();
            self.digits();
        }
        self.source[start..self.position]
            .parse::<f64>()
            .ok()
            .filter(|n| n.is_finite())
            .map(Value::Number)
            .ok_or_else(|| {
                format!("Org match at byte {start}: expected a finite number or quoted string")
            })
    }
    fn numeric_sign(&mut self) {
        if self.source[self.position..].starts_with(['+', '-']) {
            self.position += 1;
        }
    }
    fn digits(&mut self) {
        while self
            .source
            .as_bytes()
            .get(self.position)
            .is_some_and(u8::is_ascii_digit)
        {
            self.position += 1;
        }
    }
    fn quoted(&mut self) -> Result<String, String> {
        let mut text = String::new();
        while let Some(c) = self.source[self.position..].chars().next() {
            self.position += c.len_utf8();
            if c == '"' {
                return Ok(text);
            }
            if c == '\\' {
                text.push(self.quoted_escape()?);
            } else {
                text.push(c);
            }
        }
        self.fail("unterminated quoted value")
    }
    fn quoted_escape(&mut self) -> Result<char, String> {
        match self.escaped_name()? {
            '"' => Ok('"'),
            '\\' => Ok('\\'),
            'n' => Ok('\n'),
            'r' => Ok('\r'),
            't' => Ok('\t'),
            _ => self.fail("unknown quoted-value escape"),
        }
    }
}
