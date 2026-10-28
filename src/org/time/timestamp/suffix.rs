// Repeat/warning source is validated and retained here, without performing the
// completion/rescheduling behavior owned by the next delivery layer.
pub(super) fn check(text: &str) -> Result<(), String> {
    let words: Vec<_> = text.split_whitespace().collect();
    if words.len() > 2 {
        return Err("Org timestamp admits one repeater and one warning delay".into());
    }
    let mut repeater = false;
    let mut warning = false;
    for word in words {
        let (is_repeat, value) = prefix(word)?;
        let present = if is_repeat {
            &mut repeater
        } else {
            &mut warning
        };
        if *present {
            return Err("Duplicate Org timestamp suffix".into());
        }
        *present = true;
        interval(value)?;
    }
    Ok(())
}
fn prefix(text: &str) -> Result<(bool, &str), String> {
    for prefix in ["++", ".+", "+", "--", "-"] {
        if let Some(tail) = text.strip_prefix(prefix) {
            return Ok((prefix.ends_with('+'), tail));
        }
    }
    Err("Unsupported Org timestamp suffix; source is retained".into())
}
fn interval(text: &str) -> Result<(), String> {
    let (first, second) = text
        .split_once('/')
        .map_or((text, None), |(a, b)| (a, Some(b)));
    component(first)?;
    if let Some(second) = second {
        component(second)?;
    }
    Ok(())
}
fn component(text: &str) -> Result<(), String> {
    let at = text.len().checked_sub(1).ok_or("Empty Org interval")?;
    let unit = text.as_bytes()[at];
    if !b"hdwmy".contains(&unit) || !text[..at].bytes().all(|b| b.is_ascii_digit()) {
        return Err("Org interval requires a positive count and h/d/w/m/y".into());
    }
    let count: u32 = text[..at]
        .parse()
        .map_err(|_| "Org interval count overflows")?;
    if count == 0 {
        return Err("Org interval count must be positive".into());
    }
    Ok(())
}
