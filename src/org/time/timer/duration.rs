/// Countdown input admits 0..365 days, at most 64 UTF-8 bytes. Zero is only an
/// unarmed configuration/default value; selecting a countdown requires >0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimerDuration {
    seconds: u64,
}
impl TimerDuration {
    pub fn parse(source: &str) -> Result<Self, String> {
        if source.len() > 64 {
            return Err("Timer duration exceeds 64 UTF-8 bytes".into());
        }
        let source = source.trim();
        if source.is_empty() {
            return Err("Timer duration requires 1..64 UTF-8 bytes".into());
        }
        let seconds = if source.contains(':') {
            Self::stamp_seconds(source)?
        } else {
            number(source)?
                .checked_mul(60)
                .ok_or("Timer duration overflows")?
        };
        if seconds > 31536000 {
            return Err("Countdown exceeds 365 days".into());
        }
        Ok(Self { seconds })
    }
    pub fn seconds(self) -> u64 {
        self.seconds
    }
    /// Recognize source labels without limiting elapsed hours to countdown bounds.
    pub fn stamp_seconds(source: &str) -> Result<u64, String> {
        if source.len() > 64 {
            return Err("Timer stamp exceeds 64 UTF-8 bytes".into());
        }
        let words: Vec<_> = source.split(':').take(4).collect();
        if words.len() != 3 {
            return Err("Timer stamp requires H:MM:SS".into());
        }
        let hours = number(words[0])?;
        let minutes = component(words[1])?;
        let seconds = component(words[2])?;
        hours
            .checked_mul(3600)
            .and_then(|v| v.checked_add(minutes * 60 + seconds))
            .ok_or("Timer stamp overflows".into())
    }
}
fn number(value: &str) -> Result<u64, String> {
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err("Timer duration requires unsigned minutes or H:MM:SS".into());
    }
    value.parse().map_err(|_| "Timer duration overflows".into())
}
fn component(value: &str) -> Result<u64, String> {
    let value = if value.len() == 2 {
        number(value)?
    } else {
        return Err("Timer minutes/seconds require two digits".into());
    };
    if value >= 60 {
        return Err("Timer minutes/seconds require 00..59".into());
    }
    Ok(value)
}
