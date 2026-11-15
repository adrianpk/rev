use super::Transaction;
use crate::buffer::edit::Change;
use crate::pairs::Kind;
use ropey::{Rope, RopeSlice};

pub(super) struct Mutation {
    pub(super) row: usize,
    pub(super) start: usize,
    pub(super) delta: isize,
}

struct Budget<'a> {
    remaining: &'a mut usize,
}

impl Budget<'_> {
    fn charge(&mut self, bytes: usize) -> Option<()> {
        *self.remaining = self.remaining.checked_sub(bytes)?;
        Some(())
    }

    fn fragment(&mut self, text: &Rope) -> Option<()> {
        self.charge(text.len_bytes())?;
        (!text.chars().any(|ch| ch == '\r' || ch == '\n')).then_some(())
    }

    fn line<'a>(&mut self, text: &'a Rope, row: usize) -> Option<RopeSlice<'a>> {
        let line = text.get_line(row)?;
        self.charge(line.len_bytes())?;
        Some(line)
    }

    fn same_line(&mut self, before: &Rope, after: &Rope, change: &Change) -> Option<usize> {
        self.fragment(change.removed())?;
        self.fragment(change.inserted())?;
        let row = before.try_char_to_line(change.before().start).ok()?;
        if row != after.try_char_to_line(change.after().start).ok()? {
            return None;
        }
        // Reserve the complete before/after lines before format inspection.
        // Never materialize an unchecked full logical line here.
        self.line(before, row)?;
        self.line(after, row)?;
        Some(row)
    }

    fn row_delta(
        &mut self,
        before: &Rope,
        after: &Rope,
        row: usize,
        changes: &[Change],
    ) -> Option<isize> {
        let mut delta = 0isize;
        for change in changes {
            if self.same_line(before, after, change)? != row {
                return None;
            }
            let added = isize::try_from(change.inserted().len_bytes()).ok()?;
            let removed = isize::try_from(change.removed().len_bytes()).ok()?;
            delta = delta.checked_add(added.checked_sub(removed)?)?;
        }
        Some(delta)
    }

    fn paired(&mut self, changes: &[Change]) -> Option<()> {
        if changes.len() == 1 {
            return Some(());
        }
        let [opening, closing] = changes else {
            return None;
        };
        let (opening, closing) = pair_fragments(opening, closing)?;
        self.charge(opening.len_bytes())?;
        self.charge(closing.len_bytes())?;
        paired_text(opening, closing).then_some(())
    }
}

fn pair_fragments<'a>(opening: &'a Change, closing: &'a Change) -> Option<(&'a Rope, &'a Rope)> {
    if opening.removed().len_bytes() == 0 && closing.removed().len_bytes() == 0 {
        Some((opening.inserted(), closing.inserted()))
    } else if opening.inserted().len_bytes() == 0 && closing.inserted().len_bytes() == 0 {
        Some((opening.removed(), closing.removed()))
    } else {
        None
    }
}

fn paired_text(opening: &Rope, closing: &Rope) -> bool {
    let mut characters = opening.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    let Some(kind) = opening_kind(first) else {
        return false;
    };
    let (left, right) = kind.characters();
    opening.len_chars() == closing.len_chars()
        && opening.len_chars() <= crate::pairs::MAX_DEPTH
        && characters.all(|c| c == left)
        && closing.chars().all(|c| c == right)
}

fn opening_kind(c: char) -> Option<Kind> {
    match c {
        '"' => Some(Kind::DoubleQuote),
        '\'' => Some(Kind::SingleQuote),
        '`' => Some(Kind::Backtick),
        _ => Kind::structural(c)
            .filter(|(_, opening)| *opening)
            .map(|(kind, _)| kind),
    }
}

impl Mutation {
    pub(super) fn inspect(
        before: &Rope,
        after: &Rope,
        record: &Transaction,
        remaining: &mut usize,
    ) -> Option<Self> {
        let changes = record.changes();
        let first = changes.first()?;
        // A pair edit can have two ends on one row. Keep larger or multi-row
        // batches outside this finite presentation-only admission contract.
        if changes.len() > 2 || before.len_lines() != after.len_lines() {
            return None;
        }
        let mut budget = Budget { remaining };
        let row = before.try_char_to_line(first.before().start).ok()?;
        let delta = budget.row_delta(before, after, row, changes)?;
        budget.paired(changes)?;
        Some(Self {
            row,
            start: before.try_line_to_byte(row).ok()?,
            delta,
        })
    }
}
