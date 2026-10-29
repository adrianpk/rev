//! Native output is derived data, never a source edit or an implicit save.
mod blocks;
pub(super) mod headings;
mod inline;
mod notes;
use super::*;
use crate::org::{BlockKind, InlineKind};
use std::collections::BTreeSet;
pub struct Rendered {
    pub(super) text: String,
    pub(super) diagnostics: Vec<String>,
}
impl Rendered {
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }
}
impl Snapshot {
    pub fn render(&self, cancel: &Cancellation) -> Result<Rendered, String> {
        let cancel = cancel.bounded(Instant::now() + Duration::from_secs(self.job_seconds as u64));
        check(&cancel)?;
        if self.request.backend == ExportBackend::Html {
            return super::html::render(self, &cancel);
        }
        if !available(self.request.backend) {
            return Err("Selected native export renderer is not available".into());
        }
        let mut render = Render::new(
            self,
            Document {
                text: &self.text,
                nodes: &self.nodes,
            },
            &cancel,
        )?;
        let text = render.run()?;
        let text = if self.request.backend == ExportBackend::Org {
            text
        } else {
            text.replace("\r\n", "\n")
        };
        if self.request.backend == ExportBackend::Text {
            ascii(&text)?;
        }
        Ok(Rendered {
            text,
            diagnostics: render.diagnostics,
        })
    }
}
struct Document<'a> {
    text: &'a Rope,
    nodes: &'a [structure::Raw],
}
struct Render<'a> {
    text: &'a Rope,
    nodes: &'a [structure::Raw],
    fragment_depth: usize,
    observed: usize,
    snapshot: &'a Snapshot,
    cancel: &'a Cancellation,
    children: Vec<Vec<usize>>,
    roots: Vec<usize>,
    inline: inline::Inline<'a>,
    diagnostics: Vec<String>,
    headings: Vec<String>,
    numbers: Vec<usize>,
    generated: usize,
    base: usize,
}
type Change = (Range<usize>, String);
impl<'a> Render<'a> {
    fn new(
        snapshot: &'a Snapshot,
        document: Document<'a>,
        cancel: &'a Cancellation,
    ) -> Result<Self, String> {
        let (children, roots) = tree(document.nodes, cancel)?;
        let base = if matches!(snapshot.request.scope, Scope::Subtree(_)) {
            document
                .nodes
                .iter()
                .find_map(|n| {
                    if let StructureKind::Heading { level } = n.kind {
                        Some(level - 1)
                    } else {
                        None
                    }
                })
                .unwrap_or(0)
        } else {
            0
        };
        Ok(Self {
            snapshot,
            text: document.text,
            nodes: document.nodes,
            fragment_depth: 0,
            observed: document.nodes.len(),
            cancel,
            children,
            roots,
            inline: inline::Inline::new(document.text, snapshot, cancel),
            diagnostics: snapshot.diagnostics.clone(),
            headings: Vec::new(),
            numbers: Vec::new(),
            generated: 0,
            base,
        })
    }
    fn run(&mut self) -> Result<String, String> {
        let body = self.body()?;
        let mut output = self.header()?;
        append(&mut output, &body, self.snapshot.export_bytes)?;
        self.inline.notes.finish()?;
        let notes = if self.org() {
            String::new()
        } else {
            self.inline.notes.bodies(self.markdown())
        };
        append(&mut output, &notes, self.snapshot.export_bytes)?;
        self.diagnostics.append(&mut self.inline.diagnostics);
        Ok(output)
    }
    fn body(&mut self) -> Result<String, String> {
        let mut changes = Vec::new();
        for id in self.roots.clone() {
            self.paint(id, 0, &mut changes)?;
        }
        self.compose(0..self.text.len_bytes(), changes)
    }
    fn fragment(&mut self, body: &str) -> Result<String, String> {
        if self.fragment_depth == 16 {
            return Err("Export report nesting limit reached".into());
        }
        let text = Rope::from_str(body);
        let nodes = document::nodes(&text, self.cancel)?;
        let mut child = Render::new(
            self.snapshot,
            Document {
                text: &text,
                nodes: &nodes,
            },
            self.cancel,
        )?;
        child.observed = self
            .observed
            .checked_add(nodes.len())
            .ok_or("Export node count overflows")?;
        if child.observed > super::super::Limits::default().nodes {
            return Err("Export report source node limit reached".into());
        }
        child.fragment_depth = self.fragment_depth + 1;
        child.diagnostics = Vec::new();
        child.inline.notes = std::mem::take(&mut self.inline.notes);
        let output = child.body()?;
        self.observed = child.observed;
        self.inline.notes = std::mem::take(&mut child.inline.notes);
        child.diagnostics.append(&mut child.inline.diagnostics);
        for message in child.diagnostics {
            self.diagnostic(message)?;
        }
        Ok(output)
    }
    fn paint(&mut self, id: usize, depth: usize, changes: &mut Vec<Change>) -> Result<(), String> {
        check(self.cancel)?;
        if depth >= super::super::Limits::default().depth {
            return Err("Export rendering depth limit reached".into());
        }
        let node = &self.nodes[id];
        if self.org() {
            return self.org_node(id, depth, changes);
        }
        if hidden(node.kind) {
            return self.change(changes, node.source.clone(), String::new());
        }
        self.affiliated(id, depth, changes)?;
        if self.paint_element(id, depth, changes)? {
            self.descend(id, depth, changes)?;
        }
        Ok(())
    }
    fn paint_element(
        &mut self,
        id: usize,
        depth: usize,
        changes: &mut Vec<Change>,
    ) -> Result<bool, String> {
        match self.element(id, depth)? {
            Some((range, text)) => {
                self.change(changes, range, text)?;
                Ok(matches!(self.nodes[id].kind, StructureKind::Heading { .. }))
            }
            None => Ok(true),
        }
    }
    fn affiliated(
        &mut self,
        id: usize,
        depth: usize,
        changes: &mut Vec<Change>,
    ) -> Result<(), String> {
        for child in self.children[id].clone() {
            if self.nodes[child].kind == StructureKind::AffiliatedKeyword {
                self.paint(child, depth + 1, changes)?;
            }
        }
        Ok(())
    }
    fn element(&mut self, id: usize, depth: usize) -> Result<Option<Change>, String> {
        let n = &self.nodes[id];
        match n.kind {
            StructureKind::Heading { level } => self
                .heading(n, level)
                .map(|t| Some((n.opening.clone().expect("heading"), t))),
            StructureKind::Paragraph => self
                .inline
                .range(n.content.clone(), 0)
                .map(|t| Some((n.content.clone(), t))),
            StructureKind::Keyword | StructureKind::AffiliatedKeyword => {
                self.keyword(n).map(|t| Some((n.source.clone(), t)))
            }
            StructureKind::Table => self.table(id).map(|t| Some((n.source.clone(), t))),
            StructureKind::Block(kind) => self
                .block(id, kind, depth)
                .map(|t| Some((n.source.clone(), t))),
            _ => Ok(None),
        }
    }
    fn descend(
        &mut self,
        id: usize,
        depth: usize,
        changes: &mut Vec<Change>,
    ) -> Result<(), String> {
        for child in self.children[id].clone() {
            if self.nodes[child].kind != StructureKind::AffiliatedKeyword {
                self.paint(child, depth + 1, changes)?;
            }
        }
        Ok(())
    }
    fn change(
        &mut self,
        changes: &mut Vec<Change>,
        range: Range<usize>,
        text: String,
    ) -> Result<(), String> {
        self.generated = self
            .generated
            .checked_add(text.len())
            .ok_or("Export output overflows")?;
        if self.generated > self.snapshot.export_bytes {
            return Err("Export aggregate rendering byte limit reached".into());
        }
        changes.push((range, text));
        Ok(())
    }
    fn compose(&self, range: Range<usize>, mut changes: Vec<Change>) -> Result<String, String> {
        changes.sort_by_key(|(r, _)| (r.start, std::cmp::Reverse(r.end)));
        let mut out = String::new();
        let mut cursor = range.start;
        for (r, text) in changes {
            check(self.cancel)?;
            if r.start < cursor {
                continue;
            }
            self.copy(&mut out, cursor..r.start)?;
            append(&mut out, &text, self.snapshot.export_bytes)?;
            cursor = r.end;
        }
        self.copy(&mut out, cursor..range.end)?;
        Ok(out)
    }
    fn copy(&self, out: &mut String, range: Range<usize>) -> Result<(), String> {
        for chunk in self.text.byte_slice(range).chunks() {
            check(self.cancel)?;
            append(out, chunk, self.snapshot.export_bytes)?;
        }
        Ok(())
    }
    fn org(&self) -> bool {
        self.snapshot.request.backend == ExportBackend::Org
    }
    fn markdown(&self) -> bool {
        self.snapshot.request.backend == ExportBackend::Markdown
    }
    fn diagnostic(&mut self, value: String) -> Result<(), String> {
        if self.diagnostics.len() == 128 {
            return Err("Export diagnostic limit reached".into());
        }
        self.diagnostics.push(value);
        Ok(())
    }
}
fn append(out: &mut String, text: &str, limit: usize) -> Result<(), String> {
    if text.len() > limit.saturating_sub(out.len()) {
        return Err("Export output byte limit reached".into());
    }
    out.push_str(text);
    Ok(())
}
fn check(cancel: &Cancellation) -> Result<(), String> {
    document::check(cancel)
}
fn ascii(value: &str) -> Result<(), String> {
    if let Some((byte, c)) = value.char_indices().find(|(_, c)| !c.is_ascii()) {
        return Err(format!(
            "ASCII export cannot represent U+{:04X} at output byte {byte}; choose UTF-8, Markdown or Org",
            c as u32
        ));
    }
    Ok(())
}
pub(super) fn workflow(
    text: &Rope,
    nodes: &[structure::Raw],
    policy: &crate::config::Todo,
    cancel: &Cancellation,
) -> Result<Vec<String>, String> {
    let mut workflow = super::super::task::Workflow::default();
    for node in document::active(nodes, cancel)? {
        if let Some((key, value)) = document::keyword(text, node)
            && matches!(key.as_str(), "TODO" | "SEQ_TODO" | "TYP_TODO")
        {
            workflow.declaration(&value);
        }
    }
    Ok(workflow
        .effective(policy)?
        .states
        .into_iter()
        .map(|s| s.name)
        .collect())
}

fn hidden(kind: StructureKind) -> bool {
    matches!(
        kind,
        StructureKind::Drawer
            | StructureKind::PropertyDrawer
            | StructureKind::Comment
            | StructureKind::Block(BlockKind::Comment)
    )
}

fn available(backend: ExportBackend) -> bool {
    matches!(
        backend,
        ExportBackend::Text | ExportBackend::Utf8 | ExportBackend::Markdown | ExportBackend::Org
    )
}
fn tree(
    nodes: &[structure::Raw],
    cancel: &Cancellation,
) -> Result<(Vec<Vec<usize>>, Vec<usize>), String> {
    let mut children = vec![Vec::new(); nodes.len()];
    let mut roots = Vec::new();
    for (i, n) in nodes.iter().enumerate() {
        check(cancel)?;
        if let Some(parent) = n.parent {
            children[parent].push(i);
        } else {
            roots.push(i);
        }
    }
    Ok((children, roots))
}
