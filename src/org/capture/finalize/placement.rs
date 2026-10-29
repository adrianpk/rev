use super::*;
use crate::config::CaptureKind;
use crate::org::capture::Container;
mod list;

pub(super) fn insert(
    resolved: &Resolved,
    draft: &Rope,
    newline: &str,
    cancel: &Cancellation,
) -> Result<Vec<CaptureEdit>, String> {
    if let Container::List { .. } = &resolved.target.container {
        return list::insert(resolved, draft, newline, cancel);
    }
    let target = &resolved.target;
    let missing = !target.missing.is_empty();
    let at = if missing {
        target.missing_at.unwrap_or(target.subtree_end).0
    } else {
        position(resolved, cancel)?
    };
    let mut text = String::new();
    for heading in &target.missing {
        text.push_str(&"*".repeat(heading.level));
        text.push(' ');
        text.push_str(&heading.title);
        text.push_str(newline);
    }
    let payload = payload(resolved, draft, newline, cancel)?;
    text.push_str(&payload);
    separators(resolved, at, newline, &mut text);
    if text.len() > EXPANSION_BYTES * 2 {
        return Err("Capture insertion exceeds its bounded draft and target headings".into());
    }
    Ok(vec![CaptureEdit {
        bytes: at..at,
        text,
    }])
}
fn payload(
    resolved: &Resolved,
    draft: &Rope,
    newline: &str,
    cancel: &Cancellation,
) -> Result<String, String> {
    match resolved.template.kind {
        CaptureKind::Entry => entry::text(draft, resolved.target.level + 1, newline, cancel),
        CaptureKind::Item | CaptureKind::Checkitem => {
            item(draft, &resolved.target.container, newline)
        }
        CaptureKind::TableLine => table(draft, newline),
        CaptureKind::Plain => Ok(draft.to_string()),
    }
}
fn separators(resolved: &Resolved, at: usize, newline: &str, text: &mut String) {
    if resolved.template.kind == CaptureKind::Plain && resolved.target.missing.is_empty() {
        if at < resolved.source.text().len_bytes() && !text.is_empty() && !text.ends_with('\n') {
            text.push_str(newline)
        }
        return;
    }
    let source = resolved.source.text();
    let prefix = source.byte_slice(..at);
    let before = prefix.get_char(prefix.len_chars().saturating_sub(1));
    if before.is_some_and(|c| c != '\n') {
        text.insert_str(0, newline)
    }
    if !text.ends_with('\n') && !text.is_empty() {
        text.push_str(newline)
    }
}
fn position(resolved: &Resolved, cancel: &Cancellation) -> Result<usize, String> {
    let target = &resolved.target;
    let prepend = resolved.template.prepend;
    let at = match &target.container {
        Container::Table { bytes } => table_position(resolved, bytes.clone()),
        _ if resolved.template.kind == CaptureKind::Entry => {
            if prepend && target.parent.is_some() {
                target.body.end.0
            } else if prepend {
                entry::first(&resolved.source.snapshot(), cancel)?
            } else {
                target.subtree_end.0
            }
        }
        _ => {
            if prepend {
                entry::body_start(resolved, cancel)?
            } else {
                target.body.end.0
            }
        }
    };
    Ok(at)
}
fn table_position(resolved: &Resolved, bytes: std::ops::Range<crate::coordinates::Byte>) -> usize {
    let source = resolved.source.snapshot();
    let value = source.byte_slice(bytes.start.0..bytes.end.0).to_string();
    if resolved.template.prepend {
        let prefix = value
            .split_inclusive('\n')
            .take_while(|line| !line.trim_start().starts_with('|'))
            .map(str::len)
            .sum::<usize>();
        bytes.start.0 + prefix
    } else {
        bytes.start.0 + content_end(&value)
    }
}
fn item(draft: &Rope, _container: &Container, newline: &str) -> Result<String, String> {
    let value = draft.to_string();
    let first = value.lines().next().unwrap_or("");
    if !is_item(first) {
        return Err("Capture list draft must begin with a native list item".into());
    }
    let value = value.replace("\r\n", "\n").replace('\n', newline);
    Ok(value)
}
fn is_item(line: &str) -> bool {
    let line = line.trim_start();
    if ["- ", "+ ", "* "].iter().any(|p| line.starts_with(p)) {
        return true;
    }
    let digits = line.bytes().take_while(u8::is_ascii_digit).count();
    digits > 0
        && line
            .get(digits..)
            .is_some_and(|tail| tail.starts_with(". ") || tail.starts_with(") "))
}
fn table(draft: &Rope, newline: &str) -> Result<String, String> {
    let value = draft.to_string();
    if value
        .lines()
        .any(|line| !line.trim().starts_with('|') || !line.trim().ends_with('|'))
        || value.is_empty()
    {
        return Err("Capture table draft requires complete pipe-delimited rows".into());
    }
    Ok(value.replace("\r\n", "\n").replace('\n', newline))
}

fn content_end(text: &str) -> usize {
    let mut offset = 0;
    let mut end = 0;
    for line in text.split_inclusive('\n') {
        offset += line.len();
        if !line.trim().is_empty() {
            end = offset
        }
    }
    end
}
