//! Manual formats consume checked snapshots and emit escaped data only.
mod blocks;
mod headings;
mod inline;
use super::*;
use crate::org::{BlockKind, InlineKind};
struct Manual<'a> {
    snapshot: &'a Snapshot,
    cancel: &'a Cancellation,
    output: String,
    diagnostics: Vec<String>,
    numbers: Vec<usize>,
}
struct Document<'a> {
    text: &'a Rope,
    nodes: &'a [structure::Raw],
    children: Vec<Vec<usize>>,
    roots: Vec<usize>,
    levels: Vec<usize>,
}
pub(super) fn render(snapshot: &Snapshot, cancel: &Cancellation) -> Result<Rendered, String> {
    let doc = Document::new(&snapshot.text, &snapshot.nodes, cancel)?;
    let mut out = Manual {
        snapshot,
        cancel,
        output: String::new(),
        diagnostics: snapshot.diagnostics.clone(),
        numbers: Vec::new(),
    };
    out.preamble()?;
    doc.body(&mut out, 0)?;
    if out.texinfo() {
        out.emit("@bye\n")?;
    }
    Ok(Rendered {
        text: out.output,
        diagnostics: out.diagnostics,
    })
}
impl Manual<'_> {
    fn texinfo(&self) -> bool {
        self.snapshot.request.backend == ExportBackend::Texinfo
    }
    fn emit(&mut self, value: &str) -> Result<(), String> {
        document::check(self.cancel)?;
        if value.len() > self.snapshot.export_bytes.saturating_sub(self.output.len()) {
            return Err("Manual export output byte limit reached".into());
        }
        self.output.push_str(value);
        Ok(())
    }
    fn text(&mut self, value: &str) -> Result<(), String> {
        for c in value.replace("\r\n", "\n").replace('\r', "\n").chars() {
            if c.is_control() && !['\n', '\t'].contains(&c) {
                return Err("Manual export rejects formatter control scalars".into());
            }
            let mut bytes = [0; 4];
            let escaped = if self.texinfo() {
                tex_escape(c)
            } else {
                man_escape(c)
            };
            self.emit(escaped.unwrap_or_else(|| c.encode_utf8(&mut bytes)))?;
        }
        Ok(())
    }
    fn line(&mut self, value: &str) -> Result<(), String> {
        if value.contains(['\r', '\n', '\t']) {
            return Err("Manual header requires a single line".into());
        }
        self.text(value)
    }
    fn diagnostic(&mut self, message: &str) -> Result<(), String> {
        if self.diagnostics.len() >= 128 {
            return Err("Manual diagnostic limit reached".into());
        }
        self.diagnostics.push(message.into());
        Ok(())
    }
    fn literal(&mut self, value: &str) -> Result<(), String> {
        let (start, end) = if self.texinfo() {
            ("@example\n", "@end example\n")
        } else {
            (".nf\n", ".fi\n")
        };
        self.emit(start)?;
        self.text(value)?;
        if !value.ends_with('\n') {
            self.emit("\n")?;
        }
        self.emit(end)
    }
}
impl<'a> Document<'a> {
    fn new(
        text: &'a Rope,
        nodes: &'a [structure::Raw],
        cancel: &Cancellation,
    ) -> Result<Self, String> {
        let mut children = vec![Vec::new(); nodes.len()];
        let mut roots = Vec::new();
        let mut levels = vec![0; nodes.len()];
        for (i, n) in nodes.iter().enumerate() {
            document::check(cancel)?;
            if let Some(parent) = n.parent {
                children[parent].push(i);
                levels[i] = levels[parent];
            } else {
                roots.push(i);
            }
            levels[i] += usize::from(matches!(n.kind, StructureKind::Heading { .. }));
        }
        Ok(Self {
            text,
            nodes,
            children,
            roots,
            levels,
        })
    }
    fn body(&self, out: &mut Manual<'_>, depth: usize) -> Result<(), String> {
        for id in &self.roots {
            self.node(out, *id, depth)?;
        }
        Ok(())
    }
    fn children(&self, out: &mut Manual<'_>, id: usize, depth: usize) -> Result<(), String> {
        for child in &self.children[id] {
            self.node(out, *child, depth + 1)?;
        }
        Ok(())
    }
    fn node(&self, out: &mut Manual<'_>, id: usize, depth: usize) -> Result<(), String> {
        document::check(out.cancel)?;
        if depth >= super::super::Limits::default().depth {
            return Err("Manual nesting limit reached".into());
        }
        match self.nodes[id].kind {
            StructureKind::Heading { .. } => self.heading(out, id, depth),
            StructureKind::Section => self.children(out, id, depth),
            StructureKind::List { ordered } => self.list(out, id, ordered, depth),
            StructureKind::Item { .. } => self.item(out, id, depth),
            StructureKind::Paragraph => self.paragraph(out, &self.nodes[id]),
            StructureKind::Block(kind) => self.block(out, id, kind, depth),
            _ => self.leaf(out, id),
        }
    }
    fn paragraph(&self, out: &mut Manual<'_>, n: &structure::Raw) -> Result<(), String> {
        if !out.texinfo() {
            out.emit(".PP\n")?;
        }
        self.inline(out, n.content.clone(), 0, InlineContext::Standard)?;
        out.emit("\n\n")
    }
    fn leaf(&self, out: &mut Manual<'_>, id: usize) -> Result<(), String> {
        let n = &self.nodes[id];
        match n.kind {
            StructureKind::Table => {
                out.literal(&self.text.byte_slice(n.source.clone()).to_string())
            }
            StructureKind::FixedWidth | StructureKind::Planning | StructureKind::TableFormula => {
                out.literal(&self.text.byte_slice(n.content.clone()).to_string())
            }
            StructureKind::Keyword => self.keyword(out, n),
            _ => Ok(()),
        }
    }
    fn keyword(&self, out: &mut Manual<'_>, n: &structure::Raw) -> Result<(), String> {
        let (key, _) = document::keyword(self.text, n).ok_or("Manual keyword has no name")?;
        if key == "CAPTION" {
            return self.paragraph(out, n);
        }
        if super::render::headings::known(&key) && !key.starts_with("ATTR_") {
            return Ok(());
        }
        out.diagnostic("Unsupported manual keyword retained literally")?;
        out.literal(&self.text.byte_slice(n.source.clone()).to_string())
    }
}
fn tex_escape(c: char) -> Option<&'static str> {
    match c {
        '@' => Some("@@"),
        '{' => Some("@{"),
        '}' => Some("@}"),
        ',' => Some("@comma{}"),
        '\\' => Some("@backslashchar{}"),
        _ => None,
    }
}
fn man_escape(c: char) -> Option<&'static str> {
    match c {
        '\\' => Some("\\[rs]"),
        '"' => Some("\\[dq]"),
        '\'' => Some("\\[aq]"),
        '.' => Some("\\&."),
        _ => None,
    }
}
