use super::*;
impl Document {
    pub(super) fn observe(text: Rope, cancel: &Cancellation) -> Result<Self, String> {
        let raw = structure::observe(text.clone(), Limits::default(), cancel);
        if raw.coverage != Coverage::Complete
            || raw.nodes.iter().any(|n| n.kind == StructureKind::Malformed)
        {
            return Err("Document structure is incomplete or malformed; source unchanged".into());
        }
        let mut objects = Vec::new();
        for node in &raw.nodes {
            check(cancel)?;
            if let Some(context) = node.context.filter(|c| *c != InlineContext::Literal) {
                objects.extend(inline::objects(
                    &text,
                    node.content.clone(),
                    context,
                    cancel,
                )?);
            }
            if objects.len() > Limits::default().nodes {
                return Err("Document inline node limit reached".into());
            }
        }
        objects.sort_by_key(|o| o.source.start);
        objects.dedup_by(|a, b| a.source == b.source && a.kind == b.kind);
        Ok(Self {
            text,
            nodes: raw.nodes,
            objects,
        })
    }
    pub(super) fn standard(&self, range: &Range<usize>) -> Result<(), String> {
        let admitted = self.nodes.iter().any(|n| {
            n.context == Some(InlineContext::Standard)
                && n.content.start <= range.start
                && range.end <= n.content.end
        });
        if !admitted && !(self.text.len_bytes() == 0 && range.is_empty()) {
            return Err("Action requires ordinary Org text; literal or unsupported context".into());
        }
        if self
            .objects
            .iter()
            .any(|o| opaque(o.kind) && overlaps(&o.source, range))
        {
            return Err("Action crosses a literal inline object or link; source unchanged".into());
        }
        Ok(())
    }
}
pub(super) fn overlaps(a: &Range<usize>, b: &Range<usize>) -> bool {
    if b.is_empty() {
        a.start < b.start && b.start < a.end
    } else {
        a.start < b.end && b.start < a.end
    }
}
fn opaque(kind: InlineKind) -> bool {
    matches!(
        kind,
        InlineKind::Code
            | InlineKind::Verbatim
            | InlineKind::Link(_)
            | InlineKind::Target
            | InlineKind::RadioTarget
            | InlineKind::Entity
    )
}
