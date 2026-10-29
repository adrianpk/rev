use super::*;
pub(super) fn draft(value: &str, cancel: &Cancellation) -> Result<(String, String), String> {
    let mut indent: Option<String> = None;
    for line in value.split('\n') {
        active(cancel)?;
        let line = line.trim_end_matches('\r');
        if line.trim().is_empty() {
            continue;
        }
        let spaces = &line[..line.len() - line.trim_start_matches([' ', '\t']).len()];
        indent = Some(match indent {
            None => spaces.into(),
            Some(old) => old
                .chars()
                .zip(spaces.chars())
                .take_while(|(a, b)| a == b)
                .map(|(a, _)| a)
                .collect(),
        });
    }
    let indent = indent.unwrap_or_default();
    let text = value
        .split_inclusive('\n')
        .map(|line| {
            active(cancel)?;
            let body = line.trim_end_matches(['\r', '\n']);
            let body = body.strip_prefix(&indent).unwrap_or(body);
            let mut body = unescape(body);
            if line.ends_with('\n') {
                body.push('\n');
            }
            Ok(body)
        })
        .collect::<Result<String, String>>()?;
    Ok((text, indent))
}
pub(super) fn body(value: &str, indent: &str, newline: &str) -> Result<String, String> {
    if value.len() > Limits::default().bytes {
        return Err("Source draft exceeds native source byte limit".into());
    }
    let mut result = String::new();
    for line in value.split_inclusive('\n') {
        let text = line.trim_end_matches(['\r', '\n']);
        if !text.is_empty() {
            result.push_str(indent);
            result.push_str(&escape(text));
        }
        result.push_str(newline);
        if result.len() > Limits::default().bytes {
            return Err("Reinserted source body exceeds native byte limit".into());
        }
    }
    Ok(result)
}
fn marker(value: &str) -> bool {
    let value = value.trim_start_matches(',');
    value.starts_with('*') || value.starts_with("#+")
}
fn unescape(value: &str) -> String {
    let start = value.len() - value.trim_start_matches([' ', '\t']).len();
    if value[start..].starts_with(',') && marker(&value[start..]) {
        format!("{}{}", &value[..start], &value[start + 1..])
    } else {
        value.into()
    }
}
fn escape(value: &str) -> String {
    let start = value.len() - value.trim_start_matches([' ', '\t']).len();
    if marker(&value[start..]) {
        format!("{},{}", &value[..start], &value[start..])
    } else {
        value.into()
    }
}
fn active(cancel: &Cancellation) -> Result<(), String> {
    if cancel.is_cancelled() {
        Err("Source draft observation cancelled".into())
    } else {
        Ok(())
    }
}
