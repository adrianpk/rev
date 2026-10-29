use super::*;
use crate::org::inline as native;
pub(super) struct Inline<'a> {
    text: &'a Rope,
    cancel: &'a Cancellation,
    markdown: bool,
    limit: usize,
    pub notes: notes::Notes,
    pub diagnostics: Vec<String>,
}
impl<'a> Inline<'a> {
    pub fn new(text: &'a Rope, snapshot: &'a Snapshot, cancel: &'a Cancellation) -> Self {
        Self {
            text,
            cancel,
            markdown: snapshot.request.backend == ExportBackend::Markdown,
            limit: snapshot.export_bytes,
            notes: notes::Notes::default(),
            diagnostics: Vec::new(),
        }
    }
    pub fn range(&mut self, range: Range<usize>, depth: usize) -> Result<String, String> {
        check(self.cancel)?;
        if depth >= 32 {
            return Err("Export inline nesting limit reached".into());
        }
        let nodes = native::objects(
            self.text,
            range.clone(),
            InlineContext::Standard,
            self.cancel,
        )?;
        let mut out = String::new();
        let mut cursor = range.start;
        for n in nodes {
            check(self.cancel)?;
            if n.source.start < cursor {
                continue;
            }
            self.emit(&mut out, cursor, &n, depth)?;
            cursor = n.source.end;
        }
        self.plain(&mut out, cursor..range.end)?;
        Ok(out)
    }
    fn emit(
        &mut self,
        out: &mut String,
        start: usize,
        n: &native::Raw,
        depth: usize,
    ) -> Result<(), String> {
        self.plain(out, start..n.source.start)?;
        let value = self.object(n, depth + 1)?;
        append(out, &value, self.limit)
    }
    fn plain(&self, out: &mut String, range: Range<usize>) -> Result<(), String> {
        for chunk in self.text.byte_slice(range).chunks() {
            check(self.cancel)?;
            append(out, &literal(chunk, self.markdown), self.limit)?;
        }
        Ok(())
    }
    fn object(&mut self, n: &native::Raw, depth: usize) -> Result<String, String> {
        match n.kind {
            InlineKind::Link(_) => self.link(n, depth),
            InlineKind::FootnoteReference | InlineKind::FootnoteInline => self.note(n, depth),
            InlineKind::Code | InlineKind::Verbatim => self.code(n),
            InlineKind::Target | InlineKind::RadioTarget => self.target(n),
            InlineKind::Entity => Ok(self.text.byte_slice(n.source.clone()).to_string()),
            _ => {
                let value = self.range(n.content.clone(), depth)?;
                let (open, close) = markers(n.kind, self.markdown);
                Ok(format!("{open}{value}{close}"))
            }
        }
    }
    fn link(&mut self, n: &native::Raw, depth: usize) -> Result<String, String> {
        let target = self.text.byte_slice(n.content.clone()).to_string();
        if target.len() > 4096 {
            return Err("Export link target exceeds 4096 bytes".into());
        }
        let label = if let Some(r) = &n.detail {
            self.range(r.clone(), depth)?
        } else {
            literal(&target, self.markdown)
        };
        if self.markdown {
            Ok(format!("[{label}](<{}>)", destination(&target)))
        } else if label == target {
            Ok(target)
        } else {
            Ok(format!("{label} ({target})"))
        }
    }
    fn note(&mut self, n: &native::Raw, depth: usize) -> Result<String, String> {
        let label = self.text.byte_slice(n.content.clone()).to_string();
        if let Some(body) = &n.detail {
            let label = self.notes.label(&label);
            let body = self.range(body.clone(), depth)?;
            self.notes.add(label.clone(), body, self.limit)?;
            return self.notes.reference(label, false, self.markdown);
        }
        let definition = self
            .text
            .line_to_byte(self.text.byte_to_line(n.source.start))
            == n.source.start;
        self.notes.reference(label, definition, self.markdown)
    }
    fn code(&self, n: &native::Raw) -> Result<String, String> {
        let value = self.text.byte_slice(n.content.clone()).to_string();
        if !self.markdown {
            return Ok(value);
        }
        let fence = fence(&value, 1, self.cancel)?;
        let pad = if value.starts_with('`') || value.ends_with('`') {
            " "
        } else {
            ""
        };
        Ok(format!("{fence}{pad}{value}{pad}{fence}"))
    }
    fn target(&self, n: &native::Raw) -> Result<String, String> {
        let value = self.text.byte_slice(n.content.clone()).to_string();
        if !self.markdown {
            return Ok(format!("[{value}]"));
        }
        let id = value
            .replace('&', "&amp;")
            .replace('"', "&quot;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        let visible = if n.kind == InlineKind::RadioTarget {
            literal(&value, true)
        } else {
            String::new()
        };
        Ok(format!("<a id=\"{id}\"></a>{visible}"))
    }
}
pub(super) fn literal(value: &str, markdown: bool) -> String {
    if !markdown {
        return value.into();
    }
    let mut out = String::new();
    for c in value.chars() {
        if "\\`*_[]<>!#".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}
pub(super) fn fence(value: &str, minimum: usize, cancel: &Cancellation) -> Result<String, String> {
    let mut run = 0;
    let mut longest = 0;
    for (i, c) in value.chars().enumerate() {
        if i % 4096 == 0 {
            check(cancel)?;
        }
        if c == '`' {
            run += 1;
            longest = longest.max(run);
        } else {
            run = 0;
        }
    }
    if longest >= 128 {
        return Err("Export code fence exceeds 128 backticks".into());
    }
    Ok("`".repeat(minimum.max(longest + 1)))
}
fn markers(kind: InlineKind, markdown: bool) -> (&'static str, &'static str) {
    if !markdown {
        return ("", "");
    }
    match kind {
        InlineKind::Bold => ("**", "**"),
        InlineKind::Italic => ("*", "*"),
        InlineKind::Underline => ("<u>", "</u>"),
        InlineKind::Strike => ("~~", "~~"),
        _ => ("", ""),
    }
}
fn destination(value: &str) -> String {
    let mut out = String::new();
    for c in value.chars() {
        match c {
            ' ' => out.push_str("%20"),
            '<' => out.push_str("%3C"),
            '>' => out.push_str("%3E"),
            '\\' => out.push_str("%5C"),
            _ => out.push(c),
        }
    }
    out
}
