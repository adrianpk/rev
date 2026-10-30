use super::*;
pub(super) fn admit(definition: &Definition) -> Result<(), String> {
    evaluation(definition.headers.get("eval"))?;
    behavior(definition)
}
fn behavior(definition: &Definition) -> Result<(), String> {
    if definition
        .headers
        .get("session")
        .is_some_and(|s| s != "none")
    {
        return Err("Native source execution requires :session none".into());
    }
    for key in [
        "dir",
        "prologue",
        "epilogue",
        "post",
        "wrap",
        "file",
        "file-mode",
        "output-dir",
    ] {
        if definition.headers.get(key).is_some() {
            return Err(format!(
                "Source :{key} requires a supported language/header adapter"
            ));
        }
    }
    if definition.headers.get("cache").is_some_and(|s| s != "no") {
        return Err("Native execution does not use cached results".into());
    }
    Ok(())
}
pub(super) fn expand(definition: &Definition) -> Result<bool, String> {
    if definition.headers.get("no-expand").is_some() {
        return Ok(false);
    }
    match definition.headers.get("noweb").unwrap_or("no") {
        "yes" | "eval" | "no-export" | "strip-export" | "strip-tangle" => Ok(true),
        "no" | "tangle" => Ok(false),
        _ => Err("Unsupported noweb header value".into()),
    }
}

pub(super) fn evaluation(value: Option<&str>) -> Result<bool, String> {
    let value = value.unwrap_or("yes");
    if ["no", "never"].contains(&value) {
        return Err("Source :eval no/never forbids execution".into());
    }
    if !["yes", "query", "no-export", "never-export", "query-export"].contains(&value) {
        return Err("Unsupported source :eval value".into());
    }
    Ok(value == "query")
}
