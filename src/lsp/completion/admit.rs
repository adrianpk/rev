use super::{Fallback, Item, Proposal, Query, Reply, Source, proposal::Context};
use crate::lsp::{Encoding, position};
use serde_json::Value;
use std::ops::Range;
use std::sync::Arc;

pub(super) fn response(query: &Query, value: Value, wire: (&Encoding, bool)) -> Reply {
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
    let Some(context) = Context::capture(query, defaults.clone(), wire.1) else {
        return Reply::fallback(Fallback::Cancelled);
    };
    let items = ranked(query, entries, &defaults, wire.0, &context);
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
    context: &Arc<Context>,
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
    let additional = super::edits::additional(query, item, (text, &range), encoding)?;
    let sort = item.get("sortText").and_then(Value::as_str).unwrap_or("");
    Some((
        class,
        sort.to_owned(),
        Item {
            label: label.to_owned(),
            kind: item.get("kind").and_then(Value::as_u64),
            text: text.to_owned(),
            range,
            additional,
            proposal: context
                .resolve
                .then(|| Proposal::new(context.clone(), item)),
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
        && super::edits::primary(item)
        && defaults
            .get("editRange")
            .is_none_or(super::edits::default_range)
        && !["workspaceEdit", "documentChanges", "changes"]
            .iter()
            .any(|key| item.get(key).is_some())
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
    let raw = primary_range(edit)?;
    let range = replacement(query, raw, encoding)?;
    if let Some(insert) = edit.get("insert") {
        insert_range(query, insert, &range, encoding)?;
    }
    Some(range)
}

pub(super) fn default_text(item: &Value) -> Option<&str> {
    item.get("textEditText")
        .or_else(|| item.get("insertText"))
        .or_else(|| item.get("label"))?
        .as_str()
}

fn primary_range(edit: &Value) -> Option<&Value> {
    if let Some(range) = edit.get("range") {
        return (!edit.as_object()?.contains_key("replace")
            && !edit.as_object()?.contains_key("insert"))
        .then_some(range);
    }
    if let Some(replace) = edit.get("replace") {
        edit.get("insert")?;
        return Some(replace);
    }
    (!edit.as_object()?.contains_key("insert")).then_some(edit)
}

fn ranked(
    query: &Query,
    entries: &[Value],
    defaults: &Value,
    encoding: &Encoding,
    context: &Arc<Context>,
) -> Vec<Item> {
    let mut retained: Vec<(u8, String, Item)> = Vec::new();
    for item in entries
        .iter()
        .filter_map(|item| candidate(query, item, defaults, encoding, context))
    {
        let index = retained.partition_point(|old| (old.0, &old.1) <= (item.0, &item.1));
        if index < crate::completion::MAX_CANDIDATES {
            retained.insert(index, item);
            retained.truncate(crate::completion::MAX_CANDIDATES);
        }
    }
    retained.into_iter().map(|(_, _, item)| item).collect()
}

pub(super) fn resolved(query: &Query, value: Value, encoding: &Encoding) -> Reply {
    let Some(item) = query
        .resolve
        .as_ref()
        .and_then(|proposal| proposal.merge(value))
    else {
        return Reply::fallback(Fallback::NoAdmitted);
    };
    response(query, Value::Array(vec![item]), (encoding, false))
}
