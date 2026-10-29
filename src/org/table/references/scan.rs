use super::*;
mod reference;

pub(super) struct Number {
    pub(super) value: usize,
    source: Range<usize>,
}
pub(super) struct Reference {
    pub(super) row: Option<Number>,
    pub(super) column: Option<Number>,
}
pub(super) struct Scanner<'a> {
    text: &'a str,
    at: usize,
    steps: usize,
    cancel: &'a Cancellation,
    edits: Vec<(Range<usize>, String)>,
}
impl<'a> Scanner<'a> {
    pub(super) fn new(text: &'a str, cancel: &'a Cancellation) -> Self {
        Self {
            text,
            at: 0,
            steps: 0,
            cancel,
            edits: Vec::new(),
        }
    }
    pub(super) fn rewrite(mut self, axes: &Axes) -> Result<Vec<(Range<usize>, String)>, String> {
        self.space();
        while self.at < self.text.len() {
            self.step()?;
            self.assignment(axes)?;
            self.space();
            if self.at < self.text.len() && !self.take("::") {
                return Err("Expected a compatible assignment separator".into());
            }
            self.space();
            if self.at == self.text.len() && self.text.trim_end().ends_with("::") {
                return Err("Empty formula assignment".into());
            }
        }
        Ok(self.edits)
    }
    fn assignment(&mut self, axes: &Axes) -> Result<(), String> {
        let target = self.reference()?;
        if target.column.is_none() {
            return Err("Only absolute field/column assignment targets are supported".into());
        }
        axes.validate(&target)?;
        self.emit(&target, axes)?;
        self.space();
        if !self.take("=") {
            return Err("Expected an absolute field/column assignment".into());
        }
        self.expression(axes)
    }
    fn expression(&mut self, axes: &Axes) -> Result<(), String> {
        let mut frames = Vec::new();
        let mut expecting = true;
        while self.at < self.text.len() && !self.text[self.at..].starts_with("::") {
            self.step()?;
            self.space();
            if self.at == self.text.len() || self.text[self.at..].starts_with("::") {
                break;
            }
            self.token(axes, &mut frames, &mut expecting)?;
        }
        if expecting || !frames.is_empty() {
            return Err("Incomplete formula expression".into());
        }
        Ok(())
    }
    fn token(
        &mut self,
        axes: &Axes,
        frames: &mut Vec<bool>,
        expecting: &mut bool,
    ) -> Result<(), String> {
        if matches!(
            self.byte(),
            Some(b'@' | b'$' | b'0'..=b'9' | b'.' | b'a'..=b'z' | b'(')
        ) {
            operand(expecting)?;
            *expecting = self.value(axes, frames)?;
            return Ok(());
        }
        self.punctuation(frames, expecting)
    }
    fn value(&mut self, axes: &Axes, frames: &mut Vec<bool>) -> Result<bool, String> {
        match self.byte() {
            Some(b'@' | b'$') => self.field(axes).map(|_| false),
            Some(b'0'..=b'9' | b'.') => self.literal().map(|_| false),
            Some(b'a'..=b'z') => self.function(frames).map(|_| true),
            Some(b'(') => self.open(frames, false).map(|_| true),
            _ => unreachable!("classified formula value"),
        }
    }
    fn punctuation(&mut self, frames: &mut Vec<bool>, expecting: &mut bool) -> Result<(), String> {
        match self.byte() {
            Some(b'+' | b'-' | b'*' | b'/' | b'^') => self.operator(expecting),
            Some(b')') => self.close(frames, *expecting),
            Some(b',') => self.comma(frames, expecting),
            _ => Err("Unsupported or ambiguous formula token".into()),
        }?;
        self.at += 1;
        Ok(())
    }
    fn operator(&self, expecting: &mut bool) -> Result<(), String> {
        if *expecting && !matches!(self.byte(), Some(b'+' | b'-')) {
            return Err("Expected a formula value".into());
        }
        *expecting = true;
        Ok(())
    }
    fn close(&self, frames: &mut Vec<bool>, expecting: bool) -> Result<(), String> {
        if expecting {
            return Err("Empty or incomplete parenthesized expression".into());
        }
        frames.pop().ok_or("Unmatched formula parenthesis")?;
        Ok(())
    }
    fn comma(&self, frames: &[bool], expecting: &mut bool) -> Result<(), String> {
        if *expecting || frames.last() != Some(&true) {
            return Err("Unexpected formula argument separator".into());
        }
        *expecting = true;
        Ok(())
    }
    fn function(&mut self, frames: &mut Vec<bool>) -> Result<(), String> {
        let start = self.at;
        while self.byte().is_some_and(|b| b.is_ascii_alphabetic()) {
            self.progress()?;
        }
        if !matches!(
            &self.text[start..self.at],
            "sum" | "mean" | "min" | "max" | "round" | "vsum" | "vmean" | "vmin" | "vmax"
        ) {
            return Err("Unsupported formula function or named reference".into());
        }
        self.space();
        self.open(frames, true)
    }
    fn open(&mut self, frames: &mut Vec<bool>, function: bool) -> Result<(), String> {
        if frames.len() >= 64 {
            return Err("Formula reference scan exceeds depth 64".into());
        }
        if !self.take("(") {
            return Err("Expected a formula function argument list".into());
        }
        frames.push(function);
        Ok(())
    }
    fn literal(&mut self) -> Result<(), String> {
        let start = self.at;
        while self.byte().is_some_and(|b| {
            number_byte(
                b,
                self.text.as_bytes().get(self.at.wrapping_sub(1)).copied(),
            )
        }) {
            self.progress()?;
        }
        let value = self.text[start..self.at]
            .parse::<f64>()
            .map_err(|_| "Malformed numeric literal")?;
        if !value.is_finite() {
            return Err("Numeric literal is not finite binary64".into());
        }
        Ok(())
    }
    fn step(&mut self) -> Result<(), String> {
        check(self.cancel)?;
        self.steps += 1;
        if self.steps > 100000 {
            return Err("Formula reference scan exceeds 100000 tokens".into());
        }
        Ok(())
    }
    fn progress(&mut self) -> Result<(), String> {
        if self.at.is_multiple_of(64) {
            check(self.cancel)?;
        }
        self.at += 1;
        Ok(())
    }
    fn space(&mut self) {
        while self.byte().is_some_and(|b| b.is_ascii_whitespace()) {
            self.at += 1;
        }
    }
    fn take(&mut self, value: &str) -> bool {
        if self.text[self.at..].starts_with(value) {
            self.at += value.len();
            true
        } else {
            false
        }
    }
    fn byte(&self) -> Option<u8> {
        self.text.as_bytes().get(self.at).copied()
    }
}
fn operand(expecting: &mut bool) -> Result<(), String> {
    if !*expecting {
        return Err("Expected a formula operator".into());
    }
    *expecting = false;
    Ok(())
}
fn number_byte(byte: u8, previous: Option<u8>) -> bool {
    byte.is_ascii_digit()
        || matches!(byte, b'.' | b'e' | b'E')
        || matches!(byte, b'+' | b'-') && matches!(previous, Some(b'e' | b'E'))
}
