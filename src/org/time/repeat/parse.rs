use super::*;

pub(super) fn suffixes(text: &str) -> Result<Suffixes, String> {
    let mut result = Suffixes::default();
    for (index, word) in text.split_whitespace().enumerate() {
        if index >= 2 {
            return Err("Org timestamp admits one repeater and one warning delay".into());
        }
        if word.starts_with(['+', '.']) {
            if result.repeater.is_some() {
                return Err("Duplicate Org timestamp suffix".into());
            }
            result.repeater = Some(repeater(word)?);
        } else {
            if result.warning.is_some() {
                return Err("Duplicate Org timestamp suffix".into());
            }
            result.warning = Some(warning(word)?);
        }
    }
    Ok(result)
}
fn repeater(text: &str) -> Result<Repeater, String> {
    let (mode, rest) = if let Some(rest) = text.strip_prefix("++") {
        (RepeatMode::CatchUp, rest)
    } else if let Some(rest) = text.strip_prefix(".+") {
        (RepeatMode::Restart, rest)
    } else {
        (
            RepeatMode::Once,
            text.strip_prefix('+').ok_or("Invalid Org repeater")?,
        )
    };
    let (first, maximum) = rest
        .split_once('/')
        .map_or((rest, None), |(a, b)| (a, Some(b)));
    Ok(Repeater {
        mode,
        interval: interval(first)?,
        maximum: maximum.map(interval).transpose()?,
    })
}
fn warning(text: &str) -> Result<WarningDelay, String> {
    let (first_only, rest) = if let Some(rest) = text.strip_prefix("--") {
        (true, rest)
    } else {
        (
            false,
            text.strip_prefix('-')
                .ok_or("Unsupported Org timestamp suffix; source is retained")?,
        )
    };
    Ok(WarningDelay {
        interval: interval(rest)?,
        first_only,
    })
}
fn interval(text: &str) -> Result<RepeatInterval, String> {
    let unit = match text.as_bytes().last() {
        Some(b'h') => RepeatUnit::Hour,
        Some(b'd') => RepeatUnit::Day,
        Some(b'w') => RepeatUnit::Week,
        Some(b'm') => RepeatUnit::Month,
        Some(b'y') => RepeatUnit::Year,
        _ => return Err("Org interval requires a positive count and h/d/w/m/y".into()),
    };
    let digits = &text[..text.len() - 1];
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err("Org interval requires ASCII digits".into());
    }
    RepeatInterval::new(
        digits.parse().map_err(|_| "Org interval count overflows")?,
        unit,
    )
}
