//! Checked counter state and the deliberately small integer format language.
use crate::command::PrefixArgument;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Counter {
    pub current: i64,
    pub previous: i64,
    pub iteration_start: i64,
    pub format: String,
}

impl Default for Counter {
    fn default() -> Self {
        Self {
            current: 0,
            previous: 0,
            iteration_start: 0,
            format: "%d".into(),
        }
    }
}

#[derive(Clone)]
pub(crate) enum Operation {
    Insert(PrefixArgument),
    Set(i64),
    Reset,
    Add(i64),
    Restore,
    Format(String),
}

impl Operation {
    pub fn bytes(&self) -> usize {
        match self {
            Self::Format(value) => super::capture::text_bytes(value),
            _ => 96,
        }
    }
}

/// Parse and render together so accepted formats always have a bounded result.
/// Negative values use a sign and magnitude in every supported radix.
pub(crate) fn render(format: &str, value: i64) -> Result<String, &'static str> {
    admit_format(format)?;
    let mut chars = format.chars().peekable();
    let mut output = String::new();
    let mut converted = false;
    while let Some(c) = chars.next() {
        if c != '%' {
            output.push(c);
            continue;
        }
        if chars.peek() == Some(&'%') {
            chars.next();
            output.push('%');
            continue;
        }
        if converted {
            return Err("Macro counter format requires one integer conversion");
        }
        converted = true;
        Conversion::parse(&mut chars)?.render(value, &mut output)?;
    }
    if !converted {
        return Err("Macro counter format requires one integer conversion");
    }
    if output.len() > 512 {
        return Err("Rendered macro counter exceeds 512 bytes");
    }
    Ok(output)
}

#[derive(Default)]
struct Flags {
    left: bool,
    plus: bool,
    space: bool,
    alternate: bool,
    zero: bool,
}

impl Flags {
    fn parse(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Self {
        let mut flags = Self::default();
        loop {
            match chars.peek() {
                Some('-') => flags.left = true,
                Some('+') => flags.plus = true,
                Some(' ') => flags.space = true,
                Some('#') => flags.alternate = true,
                Some('0') => flags.zero = true,
                _ => break,
            }
            chars.next();
        }
        flags
    }

    fn sign(&self, value: i64) -> &'static str {
        if value < 0 {
            "-"
        } else if self.plus {
            "+"
        } else if self.space {
            " "
        } else {
            ""
        }
    }
}

struct Conversion {
    flags: Flags,
    width: usize,
    precision: Option<usize>,
    radix: char,
}

impl Conversion {
    fn parse(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Result<Self, &'static str> {
        let flags = Flags::parse(chars);
        let width = decimal(chars)?.unwrap_or(0);
        let precision = if chars.peek() == Some(&'.') {
            chars.next();
            Some(decimal(chars)?.unwrap_or(0))
        } else {
            None
        };
        let radix = chars.next().ok_or("Incomplete macro counter format")?;
        Ok(Self {
            flags,
            width,
            precision,
            radix,
        })
    }

    fn render(&self, value: i64, output: &mut String) -> Result<(), &'static str> {
        let magnitude = value.unsigned_abs();
        let mut digits = match self.radix {
            'd' => magnitude.to_string(),
            'o' => format!("{magnitude:o}"),
            'x' => format!("{magnitude:x}"),
            'X' => format!("{magnitude:X}"),
            _ => return Err("Macro counter format supports d, o, x and X only"),
        };
        if self.precision == Some(0) && magnitude == 0 {
            digits.clear();
        }
        if let Some(precision) = self.precision {
            digits = format!(
                "{}{}",
                "0".repeat(precision.saturating_sub(digits.len())),
                digits
            );
        }
        let sign = self.flags.sign(value);
        let prefix = self.prefix(magnitude, &digits);
        self.append(output, sign, prefix, &digits);
        Ok(())
    }

    fn prefix(&self, magnitude: u64, digits: &str) -> &'static str {
        if self.flags.alternate {
            match self.radix {
                'o' if !digits.starts_with('0') => "0",
                'x' if magnitude != 0 => "0x",
                'X' if magnitude != 0 => "0X",
                _ => "",
            }
        } else {
            ""
        }
    }

    fn append(&self, output: &mut String, sign: &str, prefix: &str, digits: &str) {
        let padding = self
            .width
            .saturating_sub(sign.len() + prefix.len() + digits.len());
        if !self.flags.left && (!self.flags.zero || self.precision.is_some()) {
            output.push_str(&" ".repeat(padding));
        }
        output.push_str(sign);
        output.push_str(prefix);
        if !self.flags.left && self.flags.zero && self.precision.is_none() {
            output.push_str(&"0".repeat(padding));
        }
        output.push_str(digits);
        if self.flags.left {
            output.push_str(&" ".repeat(padding));
        }
    }
}

fn decimal(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> Result<Option<usize>, &'static str> {
    let mut value = None;
    while let Some(digit) = chars.peek().and_then(|c| c.to_digit(10)) {
        chars.next();
        let next = value.unwrap_or(0usize) * 10 + digit as usize;
        if next > 128 {
            return Err("Macro counter width and precision cannot exceed 128");
        }
        value = Some(next);
    }
    Ok(value)
}

fn admit_format(format: &str) -> Result<(), &'static str> {
    if format.contains('\0') {
        return Err("Macro counter format contains NUL");
    }
    if format.len() > 256 {
        return Err("Macro counter format exceeds 256 bytes");
    }
    Ok(())
}
