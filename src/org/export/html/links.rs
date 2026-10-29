use super::*;
use crate::org::{inline::Raw, links::Target};
impl Document<'_> {
    pub(super) fn link(&self, html: &mut Html<'_>, n: &Raw, depth: usize) -> Result<(), String> {
        let value = self.text.byte_slice(n.content.clone()).to_string();
        let target = Target::parse(&value).and_then(destination);
        let Ok(target) = target else {
            html.diagnostic("Unsupported HTML link retained as literal source")?;
            return html.text(&self.text.byte_slice(n.source.clone()).to_string());
        };
        if image(n, &value) {
            return image_output(html, &value, &target);
        }
        html.emit("<a href=\"")?;
        html.text(&target)?;
        html.emit("\">")?;
        if let Some(label) = &n.detail {
            self.inline(html, label.clone(), depth)?;
        } else {
            html.text(&value)?;
        }
        html.emit("</a>")
    }
}
fn destination(target: Target) -> Result<String, String> {
    match target {
        Target::External(value) => Ok(value),
        Target::Internal(value) => {
            if value.contains(':') {
                return Err("Unsupported HTML link scheme".into());
            }
            Ok(format!("#org-target-{}", encoded(&value)))
        }
        Target::File { path, search, .. } => {
            if !search.is_empty() {
                return Err("HTML file search suffix is unsupported".into());
            }
            relative(path)
        }
        _ => Err("HTML link requires a local path, target or admitted external URL".into()),
    }
}
fn relative(path: PathBuf) -> Result<String, String> {
    use std::path::Component;
    if !path
        .components()
        .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
    {
        return Err("HTML assets require relative paths without parent traversal".into());
    }
    let value = path.to_str().ok_or("HTML asset path is not UTF-8")?;
    if value.is_empty() || value.contains(':') {
        return Err("Invalid HTML asset path".into());
    }
    Ok(crate::org::links::encode(value))
}

fn image(n: &Raw, value: &str) -> bool {
    if n.detail.is_some() {
        return false;
    }
    let Ok(Target::File { path, .. }) = Target::parse(value) else {
        return false;
    };
    path.extension().and_then(|e| e.to_str()).is_some_and(|e| {
        ["png", "jpg", "jpeg", "gif", "webp", "avif"].contains(&e.to_ascii_lowercase().as_str())
    })
}

fn image_output(html: &mut Html<'_>, value: &str, target: &str) -> Result<(), String> {
    html.emit("<img src=\"")?;
    html.text(target)?;
    html.emit("\" alt=\"")?;
    html.text(value)?;
    html.emit("\" loading=\"lazy\">")
}
