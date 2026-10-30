use super::*;
use ropey::Rope;
pub(super) struct Previous {
    pub range: Range<usize>,
    pub prefix: String,
    pub body: String,
}
pub(super) fn find(text: &Rope, end: usize, name: Option<&str>) -> Result<Previous, String> {
    let suffix = text.byte_slice(end..).to_string();
    let gap = suffix.len() - suffix.trim_start_matches(['\r', '\n']).len();
    let tail = &suffix[gap..];
    let header_end = tail.find('\n').map_or(tail.len(), |n| n + 1);
    let header = tail[..header_end].trim();
    let Some(label) = result_label(header) else {
        let separator = if end == 0 || text.byte(end - 1) == b'\n' {
            "\n"
        } else {
            "\n\n"
        };
        return Ok(Previous {
            range: end..end,
            prefix: format!(
                "{separator}#+RESULTS:{}\n",
                name.map(|s| format!(" {s}")).unwrap_or_default()
            ),
            body: String::new(),
        });
    };
    if !label.is_empty() && Some(label) != name {
        return Err("Following RESULTS belongs to another named source".into());
    }
    let rest = &tail[header_end..];
    let used = body_end(rest)?;
    let body = rest[..used].replace("\r\n", "\n");
    Ok(Previous {
        range: end + gap..end + gap + header_end + used,
        prefix: format!(
            "#+RESULTS:{}\n",
            name.map(|s| format!(" {s}")).unwrap_or_default()
        ),
        body,
    })
}
fn result_label(header: &str) -> Option<&str> {
    if !header.get(..9)?.eq_ignore_ascii_case("#+RESULTS") {
        return None;
    }
    let after = &header[9..];
    let after = if after.starts_with('[') {
        &after[after.find(']')? + 1..]
    } else {
        after
    };
    after.strip_prefix(':').map(str::trim)
}
fn body_end(value: &str) -> Result<usize, String> {
    let first = value.lines().next().unwrap_or("").trim();
    if first.eq_ignore_ascii_case(":RESULTS:") {
        return delimited(value, ":END:");
    }
    if first.to_ascii_uppercase().starts_with("#+BEGIN_") {
        let name = first[8..]
            .split_whitespace()
            .next()
            .ok_or("Malformed previous result block")?;
        return delimited(value, &format!("#+END_{name}"));
    }
    let mut used = 0;
    for line in value.split_inclusive('\n') {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if !trimmed.starts_with([':', '|']) {
            return Err("Previous result boundary is ambiguous; source unchanged".into());
        }
        used += line.len();
    }
    Ok(used)
}
fn delimited(value: &str, closing: &str) -> Result<usize, String> {
    let mut used = 0;
    for line in value.split_inclusive('\n') {
        used += line.len();
        if line.trim().eq_ignore_ascii_case(closing) {
            return Ok(used);
        }
    }
    Err("Unclosed previous result; source unchanged".into())
}
