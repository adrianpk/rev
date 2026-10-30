use super::*;
const XML: &str = "http://www.w3.org/XML/1998/namespace";
const XMLNS: &str = "http://www.w3.org/2000/xmlns/";
pub(super) fn bindings(
    previous: Option<&BTreeMap<String, String>>,
    attrs: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>, String> {
    let mut result = previous.cloned().unwrap_or_default();
    result.insert("xml".into(), XML.into());
    for (key, value) in attrs {
        let Some(prefix) = prefix(key) else {
            continue;
        };
        if value == XMLNS || prefix == "xmlns" || (prefix == "xml") != (value == XML) {
            return Err("Invalid reserved XML namespace binding".into());
        }
        result.insert(prefix.into(), value.clone());
    }
    if result.len() > 64 {
        return Err("XML namespace count exceeds 64".into());
    }
    Ok(result)
}
fn prefix(value: &str) -> Option<&str> {
    if value == "xmlns" {
        Some("")
    } else {
        value.strip_prefix("xmlns:")
    }
}
pub(super) fn resolve(
    raw: &str,
    bindings: &BTreeMap<String, String>,
    attr: bool,
) -> Result<Name, String> {
    let (prefix, local) = raw.split_once(':').unwrap_or(("", raw));
    if local.contains(':') || local.is_empty() {
        return Err("Invalid XML qualified name".into());
    }
    let space = if prefix.is_empty() && attr {
        String::new()
    } else {
        bindings.get(prefix).cloned().unwrap_or_default()
    };
    if !prefix.is_empty() && space.is_empty() {
        return Err("Undeclared XML namespace prefix".into());
    }
    Ok(Name {
        space,
        local: local.into(),
    })
}
pub(super) fn attributes(
    raw: &BTreeMap<String, String>,
    names: &BTreeMap<String, String>,
) -> Result<BTreeMap<Name, String>, String> {
    let mut result = BTreeMap::new();
    for (key, value) in raw {
        if prefix(key).is_some() {
            continue;
        }
        if result
            .insert(resolve(key, names, true)?, value.clone())
            .is_some()
        {
            return Err("Duplicate expanded XML attribute".into());
        }
    }
    Ok(result)
}
