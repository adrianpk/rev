use super::*;
use crate::org::{inline::Raw, links::Target};
impl Document<'_> {
    pub(super) fn link(&self, html: &mut Html<'_>, n: &Raw, depth: usize) -> Result<(), String> {
        let value = self.text.byte_slice(n.content.clone()).to_string();
        let target = Target::parse(&value).and_then(|target| self.destination(target));
        let Ok(target) = target else {
            html.diagnostic("Unsupported HTML link retained as literal source")?;
            return html.text(&self.text.byte_slice(n.source.clone()).to_string());
        };
        if image(n, &value) {
            return image_output(html, &value, &target, &Properties::default());
        }
        if let Some(anchor) = target.strip_prefix('#') {
            html.references.insert(anchor.into());
        }
        html.emit("<a href=\"")?;
        html.text(&target)?;
        html.emit("\">")?;
        if let Some(label) = &n.detail {
            self.objects(html, label.clone(), depth, InlineContext::Minimal)?;
        } else {
            html.text(&value)?;
        }
        html.emit("</a>")
    }
}
impl Document<'_> {
    fn destination(&self, target: Target) -> Result<String, String> {
        match target {
            Target::External(value) => Ok(value),
            Target::Internal(value) => self.internal(&value),
            Target::File { path, search, .. } => {
                if !search.is_empty() {
                    return Err("HTML file search suffix is unsupported".into());
                }
                relative(path)
            }
            _ => Err("HTML link requires a local path, target or admitted external URL".into()),
        }
    }
    fn internal(&self, value: &str) -> Result<String, String> {
        if value.contains(':') {
            return Err("Unsupported HTML link scheme".into());
        }
        if let Some(heading) = value.strip_prefix('*') {
            return self
                .heading_links
                .get(heading.trim_start())
                .and_then(|id| id.as_ref())
                .map(|id| format!("#{id}"))
                .ok_or("Missing or ambiguous HTML heading link".into());
        }
        if let Some(id) = value.strip_prefix('#') {
            return Ok(format!("#{id}"));
        }
        Ok(format!("#org-target-{}", encoded(value)))
    }
    pub(super) fn standalone_image(
        &self,
        html: &mut Html<'_>,
        n: &structure::Raw,
        props: &Properties,
    ) -> Result<bool, String> {
        let objects = crate::org::inline::objects(
            self.text,
            n.content.clone(),
            InlineContext::Standard,
            html.cancel,
        )?;
        let Some(first) = objects
            .first()
            .filter(|n| matches!(n.kind, InlineKind::Link(_)))
        else {
            return Ok(false);
        };
        let value = self.text.byte_slice(first.content.clone()).to_string();
        let trailing = self
            .text
            .byte_slice(first.source.end..n.content.end)
            .to_string();
        if first.source.start != n.content.start
            || !trailing.trim().is_empty()
            || !image(first, &value)
        {
            return Ok(false);
        }
        let target = Target::parse(&value).and_then(|target| self.destination(target));
        let Ok(target) = target else {
            return Ok(false);
        };
        image_output(html, &value, &target, props)?;
        Ok(true)
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

fn image_output(
    html: &mut Html<'_>,
    value: &str,
    target: &str,
    props: &Properties,
) -> Result<(), String> {
    props.admit("img")?;
    props.anchor(html)?;
    let mut attributes = props.values.clone();
    attributes
        .entry("alt".into())
        .or_insert_with(|| value.into());
    attributes.insert("src".into(), target.into());
    attributes.insert("loading".into(), "lazy".into());
    html.open("img", &attributes)
}
