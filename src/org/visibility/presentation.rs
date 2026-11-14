use super::Outline;
use crate::org::structure::{HeadingPrefix, LinePresentation, presentation_line};
use ropey::RopeSlice;

/// Canonical same-line before/after source, qualified at a checked base byte.
pub(crate) struct PresentationEdit<'a> {
    pub at: usize,
    pub before: RopeSlice<'a>,
    pub after: RopeSlice<'a>,
    pub changed: usize,
}

impl Outline {
    pub(crate) fn admit_presentation(
        &self,
        edit: &PresentationEdit<'_>,
        remaining: &mut usize,
    ) -> Option<()> {
        self.complete().ok()?;
        let (before, after) = presentation_lines(edit, remaining)?;
        let old = presentation_line(&before, remaining)?;
        let new = presentation_line(&after, remaining)?;
        match (old, new) {
            (LinePresentation::Body, LinePresentation::Body) => {
                (self.heading_level_at(edit.at).is_none() && !self.blocked_at(edit.at))
                    .then_some(())
            }
            (LinePresentation::Heading(old), LinePresentation::Heading(new)) => {
                self.admit_title(edit, (&before, &after), old, new)
            }
            _ => None,
        }
    }

    fn admit_title(
        &self,
        edit: &PresentationEdit<'_>,
        lines: (&str, &str),
        old: HeadingPrefix,
        new: HeadingPrefix,
    ) -> Option<()> {
        (old.level == new.level
            && old.end == new.end
            && edit.changed >= old.end
            && self.heading_level_at(edit.at) == Some(old.level)
            && lines.0.get(..old.end) == lines.1.get(..new.end))
        .then_some(())
    }

    fn blocked_at(&self, at: usize) -> bool {
        let end = self.blocked.partition_point(|range| range.start <= at);
        end.checked_sub(1)
            .is_some_and(|i| self.blocked[i].contains(&at))
    }

    pub(crate) fn presentation_limit(&self, bytes: usize) -> Option<()> {
        (bytes <= crate::org::Limits::default().bytes).then_some(())
    }

    pub(crate) fn same_presentation(
        &self,
        other: &Self,
        map: impl Fn(usize) -> Option<usize>,
    ) -> bool {
        self.complete().is_ok()
            && other.complete().is_ok()
            && self.headings.len() == other.headings.len()
            && self
                .headings
                .iter()
                .zip(&other.headings)
                .all(|(old, new)| old.level == new.level && map(old.start) == Some(new.start))
    }
}

fn presentation_lines(
    edit: &PresentationEdit<'_>,
    remaining: &mut usize,
) -> Option<(String, String)> {
    let probes = edit
        .before
        .len_bytes()
        .checked_add(edit.after.len_bytes())?;
    *remaining = remaining.checked_sub(probes)?;
    Some((line_text(edit.before)?, line_text(edit.after)?))
}

// Inspection reserved these complete lines before reaching the Org owner.
// Allocation refusal drops optional presentation without affecting the edit.
fn line_text(line: RopeSlice<'_>) -> Option<String> {
    let mut text = String::new();
    text.try_reserve_exact(line.len_bytes()).ok()?;
    for chunk in line.chunks() {
        text.push_str(chunk);
    }
    Some(text)
}
