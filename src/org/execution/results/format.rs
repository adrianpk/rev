use super::*;
pub(super) fn render(policy: &Policy, out: &Outcome) -> Result<String, String> {
    if out.value.len() > crate::org::Limits::default().bytes {
        return Err("Result data exceed native source byte limit".into());
    }
    let raw =
        std::str::from_utf8(&out.value).map_err(|_| "Result adapter returned invalid UTF-8")?;
    if raw
        .chars()
        .any(|c| c.is_control() && !['\n', '\r', '\t'].contains(&c))
    {
        return Err("Result adapter returned unsupported control bytes".into());
    }
    let raw = raw.replace("\r\n", "\n");
    if raw.contains('\r') {
        return Err("Result adapter returned bare carriage returns".into());
    }
    let raw = raw.trim_end_matches('\n');
    let body = if policy.table || out.result == config::SourceResult::Table {
        table(raw)?
    } else {
        raw.split('\n')
            .map(|l| {
                if l.is_empty() {
                    ":\n".into()
                } else {
                    format!(": {l}\n")
                }
            })
            .collect()
    };
    wrap(policy, raw, &body)
}
fn table(raw: &str) -> Result<String, String> {
    let mut text = String::new();
    let mut width = None;
    for row in raw.split('\n') {
        if row.contains('|') {
            return Err("Table adapter requires tab-delimited cells without pipes".into());
        }
        let cells: Vec<_> = row.split('\t').collect();
        if cells.len() > 256 || width.is_some_and(|n| n != cells.len()) {
            return Err("Table adapter returned ragged or oversized rows".into());
        }
        width = Some(cells.len());
        text.push_str(&format!("| {} |\n", cells.join(" | ")));
    }
    if text.lines().count() > 10000 {
        return Err("Table result exceeds 10000 rows".into());
    }
    Ok(text)
}
fn wrap(policy: &Policy, raw: &str, body: &str) -> Result<String, String> {
    let (open, close) = match policy.format.as_str() {
        "drawer" => (":RESULTS:", ":END:"),
        "html" => ("#+BEGIN_EXPORT html", "#+END_EXPORT"),
        "latex" => ("#+BEGIN_EXPORT latex", "#+END_EXPORT"),
        "org" => ("#+BEGIN_SRC org", "#+END_SRC"),
        "code" => ("#+BEGIN_EXAMPLE", "#+END_EXAMPLE"),
        _ => return Ok(body.into()),
    };
    if raw.lines().any(|l| l.trim().eq_ignore_ascii_case(close)) {
        return Err("Result contains its wrapping delimiter; prior results retained".into());
    }
    Ok(format!("{open}\n{raw}\n{close}\n"))
}
