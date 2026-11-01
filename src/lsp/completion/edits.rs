//! Whole-item admission before any canonical transaction can be prepared.
use super::{Edit, Query};
use crate::lsp::{Encoding, position};
use serde::Deserialize;
use serde_json::Value;
use std::ops::Range;

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct TextEdit {
    range: WireRange,
    new_text: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireRange {
    start: position::Position,
    end: position::Position,
}

pub(super) fn additional(
    query: &Query,
    item: &Value,
    primary: (&str, &Range<usize>),
    encoding: &Encoding,
) -> Option<Vec<Edit>> {
    let Some(value) = item.get("additionalTextEdits") else {
        return Some(Vec::new());
    };
    let values = value.as_array()?;
    if values.len() > 32 {
        return None;
    }
    let mut bytes = primary.0.len();
    let mut edits = Vec::new();
    for value in values {
        let edit: TextEdit = serde_json::from_value(value.clone()).ok()?;
        bytes = bytes.checked_add(edit.new_text.len())?;
        if bytes > 256 * 1024 {
            return None;
        }
        let start = position::decode(&query.text, edit.range.start, encoding).ok()?;
        let end = position::decode(&query.text, edit.range.end, encoding).ok()?;
        if start > end {
            return None;
        }
        edits.push(Edit {
            range: start..end,
            text: edit.new_text,
        });
    }
    disjoint(primary.1, &edits).then_some(edits)
}
fn disjoint(primary: &Range<usize>, edits: &[Edit]) -> bool {
    let mut ranges: Vec<_> = edits
        .iter()
        .map(|edit| &edit.range)
        .chain(std::iter::once(primary))
        .collect();
    ranges.sort_by_key(|range| (range.start, range.end));
    ranges
        .windows(2)
        .all(|pair| pair[0].start != pair[1].start && pair[0].end <= pair[1].start)
}

pub(super) fn primary(item: &Value) -> bool {
    item.get("textEdit").is_none_or(|edit| {
        edit.as_object().is_some_and(|object| {
            object
                .keys()
                .all(|key| matches!(key.as_str(), "range" | "newText" | "insert" | "replace"))
        })
    })
}

pub(super) fn default_range(value: &Value) -> bool {
    value.as_object().is_some_and(|object| {
        object
            .keys()
            .all(|key| matches!(key.as_str(), "start" | "end" | "insert" | "replace"))
    })
}
