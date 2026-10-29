use std::path::{Component, Path, PathBuf};
pub(crate) fn local(base: &Path, value: &str) -> Result<PathBuf, String> {
    crate::config::capture::path(value)?;
    if value.is_empty()
        || value.starts_with('~')
        || value.starts_with("//")
        || value.starts_with("/ssh:")
    {
        return Err("Attachment location requires a literal local path".into());
    }
    let path = Path::new(value);
    let path = if path.is_absolute() {
        path.to_owned()
    } else {
        base.join(path)
    };
    crate::config::capture::path(path.to_str().ok_or("Attachment location is not UTF-8")?)?;
    normalize(&path)
}
fn normalize(path: &Path) -> Result<PathBuf, String> {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                if !out.pop() {
                    return Err("Attachment path escapes filesystem root".into());
                }
            }
            Component::CurDir => (),
            _ => out.push(component.as_os_str()),
        }
    }
    Ok(out)
}
pub(crate) fn location(
    dir: Option<&str>,
    id: Option<&str>,
    base: &Path,
    policy: &crate::config::Org,
) -> Result<PathBuf, String> {
    if let Some(dir) = dir {
        return local(base, dir);
    }
    let id = id.ok_or("Attachment location requires a DIR or ID property")?;
    if id.len() < 3
        || id.len() > 128
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
    {
        return Err("Attachment ID cannot form a safe directory".into());
    }
    Ok(local(base, &policy.attachments.directory)?
        .join(&id[..2])
        .join(&id[2..]))
}
pub(super) fn name(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 255
        || value.chars().any(char::is_control)
        || Path::new(value).components().count() != 1
        || !matches!(
            Path::new(value).components().next(),
            Some(Component::Normal(_))
        )
    {
        return Err("Select one attachment filename without traversal or controls".into());
    }
    Ok(())
}
