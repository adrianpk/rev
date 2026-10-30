//! Literal argument data is never interpolated as program source accidentally.
use super::*;
use std::collections::BTreeMap;
pub(super) fn values(definition: &Definition) -> Result<BTreeMap<String, String>, String> {
    let mut values = BTreeMap::new();
    for header in definition.headers.iter().filter(|h| h.key == "var") {
        for assignment in split(&header.value, ' ')? {
            assignment_value(&mut values, &assignment)?;
        }
    }
    Ok(values)
}
fn assignment_value(values: &mut BTreeMap<String, String>, assignment: &str) -> Result<(), String> {
    let (name, value) = assignment
        .split_once('=')
        .ok_or("Source variable requires name=literal")?;
    if name.is_empty()
        || name.len() > 128
        || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        || name.as_bytes()[0].is_ascii_digit()
    {
        return Err("Source variable requires an ASCII identifier".into());
    }
    values.insert(name.into(), literal(value.trim())?);
    Ok(())
}
fn literal(value: &str) -> Result<String, String> {
    if value.starts_with('"') {
        let document: toml::Table = toml::from_str(&format!("value={value}"))
            .map_err(|_| "Source argument requires a quoted string or finite number")?;
        let value = document
            .get("value")
            .and_then(toml::Value::as_str)
            .ok_or("Source argument is not a string")?;
        if value.contains('\0') {
            return Err("Source variable contains NUL".into());
        }
        return Ok(value.into());
    }
    value
        .parse::<f64>()
        .ok()
        .filter(|n| n.is_finite())
        .map(|_| value.into())
        .ok_or("Source argument requires a quoted string or finite number".into())
}
pub(in crate::org::execution) fn split(
    value: &str,
    separator: char,
) -> Result<Vec<String>, String> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut lex = LiteralLex::default();
    for (i, c) in value.char_indices() {
        if lex.separator(c, separator) {
            push(&mut parts, &value[start..i])?;
            start = i + 1;
        }
    }
    if lex.quote || lex.escape {
        return Err("Unclosed source argument literal".into());
    }
    push(&mut parts, &value[start..])?;
    Ok(parts)
}
fn push(parts: &mut Vec<String>, value: &str) -> Result<(), String> {
    if parts.len() >= 256 {
        return Err("Source arguments exceed 256 values".into());
    }
    if !value.trim().is_empty() {
        parts.push(value.trim().into());
    }
    Ok(())
}
pub(super) fn body(
    definition: &Definition,
    body: &str,
    profile: &SourceProgram,
) -> Result<String, String> {
    let values = values(definition)?;
    if values.is_empty() {
        return Ok(body.into());
    }
    if matches!(
        definition.language.as_str(),
        "sh" | "shell" | "bash" | "dash" | "zsh"
    ) {
        let prefix: String = values
            .iter()
            .map(|(k, v)| format!("{k}='{}'\n", v.replace('\'', "'\\''")))
            .collect();
        return Ok(prefix + body);
    }
    for name in values.keys() {
        let token = format!("{{var:{name}}}");
        if !profile.argv.contains(&token) && !profile.steps.iter().any(|s| s.argv.contains(&token))
        {
            return Err(format!(
                "Declared adapter needs an explicit {token} argument"
            ));
        }
    }
    Ok(body.into())
}
pub(super) fn token(definition: &Definition, value: &str) -> Result<Option<String>, String> {
    let Some(name) = value
        .strip_prefix("{var:")
        .and_then(|s| s.strip_suffix('}'))
    else {
        return Ok(None);
    };
    values(definition)?
        .remove(name)
        .map(Some)
        .ok_or_else(|| format!("Missing source variable {name}"))
}

#[derive(Default)]
struct LiteralLex {
    quote: bool,
    escape: bool,
}
impl LiteralLex {
    fn separator(&mut self, c: char, separator: char) -> bool {
        if self.escape {
            self.escape = false;
            return false;
        }
        if c == '\\' {
            self.escape = true;
            return false;
        }
        if c == '"' {
            self.quote = !self.quote;
        }
        c == separator && !self.quote
    }
}
