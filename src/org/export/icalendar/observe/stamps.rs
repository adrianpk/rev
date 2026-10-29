use super::*;
pub(super) fn planning(entry: &mut Entry, text: &str) -> Result<(), String> {
    for (name, target) in [
        ("SCHEDULED:", &mut entry.scheduled),
        ("DEADLINE:", &mut entry.deadline),
    ] {
        let mut found = text.match_indices(name);
        if let Some((at, _)) = found.next() {
            if found.next().is_some() || target.is_some() {
                return Err("Duplicate calendar planning timestamp".into());
            }
            let (stamp, _) = prefix(text[at + name.len()..].trim_start())?;
            if stamp.start.active() {
                *target = Some(stamp);
            }
        }
    }
    Ok(())
}
pub(super) fn appointments(
    entry: &mut Entry,
    snapshot: &Snapshot,
    range: Range<usize>,
    cancel: &Cancellation,
) -> Result<(), String> {
    let nodes = crate::org::inline::objects(
        &snapshot.text,
        range.clone(),
        InlineContext::Standard,
        cancel,
    )?;
    let protected: Vec<_> = nodes
        .iter()
        .filter(|n| {
            matches!(
                n.kind,
                crate::org::InlineKind::Code
                    | crate::org::InlineKind::Verbatim
                    | crate::org::InlineKind::Link(_)
                    | crate::org::InlineKind::Target
                    | crate::org::InlineKind::RadioTarget
            )
        })
        .map(|n| n.source.clone())
        .collect();
    let text = snapshot.text.byte_slice(range.clone()).to_string();
    let mut at = 0;
    while let Some(offset) = text[at..].find('<') {
        document::check(cancel)?;
        let start = at + offset;
        if let Some(end) = protected
            .iter()
            .find(|r| r.contains(&(range.start + start)))
            .map(|r| r.end)
        {
            at = end - range.start;
            continue;
        }
        if text
            .as_bytes()
            .get(start + 1)
            .is_some_and(u8::is_ascii_digit)
        {
            let (stamp, length) = prefix(&text[start..])?;
            if entry.appointments.len() >= 4096 {
                return Err("Calendar heading timestamp limit reached".into());
            }
            entry.appointments.push(stamp);
            at = start + length;
        } else {
            at = start + 1;
        }
    }
    Ok(())
}
fn prefix(text: &str) -> Result<(TimestampRange, usize), String> {
    let close = match text.as_bytes().first() {
        Some(b'<') => '>',
        Some(b'[') => ']',
        _ => return Err("Expected calendar planning timestamp".into()),
    };
    let mut length = text.find(close).ok_or("Unclosed calendar timestamp")? + 1;
    if text[length..].starts_with("--") {
        let tail = &text[length + 2..];
        length += 2
            + tail
                .find(close)
                .ok_or("Unclosed calendar timestamp range")?
            + 1;
    }
    Ok((TimestampRange::parse(&text[..length])?, length))
}
