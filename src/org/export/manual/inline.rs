use super::*;
use crate::org::inline as native;
impl Document<'_> {
    pub(super) fn inline(
        &self,
        out: &mut Manual<'_>,
        range: Range<usize>,
        depth: usize,
        context: InlineContext,
    ) -> Result<(), String> {
        if depth >= 32 {
            return Err("Manual inline nesting limit reached".into());
        }
        let nodes = native::objects(self.text, range.clone(), context, out.cancel)?;
        let mut cursor = range.start;
        for n in nodes {
            document::check(out.cancel)?;
            if n.source.start < cursor {
                continue;
            }
            out.text(&self.text.byte_slice(cursor..n.source.start).to_string())?;
            self.object(out, &n, depth + 1, context)?;
            cursor = n.source.end;
        }
        out.text(&self.text.byte_slice(cursor..range.end).to_string())
    }
    fn object(
        &self,
        out: &mut Manual<'_>,
        n: &native::Raw,
        depth: usize,
        context: InlineContext,
    ) -> Result<(), String> {
        let pair = decoration(n.kind, out.texinfo());
        let Some((start, end)) = pair else {
            out.diagnostic("Unsupported manual inline form retained literally")?;
            return out.text(&self.text.byte_slice(n.source.clone()).to_string());
        };
        out.emit(start)?;
        if matches!(n.kind, InlineKind::Code | InlineKind::Verbatim) {
            out.text(&self.text.byte_slice(n.content.clone()).to_string())?;
        } else {
            self.inline(out, n.content.clone(), depth, context)?;
        }
        out.emit(end)
    }
}
fn decoration(kind: InlineKind, texinfo: bool) -> Option<(&'static str, &'static str)> {
    let index = match kind {
        InlineKind::Bold => 0,
        InlineKind::Italic => 1,
        InlineKind::Code | InlineKind::Verbatim => 2,
        _ => return None,
    };
    Some(if texinfo {
        [("@strong{", "}"), ("@emph{", "}"), ("@code{", "}")][index]
    } else {
        [("\\fB", "\\fP"), ("\\fI", "\\fP"), ("\\f(CW", "\\fP")][index]
    })
}
