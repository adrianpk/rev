use std::ops::Range;

use ropey::RopeSlice;

use super::{Kind, Marker, Observation};

/// Borrowed canonical lines and the matching complete base's physical row.
pub(crate) struct PresentationEdit<'a> {
    pub line: Range<usize>,
    pub before: RopeSlice<'a>,
    pub after: RopeSlice<'a>,
    pub changed: usize,
}

/// Display coordinates only; this carries no editable item/subtree authority.
pub(crate) struct DisplayItem {
    pub marker: Range<usize>,
    pub depth: usize,
    pub bullet: Option<u8>,
}

impl Observation {
    pub(crate) fn admit_presentation(
        &self,
        edit: &PresentationEdit<'_>,
        remaining: &mut usize,
    ) -> Option<()> {
        let (before, after) = lines(edit, remaining, self.nodes.len())?;
        if let Some((_, Some(prefix))) = self.presentation_heading(edit.line.clone()) {
            let end = prefix.end.checked_sub(edit.line.start)?;
            return title(&before, &after, end, edit.changed);
        }
        self.body_context(edit.line.clone())?;
        let old = body(&before)?;
        let new = body(&after)?;
        (old == new).then_some(())
    }

    fn body_context(&self, line: Range<usize>) -> Option<()> {
        let mut paragraph = false;
        for node in self
            .nodes
            .iter()
            .filter(|n| n.bytes.start < line.end && n.bytes.end > line.start)
        {
            match node.kind {
                Kind::Paragraph => paragraph = true,
                Kind::Semantic(
                    crate::syntax::MarkdownRole::Emphasis | crate::syntax::MarkdownRole::Link,
                ) => (),
                _ => return None,
            }
        }
        paragraph.then_some(())
    }

    pub(crate) fn presentation_heading(
        &self,
        line: Range<usize>,
    ) -> Option<(u8, Option<Range<usize>>)> {
        self.nodes.iter().find_map(|node| match &node.kind {
            Kind::Heading { level, prefix }
                if node.bytes.start < line.end && node.bytes.end > line.start =>
            {
                Some((*level, prefix.clone()))
            }
            _ => None,
        })
    }

    pub(crate) fn presentation_items(
        &self,
        line: Range<usize>,
    ) -> impl Iterator<Item = DisplayItem> + '_ {
        self.items()
            .iter()
            .filter(move |item| line.contains(&item.marker.start))
            .map(|item| DisplayItem {
                marker: item.marker.clone(),
                depth: item.depth,
                bullet: match item.form {
                    Marker::Bullet(byte) => Some(byte),
                    Marker::Ordered { .. } => None,
                },
            })
    }
}

fn lines(
    edit: &PresentationEdit<'_>,
    remaining: &mut usize,
    nodes: usize,
) -> Option<(String, String)> {
    // Shape inspection reserved the two copies. Reserve three additional full
    // passes (leading whitespace/nonblank, structural guards, prefix equality).
    let bytes = edit
        .before
        .len_bytes()
        .checked_add(edit.after.len_bytes())?;
    // Two existing-node lookups also consume allowance, bounding metadata work
    // without adding another whole-document index to the retained observation.
    let probes = bytes.checked_mul(3)?.checked_add(nodes.checked_mul(2)?)?;
    *remaining = remaining.checked_sub(probes)?;
    Some((line_text(edit.before)?, line_text(edit.after)?))
}

fn line_text(line: RopeSlice<'_>) -> Option<String> {
    let mut text = String::new();
    text.try_reserve_exact(line.len_bytes()).ok()?;
    for chunk in line.chunks() {
        text.push_str(chunk);
    }
    Some(text)
}

// This is only a refusal filter over parser-admitted paragraphs. It does not
// recognize a heading, list, table, definition or container from source text.
fn body(line: &str) -> Option<&str> {
    let indent = line
        .bytes()
        .take_while(|byte| matches!(byte, b' ' | b'\t'))
        .count();
    let prefix = line.get(..indent)?;
    if indent > 3 || prefix.contains('\t') {
        return None;
    }
    let text = line
        .get(indent..)?
        .trim_end_matches([' ', '\t', '\r', '\n']);
    let first = *text.as_bytes().first()?;
    if first.is_ascii_digit()
        || b"#>-+*_=~`[<|!\\:'\"".contains(&first)
        || text.contains('|')
        || text.starts_with('\u{feff}')
    {
        return None;
    }
    Some(prefix)
}

fn title(before: &str, after: &str, end: usize, changed: usize) -> Option<()> {
    let separator = before.as_bytes().get(end)?;
    let prefix = end.checked_add(1)?;
    (matches!(separator, b' ' | b'\t')
        && changed >= prefix
        && before.get(..prefix) == after.get(..prefix))
    .then_some(())
}
