use super::*;
use std::path::Component;
pub(super) fn check(cancel: &Cancellation) -> Result<(), String> {
    if cancel.is_cancelled() {
        Err("Tangle cancelled or expired".into())
    } else {
        Ok(())
    }
}
pub(super) fn literal(value: &str) -> Result<String, String> {
    let value = value.trim();
    if !value.starts_with('"') {
        if value.starts_with(['(', '\'', '`']) {
            return Err("Executable header forms are unsupported for native tangling".into());
        }
        return Ok(value.into());
    }
    let value = value
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .ok_or("Unclosed literal tangle header")?;
    let mut result = String::new();
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        result.push(if c == '\\' {
            escape(chars.next().ok_or("Incomplete header escape")?)?
        } else {
            c
        });
    }
    Ok(result)
}
pub(super) fn yes(block: &Definition, key: &str, default: bool) -> Result<bool, String> {
    match block.headers.get(key) {
        None => Ok(default),
        Some("yes") => Ok(true),
        Some("no") => Ok(false),
        _ => Err(format!("Tangle {key} requires yes or no")),
    }
}
pub(super) fn expands(block: &Definition) -> Result<bool, String> {
    if block.headers.get("no-expand").is_some() {
        return Ok(false);
    }
    match block.headers.get("noweb").unwrap_or("no") {
        "yes" | "tangle" | "no-export" | "strip-export" => Ok(true),
        "no" | "eval" | "strip-tangle" => Ok(false),
        _ => Err("Unsupported noweb header value".into()),
    }
}
pub(super) fn supported(block: &Definition) -> Result<(), String> {
    for key in ["comments", "shebang", "tangle-mode"] {
        if block
            .headers
            .get(key)
            .is_some_and(|s| !s.is_empty() && s != "no")
        {
            return Err(format!(
                "Native tangling does not interpret {key}; no files written"
            ));
        }
    }
    if block.headers.get("no-expand").is_none() && block.headers.iter().any(|h| h.key == "var") {
        return Err("Variable expansion requires a language adapter; no files written".into());
    }
    Ok(())
}
pub(super) fn destination(
    block: &Definition,
    source: Option<&Path>,
) -> Result<Option<PathBuf>, String> {
    let declared = literal(block.headers.get("tangle").unwrap_or("no"))?;
    let Some(path) = declared_path(&declared, source, &block.language)? else {
        return Ok(None);
    };
    admit_path(&path)?;
    Ok(Some(path.components().collect()))
}
fn declared_path(
    declared: &str,
    source: Option<&Path>,
    language: &str,
) -> Result<Option<PathBuf>, String> {
    Ok(Some(match declared {
        "no" => return Ok(None),
        "yes" => default_path(source, language)?,
        "" => return Err("Empty tangle destination".into()),
        _ => {
            let path = PathBuf::from(declared);
            if path.is_absolute() {
                path
            } else {
                source
                    .and_then(Path::parent)
                    .ok_or("Relative tangle target requires an associated Org file")?
                    .join(path)
            }
        }
    }))
}
fn admit_path(path: &Path) -> Result<(), String> {
    if path.to_string_lossy().len() > 4096
        || path.to_string_lossy().chars().any(char::is_control)
        || path.components().any(|c| matches!(c, Component::ParentDir))
    {
        return Err("Unsafe tangle destination".into());
    }
    if path.file_name().is_none() {
        return Err("Tangle destination requires a file name".into());
    }
    Ok(())
}
fn default_path(source: Option<&Path>, language: &str) -> Result<PathBuf, String> {
    let source = source.ok_or("Tangle yes requires an associated Org file")?;
    if language.is_empty() {
        return Err("Tangle yes requires a source language".into());
    }
    let aliases = [
        ("emacs-lisp", "el"),
        ("elisp", "el"),
        ("python", "py"),
        ("rust", "rs"),
        ("javascript", "js"),
        ("C", "c"),
        ("C++", "cpp"),
        ("bash", "sh"),
        ("shell", "sh"),
    ];
    let extension = aliases
        .into_iter()
        .find_map(|(name, ext)| (name == language).then_some(ext))
        .unwrap_or(language);
    Ok(source.with_extension(extension))
}

fn escape(c: char) -> Result<char, String> {
    match c {
        'n' => Ok('\n'),
        'r' => Ok('\r'),
        't' => Ok('\t'),
        '\\' => Ok('\\'),
        '"' => Ok('"'),
        _ => Err("Unsupported literal header escape".into()),
    }
}
