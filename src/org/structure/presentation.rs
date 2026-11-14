use super::{Cancellation, Instant, Work, tokens};

pub(in crate::org) enum LinePresentation {
    Body,
    Heading(HeadingPrefix),
}

#[derive(Clone, Copy)]
pub(in crate::org) struct HeadingPrefix {
    pub level: usize,
    pub end: usize,
}

/// Reuse the actual Org token classifier with a caller-owned byte allowance.
/// Reserve all fixed keyword probes and an extra whitespace/guard pass before
/// entering the classifier; its variable scans charge up to four bytes/tick.
pub(in crate::org) fn presentation_line(
    line: &str,
    remaining: &mut usize,
) -> Option<LinePresentation> {
    let probes = line.len().checked_add(1024)?;
    *remaining = remaining.checked_sub(probes)?;
    let cancel = Cancellation::default();
    let mut work = Work {
        cancellation: &cancel,
        yielded: Instant::now(),
        remaining: Some(remaining),
    };
    let facts = tokens::classify(line, &mut work).ok()?;
    match facts.token {
        tokens::Token::Heading(level, title) => Some(LinePresentation::Heading(HeadingPrefix {
            level,
            end: title.start,
        })),
        tokens::Token::Text if ordinary(line) => Some(LinePresentation::Body),
        _ => None,
    }
}

fn ordinary(line: &str) -> bool {
    // Text-like malformed boundaries and definitions remain uncertain even
    // when the native classifier correctly leaves them as ordinary source.
    !line
        .trim_start_matches([' ', '\t'])
        .starts_with(['#', ':', '|', '*', '-', '+', '[', '<', '>'])
}
