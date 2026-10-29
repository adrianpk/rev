mod format;
use super::*;
#[derive(Debug, Clone)]
pub(super) enum Kind {
    Number(f64),
    Reference(Reference),
    Name,
    Operator(Operator),
    Open,
    Close,
    Comma,
    Equal,
    Range,
    Assignments,
    Format(usize),
    End,
}
#[derive(Debug, Clone)]
pub(super) struct Token {
    pub kind: Kind,
    pub source: Range<usize>,
}
pub(super) struct Budget<'a> {
    pub limits: Limits,
    pub cancel: &'a Cancellation,
    steps: usize,
}
pub(super) struct Lexer<'a> {
    pub text: &'a str,
    pub at: usize,
    pub budget: Budget<'a>,
}
impl<'a> Budget<'a> {
    pub fn new(limits: Limits, cancel: &'a Cancellation) -> Self {
        Self {
            limits,
            cancel,
            steps: 0,
        }
    }
    pub fn step(&mut self, at: usize) -> Result<(), Error> {
        if self.cancel.is_cancelled() {
            return Err(Error::new(
                ErrorKind::Cancelled,
                at..at,
                "cancelled or timed out",
            ));
        }
        self.steps += 1;
        if self.steps > self.limits.steps {
            return Err(Error::new(
                ErrorKind::Limit,
                at..at,
                "formula step budget exhausted",
            ));
        }
        Ok(())
    }
}
impl<'a> Lexer<'a> {
    pub fn new(text: &'a str, limits: Limits, cancel: &'a Cancellation) -> Result<Self, Error> {
        if text.len() > limits.bytes {
            return Err(Error::new(
                ErrorKind::Limit,
                0..0,
                "formula source byte limit exceeded",
            ));
        }
        Ok(Self {
            text,
            at: 0,
            budget: Budget::new(limits, cancel),
        })
    }
    pub fn scan(&mut self) -> Result<Vec<Token>, Error> {
        let mut tokens = Vec::new();
        loop {
            self.space()?;
            self.budget.step(self.at)?;
            if self.at < self.text.len() && tokens.len() >= 100000 {
                return Err(self.error(ErrorKind::Limit, "formula exceeds 100000 tokens"));
            }
            let start = self.at;
            let kind = self.token()?;
            let done = matches!(kind, Kind::End);
            tokens.push(Token {
                kind,
                source: start..self.at,
            });
            if done {
                return Ok(tokens);
            }
        }
    }
    fn token(&mut self) -> Result<Kind, Error> {
        match self.byte() {
            None => Ok(Kind::End),
            Some(b'@' | b'$') => self.reference().map(Kind::Reference),
            Some(b';') => self.format(),
            Some(b'0'..=b'9') => self.number(),
            Some(b'.') => self.dots_or_number(),
            Some(b'a'..=b'z' | b'A'..=b'Z' | b'_') => self.name(),
            _ => self.punctuation(),
        }
    }
    fn dots_or_number(&mut self) -> Result<Kind, Error> {
        if self.text[self.at..].starts_with("..") {
            self.punctuation()
        } else {
            self.number()
        }
    }
    fn punctuation(&mut self) -> Result<Kind, Error> {
        for (spelling, kind) in [("..", Kind::Range), ("::", Kind::Assignments)] {
            if self.text[self.at..].starts_with(spelling) {
                self.at += 2;
                return Ok(kind);
            }
        }
        let kind = self.symbol()?;
        self.at += 1;
        Ok(kind)
    }
    fn symbol(&self) -> Result<Kind, Error> {
        if let Some(operator) = self.byte().and_then(Operator::symbol) {
            return Ok(Kind::Operator(operator));
        }
        match self.byte() {
            Some(b'(') => Ok(Kind::Open),
            Some(b')') => Ok(Kind::Close),
            Some(b',') => Ok(Kind::Comma),
            Some(b'=') => Ok(Kind::Equal),
            _ => Err(self.error(
                ErrorKind::Unsupported,
                "unsupported Calc/Lisp or formula token",
            )),
        }
    }
    fn name(&mut self) -> Result<Kind, Error> {
        while self
            .byte()
            .is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            self.advance()?;
        }
        Ok(Kind::Name)
    }
    fn number(&mut self) -> Result<Kind, Error> {
        let start = self.at;
        self.digits()?;
        if self.byte() == Some(b'.') && !self.text[self.at..].starts_with("..") {
            self.advance()?;
            self.digits()?;
        }
        self.exponent()?;
        let value = self.text[start..self.at]
            .parse::<f64>()
            .map_err(|_| self.error(ErrorKind::Syntax, "malformed numeric literal"))?;
        if !value.is_finite() {
            return Err(Error::new(
                ErrorKind::Syntax,
                start..self.at,
                "literal is not finite binary64",
            ));
        }
        Ok(Kind::Number(value))
    }
    fn exponent(&mut self) -> Result<(), Error> {
        if matches!(self.byte(), Some(b'e' | b'E')) {
            self.advance()?;
            if matches!(self.byte(), Some(b'+' | b'-')) {
                self.advance()?;
            }
            self.digits()?;
        }
        Ok(())
    }
    pub fn digits(&mut self) -> Result<(), Error> {
        while self.byte().is_some_and(|b| b.is_ascii_digit()) {
            self.advance()?;
        }
        Ok(())
    }
    fn space(&mut self) -> Result<(), Error> {
        while self.byte().is_some_and(|b| b.is_ascii_whitespace()) {
            self.advance()?;
        }
        Ok(())
    }
    pub fn advance(&mut self) -> Result<(), Error> {
        self.budget.step(self.at)?;
        self.at += 1;
        Ok(())
    }
    pub fn byte(&self) -> Option<u8> {
        self.text.as_bytes().get(self.at).copied()
    }
    pub fn error(&self, kind: ErrorKind, message: &str) -> Error {
        let width = self.text[self.at..]
            .chars()
            .next()
            .map_or(0, char::len_utf8);
        Error::new(kind, self.at..self.at + width, message)
    }
}

impl Operator {
    fn symbol(byte: u8) -> Option<Self> {
        match byte {
            b'+' => Some(Self::Add),
            b'-' => Some(Self::Subtract),
            b'*' => Some(Self::Multiply),
            b'/' => Some(Self::Divide),
            b'^' => Some(Self::Power),
            _ => None,
        }
    }
}
