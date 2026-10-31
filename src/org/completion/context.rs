use super::*;
use crate::org::{BlockKind, InlineContext, InlineKind, StructureKind};

#[derive(Default)]
struct Position {
    heading: Option<Range<usize>>,
    property: bool,
    standard: bool,
    literal: bool,
    fixed: bool,
}
impl Position {
    fn observe(
        structure: &Structure,
        text: &ropey::Rope,
        byte: usize,
        cancel: &Cancellation,
    ) -> Result<Self, String> {
        let mut position = Self::default();
        for node in structure.raw_nodes() {
            check(cancel)?;
            if protected(node, text, byte) {
                position.literal = true;
            }
            if within(&node.content, byte) {
                position.collect(node);
            }
        }
        Ok(position)
    }
    fn local(&self, line: &str, at: usize, base: usize) -> Option<Local> {
        if self.property {
            return property(line, at);
        }
        if self.fixed {
            return None;
        }
        if let Some(heading) = &self.heading {
            return heading_target(line, at, heading.start - base);
        }
        directive(line, at)
    }
    fn links(&self) -> bool {
        self.standard && !self.property && !self.fixed
    }
    fn collect(&mut self, node: &crate::org::structure::Raw) {
        self.property |= node.property_candidate;
        match node.kind {
            StructureKind::Heading { .. } => self.heading = Some(node.content.clone()),
            StructureKind::PropertyDrawer => self.property = true,
            StructureKind::FixedWidth => self.fixed = true,
            _ => {}
        }
        self.standard |= node.context == Some(InlineContext::Standard);
    }
}
fn within(range: &Range<usize>, byte: usize) -> bool {
    range.start <= byte && byte <= range.end
}
fn protected(node: &crate::org::structure::Raw, text: &ropey::Rope, byte: usize) -> bool {
    if node.kind == StructureKind::Malformed && node.context == Some(InlineContext::Literal) {
        // The incomplete opener may itself be a directive target. Its body
        // retains literal protection through EOF, including unclosed blocks.
        return byte >= node.source.start
            && text.byte_to_line(byte) != text.byte_to_line(node.source.start);
    }
    if !within(&node.source, byte) {
        return false;
    }
    match node.kind {
        StructureKind::Block(BlockKind::Greater) => !node.name.as_ref().is_some_and(|name| {
            let name = text.byte_slice(name.clone()).to_string();
            name.eq_ignore_ascii_case("QUOTE") || name.eq_ignore_ascii_case("CENTER")
        }),
        StructureKind::Block(BlockKind::Verse) => false,
        StructureKind::Block(_) | StructureKind::Comment => true,
        _ => false,
    }
}

pub(super) fn at(
    structure: &Structure,
    text: &ropey::Rope,
    byte: usize,
    cancel: &Cancellation,
) -> Result<Option<Target>, String> {
    let position = Position::observe(structure, text, byte, cancel)?;
    if position.literal {
        return Ok(None);
    }
    let row = text.byte_to_line(byte);
    let line = text.line(row);
    if line.len_bytes() > 4096 {
        return Err("Native Org completion line limit reached".into());
    }
    let base = text.line_to_byte(row);
    let line = line.to_string();
    let at = byte - base;
    let local = position.local(&line, at, base);
    let local = match local {
        Some(local) => Some(local),
        None if position.links() => link(text, &line, base, at, cancel)?,
        _ => None,
    };
    local
        .map(|(range, kind)| target(&line, base, at, range, kind))
        .transpose()
}

