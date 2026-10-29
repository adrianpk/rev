//! Native TeX is escaped data; rendering cannot launch a compiler.
mod blocks;
mod inline;
mod math;
mod notes;
use super::*;
use crate::org::{BlockKind, InlineKind};
use std::collections::{BTreeMap, BTreeSet};
struct Tex<'a> {
    snapshot: &'a Snapshot,
    cancel: &'a Cancellation,
    output: String,
    diagnostics: Vec<String>,
    frame: bool,
    ordered: usize,
    note_count: usize,
    used: BTreeMap<String, usize>,
    active: BTreeSet<String>,
    observed: usize,
}
struct Document<'a> {
    text: &'a Rope,
    nodes: &'a [structure::Raw],
    children: Vec<Vec<usize>>,
    roots: Vec<usize>,
    base: usize,
    notes: BTreeMap<String, Range<usize>>,
}
pub(super) fn render(snapshot: &Snapshot, cancel: &Cancellation) -> Result<Rendered, String> {
    let doc = Document::new(&snapshot.text, &snapshot.nodes, snapshot, cancel)?;
    let mut tex = Tex {
        snapshot,
        cancel,
        output: String::new(),
        diagnostics: snapshot.diagnostics.clone(),
        frame: false,
        ordered: 0,
        note_count: 0,
        used: BTreeMap::new(),
        active: BTreeSet::new(),
        observed: snapshot.nodes.len(),
    };
    tex.preamble()?;
    doc.body(&mut tex, 0)?;
    tex.emit("\\end{document}\n")?;
    Ok(Rendered {
        text: tex.output,
        diagnostics: tex.diagnostics,
    })
}
impl Tex<'_> {
    fn beamer(&self) -> bool {
        self.snapshot.request.backend == ExportBackend::Beamer
    }
    fn emit(&mut self, value: &str) -> Result<(), String> {
        document::check(self.cancel)?;
        if value.len() > self.snapshot.export_bytes.saturating_sub(self.output.len()) {
            return Err("TeX export output byte limit reached".into());
        }
        self.output.push_str(value);
        Ok(())
    }
    fn text(&mut self, value: &str) -> Result<(), String> {
        for c in value.replace("\r\n", "\n").replace('\r', "\n").chars() {
            let mut bytes = [0; 4];
            self.emit(escape(c).unwrap_or_else(|| c.encode_utf8(&mut bytes)))?;
        }
        Ok(())
    }
    fn diagnostic(&mut self, message: &str) -> Result<(), String> {
        if self.diagnostics.len() == 128 {
            return Err("TeX diagnostic limit reached".into());
        }
        self.diagnostics.push(message.into());
        Ok(())
    }
    fn preamble(&mut self) -> Result<(), String> {
        let class = if self.beamer() { "beamer" } else { "article" };
        self.emit(&format!("\\documentclass{{{class}}}\n"))?;
        self.emit("\\usepackage[T1]{fontenc}\n\\usepackage[utf8]{inputenc}\n\\usepackage{textcomp}\n\\usepackage{amsmath}\n\\usepackage{hyperref}\n")?;
        self.metadata()?;
        self.emit("\\begin{document}\n")?;
        self.frontmatter()
    }
    fn metadata(&mut self) -> Result<(), String> {
        let m = &self.snapshot.metadata;
        for (name, value, enabled) in [
            ("title", &m.title, self.snapshot.options.title),
            ("author", &m.author, self.snapshot.options.author),
            ("date", &m.date, true),
        ] {
            self.emit(&format!("\\{name}{{"))?;
            if let Some(value) = value.as_ref().filter(|_| enabled) {
                self.text(value)?;
            }
            self.emit("}\n")?;
        }
        Ok(())
    }
    fn frontmatter(&mut self) -> Result<(), String> {
        if matches!(self.snapshot.request.scope, Scope::Region(_)) {
            return Ok(());
        }
        self.title_page()?;
        self.contents()
    }
    fn title_page(&mut self) -> Result<(), String> {
        if self.snapshot.options.title && self.snapshot.metadata.title.is_some() {
            let value = if self.beamer() {
                "\\begin{frame}\n\\titlepage\n\\end{frame}\n"
            } else {
                "\\maketitle\n"
            };
            self.emit(value)?;
        }
        Ok(())
    }
    fn contents(&mut self) -> Result<(), String> {
        if self.snapshot.options.toc {
            let value = if self.beamer() {
                "\\begin{frame}{Contents}\n\\tableofcontents\n\\end{frame}\n"
            } else {
                "\\tableofcontents\n"
            };
            self.emit(value)?;
        }
        Ok(())
    }
    fn literal(&mut self, value: &str) -> Result<(), String> {
        self.emit("\\begin{quote}\\ttfamily\n")?;
        for line in value.lines() {
            self.literal_line(line)?;
        }
        self.emit("\\end{quote}\n")
    }
    fn literal_line(&mut self, line: &str) -> Result<(), String> {
        self.emit("\\noindent ")?;
        for c in line.chars() {
            match c {
                ' ' => self.emit("\\ ")?,
                '\t' => self.emit("\\ \\ \\ \\ \\ \\ \\ \\ ")?,
                _ => self.text(&c.to_string())?,
            }
        }
        self.emit("\\par\n")
    }
}
impl<'a> Document<'a> {
    fn new(
        text: &'a Rope,
        nodes: &'a [structure::Raw],
        snapshot: &Snapshot,
        cancel: &Cancellation,
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
            notes: BTreeMap::new(),
        };
        doc.definitions(cancel)?;
        Ok(doc)
    }
    fn body(&self, tex: &mut Tex<'_>, depth: usize) -> Result<(), String> {
        for id in &self.roots {
            self.node(tex, *id, depth)?;
        }
        Ok(())
    }
    fn children(&self, tex: &mut Tex<'_>, id: usize, depth: usize) -> Result<(), String> {
        for child in &self.children[id] {
            self.node(tex, *child, depth + 1)?;
        }
        Ok(())
    }
    fn node(&self, tex: &mut Tex<'_>, id: usize, depth: usize) -> Result<(), String> {
        document::check(tex.cancel)?;
        if depth >= super::super::Limits::default().depth {
            return Err("TeX nesting limit reached".into());
        }
        let n = &self.nodes[id];
        if tex.beamer() && !tex.frame && leaf_frame(n.kind) {
            return self.leaf_frame(tex, id, depth);
        }
        self.element(tex, id, depth)
    }
    fn element(&self, tex: &mut Tex<'_>, id: usize, depth: usize) -> Result<(), String> {
        let n = &self.nodes[id];
        match n.kind {
            StructureKind::Heading { level } => self.heading(tex, id, level, depth),
            StructureKind::Section => self.children(tex, id, depth),
            StructureKind::List { ordered } => self.list(tex, id, ordered, depth),
            StructureKind::Item { .. } => {
                tex.emit("\\item ")?;
                self.children(tex, id, depth)
            }
            StructureKind::Paragraph => self.paragraph(tex, n),
            StructureKind::Table => self.table(tex, id),
            StructureKind::Block(kind) => self.block(tex, id, kind, depth),
            _ => self.leaf(tex, n),
        }
    }
    fn paragraph(&self, tex: &mut Tex<'_>, n: &structure::Raw) -> Result<(), String> {
        if self.definition(n, tex.cancel)?.is_some() {
            return Ok(());
        }
        self.inline(tex, n.content.clone(), 0)?;
        tex.emit("\n\n")
    }
    fn leaf(&self, tex: &mut Tex<'_>, n: &structure::Raw) -> Result<(), String> {
        match n.kind {
            StructureKind::FixedWidth | StructureKind::Planning | StructureKind::TableFormula => {
                tex.literal(&self.text.byte_slice(n.content.clone()).to_string())
            }
            StructureKind::Keyword => self.keyword(tex, n),
            _ => Ok(()),
        }
    }
    fn keyword(&self, tex: &mut Tex<'_>, n: &structure::Raw) -> Result<(), String> {
        let (key, _) = document::keyword(self.text, n).ok_or("TeX keyword has no name")?;
        if super::render::headings::known(&key) {
            return Ok(());
        }
        tex.diagnostic("Unsupported TeX keyword retained as escaped data")?;
        tex.literal(&self.text.byte_slice(n.source.clone()).to_string())
    }
    fn heading(
        &self,
        tex: &mut Tex<'_>,
        id: usize,
        level: usize,
        depth: usize,
    ) -> Result<(), String> {
        let level = level.saturating_sub(self.base).max(1);
        let title = super::render::headings::title_range(
            self.text,
            &self.nodes[id],
            &tex.snapshot.options,
            &tex.snapshot.todo,
        );
        if tex.beamer() {
            return self.beamer_heading(tex, (id, level), title, depth);
        }
        self.section(tex, level, title)?;
        self.children(tex, id, depth)
    }
    fn section(&self, tex: &mut Tex<'_>, level: usize, title: Range<usize>) -> Result<(), String> {
        let name = [
            "section",
            "subsection",
            "subsubsection",
            "paragraph",
            "subparagraph",
        ][level.saturating_sub(1).min(4)];
        let star = if tex.snapshot.options.numbered {
            ""
        } else {
            "*"
        };
        tex.emit(&format!("\\{name}{star}{{"))?;
        self.inline(tex, title, 0)?;
        tex.emit("}\n")
    }
    fn beamer_heading(
        &self,
        tex: &mut Tex<'_>,
        heading: (usize, usize),
        title: Range<usize>,
        depth: usize,
    ) -> Result<(), String> {
        let (id, level) = heading;
        if level < tex.snapshot.options.frame_level {
            self.section(tex, level, title)?;
            return self.children(tex, id, depth);
        }
        let old = tex.frame;
        let (open, close) = if old {
            ("\\begin{block}{", "\\end{block}\n")
        } else {
            ("\\begin{frame}[fragile]{", "\\end{frame}\n")
        };
        tex.emit(open)?;
        self.inline(tex, title, 0)?;
        tex.emit("}\n")?;
        tex.frame = true;
        self.children(tex, id, depth)?;
        tex.frame = old;
        tex.emit(close)
    }
    fn leaf_frame(&self, tex: &mut Tex<'_>, id: usize, depth: usize) -> Result<(), String> {
        tex.emit("\\begin{frame}[fragile]\n")?;
        tex.frame = true;
        self.element(tex, id, depth)?;
        tex.frame = false;
        tex.emit("\\end{frame}\n")
    }
}
fn leaf_frame(kind: StructureKind) -> bool {
    matches!(
        kind,
        StructureKind::Paragraph
            | StructureKind::List { .. }
            | StructureKind::Table
            | StructureKind::FixedWidth
            | StructureKind::Block(
                BlockKind::Source
                    | BlockKind::Example
                    | BlockKind::Export
                    | BlockKind::Verse
                    | BlockKind::Greater
                    | BlockKind::Dynamic
            )
    )
}
fn escape(c: char) -> Option<&'static str> {
    match c {
        '\\' => Some("\\textbackslash{}"),
        '{' => Some("\\{"),
        '}' => Some("\\}"),
        '$' => Some("\\$"),
        '%' => Some("\\%"),
        '#' => Some("\\#"),
        '&' => Some("\\&"),
        '_' => Some("\\_"),
        _ => accent(c),
    }
}
fn accent(c: char) -> Option<&'static str> {
    match c {
        '~' => Some("\\textasciitilde{}"),
        '^' => Some("\\textasciicircum{}"),
        _ => None,
    }
}
