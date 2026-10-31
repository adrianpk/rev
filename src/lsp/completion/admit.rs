use super::{Fallback, Item, Query, Reply, Source};
use crate::lsp::{Encoding, position};
use serde_json::Value;
use std::ops::Range;

pub(super) fn response(query: &Query, value: Value, encoding: &Encoding) -> Reply {
    let incomplete = value
        .get("isIncomplete")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let defaults = value.get("itemDefaults").cloned().unwrap_or(Value::Null);
    let entries = value.as_array().or_else(|| value.get("items")?.as_array());
    let Some(entries) = entries else {
        return Reply::fallback(if value.is_null() {
            Fallback::Empty
        } else {
            Fallback::Unavailable
        });
    };
    if entries.is_empty() {
        return Reply::fallback(Fallback::Empty);
    }
    let mut items: Vec<_> = entries
        .iter()
        .take(crate::completion::MAX_SYMBOLS)
        .filter_map(|item| candidate(query, item, &defaults, encoding))
        .collect();
    items.sort_by(|a, b| (a.0, &a.1).cmp(&(b.0, &b.1)));
    let mut bytes = 0;
    let items: Vec<_> = items
        .into_iter()
        .map(|(_, _, item)| item)
        .take(crate::completion::MAX_CANDIDATES)
        .take_while(|item| {
            bytes += item.label.len() + item.text.len();
            bytes <= crate::completion::MAX_BYTES
        })
        .collect();
    if items.is_empty() {
        return Reply::fallback(Fallback::NoAdmitted);
    }
    Reply {
        items,
        source: Source::Semantic { incomplete },
        version: None,
    }
}

fn candidate(
    query: &Query,
    item: &Value,
    defaults: &Value,
    encoding: &Encoding,
) -> Option<(u8, String, Item)> {
    if !supported(item, defaults) {
        return None;
    }
    let label = item.get("label")?.as_str()?;
    if label.len() > crate::completion::MAX_LABEL {
        return None;
    }
    let filter = item
        .get("filterText")
        .and_then(Value::as_str)
        .unwrap_or(label);
    let class = crate::completion::matching::class(filter, &query.prefix)?;
    let (text, range) = insertion(query, item, defaults, encoding)?;
    if text.len() > 64 * 1024 {
        return None;
    }
    let sort = item.get("sortText").and_then(Value::as_str).unwrap_or("");
    Some((
        class,
        sort.to_owned(),
        Item {
            label: label.to_owned(),
            text: text.to_owned(),
            range,
        },
    ))
}

fn supported(item: &Value, defaults: &Value) -> bool {
    let format = item
        .get("insertTextFormat")
        .or_else(|| defaults.get("insertTextFormat"));
    let mode = item
        .get("insertTextMode")
        .or_else(|| defaults.get("insertTextMode"));
    format.is_none_or(|value| value == 1)
        && mode.is_none_or(|value| value == 1)
        && item.get("command").is_none_or(Value::is_null)
        && item
            .get("additionalTextEdits")
            .is_none_or(|value| value.as_array().is_some_and(Vec::is_empty))
}

fn insertion<'a>(
    query: &Query,
    item: &'a Value,
    defaults: &Value,
    encoding: &Encoding,
) -> Option<(&'a str, Range<usize>)> {
    if let Some(edit) = item.get("textEdit") {
        let range = edit_range(query, edit, encoding)?;
        return Some((edit.get("newText")?.as_str()?, range));
    }
    if let Some(edit) = defaults.get("editRange") {
        let range = edit_range(query, edit, encoding)?;
        let text = default_text(item)?;
        return Some((text, range));
    }
    Some((
        item.get("insertText")
            .or_else(|| item.get("label"))?
            .as_str()?,
        query.range.clone(),
    ))
}

fn decode(query: &Query, value: &Value, encoding: &Encoding) -> Option<Range<usize>> {
    let start = serde_json::from_value(value.get("start")?.clone()).ok()?;
    let end = serde_json::from_value(value.get("end")?.clone()).ok()?;
    let start = position::decode(&query.text, start, encoding).ok()?;
    let end = position::decode(&query.text, end, encoding).ok()?;
    (start <= end && query.text.char_to_line(start) == query.text.char_to_line(end))
        .then_some(start..end)
}

fn replacement(query: &Query, value: &Value, encoding: &Encoding) -> Option<Range<usize>> {
    let range = decode(query, value, encoding)?;
    (range.start <= query.range.start && range.end >= query.point).then_some(range)
}

fn insert_range(
    query: &Query,
    value: &Value,
    replace: &Range<usize>,
    encoding: &Encoding,
) -> Option<()> {
    let insert = replacement(query, value, encoding)?;
    (insert.start == replace.start && insert.end <= replace.end).then_some(())
}

fn edit_range(query: &Query, edit: &Value, encoding: &Encoding) -> Option<Range<usize>> {
    let raw = edit
        .get("range")
        .or_else(|| edit.get("replace"))
        .unwrap_or(edit);
    let range = replacement(query, raw, encoding)?;
    if let Some(insert) = edit.get("insert") {
        insert_range(query, insert, &range, encoding)?;
    }
    Some(range)
}

fn default_text(item: &Value) -> Option<&str> {
    item.get("textEditText")
        .or_else(|| item.get("insertText"))
        .or_else(|| item.get("label"))?
        .as_str()
}
