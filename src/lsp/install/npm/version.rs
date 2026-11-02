pub(super) fn resolved(bytes: &[u8], major: Option<u64>) -> Result<String, String> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| "invalid npm version response")?;
    let values = match &value {
        serde_json::Value::String(_) => std::slice::from_ref(&value),
        serde_json::Value::Array(values) if values.len() <= 4096 => values,
        _ => return Err("npm version response requires a bounded exact version list".into()),
    };
    values
        .iter()
        .filter_map(serde_json::Value::as_str)
        .filter_map(|v| stable(v).map(|key| (key, v)))
        .filter(|(key, _)| major.is_none_or(|major| key[0] == major))
        .max_by_key(|(key, _)| *key)
        .map(|(_, version)| version.into())
        .ok_or_else(|| "npm resolver lacks an exact stable version in the required major".into())
}

fn stable(version: &str) -> Option<[u64; 3]> {
    if version.len() > 128 {
        return None;
    }
    let mut parts = version.split('.');
    let numbers = [
        number(parts.next()?)?,
        number(parts.next()?)?,
        number(parts.next()?)?,
    ];
    parts.next().is_none().then_some(numbers)
}

fn number(text: &str) -> Option<u64> {
    if text.is_empty()
        || !text.bytes().all(|b| b.is_ascii_digit())
        || (text.len() > 1 && text.starts_with('0'))
    {
        return None;
    }
    text.parse().ok()
}

pub(super) fn exact(value: &str) -> bool {
    value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b".-+".contains(&b))
        && value.split(['-', '+']).next().and_then(stable).is_some()
}
