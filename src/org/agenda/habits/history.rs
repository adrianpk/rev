use crate::org::{task::Workflow, time::Timestamp};

pub(crate) fn inactive(text: &str) -> Result<Timestamp, String> {
    let value = Timestamp::parse(text)?;
    if value.active()
        || value.end_time().is_some()
        || value.repeater()?.is_some()
        || value.warning_delay()?.is_some()
    {
        return Err("Org habit completion requires one inactive timestamp".into());
    }
    Ok(value)
}
/// None means this source line makes no completion assertion. An invalid actual
/// DONE record is unresolved, never silently converted into absence or success.
pub(crate) fn completion(line: &str, workflow: &Workflow) -> Result<Option<Timestamp>, String> {
    let Some(body) = line.trim_start().strip_prefix('-') else {
        return Ok(None);
    };
    if !body.starts_with([' ', '\t']) {
        return Ok(None);
    }
    let body = body.trim_start();
    let tail = if let Some(state) = body.strip_prefix("State ") {
        let Some((word, tail)) = quoted(state) else {
            return Ok(None);
        };
        if !workflow.state(word).is_some_and(|s| s.done) {
            return Ok(None);
        }
        from_state(tail)?
    } else if let Some(tail) = body.strip_prefix("CLOSING NOTE ") {
        tail.trim_start()
    } else {
        return Ok(None);
    };
    timestamp(tail).map(Some)
}
fn quoted(text: &str) -> Option<(&str, &str)> {
    text.trim_start().strip_prefix('"')?.split_once('"')
}
fn from_state(tail: &str) -> Result<&str, String> {
    let tail = tail.trim_start();
    if let Some(from) = tail.strip_prefix("from ") {
        quoted(from)
            .map(|(_, tail)| tail.trim_start())
            .ok_or("Malformed Org habit state origin".into())
    } else {
        Ok(tail)
    }
}
fn timestamp(tail: &str) -> Result<Timestamp, String> {
    if !tail.starts_with('[') {
        return Err("Missing Org habit completion timestamp".into());
    }
    let end = tail
        .find(']')
        .ok_or("Unclosed Org habit completion timestamp")?;
    let rest = tail[end + 1..].trim();
    if !rest.is_empty() && rest != "\\\\" {
        return Err("Unexpected Org habit history suffix".into());
    }
    inactive(&tail[..=end])
}
