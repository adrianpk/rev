use super::*;
use crate::org::{Coverage, Limits, StructureKind, structure};
use std::ops::Range;
pub(in crate::org::execution) struct Call {
    layers: crate::org::property::Properties,
    pub name: String,
    pub inside: String,
    pub end_headers: String,
    pub args: String,
    pub range: Range<usize>,
    pub inline: bool,
}
impl Call {
    pub fn observe(scan: &Scan, cancel: &Cancellation) -> Result<Self, String> {
        let text = scan.text();
        let document = structure::observe(text.clone(), Limits::default(), cancel);
        if document.coverage != Coverage::Complete {
            return Err("Incomplete call source".into());
        }
        let properties = crate::org::property::Properties::observe_nodes(
            document.nodes.iter(),
            true,
            text,
            cancel,
        );
        let node = document
            .nodes
            .iter()
            .rev()
            .find(|n| contains(n, scan.at(), text.len_bytes()))
            .ok_or("No call at point")?;
        if !matches!(node.kind, StructureKind::Keyword | StructureKind::Paragraph) {
            return Err("No executable named call at point".into());
        }
        admitted(text, &document, node)?;
        let line_index = text.byte_to_line(scan.at());
        let start = text.line_to_byte(line_index);
        let line = text.line(line_index).to_string();
        if line.len() > 4096 {
            return Err("Call line exceeds 4096 bytes".into());
        }
        let trimmed = line.trim_start();
        let offset = line.len() - trimmed.len();
        if trimmed
            .get(..7)
            .is_some_and(|s| s.eq_ignore_ascii_case("#+CALL:"))
        {
            let mut call = parse(trimmed[7..].trim(), false)?;
            call.layers = properties;
            call.range = start + offset..start + line.len();
            return Ok(call);
        }
        let mut call = inline(&line, start, scan.at())?;
        call.layers = properties;
        Ok(call)
    }
    pub fn restrictions(
        &self,
        scan: &Scan,
        language: &str,
        cancel: &Cancellation,
    ) -> Result<bool, String> {
        let mut headers = crate::org::block::Headers::default();
        for layer in self
            .layers
            .header_layers(scan.text(), self.range.start, language)?
        {
            headers.apply(&layer, cancel)?;
        }
        super::super::headers::evaluation(headers.get("eval"))
    }
    pub fn arguments(
        &self,
        definition: &mut Definition,
        cancel: &Cancellation,
    ) -> Result<(), String> {
        for value in super::super::variables::split(&self.args, ',')? {
            definition.headers.apply(&format!(":var {value}"), cancel)?;
        }
        Ok(())
    }
    pub fn library_name<'a>(&'a self, path: &Path, source: Option<&Path>) -> Option<&'a str> {
        let Some((file, name)) = self.name.rsplit_once(':') else {
            return Some(&self.name);
        };
        let requested = source
            .and_then(Path::parent)
            .unwrap_or(Path::new("."))
            .join(file);
        crate::file::resolve(&requested)
            .ok()
            .filter(|p| p.path() == path)
            .map(|_| name)
    }
}
fn parse(value: &str, inline: bool) -> Result<Call, String> {
    let end = value
        .find(['[', '('])
        .ok_or("Call requires name(arguments)")?;
    let name = &value[..end];
    if name.is_empty() || name.len() > 4096 || name.chars().any(char::is_whitespace) {
        return Err("Invalid named call identifier".into());
    }
    let (inside, rest) = optional(&value[end..], '[', ']')?;
    let (args, rest) = required(rest, '(', ')')?;
    let end_headers = if inline {
        let (headers, rest) = optional(rest, '[', ']')?;
        if !rest.is_empty() {
            return Err("Unexpected inline call suffix".into());
        }
        headers
    } else {
        rest.trim().into()
    };
    Ok(Call {
        layers: Default::default(),
        name: name.into(),
        inside,
        end_headers,
        args,
        range: 0..0,
        inline,
    })
}
fn optional(value: &str, open: char, close: char) -> Result<(String, &str), String> {
    if value.starts_with(open) {
        required(value, open, close)
    } else {
        Ok((String::new(), value))
    }
}
fn required(value: &str, open: char, close: char) -> Result<(String, &str), String> {
    if !value.starts_with(open) {
        return Err("Malformed call argument delimiters".into());
    }
    let mut quote = false;
    let mut escape = false;
    for (i, c) in value[1..].char_indices() {
        if escape {
            escape = false;
            continue;
        }
        if c == '\\' {
            escape = true;
            continue;
        }
        if c == '"' {
            quote = !quote;
        }
        if c == close && !quote {
            return Ok((value[1..i + 1].into(), &value[i + 2..]));
        }
        if c == open && !quote {
            return Err("Evaluated/nested call arguments are unsupported".into());
        }
    }
    Err("Unclosed named call arguments".into())
}
fn inline(line: &str, start: usize, at: usize) -> Result<Call, String> {
    for (i, _) in line.match_indices("call_") {
        if i > 0
            && line[..i]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_alphanumeric() || c == '_')
        {
            continue;
        }
        let tail = &line[i + 5..];
        let (_, rest) = required_tail(tail)?;
        let end = line.len() - rest.len();
        if at < start + i || at > start + end {
            continue;
        }
        let mut call = parse(&line[i + 5..end], true)?;
        call.range = start + i..start + end;
        return Ok(call);
    }
    Err("No named call at point".into())
}
fn required_tail(value: &str) -> Result<(String, &str), String> {
    let end = value.find(['[', '(']).ok_or("Malformed inline call")?;
    let (_, rest) = optional(&value[end..], '[', ']')?;
    let (_, rest) = required(rest, '(', ')')?;
    optional(rest, '[', ']')
}

pub(in crate::org::execution) fn at(scan: &Scan) -> bool {
    let text = scan.text();
    let index = text.byte_to_line(scan.at());
    let start = text.line_to_byte(index);
    let line = text.line(index).to_string();
    line.trim_start()
        .get(..7)
        .is_some_and(|s| s.eq_ignore_ascii_case("#+CALL:"))
        || inline(&line, start, scan.at()).is_ok()
}

fn admitted(
    text: &ropey::Rope,
    document: &structure::Document,
    node: &structure::Raw,
) -> Result<(), String> {
    let mut parent = node.parent;
    while let Some(index) = parent {
        let ancestor = &document.nodes[index];
        if text
            .byte_slice(ancestor.content.clone())
            .to_string()
            .split_whitespace()
            .next()
            == Some("COMMENT")
        {
            return Err("Calls under COMMENT are excluded from execution".into());
        }
        parent = ancestor.parent;
    }
    Ok(())
}

fn contains(node: &structure::Raw, at: usize, end: usize) -> bool {
    node.source.contains(&at) || at == end && node.source.end == at
}
