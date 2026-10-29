//! Native structure becomes ODF elements; unsupported rich data stays literal.
mod blocks;
mod headings;
mod inline;
mod math;
mod notes;
use super::*;
use crate::org::{BlockKind, InlineKind};
use std::collections::{BTreeMap, BTreeSet};
struct Odt<'a> {
    xml: Xml<'a>,
    snapshot: &'a Snapshot,
    cancel: &'a Cancellation,
    diagnostics: Vec<String>,
    ids: BTreeSet<String>,
    references: BTreeSet<String>,
    used: BTreeMap<(usize, String), usize>,
    active: BTreeSet<(usize, String)>,
    note_count: usize,
    observed: usize,
    lists: usize,
    tables: usize,
}
struct Document<'a> {
    text: &'a Rope,
    nodes: &'a [structure::Raw],
    children: Vec<Vec<usize>>,
    roots: Vec<usize>,
    base: usize,
    prefix: usize,
    notes: BTreeMap<String, Range<usize>>,
    headings: BTreeMap<usize, (String, Range<usize>)>,
    heading_links: BTreeMap<String, Option<String>>,
}
pub(super) fn render(
    snapshot: &Snapshot,
    cancel: &Cancellation,
) -> Result<(String, Vec<String>), String> {
    let doc = Document::new(&snapshot.text, &snapshot.nodes, snapshot, cancel, 0)?;
    let mut odt = Odt {
        xml: Xml::new(snapshot.export_bytes, cancel),
        snapshot,
        cancel,
        diagnostics: snapshot.diagnostics.clone(),
        ids: BTreeSet::new(),
        references: BTreeSet::new(),
        used: BTreeMap::new(),
        active: BTreeSet::new(),
        note_count: 0,
        observed: snapshot.nodes.len(),
        lists: 0,
        tables: 0,
    };
    odt.xml.start("document-content")?;
    odt.xml.emit("<office:body><office:text>")?;
    doc.header(&mut odt)?;
    doc.body(&mut odt, 0)?;
    odt.targets()?;
    odt.xml
        .emit("</office:text></office:body></office:document-content>")?;
    Ok((odt.xml.output, odt.diagnostics))
}
impl Odt<'_> {
    fn diagnostic(&mut self, value: &str) -> Result<(), String> {
        if self.diagnostics.len() == 128 {
            return Err("ODT diagnostic limit reached".into());
        }
        self.diagnostics.push(value.into());
        Ok(())
    }
    fn bookmark(&mut self, name: &str) -> Result<(), String> {
        if !self.ids.insert(name.into()) {
            return Err("Duplicate ODT bookmark".into());
        }
        self.xml
            .emit(&format!("<text:bookmark text:name=\"{name}\"/>"))
    }
    fn targets(&self) -> Result<(), String> {
        if self.references.iter().any(|r| !self.ids.contains(r)) {
            return Err("Unresolved ODT local target; include its definition".into());
        }
        Ok(())
    }
    fn literal(&mut self, value: &str) -> Result<(), String> {
        self.xml.emit("<text:p text:style-name=\"Literal\">")?;
        self.xml.prose(value)?;
        self.xml.emit("</text:p>")
    }
    fn unsupported(&mut self, value: &str) -> Result<(), String> {
        self.diagnostic("Unsupported ODT rich form retained as literal data")?;
        self.literal(value)
    }
}
impl<'a> Document<'a> {
    fn new(
        text: &'a Rope,
        nodes: &'a [structure::Raw],
        snapshot: &Snapshot,
        cancel: &Cancellation,
        prefix: usize,
    ) -> Result<Self, String> {
        let mut children = vec![Vec::new(); nodes.len()];
        let mut roots = Vec::new();
        for (id, n) in nodes.iter().enumerate() {
            document::check(cancel)?;
            if let Some(parent) = n.parent {
                children[parent].push(id);
            } else {
                roots.push(id);
            }
        }
        let base = if matches!(snapshot.request.scope, Scope::Subtree(_)) {
            nodes
                .iter()
                .find_map(|n| match n.kind {
                    StructureKind::Heading { level } => Some(level - 1),
                    _ => None,
                })
                .unwrap_or(0)
        } else {
            0
        };
        let mut doc = Self {
            text,
            nodes,
            children,
            roots,
            base,
            prefix,
            notes: BTreeMap::new(),
            headings: BTreeMap::new(),
            heading_links: BTreeMap::new(),
        };
        doc.definitions(cancel)?;
        doc.index(snapshot, cancel)?;
        Ok(doc)
    }
    fn body(&self, odt: &mut Odt<'_>, depth: usize) -> Result<(), String> {
        for id in &self.roots {
            self.node(odt, *id, depth)?;
        }
        Ok(())
    }
    fn children(&self, odt: &mut Odt<'_>, id: usize, depth: usize) -> Result<(), String> {
        for child in &self.children[id] {
            self.node(odt, *child, depth + 1)?;
        }
        Ok(())
    }
    fn node(&self, odt: &mut Odt<'_>, id: usize, depth: usize) -> Result<(), String> {
        document::check(odt.cancel)?;
        if depth >= super::super::super::Limits::default().depth {
            return Err("ODT nesting limit reached".into());
        }
        self.properties(odt, id)?;
        self.element(odt, id, depth)
    }
    fn element(&self, odt: &mut Odt<'_>, id: usize, depth: usize) -> Result<(), String> {
        match self.nodes[id].kind {
            StructureKind::Section => self.children(odt, id, depth),
            StructureKind::Heading { level } => self.heading(odt, (id, level), depth),
            StructureKind::Paragraph => self.paragraph(odt, id),
            StructureKind::List { ordered } => self.list(odt, (id, ordered), depth),
            StructureKind::Item { .. } => self.item(odt, id, depth),
            StructureKind::Table => self.table(odt, id),
            StructureKind::Block(kind) => self.block(odt, (id, kind), depth),
            _ => self.leaf(odt, id),
        }
    }
    fn paragraph(&self, odt: &mut Odt<'_>, id: usize) -> Result<(), String> {
        let n = &self.nodes[id];
        if self.definition(n, odt.cancel)?.is_some() {
            return Ok(());
        }
        odt.xml.emit("<text:p>")?;
        self.inline(odt, n.content.clone(), 0, InlineContext::Standard)?;
        odt.xml.emit("</text:p>")
    }
    fn leaf(&self, odt: &mut Odt<'_>, id: usize) -> Result<(), String> {
        let n = &self.nodes[id];
        match n.kind {
            StructureKind::FixedWidth | StructureKind::Planning | StructureKind::TableFormula => {
                odt.literal(&self.text.byte_slice(n.content.clone()).to_string())
            }
            StructureKind::Keyword => {
                let (key, _) = document::keyword(self.text, n).ok_or("ODT keyword has no name")?;
                if super::super::render::headings::known(&key) && !key.starts_with("ATTR_") {
                    return Ok(());
                }
                odt.unsupported(&self.text.byte_slice(n.source.clone()).to_string())
            }
            _ => Ok(()),
        }
    }
    fn properties(&self, odt: &mut Odt<'_>, id: usize) -> Result<(), String> {
        for child in &self.children[id] {
            let n = &self.nodes[*child];
            if n.kind == StructureKind::AffiliatedKeyword {
                self.property(odt, n)?;
            }
        }
        Ok(())
    }
    fn property(&self, odt: &mut Odt<'_>, n: &structure::Raw) -> Result<(), String> {
        let key = n
            .name
            .as_ref()
            .map(|r| {
                self.text
                    .byte_slice(r.clone())
                    .to_string()
                    .to_ascii_uppercase()
            })
            .unwrap_or_default();
        match key.as_str() {
            "NAME" => {
                let name = self.text.byte_slice(n.content.clone()).to_string();
                odt.xml.emit("<text:p>")?;
                odt.bookmark(&inline::identity(name.trim())?)?;
                odt.xml.emit("</text:p>")
            }
            "CAPTION" => {
                odt.xml.emit("<text:p text:style-name=\"Caption\">")?;
                self.inline(odt, n.content.clone(), 0, InlineContext::Standard)?;
                odt.xml.emit("</text:p>")
            }
            _ => odt.unsupported(&self.text.byte_slice(n.source.clone()).to_string()),
        }
    }
}
