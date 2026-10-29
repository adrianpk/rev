use std::path::{Path, PathBuf};
pub fn location(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 4096
        || value.chars().any(char::is_control)
        || value.contains("://")
    {
        return Err("Archive location requires a local file::heading of at most 4096 bytes".into());
    }
    let (file, heading) = value
        .split_once("::")
        .ok_or("Archive location requires file::heading")?;
    if heading.contains("::") || heading.trim_start().starts_with("datetree") {
        return Err("Unsupported archive heading syntax".into());
    }
    if file.replace("%s", "").contains('%') {
        return Err("Archive location supports only the %s source-file escape".into());
    }
    Ok(())
}
pub(super) fn expand(value: &str, source: &Path) -> Result<(PathBuf, String), String> {
    location(value)?;
    let (file, heading) = value.split_once("::").expect("validated archive location");
    let source_name = source.to_str().ok_or("Archive source path must be UTF-8")?;
    let file = if file.is_empty() {
        source_name.to_owned()
    } else {
        file.replace("%s", source_name)
    };
    crate::config::capture::path(&file)?;
    let path = PathBuf::from(file);
    let path = if path.is_relative() {
        source
            .parent()
            .ok_or("Archive origin has no directory")?
            .join(path)
    } else {
        path
    };
    if path.as_os_str().len() > 4096 {
        return Err("Expanded archive path exceeds 4096 bytes".into());
    }
    let heading = heading.trim().trim_start_matches('*').trim().to_owned();
    Ok((path, heading))
}
