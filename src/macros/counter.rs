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
    if format.contains('\0') {
        return Err("Macro counter format contains NUL");
    }
    if format.len() > 256 {
        return Err("Macro counter format exceeds 256 bytes");
    }
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
        let (mut left, mut plus, mut space, mut alternate, mut zero) =
            (false, false, false, false, false);
        loop {
            match chars.peek() {
                Some('-') => left = true,
                Some('+') => plus = true,
                Some(' ') => space = true,
                Some('#') => alternate = true,
                Some('0') => zero = true,
                _ => break,
            }
            chars.next();
        }
        let width = decimal(&mut chars)?.unwrap_or(0);
        let precision = if chars.peek() == Some(&'.') {
            chars.next();
            Some(decimal(&mut chars)?.unwrap_or(0))
        } else {
            None
        };
        let conversion = chars.next().ok_or("Incomplete macro counter format")?;
        let magnitude = value.unsigned_abs();
        let mut digits = match conversion {
            'd' => magnitude.to_string(),
            'o' => format!("{magnitude:o}"),
            'x' => format!("{magnitude:x}"),
            'X' => format!("{magnitude:X}"),
            _ => return Err("Macro counter format supports d, o, x and X only"),
        };
        if precision == Some(0) && magnitude == 0 {
            digits.clear();
        }
        if let Some(precision) = precision {
            digits = format!(
                "{}{}",
                "0".repeat(precision.saturating_sub(digits.len())),
                digits
            );
        }
        let sign = if value < 0 {
            "-"
        } else if plus {
            "+"
        } else if space {
            " "
        } else {
            ""
        };
        let prefix = if alternate {
            match conversion {
                'o' if !digits.starts_with('0') => "0",
                'x' if magnitude != 0 => "0x",
                'X' if magnitude != 0 => "0X",
                _ => "",
            }
        } else {
            ""
        };
        let padding = width.saturating_sub(sign.len() + prefix.len() + digits.len());
        if !left && (!zero || precision.is_some()) {
            output.push_str(&" ".repeat(padding));
        }
        output.push_str(sign);
        output.push_str(prefix);
        if !left && zero && precision.is_none() {
            output.push_str(&"0".repeat(padding));
        }
        output.push_str(&digits);
        if left {
            output.push_str(&" ".repeat(padding));
        }
    }
    if !converted {
        return Err("Macro counter format requires one integer conversion");
    }
    if output.len() > 512 {
        return Err("Rendered macro counter exceeds 512 bytes");
    }
    Ok(output)
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
