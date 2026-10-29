use super::*;
use crate::org::{inline as native, links::Target};
impl Document<'_> {
    pub(super) fn index(&mut self, cancel: &Cancellation) -> Result<(), String> {
        let ids: BTreeMap<_, _> = self
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| matches!(n.kind, StructureKind::Heading { .. }))
            .map(|(i, n)| (n.source.start, i))
            .collect();
        for n in document::active(self.nodes, cancel)? {
            self.index_affiliation(n)?;
            self.index_node(n, ids.get(&n.source.start).copied(), cancel)?;
            if n.kind == StructureKind::Paragraph {
                self.index_targets(n, cancel)?;
            }
        }
        Ok(())
    }
    fn index_node(
        &mut self,
        n: &structure::Raw,
        id: Option<usize>,
        cancel: &Cancellation,
    ) -> Result<(), String> {
        match n.kind {
            StructureKind::Heading { .. } => self.index_title(n, id.expect("native heading index")),
            StructureKind::Keyword => return self.index_name(n),
            StructureKind::Paragraph => return self.index_note(n, cancel),
            _ => {}
        }
        Ok(())
    }
    fn index_title(&mut self, n: &structure::Raw, id: usize) {
        let title = self.text.byte_slice(n.content.clone()).to_string();
        self.titles.entry(title).or_default().push(id);
    }
    fn index_name(&mut self, n: &structure::Raw) -> Result<(), String> {
        let Some((key, value)) = document::keyword(self.text, n) else {
            return Ok(());
        };
        if key == "NAME" {
            self.admit_target(&value)?;
        }
        Ok(())
    }
    fn index_targets(&mut self, n: &structure::Raw, cancel: &Cancellation) -> Result<(), String> {
        for object in native::objects(
            self.text,
            n.content.clone(),
            InlineContext::Standard,
            cancel,
        )? {
            if matches!(object.kind, InlineKind::Target | InlineKind::RadioTarget) {
                self.admit_target(&self.text.byte_slice(object.content).to_string())?;
            }
        }
        Ok(())
    }
    pub(super) fn admit_target(&mut self, label: &str) -> Result<(), String> {
        if label.is_empty() || label.len() > 1024 || self.targets.len() >= 4096 {
            return Err("Manual target label or count limit reached".into());
        }
        let reference = Reference {
            id: format!("org-target-{}", self.targets.len()),
            label: label.into(),
        };
        if self.targets.insert(label.into(), reference).is_some() {
            return Err("Duplicate manual target".into());
        }
        Ok(())
    }
    pub(super) fn named(&self, out: &mut Manual<'_>, n: &structure::Raw) -> Result<(), String> {
        let label = self.text.byte_slice(n.content.clone()).to_string();
        self.anchor(out, &label)?;
        if out.texinfo() {
            out.emit("\n")?;
        }
        Ok(())
    }
    pub(super) fn anchor(&self, out: &mut Manual<'_>, label: &str) -> Result<(), String> {
        let reference = self.targets.get(label).ok_or("Unresolved manual target")?;
        if out.texinfo() {
            out.emit(&format!("@anchor{{{}}}", reference.id))?;
        }
        Ok(())
    }
    pub(super) fn target(&self, out: &mut Manual<'_>, n: &native::Raw) -> Result<(), String> {
        let label = self.text.byte_slice(n.content.clone()).to_string();
        self.anchor(out, &label)?;
        if n.kind == InlineKind::RadioTarget {
            out.text(&label)?;
        }
        Ok(())
    }
    fn reference(&self, value: &str) -> Result<Reference, String> {
        if let Some(reference) = self.targets.get(value) {
            return Ok(reference.clone());
        }
        let title = value.strip_prefix('*').unwrap_or(value);
        let ids = self
            .titles
            .get(title)
            .ok_or("Unresolved manual heading or target")?;
        if ids.len() != 1 {
            return Err("Ambiguous manual heading reference".into());
        }
        Ok(Reference {
            id: format!("org-heading-{}", ids[0]),
            label: title.into(),
        })
    }
    pub(super) fn link(
        &self,
        out: &mut Manual<'_>,
        n: &native::Raw,
        depth: usize,
    ) -> Result<(), String> {
        let value = self.text.byte_slice(n.content.clone()).to_string();
        match Target::parse(&value) {
            Ok(Target::Internal(target)) => self.local_link(out, n, (&value, &target), depth),
            Ok(target) => self.external_link(out, n, (&value, target), depth),
            Err(_) => self.literal_link(out, n),
        }
    }
    fn local_link(
        &self,
        out: &mut Manual<'_>,
        n: &native::Raw,
        values: (&str, &str),
        depth: usize,
    ) -> Result<(), String> {
        if unsupported(values.1) && !self.targets.contains_key(values.1) {
            return self.literal_link(out, n);
        }
        let reference = self.reference(values.1)?;
        if out.texinfo() {
            out.emit(&format!("@ref{{{},,", reference.id))?;
        }
        self.label(out, n, values.0, depth)?;
        if out.texinfo() {
            out.emit("}")
        } else {
            self.man_reference(out, &reference.label)
        }
    }
    fn man_reference(&self, out: &mut Manual<'_>, label: &str) -> Result<(), String> {
        out.emit(" (see ")?;
        out.text(label)?;
        out.emit(")")
    }
    fn external_link(
        &self,
        out: &mut Manual<'_>,
        n: &native::Raw,
        values: (&str, Target),
        depth: usize,
    ) -> Result<(), String> {
        let Some(uri) = uri(values.1) else {
            return self.literal_link(out, n);
        };
        if out.texinfo() {
            out.emit("@url{")?;
            out.line(&uri)?;
            out.emit(",")?;
        }
        self.label(out, n, values.0, depth)?;
        if out.texinfo() {
            out.emit("}")
        } else {
            out.emit(" <")?;
            out.line(&uri)?;
            out.emit(">")
        }
    }
    fn label(
        &self,
        out: &mut Manual<'_>,
        n: &native::Raw,
        value: &str,
        depth: usize,
    ) -> Result<(), String> {
        if let Some(label) = &n.detail {
            self.inline(out, label.clone(), depth, InlineContext::Minimal)
        } else {
            out.text(value)
        }
    }
    fn literal_link(&self, out: &mut Manual<'_>, n: &native::Raw) -> Result<(), String> {
        out.diagnostic("Unsupported manual link retained literally; no resource read")?;
        out.text(&self.text.byte_slice(n.source.clone()).to_string())
    }
}
fn uri(target: Target) -> Option<String> {
    match target {
        Target::External(value) => Some(value),
        Target::File { path, search, .. } if search.is_empty() => {
            Some(crate::org::links::encode(path.to_str()?))
        }
        _ => None,
    }
}

fn unsupported(value: &str) -> bool {
    !value.starts_with('*') && (value.contains(':') || value.starts_with('#'))
}
