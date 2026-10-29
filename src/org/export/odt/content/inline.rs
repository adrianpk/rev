use super::*;
use crate::org::{inline as native, links::Target};
impl Document<'_> {
    pub(super) fn inline(
        &self,
        odt: &mut Odt<'_>,
        range: Range<usize>,
        depth: usize,
        context: InlineContext,
    ) -> Result<(), String> {
        if depth >= 32 {
            return Err("ODT inline nesting limit reached".into());
        }
        let nodes = native::objects(self.text, range.clone(), context, odt.cancel)?;
        let mut cursor = range.start;
        for n in nodes {
            if n.source.start < cursor {
                continue;
            }
            odt.xml
                .prose(&self.text.byte_slice(cursor..n.source.start).to_string())?;
            self.object(odt, &n, depth + 1, context)?;
            cursor = n.source.end;
        }
        odt.xml
            .prose(&self.text.byte_slice(cursor..range.end).to_string())
    }
    fn object(
        &self,
        odt: &mut Odt<'_>,
        n: &native::Raw,
        depth: usize,
        context: InlineContext,
    ) -> Result<(), String> {
        match n.kind {
            InlineKind::Link(_) => self.link(odt, n, depth),
            InlineKind::FootnoteInline | InlineKind::FootnoteReference => self.note(odt, n, depth),
            InlineKind::Target | InlineKind::RadioTarget => self.target(odt, n),
            InlineKind::Entity => {
                odt.diagnostic("ODT entity/math spelling retained as literal data")?;
                odt.xml
                    .prose(&self.text.byte_slice(n.source.clone()).to_string())
            }
            _ => self.markup(odt, n, depth, context),
        }
    }
    fn target(&self, odt: &mut Odt<'_>, n: &native::Raw) -> Result<(), String> {
        odt.bookmark(&identity(
            &self.text.byte_slice(n.content.clone()).to_string(),
        )?)?;
        if n.kind == InlineKind::RadioTarget {
            odt.xml
                .prose(&self.text.byte_slice(n.content.clone()).to_string())?;
        }
        Ok(())
    }
    fn markup(
        &self,
        odt: &mut Odt<'_>,
        n: &native::Raw,
        depth: usize,
        context: InlineContext,
    ) -> Result<(), String> {
        let style = match n.kind {
            InlineKind::Bold => "Bold",
            InlineKind::Italic => "Italic",
            InlineKind::Underline => "Underline",
            InlineKind::Strike => "Strike",
            _ => "Code",
        };
        odt.xml
            .emit(&format!("<text:span text:style-name=\"{style}\">"))?;
        if matches!(n.kind, InlineKind::Code | InlineKind::Verbatim) {
            odt.xml
                .prose(&self.text.byte_slice(n.content.clone()).to_string())?;
        } else {
            self.inline(odt, n.content.clone(), depth, context)?;
        }
        odt.xml.emit("</text:span>")
    }
    fn link(&self, odt: &mut Odt<'_>, n: &native::Raw, depth: usize) -> Result<(), String> {
        let value = self.text.byte_slice(n.content.clone()).to_string();
        let Ok(target) = Target::parse(&value).and_then(|t| self.destination(odt, t)) else {
            odt.diagnostic(
                "Unsupported ODT resource retained as a literal link; no asset embedded",
            )?;
            return odt
                .xml
                .prose(&self.text.byte_slice(n.source.clone()).to_string());
        };
        odt.xml
            .emit("<text:a xlink:type=\"simple\" xlink:href=\"")?;
        odt.xml.text(&target)?;
        odt.xml.emit("\">")?;
        if let Some(label) = &n.detail {
            self.inline(odt, label.clone(), depth, InlineContext::Minimal)?;
        } else {
            odt.xml.prose(&value)?;
        }
        odt.xml.emit("</text:a>")
    }
    fn destination(&self, odt: &mut Odt<'_>, target: Target) -> Result<String, String> {
        match target {
            Target::External(value) => Ok(value),
            Target::Internal(value) => {
                let target = if let Some(title) = value.strip_prefix('*') {
                    self.heading_links
                        .get(title)
                        .and_then(Clone::clone)
                        .ok_or("Unresolved ODT heading link")?
                } else {
                    identity(&value)?
                };
                odt.references.insert(target.clone());
                Ok(format!("#{target}"))
            }
            Target::File { path, search, .. } => {
                if !search.is_empty() {
                    return Err("ODT file search is unsupported".into());
                }
                let value = path.to_str().ok_or("ODT file link is not UTF-8")?;
                if media(value) {
                    return Err("ODT media embedding is unsupported".into());
                }
                Ok(crate::org::links::encode(value))
            }
            _ => Err("ODT resource kind is unsupported".into()),
        }
    }
}
pub(super) fn identity(value: &str) -> Result<String, String> {
    use std::fmt::Write;
    if value.is_empty() || value.len() > 1024 {
        return Err("ODT bookmark requires 1..1024 bytes".into());
    }
    let mut id = String::from("org-target-");
    for b in value.bytes() {
        write!(id, "{b:02x}").expect("write string");
    }
    Ok(id)
}
fn media(value: &str) -> bool {
    Path::new(value)
        .extension()
        .and_then(|v| v.to_str())
        .is_some_and(|v| {
            ["png", "jpg", "jpeg", "gif", "svg", "webp", "avif"]
                .contains(&v.to_ascii_lowercase().as_str())
        })
}
