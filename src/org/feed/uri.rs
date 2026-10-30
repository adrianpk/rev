//! Literal HTTP-relative link resolution; no link is fetched during parsing.
pub(super) fn resolve(base: Option<&str>, value: &str) -> Result<String, String> {
    if value.len() > 4096 || value.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err("Invalid feed link/base URI".into());
    }
    if value
        .split(['/', '?', '#'])
        .next()
        .is_some_and(|s| s.contains(':'))
    {
        return Ok(value.into());
    }
    let Some(base) = base else {
        return Ok(value.into());
    };
    relative(base, value)
}
fn relative(base: &str, value: &str) -> Result<String, String> {
    let Some((scheme, tail)) = base.split_once("://") else {
        return Ok(value.into());
    };
    let end = tail.find(['/', '?', '#']).unwrap_or(tail.len());
    let authority = &tail[..end];
    if value.starts_with("//") {
        return Ok(format!("{scheme}:{value}"));
    }
    let root = format!("{scheme}://{authority}");
    let old = tail[end..].split('#').next().unwrap_or("");
    if value.is_empty() {
        return Ok(format!("{root}{old}"));
    }
    if value.starts_with('#') {
        return Ok(format!("{root}{old}{value}"));
    }
    if value.starts_with('?') {
        return Ok(format!(
            "{root}{}{value}",
            old.split('?').next().unwrap_or("/")
        ));
    }
    let joined = if value.starts_with('/') {
        value.into()
    } else {
        join_path(old, value)
    };
    let (path, suffix) = split_suffix(&joined);
    let path = normalize(path);
    Ok(format!("{root}{path}{suffix}"))
}
fn normalize(path: &str) -> String {
    let mut parts = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => (),
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    let trailing = path.ends_with('/') || path.ends_with("/.") || path.ends_with("/..");
    let end = if trailing && !parts.is_empty() {
        "/"
    } else {
        ""
    };
    format!("/{}{end}", parts.join("/"))
}

fn join_path(old: &str, value: &str) -> String {
    let old = old.split('?').next().unwrap_or("/");
    let directory = old.rsplit_once('/').map_or("", |(p, _)| p);
    format!("{directory}/{value}")
}
fn split_suffix(value: &str) -> (&str, &str) {
    let end = value.find(['?', '#']).unwrap_or(value.len());
    (&value[..end], &value[end..])
}