type Local = (Range<usize>, Kind);
fn property(line: &str, at: usize) -> Option<Local> {
    let start = line.len() - line.trim_start().len();
    if !line[start..].starts_with(':') {
        return None;
    }
    word(line, at, start + 1, Kind::Property)
}
fn directive(line: &str, at: usize) -> Option<Local> {
    let start = line.len() - line.trim_start().len();
    if !line[start..].starts_with("#+") {
        return None;
    }
    word(line, at, start + 2, Kind::Directive)
}
fn heading_target(line: &str, at: usize, title: usize) -> Option<Local> {
    if at < title {
        return None;
    }
    let prefix = &line[title..at];
    let start = prefix
        .char_indices()
        .rev()
        .find(|(_, ch)| ch.is_whitespace())
        .map_or(title, |(offset, ch)| title + offset + ch.len_utf8());
    if line[start..at].starts_with(':') {
        let tag = line[start..at].rfind(':')? + start + 1;
        return word(line, at, tag, Kind::Tag);
    }
    if !prefix.chars().any(char::is_whitespace) {
        return word(line, at, title, Kind::Todo);
    }
    None
}
fn word(line: &str, at: usize, start: usize, kind: Kind) -> Option<Local> {
    if at < start {
        return None;
    }
    let end = line[start..]
        .find(|ch: char| ch == ':' || ch.is_whitespace() || (kind == Kind::Link && ch == ']'))
        .map_or(line.len(), |end| start + end);
    if at > end {
        return None;
    }
    Some((start..end, kind))
}
fn link(
    text: &ropey::Rope,
    line: &str,
    base: usize,
    at: usize,
    cancel: &Cancellation,
) -> Result<Option<Local>, String> {
    let Some(start) = line[..at].rfind("[[") else {
        return Ok(None);
    };
    if line[..start]
        .chars()
        .rev()
        .take_while(|ch| *ch == '\\')
        .count()
        % 2
        == 1
    {
        return Ok(None);
    }
    let objects = crate::org::inline::objects(
        text,
        base..base + line.len(),
        InlineContext::Standard,
        cancel,
    )?;
    if objects.iter().any(|object| opaque(object, base + at)) {
        return Ok(None);
    }
    let start = start + 2;
    if line[start..at].contains([']', '\n', '\r']) {
        return Ok(None);
    }
    if line[start..].starts_with('*') {
        let start = start + 1;
        if at < start {
            return Ok(None);
        }
        let end = line[start..]
            .find([']', '\n', '\r'])
            .map_or(line.len(), |end| start + end);
        return Ok(Some((start..end, Kind::Heading)));
    }
    Ok(word(line, at, start, Kind::Link))
}
fn opaque(object: &crate::org::inline::Raw, byte: usize) -> bool {
    if matches!(object.kind, InlineKind::Code | InlineKind::Verbatim)
        && within(&object.source, byte)
    {
        return true;
    }
    object
        .detail
        .as_ref()
        .is_some_and(|detail| within(detail, byte))
}
fn target(
    line: &str,
    base: usize,
    at: usize,
    range: Range<usize>,
    kind: Kind,
) -> Result<Target, String> {
    if range.len() > MAX_LABEL {
        return Err("Native completion label limit reached".into());
    }
    let closing = line[range.end..].starts_with(':');
    let mut original = line[range.clone()].to_owned();
    let mut prefix = line[range.start..at].to_owned();
    if kind == Kind::Heading {
        prefix = crate::org::links::decode(&prefix)?;
        original = crate::org::links::decode(&original)?;
    }
    if !closing && kind.colon(&original) {
        original.clear();
    }
    Ok(Target {
        range: base + range.start..base + range.end,
        prefix,
        original,
        kind,
        closing,
        symbols: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // An incomplete property key retains its surrounding native drawer context;
    // the failed token never becomes authority to invent an observed name.
    #[test]
    fn partial_property() {
        let source = "* One\n:PROPERTIES:\n:OWNER: Ada\n:END:\n* Two\n:PROPERTIES:\n:OW\n:END:\n";
        let mut buffers = crate::buffer::Buffers::new().unwrap();
        let id = buffers.create(source).unwrap();
        let buffer = buffers.get(id).unwrap();
        let cancel = Cancellation::default();
        let structure = StructureScan::new(buffer, Limits::default())
            .unwrap()
            .run(&cancel);
        let text: ropey::Rope = buffer
            .slice(&buffer.span(0..buffer.len_chars()).unwrap())
            .unwrap()
            .into();
        let nodes: Vec<_> = structure
            .raw_nodes()
            .map(|node| (node.kind, node.content.clone(), node.name.clone()))
            .collect();
        let byte = source.rfind(":OW\n").unwrap() + 3;
        let target = at(&structure, &text, byte, &cancel)
            .unwrap()
            .map(|target| target.kind);
        assert_eq!(target, Some(Kind::Property), "{nodes:?}");
    }
}
