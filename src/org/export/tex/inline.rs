use super::*;
use crate::org::{inline as native, links::Target};
impl Document<'_> {
    pub(super) fn inline(
        &self,
        tex: &mut Tex<'_>,
        range: Range<usize>,
        depth: usize,
    ) -> Result<(), String> {
        self.math(tex, range, depth, InlineContext::Standard)
    }
    pub(super) fn objects(
        &self,
        tex: &mut Tex<'_>,
        range: Range<usize>,
        depth: usize,
        context: InlineContext,
    ) -> Result<(), String> {
        if depth >= 32 {
            return Err("TeX inline nesting limit reached".into());
        }
        let nodes = native::objects(self.text, range.clone(), context, tex.cancel)?;
        let mut cursor = range.start;
        for n in nodes {
            document::check(tex.cancel)?;
            if n.source.start < cursor {
                continue;
            }
            tex.text(&self.text.byte_slice(cursor..n.source.start).to_string())?;
            self.object(tex, &n, depth + 1, context)?;
            cursor = n.source.end;
        }
        tex.text(&self.text.byte_slice(cursor..range.end).to_string())
    }
    fn object(
        &self,
        tex: &mut Tex<'_>,
        n: &native::Raw,
        depth: usize,
        context: InlineContext,
    ) -> Result<(), String> {
        match n.kind {
            InlineKind::Link(_) => self.link(tex, n, depth),
            InlineKind::FootnoteReference | InlineKind::FootnoteInline => self.note(tex, n, depth),
            _ => self.decoration(tex, n, depth, context),
        }
    }
    fn decoration(
        &self,
        tex: &mut Tex<'_>,
        n: &native::Raw,
        depth: usize,
        context: InlineContext,
    ) -> Result<(), String> {
        if matches!(n.kind, InlineKind::Target | InlineKind::RadioTarget) {
            return self.target(tex, n);
        }
        let command = command(n.kind);
        tex.emit(&format!("\\{command}{{"))?;
        if matches!(
            n.kind,
            InlineKind::Code | InlineKind::Verbatim | InlineKind::Entity
        ) {
            tex.text(&self.text.byte_slice(n.content.clone()).to_string())?;
        } else {
            self.math(tex, n.content.clone(), depth, context)?;
        }
        tex.emit("}")
    }
    fn target(&self, tex: &mut Tex<'_>, n: &native::Raw) -> Result<(), String> {
        let value = self.text.byte_slice(n.content.clone()).to_string();
        tex.emit(&format!("\\hypertarget{{rev-{}}}{{", encoded(&value)))?;
        if n.kind == InlineKind::RadioTarget {
            tex.text(&value)?;
        }
        tex.emit("}")
    }
    fn link(&self, tex: &mut Tex<'_>, n: &native::Raw, depth: usize) -> Result<(), String> {
        let value = self.text.byte_slice(n.content.clone()).to_string();
        let target = Target::parse(&value).and_then(destination);
        let Ok((command, target)) = target else {
            tex.diagnostic("Unsupported TeX link retained as literal data")?;
            return tex.text(&self.text.byte_slice(n.source.clone()).to_string());
        };
        tex.emit(&format!("\\{command}{{"))?;
        tex.text(&target)?;
        tex.emit("}{")?;
        self.link_label(tex, n, depth, &value)?;
        tex.emit("}")
    }
    fn link_label(
        &self,
        tex: &mut Tex<'_>,
        n: &native::Raw,
        depth: usize,
        value: &str,
    ) -> Result<(), String> {
        if let Some(label) = &n.detail {
            self.math(tex, label.clone(), depth, InlineContext::Minimal)
        } else {
            tex.text(value)
        }
    }
}
fn command(kind: InlineKind) -> &'static str {
    match kind {
        InlineKind::Bold => "textbf",
        InlineKind::Italic => "emph",
        InlineKind::Underline => "underline",
        InlineKind::Code | InlineKind::Verbatim => "texttt",
        _ => "textrm",
    }
}
fn destination(target: Target) -> Result<(&'static str, String), String> {
    match target {
        Target::External(uri) => Ok(("href", uri)),
        Target::Internal(value) => {
            if value.contains(':') || value.starts_with('*') {
                return Err("Unsupported TeX internal target".into());
            }
            Ok(("hyperlink", format!("rev-{}", encoded(&value))))
        }
        Target::File { path, search, .. } => {
            if !search.is_empty() {
                return Err("TeX file search suffix is unsupported".into());
            }
            let value = path.to_str().ok_or("TeX file path is not UTF-8")?;
            Ok(("href", crate::org::links::encode(value)))
        }
        _ => Err("Unsupported TeX link kind".into()),
    }
}
fn encoded(value: &str) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    for b in value.bytes() {
        write!(out, "{b:02x}").expect("write string");
    }
    out
}
