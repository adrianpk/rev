use super::*;
use crate::org::inline as native;
impl Document<'_> {
    pub(super) fn inline(
        &self,
        html: &mut Html<'_>,
        range: Range<usize>,
        depth: usize,
    ) -> Result<(), String> {
        self.objects(html, range, depth, InlineContext::Standard)
    }
    pub(super) fn objects(
        &self,
        html: &mut Html<'_>,
        range: Range<usize>,
        depth: usize,
        context: InlineContext,
    ) -> Result<(), String> {
        if depth >= 32 {
            return Err("HTML inline nesting limit reached".into());
        }
        let nodes = native::objects(self.text, range.clone(), context, html.cancel)?;
        let mut cursor = range.start;
        for n in nodes {
            document::check(html.cancel)?;
            if n.source.start < cursor {
                continue;
            }
            html.text(&self.text.byte_slice(cursor..n.source.start).to_string())?;
            self.object(html, &n, depth + 1, context)?;
            cursor = n.source.end;
        }
        html.text(&self.text.byte_slice(cursor..range.end).to_string())
    }
    pub(super) fn capture(
        &self,
        html: &mut Html<'_>,
        range: Range<usize>,
        depth: usize,
    ) -> Result<String, String> {
        let old = std::mem::take(&mut html.output);
        let result = self.inline(html, range, depth);
        let output = std::mem::replace(&mut html.output, old);
        result?;
        Ok(output)
    }
    fn object(
        &self,
        html: &mut Html<'_>,
        n: &native::Raw,
        depth: usize,
        context: InlineContext,
    ) -> Result<(), String> {
        match n.kind {
            InlineKind::Link(_) => self.link(html, n, depth),
            InlineKind::FootnoteReference | InlineKind::FootnoteInline => self.note(html, n, depth),
            InlineKind::Target | InlineKind::RadioTarget => self.target(html, n),
            _ => self.decoration(html, n, depth, context),
        }
    }
    fn decoration(
        &self,
        html: &mut Html<'_>,
        n: &native::Raw,
        depth: usize,
        context: InlineContext,
    ) -> Result<(), String> {
        if n.kind == InlineKind::Entity {
            return html.text(&self.text.byte_slice(n.source.clone()).to_string());
        }
        let tag = tag(n.kind);
        html.emit(&format!("<{tag}>"))?;
        if matches!(n.kind, InlineKind::Code | InlineKind::Verbatim) {
            html.text(&self.text.byte_slice(n.content.clone()).to_string())?;
        } else {
            self.objects(html, n.content.clone(), depth, context)?;
        }
        html.emit(&format!("</{tag}>"))
    }

    fn target(&self, html: &mut Html<'_>, n: &native::Raw) -> Result<(), String> {
        let value = self.text.byte_slice(n.content.clone()).to_string();
        let anchor = format!("org-target-{}", encoded(&value));
        html.identity(&anchor)?;
        html.emit(&format!("<span id=\"{anchor}\">"))?;
        if n.kind == InlineKind::RadioTarget {
            html.text(&value)?;
        }
        html.emit("</span>")
    }
}
fn tag(kind: InlineKind) -> &'static str {
    match kind {
        InlineKind::Bold => "strong",
        InlineKind::Italic => "em",
        InlineKind::Underline => "u",
        InlineKind::Strike => "del",
        InlineKind::Code | InlineKind::Verbatim => "code",
        _ => "span",
    }
}
