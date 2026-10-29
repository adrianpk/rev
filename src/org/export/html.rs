//! HTML is inert derived output. Only the ordinary publisher can write it.
mod attributes;
mod blocks;
use attributes::Properties;
mod inline;
mod links;
mod notes;
use super::*;
use crate::org::{BlockKind, InlineKind};
use std::collections::{BTreeMap, BTreeSet};
struct Html<'a> {
    snapshot: &'a Snapshot,
    cancel: &'a Cancellation,
    output: String,
    diagnostics: Vec<String>,
    ids: BTreeSet<String>,
    references: BTreeSet<String>,
    headings: Vec<(usize, String, String)>,
    numbers: Vec<usize>,
    notes: notes::Notes,
    observed: usize,
}
struct Document<'a> {
    text: &'a Rope,
    nodes: &'a [structure::Raw],
    children: Vec<Vec<usize>>,
    roots: Vec<usize>,
    base: usize,
    heading_ids: BTreeMap<usize, String>,
    heading_links: BTreeMap<String, Option<String>>,
}
pub(super) fn render(snapshot: &Snapshot, cancel: &Cancellation) -> Result<Rendered, String> {
    let doc = Document::new(&snapshot.text, &snapshot.nodes, snapshot, cancel, 0)?;
    let mut html = Html {
        snapshot,
        cancel,
        output: String::new(),
        diagnostics: snapshot.diagnostics.clone(),
        ids: BTreeSet::new(),
        references: BTreeSet::new(),
        headings: Vec::new(),
        numbers: Vec::new(),
        notes: notes::Notes::default(),
        observed: snapshot.nodes.len(),
    };
    doc.body(&mut html, 0)?;
    html.footnotes()?;
    html.targets_current()?;
    let body = std::mem::take(&mut html.output);
    html.header()?;
    html.emit(&body)?;
    html.emit("</main>\n</body>\n</html>\n")?;
    Ok(Rendered {
        text: html.output,
        diagnostics: html.diagnostics,
    })
}
impl Html<'_> {
    fn emit(&mut self, value: &str) -> Result<(), String> {
        document::check(self.cancel)?;
        if value.len() > self.snapshot.export_bytes.saturating_sub(self.output.len()) {
            return Err("HTML export output byte limit reached".into());
        }
        self.output.push_str(value);
        Ok(())
    }
    fn text(&mut self, value: &str) -> Result<(), String> {
        let value = value.replace("\r\n", "\n").replace('\r', "\n");
        for (i, c) in value.char_indices() {
            if i % 4096 == 0 {
                document::check(self.cancel)?;
            }
            let mut bytes = [0; 4];
            self.emit(match c {
                '&' => "&amp;",
                '<' => "&lt;",
                '>' => "&gt;",
                '"' => "&quot;",
                '\'' => "&#39;",
                _ => c.encode_utf8(&mut bytes),
            })?;
        }
        Ok(())
    }
    fn diagnostic(&mut self, message: &str) -> Result<(), String> {
        if self.diagnostics.len() == 128 {
            return Err("HTML export diagnostic limit reached".into());
        }
        self.diagnostics.push(message.into());
        Ok(())
    }
    fn identity(&mut self, id: &str) -> Result<(), String> {
        if !self.ids.insert(id.into()) {
            return Err("Duplicate HTML export identifier".into());
        }
        Ok(())
    }
    fn targets_current(&self) -> Result<(), String> {
        if self.references.iter().any(|id| !self.ids.contains(id)) {
            return Err(
                "Unresolved HTML local target; include its definition in the selected scope".into(),
            );
        }
        Ok(())
    }
    fn header(&mut self) -> Result<(), String> {
        let m = &self.snapshot.metadata;
        let language = m.language.as_deref().unwrap_or("en");
        self.emit("<!DOCTYPE html>\n<html lang=\"")?;
        self.text(language)?;
        self.emit("\">\n<head>\n<meta charset=\"utf-8\">\n<title>")?;
        let title = m
            .title
            .as_deref()
            .filter(|_| self.snapshot.options.title)
            .unwrap_or("Org document");
        self.text(title)?;
        self.emit("</title>\n")?;
        self.metadata()?;
        self.emit("</head>\n<body>\n<main>\n")?;
        self.preamble()?;
        self.contents()
    }
    fn metadata(&mut self) -> Result<(), String> {
        let m = &self.snapshot.metadata;
        for (name, value, enabled) in [
            ("author", &m.author, self.snapshot.options.author),
            ("email", &m.email, self.snapshot.options.author),
            ("date", &m.date, true),
        ] {
            if let Some(value) = value.as_ref().filter(|_| enabled) {
                self.emit(&format!("<meta name=\"{name}\" content=\""))?;
                self.text(value)?;
                self.emit("\">\n")?;
            }
        }
        Ok(())
    }
    fn preamble(&mut self) -> Result<(), String> {
        if matches!(self.snapshot.request.scope, Scope::Region(_)) {
            return Ok(());
        }
        let m = &self.snapshot.metadata;
        for (tag, value, enabled) in [
            ("h1", &m.title, self.snapshot.options.title),
            ("p", &m.author, self.snapshot.options.author),
            ("p", &m.date, true),
        ] {
            if let Some(value) = value.as_ref().filter(|_| enabled) {
                self.emit(&format!("<{tag}>"))?;
                self.text(value)?;
                self.emit(&format!("</{tag}>\n"))?;
            }
        }
        Ok(())
    }
    fn contents(&mut self) -> Result<(), String> {
        if !self.snapshot.options.toc || self.headings.is_empty() {
            return Ok(());
        }
        self.emit("<nav aria-label=\"Contents\"><ul>\n")?;
        for (_, id, title) in self.headings.clone() {
            self.emit(&format!("<li><a href=\"#{id}\">"))?;
            self.text(&title)?;
            self.emit("</a></li>\n")?;
        }
        self.emit("</ul></nav>\n")
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
        for (i, n) in nodes.iter().enumerate() {
            document::check(cancel)?;
            if let Some(parent) = n.parent {
                children[parent].push(i);
            } else {
                roots.push(i);
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
            heading_ids: BTreeMap::new(),
            heading_links: BTreeMap::new(),
        };
        doc.index_headings(snapshot, cancel, prefix)?;
        Ok(doc)
    }
    fn index_headings(
        &mut self,
        snapshot: &Snapshot,
        cancel: &Cancellation,
        prefix: usize,
    ) -> Result<(), String> {
        let options = Options {
            todo: false,
            tags: false,
            ..snapshot.options.clone()
        };
        for (id, n) in self
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| matches!(n.kind, StructureKind::Heading { .. }))
        {
            document::check(cancel)?;
            let anchor = format!("org-heading-{prefix}-{}", self.heading_ids.len() + 1);
            self.heading_ids.insert(id, anchor.clone());
            let title =
                super::render::headings::title_range(self.text, n, &options, &snapshot.todo);
            for range in [n.content.clone(), title] {
                let key = self.text.byte_slice(range).to_string();
                self.heading_links
                    .entry(key)
                    .and_modify(|old| {
                        if old.as_ref() != Some(&anchor) {
                            *old = None;
                        }
                    })
                    .or_insert(Some(anchor.clone()));
            }
        }
        Ok(())
    }
    fn body(&self, html: &mut Html<'_>, depth: usize) -> Result<(), String> {
        for id in &self.roots {
            self.node(html, *id, depth)?;
        }
        Ok(())
    }
    fn children(&self, html: &mut Html<'_>, id: usize, depth: usize) -> Result<(), String> {
        for child in &self.children[id] {
            self.node(html, *child, depth + 1)?;
        }
        Ok(())
    }
    fn node(&self, html: &mut Html<'_>, id: usize, depth: usize) -> Result<(), String> {
        document::check(html.cancel)?;
        if depth >= super::super::Limits::default().depth {
            return Err("HTML export nesting limit reached".into());
        }
        let n = &self.nodes[id];
        match n.kind {
            StructureKind::Heading { level } => self.heading(html, id, level, depth),
            StructureKind::Section => self.children(html, id, depth),
            StructureKind::List { ordered } => self.container(html, id, list_tag(ordered), depth),
            StructureKind::Item { .. } => self.container(html, id, "li", depth),
            StructureKind::Paragraph => self.paragraph(html, id),
            StructureKind::Table => self.table(html, id),
            StructureKind::Block(kind) => self.block(html, id, kind, depth),
            _ => self.leaf(html, n),
        }
    }
    fn container(
        &self,
        html: &mut Html<'_>,
        id: usize,
        tag: &str,
        depth: usize,
    ) -> Result<(), String> {
        let props = self.properties(html, id)?;
        self.open_container(html, id, tag, &props)?;
        html.emit("\n")?;
        self.children(html, id, depth)?;
        html.emit(&format!("</{tag}>\n"))
    }
    fn open_container(
        &self,
        html: &mut Html<'_>,
        id: usize,
        tag: &str,
        props: &Properties,
    ) -> Result<(), String> {
        props.admit(tag)?;
        props.anchor(html)?;
        let mut values = props.values.clone();
        if tag == "ol" {
            values.insert("start".into(), self.list_start(id)?);
        }
        html.open(tag, &values)
    }
    fn list_start(&self, id: usize) -> Result<String, String> {
        let item = self.children[id]
            .iter()
            .find_map(|i| self.nodes[*i].opening.as_ref())
            .ok_or("Ordered list has no item marker")?;
        let marker = self.text.byte_slice(item.clone()).to_string();
        let number = marker.trim().trim_end_matches(['.', ')']);
        let start = number
            .parse::<u64>()
            .map_err(|_| "Ordered list start exceeds HTML integer range")?;
        Ok(start.to_string())
    }
    fn heading(
        &self,
        html: &mut Html<'_>,
        id: usize,
        level: usize,
        depth: usize,
    ) -> Result<(), String> {
        let level = level.saturating_sub(self.base).max(1);
        if level > 256 {
            return Err("HTML heading level exceeds 256".into());
        }
        while html.numbers.len() < level {
            html.numbers.push(0);
        }
        html.numbers.truncate(level);
        html.numbers[level - 1] += 1;
        let range = super::render::headings::title_range(
            self.text,
            &self.nodes[id],
            &html.snapshot.options,
            &html.snapshot.todo,
        );
        let mut plain = self.text.byte_slice(range.clone()).to_string();
        let mut title = self.capture(html, range, 0)?;
        if html.snapshot.options.numbered {
            let number = html
                .numbers
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(".");
            title = format!("{number} {title}");
            plain = format!("{number} {plain}");
        }
        let anchor = self.heading_ids[&id].clone();
        html.identity(&anchor)?;
        html.headings.push((level, anchor.clone(), plain));
        let tag = level.min(6);
        html.emit(&format!("<h{tag} id=\"{anchor}\">{title}</h{tag}>\n"))?;
        self.children(html, id, depth)
    }
    fn paragraph(&self, html: &mut Html<'_>, id: usize) -> Result<(), String> {
        let n = &self.nodes[id];
        if self.definition(html, n)? {
            return Ok(());
        }
        let props = self.properties(html, id)?;
        props.figure(html)?;
        self.paragraph_content(html, n, &props)?;
        self.caption(html, props.caption.as_ref(), "figcaption")?;
        props.end_figure(html)
    }
    fn paragraph_content(
        &self,
        html: &mut Html<'_>,
        n: &structure::Raw,
        props: &Properties,
    ) -> Result<(), String> {
        if self.standalone_image(html, n, props)? {
            return Ok(());
        }
        props.open(html, "p")?;
        self.inline(html, n.content.clone(), 0)?;
        html.emit("</p>\n")
    }
    fn leaf(&self, html: &mut Html<'_>, n: &structure::Raw) -> Result<(), String> {
        match n.kind {
            StructureKind::FixedWidth | StructureKind::Planning | StructureKind::TableFormula => {
                html.emit("<pre>")?;
                html.text(&self.text.byte_slice(n.content.clone()).to_string())?;
                html.emit("</pre>\n")
            }
            StructureKind::Keyword => self.keyword(html, n),
            _ => Ok(()),
        }
    }
    fn keyword(&self, html: &mut Html<'_>, n: &structure::Raw) -> Result<(), String> {
        let key = document::keyword(self.text, n)
            .map(|(k, _)| k)
            .unwrap_or_default();
        if super::render::headings::known(&key) {
            return Ok(());
        }
        html.diagnostic("Unsupported HTML keyword retained literally")?;
        html.emit("<pre>")?;
        html.text(&self.text.byte_slice(n.source.clone()).to_string())?;
        html.emit("</pre>\n")
    }
}
fn encoded(value: &str) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    for byte in value.bytes() {
        write!(out, "{byte:02x}").expect("write string");
    }
    out
}

fn list_tag(ordered: bool) -> &'static str {
    if ordered { "ol" } else { "ul" }
}
