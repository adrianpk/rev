use super::*;
use std::collections::{BTreeMap, BTreeSet};
struct Note {
    label: String,
    label_range: Range<usize>,
    source: Range<usize>,
    definition: Option<Range<usize>>,
}
impl Document {
    fn notes(&self, cancel: &Cancellation) -> Result<Vec<Note>, String> {
        let mut notes = Vec::new();
        for object in &self.objects {
            check(cancel)?;
            if !matches!(
                object.kind,
                InlineKind::FootnoteReference | InlineKind::FootnoteInline
            ) {
                continue;
            }
            if object.content.len() > 1024 || notes.len() >= 4096 {
                return Err("Footnotes exceed 4096 records or 1024-byte labels".into());
            }
            let mut definition = object.detail.clone();
            if object.kind == InlineKind::FootnoteReference && self.column_zero(object.source.start)
            {
                definition =
                    Some(object.source.end..self.definition_end(object.source.end, cancel)?);
            }
            notes.push(Note {
                label: self.slice(object.content.clone()),
                label_range: object.content.clone(),
                source: object.source.clone(),
                definition,
            });
        }
        Ok(notes)
    }
    pub(super) fn footnote_context(
        &self,
        at: usize,
        cancel: &Cancellation,
    ) -> Result<Option<Output>, String> {
        let notes = self.notes(cancel)?;
        let Some(note) = self.selected_note(&notes, at) else {
            return Ok(None);
        };
        self.note_syntax(cancel)?;
        validate(&notes)?;
        follow(&notes, note).map(Some)
    }
    fn note_syntax(&self, cancel: &Cancellation) -> Result<(), String> {
        for node in self
            .nodes
            .iter()
            .filter(|n| n.context == Some(InlineContext::Standard))
        {
            check(cancel)?;
            let source = self.slice(node.content.clone());
            for (offset, _) in source.match_indices("[fn:") {
                check(cancel)?;
                let at = node.content.start + offset;
                if self.opaque_note(at) {
                    continue;
                }
                if !self.objects.iter().any(|o| {
                    o.source.start == at
                        && matches!(
                            o.kind,
                            InlineKind::FootnoteReference | InlineKind::FootnoteInline
                        )
                }) {
                    return Err("Malformed footnote reference; source unchanged".into());
                }
            }
        }
        Ok(())
    }
    fn opaque_note(&self, at: usize) -> bool {
        super::observe::protected_range(&self.protected, &(at..at))
            || self.objects.iter().any(|o| {
                o.source.contains(&at)
                    && matches!(
                        o.kind,
                        InlineKind::Code
                            | InlineKind::Verbatim
                            | InlineKind::Link(_)
                            | InlineKind::RadioTarget
                            | InlineKind::Target
                    )
            })
    }
    fn selected_note<'a>(&self, notes: &'a [Note], at: usize) -> Option<&'a Note> {
        notes.iter().find(|n| {
            at_note(n, at)
                || at == self.text.len_bytes() && n.definition.as_ref().is_some_and(|d| d.end == at)
        })
    }
    fn reference(&self, note: &Note) -> bool {
        note.definition.is_none()
            || !self.column_zero(note.source.start)
            || note.source.end > note.label_range.end + 1
    }
    fn column_zero(&self, at: usize) -> bool {
        self.text.line_to_byte(self.text.byte_to_line(at)) == at
    }
    fn definition_end(&self, start: usize, cancel: &Cancellation) -> Result<usize, String> {
        let row = self.text.byte_to_line(start);
        let mut blank = false;
        for r in row + 1..self.text.len_lines() {
            check(cancel)?;
            let at = self.text.line_to_byte(r);
            let line = self.text.line(r).to_string();
            if line.starts_with("[fn:") || heading(&line) {
                return Ok(at);
            }
            if line.trim().is_empty() && blank {
                return Ok(at);
            }
            blank = line.trim().is_empty();
        }
        Ok(self.text.len_bytes())
    }
    pub(super) fn footnote(
        &self,
        at: usize,
        force: bool,
        cancel: &Cancellation,
    ) -> Result<Output, String> {
        self.context_allowed(at)?;
        self.note_syntax(cancel)?;
        let notes = self.notes(cancel)?;
        validate(&notes)?;
        if !force && let Some(note) = self.selected_note(&notes, at) {
            return follow(&notes, note);
        }
        self.standard(&(at..at))?;
        if notes.iter().any(|n| n.source.contains(&at)) {
            return Err("Move outside the existing footnote before creating another".into());
        }
        self.new_note(at, &notes)
    }
    fn new_note(&self, at: usize, notes: &[Note]) -> Result<Output, String> {
        let used: BTreeSet<_> = notes.iter().map(|n| n.label.as_str()).collect();
        let number = (1..=4097)
            .find(|n| !used.contains(n.to_string().as_str()))
            .ok_or("Footnote label limit reached")?;
        let marker = format!("[fn:{number}]");
        let reference = if self.column_zero(at) {
            format!(" {marker}")
        } else {
            marker.clone()
        };
        let (end, section) = self.footnote_section();
        let nl = self.newline();
        let mut suffix = if section {
            format!("{nl}{nl}{marker} ")
        } else {
            format!("{nl}{nl}* Footnotes{nl}{nl}{marker} ")
        };
        let offset = suffix.chars().count();
        if end < self.text.len_bytes() {
            suffix.push_str(nl);
        }
        if at == end {
            let text = format!("{reference}{suffix}");
            let offset = reference.chars().count() + offset;
            return self.change(
                vec![Edit {
                    bytes: at..at,
                    text,
                }],
                Some((at, offset)),
            );
        }
        self.change(
            vec![
                Edit {
                    bytes: at..at,
                    text: reference,
                },
                Edit {
                    bytes: end..end,
                    text: suffix,
                },
            ],
            Some((end, offset)),
        )
    }
    fn footnote_section(&self) -> (usize, bool) {
        self.nodes
            .iter()
            .find(|n| {
                matches!(n.kind, StructureKind::Heading { level: 1 })
                    && self
                        .text
                        .line(self.text.byte_to_line(n.content.start))
                        .to_string()
                        .trim_end()
                        == "* Footnotes"
            })
            .map_or((self.text.len_bytes(), false), |n| (n.source.end, true))
    }
    pub(super) fn renumber(&self, cancel: &Cancellation) -> Result<Output, String> {
        self.note_syntax(cancel)?;
        let notes = self.notes(cancel)?;
        validate(&notes)?;
        let mut labels = BTreeMap::new();
        for note in notes.iter().filter(|n| self.reference(n)) {
            check(cancel)?;
            if numeric(&note.label) {
                let next = labels.len() + 1;
                labels.entry(note.label.clone()).or_insert(next);
            }
        }
        let edits = numbered_edits(&notes, &labels, cancel)?;
        self.change(edits, None)
    }
}
fn heading(line: &str) -> bool {
    let count = line.bytes().take_while(|c| *c == b'*').count();
    count > 0 && line.as_bytes().get(count) == Some(&b' ')
}
fn numeric(label: &str) -> bool {
    !label.is_empty() && label.bytes().all(|c| c.is_ascii_digit())
}
fn at_note(note: &Note, at: usize) -> bool {
    note.source.contains(&at) || note.definition.as_ref().is_some_and(|r| r.contains(&at))
}
fn validate(notes: &[Note]) -> Result<(), String> {
    let mut definitions = BTreeSet::new();
    for note in notes
        .iter()
        .filter(|n| n.definition.is_some() && !n.label.is_empty())
    {
        if !definitions.insert(&note.label) {
            return Err(format!("Ambiguous footnote definition: {}", note.label));
        }
    }
    for note in notes.iter().filter(|n| n.definition.is_none()) {
        if !definitions.contains(&note.label) {
            return Err(format!("Unresolved footnote reference: {}", note.label));
        }
    }
    Ok(())
}
fn follow(notes: &[Note], note: &Note) -> Result<Output, String> {
    if note.definition.is_none() {
        return notes
            .iter()
            .find(|n| n.label == note.label && n.definition.is_some())
            .map(|n| Output::Move(n.source.start))
            .ok_or_else(|| "Unresolved footnote reference".into());
    }
    if note.label.is_empty() {
        return Ok(Output::Move(
            note.definition.as_ref().expect("inline definition").start,
        ));
    }
    notes
        .iter()
        .find(|n| n.label == note.label && n.definition.is_none())
        .map(|n| Output::Move(n.source.start))
        .ok_or_else(|| "No separate reference for this inline footnote".into())
}

fn numbered_edits(
    notes: &[Note],
    labels: &BTreeMap<String, usize>,
    cancel: &Cancellation,
) -> Result<Vec<Edit>, String> {
    let mut edits = Vec::new();
    for note in notes {
        check(cancel)?;
        if numeric(&note.label) {
            let number = labels
                .get(&note.label)
                .ok_or("Unreferenced numeric footnote; renumber refused")?;
            edits.push(Edit {
                bytes: note.label_range.clone(),
                text: number.to_string(),
            });
        }
    }

    Ok(edits)
}
