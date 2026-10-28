//! Nonnegative Org work durations, kept distinct from calendar scheduling.
//! Parsing validates entered data; editing retains the original source spelling.
use crate::config::DurationUnits;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Duration {
    minutes: f64,
}
impl Duration {
    pub fn parse(text: &str, units: &DurationUnits) -> Result<Self, String> {
        let text = text.trim();
        if text.is_empty() || text.len() > 128 {
            return Err("Org duration requires 1..128 UTF-8 bytes".into());
        }
        let minutes = if text.contains(':') {
            clock(text)?
        } else if text.bytes().all(number_byte) {
            number(text)?
        } else {
            terms(text, units)?
        };
        if !minutes.is_finite() || minutes < 0.0 {
            return Err("Org duration must be finite and nonnegative".into());
        }
        Ok(Self { minutes })
    }
    pub fn minutes(self) -> f64 {
        self.minutes
    }
}
fn clock(text: &str) -> Result<f64, String> {
    let values: Vec<_> = text
        .split(':')
        .take(4)
        .map(number)
        .collect::<Result<_, _>>()?;
    if !(2..=3).contains(&values.len()) {
        return Err("Org duration requires H:MM or H:MM:SS".into());
    }
    Ok(values[0] * 60.0 + values[1] + values.get(2).copied().unwrap_or(0.0) / 60.0)
}
fn number_byte(byte: u8) -> bool {
    byte.is_ascii_digit() || byte == b'.'
}
fn number(text: &str) -> Result<f64, String> {
    if text.is_empty() || !text.bytes().all(number_byte) {
        return Err("Org duration requires an unsigned decimal number".into());
    }
    let value: f64 = text.parse().map_err(|_| "Invalid Org duration number")?;
    if !value.is_finite() {
        return Err("Org duration number overflows".into());
    }
    Ok(value)
}
fn terms(mut text: &str, units: &DurationUnits) -> Result<f64, String> {
    let mut total = 0.0;
    let mut count = 0;
    while !text.is_empty() {
        count += 1;
        if count > 32 {
            return Err("Org duration exceeds 32 components".into());
        }
        let end = text.bytes().take_while(|b| number_byte(*b)).count();
        let value = number(&text[..end])?;
        let rest = &text[end..];
        let (name, scale) = unit(rest, units)?;
        total += value * scale;
        text = rest[name.len()..].trim_start();
    }
    Ok(total)
}
fn unit(text: &str, units: &DurationUnits) -> Result<(&'static str, f64), String> {
    [
        ("min", units.min),
        ("h", units.h),
        ("d", units.d),
        ("w", units.w),
        ("m", units.m),
        ("y", units.y),
    ]
    .into_iter()
    .find(|(name, _)| text.starts_with(name))
    .ok_or_else(|| "Unknown Org duration unit; use min/h/d/w/m/y".into())
}
