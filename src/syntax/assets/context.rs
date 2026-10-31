use super::super::{Source, Syntax};
use std::ops::Range;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use tree_sitter::{QueryCursor, QueryCursorOptions, StreamingIterator};

impl Syntax {
    pub(crate) fn external_literal(
        &self,
        text: ropey::RopeSlice<'_>,
        byte: usize,
        deadline: Instant,
        cancel: &AtomicBool,
    ) -> Result<Option<&'static str>, String> {
        let Some(parsed) = self.parsed.as_ref().filter(|parsed| parsed.external) else {
            return Ok(None);
        };
        let Some(tree) = &parsed.tree else {
            return Ok(None);
        };
        let query = parsed.completion.as_ref().expect("admitted external query");
        let mut cursor = QueryCursor::new();
        cursor.set_match_limit(4096);
        cursor.set_byte_range(byte.saturating_sub(1)..byte);
        let mut progress = |_: &tree_sitter::QueryCursorState| {
            if Instant::now() >= deadline || cancel.load(Ordering::Acquire) {
                std::ops::ControlFlow::Break(())
            } else {
                std::ops::ControlFlow::Continue(())
            }
        };
        let options = QueryCursorOptions::new().progress_callback(&mut progress);
        let mut captures =
            cursor.captures_with_options(query, tree.root_node(), Source(text), options);
        let mut kind = None;
        while let Some((matched, index)) = captures.next() {
            let capture = matched.captures()[*index];
            if within(capture.node.byte_range(), byte) {
                kind = literal(query.capture_names()[capture.index as usize]).or(kind);
            }
        }
        if cursor.did_exceed_match_limit()
            || Instant::now() >= deadline
            || cancel.load(Ordering::Acquire)
        {
            return Err("Native completion work limit reached".into());
        }
        Ok(kind)
    }
}

fn within(range: Range<usize>, byte: usize) -> bool {
    range.start < byte && byte <= range.end
}
fn literal(name: &str) -> Option<&'static str> {
    match name {
        "comment" => Some("comment"),
        "string" => Some("string"),
        _ => None,
    }
}
