use std::ops::Range;

pub(crate) enum Marker {
    Bullet(u8),
    Ordered {
        number: u32,
        delimiter: u8,
        width: usize,
    },
}

/// Editor-owned source spans, including the complete item subtree. Parser
/// container identity supplies depth and parentage; indentation never guesses it.
pub(crate) struct Item {
    pub(super) bytes: Range<usize>,
    pub(super) line_start: usize,
    pub(crate) marker: Range<usize>,
    pub(super) content: usize,
    pub(super) nonempty: bool,
    pub(crate) depth: usize,
    pub(super) parent: Option<usize>,
    pub(super) group: usize,
    pub(crate) form: Marker,
    pub(super) quotes: usize,
    pub(super) marker_column: usize,
    pub(super) content_column: usize,
}

impl Item {
    pub(super) fn parse(source: &str, bytes: Range<usize>, depth: usize) -> Option<Self> {
        let line_start = source[..bytes.start].rfind('\n').map_or(0, |at| at + 1);
        let first = source.get(bytes.clone())?.split('\n').next()?;
        let start = bytes.start
            + first
                .bytes()
                .take_while(|b| matches!(b, b' ' | b'\t'))
                .count();
        let raw = source.get(start..bytes.end)?;
        let (form, length) = marker(raw)?;
        let end = start + length;
        let padding = source[end..bytes.end]
            .bytes()
            .take_while(|b| matches!(b, b' ' | b'\t'))
            .count();
        let content = end + padding;
        let prefix = &source[line_start..start];
        let quotes = prefix.bytes().filter(|b| *b == b'>').count();
        let base = quote_column(prefix);
        let marker_column = columns(prefix) - base;
        let padding_columns =
            columns(&source[line_start..content]) - columns(&source[line_start..end]);
        let padding_columns = if (1..=4).contains(&padding_columns) {
            padding_columns
        } else {
            1
        };
        Some(Self {
            bytes,
            line_start,
            marker: start..end,
            content,
            nonempty: false,
            depth,
            parent: None,
            group: 0,
            quotes,
            marker_column,
            content_column: marker_column + length + padding_columns,
            form,
        })
    }
}

fn marker(source: &str) -> Option<(Marker, usize)> {
    let bytes = source.as_bytes();
    let first = *bytes.first()?;
    if matches!(first, b'-' | b'+' | b'*') {
        return Some((Marker::Bullet(first), 1));
    }
    let digits = bytes.iter().take_while(|b| b.is_ascii_digit()).count();
    if !(1..=9).contains(&digits) {
        return None;
    }
    let delimiter = *bytes.get(digits)?;
    if !matches!(delimiter, b'.' | b')') {
        return None;
    }
    let number = source[..digits].parse().ok()?;
    Some((
        Marker::Ordered {
            number,
            delimiter,
            width: digits,
        },
        digits + 1,
    ))
}

pub(super) fn columns(source: &str) -> usize {
    source.bytes().fold(0, |column, byte| {
        if byte == b'\t' {
            column + 4 - column % 4
        } else {
            column + 1
        }
    })
}

fn quote_column(prefix: &str) -> usize {
    let Some(at) = prefix.rfind('>') else {
        return 0;
    };
    let end = at + 1 + usize::from(prefix.as_bytes().get(at + 1) == Some(&b' '));
    columns(&prefix[..end])
}

pub(super) fn quote_end(line: &str, count: usize) -> usize {
    // Quote ownership is already admitted. A surrounding list can put its
    // retained quote prefix after more than three spaces or a tab stop.
    let bytes = line.as_bytes();
    let mut end = 0;
    for _ in 0..count {
        let spaces = bytes[end..]
            .iter()
            .take_while(|b| matches!(b, b' ' | b'\t'))
            .count();
        let at = end + spaces;
        if bytes.get(at) != Some(&b'>') {
            break;
        }
        end = at + 1;
        if bytes.get(end) == Some(&b' ') {
            end += 1;
        }
    }
    end
}
