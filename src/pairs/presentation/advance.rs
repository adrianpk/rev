//! Fixed-size proof prepared before native source installation. No second copy
//! of source or endpoint arrays escapes the buffer's canonical transaction owner.
use ropey::{Rope, RopeSlice};

use crate::buffer::{
    Stamp,
    edit::{Change, Transaction},
};
use crate::pairs::{MAX_BYTES, Profile, presentation::store::Snapshot};

#[derive(Clone, Copy, Default)]
struct Delta {
    start: usize,
    end: usize,
    output_end: usize,
}

pub(crate) struct Advance {
    pub publication: u64,
    pub before: Stamp,
    pub after: Stamp,
    deltas: [Delta; 16],
    count: usize,
}

impl Advance {
    pub(in crate::pairs) fn prepare(
        source: (&Snapshot, u64),
        record: &Transaction,
        texts: (&Rope, &Rope),
    ) -> Option<Self> {
        let (snapshot, publication) = source;
        if snapshot.stamp != record.before {
            return None;
        }
        if record.changes().len() > 16 || snapshot.profile == Profile::Html {
            return None;
        }
        let mut proof = Self {
            publication,
            before: record.before,
            after: record.after,
            deltas: [Delta::default(); 16],
            count: 0,
        };
        let mut budget = MAX_BYTES;
        for change in record.changes() {
            validate_change(snapshot, change, texts, &mut budget)?;
            let before = change.before();
            let after = change.after();
            proof.deltas[proof.count] = Delta {
                start: before.start,
                end: before.end,
                output_end: after.end,
            };
            proof.count += 1;
        }
        proof.validate_snapshot(snapshot)?;
        Some(proof)
    }

    fn validate_snapshot(&self, snapshot: &Snapshot) -> Option<()> {
        for endpoint in &snapshot.spans {
            self.map(endpoint.at.0)?;
            self.map(endpoint.mate.0)?;
        }
        self.map(snapshot.inspected)?;
        self.map(snapshot.target())?;
        Some(())
    }

    pub(in crate::pairs) fn map(&self, position: usize) -> Option<usize> {
        let mut previous = None;
        for delta in &self.deltas[..self.count] {
            if position < delta.start {
                break;
            }
            if position < delta.end {
                return None;
            }
            previous = Some(delta);
        }
        let Some(delta) = previous else {
            return Some(position);
        };
        delta
            .output_end
            .checked_add(position.checked_sub(delta.end)?)
    }
}

fn validate_change(
    snapshot: &Snapshot,
    change: &Change,
    texts: (&Rope, &Rope),
    budget: &mut usize,
) -> Option<()> {
    let before = change.before();
    if !contained(snapshot, &before) {
        return None;
    }
    neutral_payload(change.removed(), budget)?;
    neutral_payload(change.inserted(), budget)?;
    interior_word(texts.0.slice(..), before, budget)?;
    interior_word(texts.1.slice(..), change.after(), budget)?;
    prose_change(snapshot.profile, change, texts, budget)?;
    Some(())
}

fn word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn neutral_payload(text: &Rope, budget: &mut usize) -> Option<()> {
    *budget = budget.checked_sub(text.len_bytes())?;
    text.chars().all(word).then_some(())
}

fn interior_word(
    text: RopeSlice<'_>,
    range: std::ops::Range<usize>,
    budget: &mut usize,
) -> Option<()> {
    *budget = budget.checked_sub(4)?;
    let left = range.start.checked_sub(1)?;
    (word(text.get_char(left)?) && word(text.get_char(range.end)?)).then_some(())
}

fn prose_line(text: RopeSlice<'_>, at: usize, budget: &mut usize) -> Option<()> {
    let start = text.line_to_char(text.char_to_line(at));
    let mut chars = text.slice(start..at).chars();
    for _ in 0..256 {
        let c = chars.next()?;
        *budget = budget.checked_sub(c.len_utf8())?;
        if !c.is_whitespace() {
            return c.is_ascii_alphabetic().then_some(());
        }
    }
    None
}

fn contained(snapshot: &Snapshot, range: &std::ops::Range<usize>) -> bool {
    snapshot
        .spans
        .iter()
        .any(|s| s.opening && !s.kind.quote() && s.at.0 < range.start && range.end <= s.mate.0)
}

fn prose_change(
    profile: Profile,
    change: &Change,
    texts: (&Rope, &Rope),
    budget: &mut usize,
) -> Option<()> {
    if matches!(profile, Profile::Org | Profile::Markdown) {
        prose_line(texts.0.slice(..), change.before().start, budget)?;
    }
    if profile == Profile::Org {
        // Underscored Org words can introduce inline source syntax. Restrict
        // advancement to ordinary alphanumeric words instead of guessing it.
        ordinary_word(texts.0.slice(..), change.before().start, budget)?;
        ordinary_word(texts.1.slice(..), change.after().start, budget)?;
    }
    Some(())
}

fn ordinary_word(text: RopeSlice<'_>, at: usize, budget: &mut usize) -> Option<()> {
    let mut start = at;
    while start > 0 {
        let c = text.char(start - 1);
        *budget = budget.checked_sub(c.len_utf8())?;
        if c == '_' {
            return None;
        }
        if !c.is_ascii_alphanumeric() {
            break;
        }
        start -= 1;
    }
    for c in text.slice(start..).chars() {
        *budget = budget.checked_sub(c.len_utf8())?;
        if c == '_' {
            return None;
        }
        if !c.is_ascii_alphanumeric() {
            return Some(());
        }
    }
    Some(())
}
