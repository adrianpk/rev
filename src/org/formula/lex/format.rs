use super::*;
impl Lexer<'_> {
    pub(super) fn format(&mut self) -> Result<Kind, Error> {
        self.advance()?;
        self.space()?;
        let start = self.at;
        while self.at < self.text.len() && !self.text[self.at..].starts_with("::") {
            if !self.byte().is_some_and(|b| b.is_ascii()) {
                return Err(self.error(
                    ErrorKind::Unsupported,
                    "unsupported decimal format character",
                ));
            }
            self.advance()?;
        }
        let text = self.text[start..self.at].trim_end();
        let source = start..start + text.len();
        let digits = decimal_digits(text).ok_or_else(|| {
            Error::new(
                ErrorKind::Unsupported,
                source.clone(),
                "only fN or %.Nf fixed decimal formatting is supported",
            )
        })?;
        let decimals = precision(digits, source.clone())?;
        if decimals > self.budget.limits.decimal_places {
            return Err(Error::new(
                ErrorKind::Limit,
                source,
                "fixed decimal places exceed configured limit",
            ));
        }
        Ok(Kind::Format(decimals))
    }
}
fn decimal_digits(text: &str) -> Option<&str> {
    if let Some(digits) = text.strip_prefix('f') {
        return Some(digits);
    }
    text.strip_prefix("%.")?.strip_suffix('f')
}

fn precision(digits: &str, source: Range<usize>) -> Result<usize, Error> {
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err(Error::new(
            ErrorKind::Syntax,
            source,
            "expected unsigned decimal places",
        ));
    }
    digits
        .parse()
        .map_err(|_| Error::new(ErrorKind::Syntax, source, "decimal places overflow"))
}
