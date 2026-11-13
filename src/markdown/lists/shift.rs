use super::builder::{Builder, Layout, Shift};
use super::source::{Source, separator};
use super::{super::Observation, Edit};

pub(super) fn indent(
    model: &Observation,
    source: &Source<'_>,
    index: usize,
) -> Result<Edit, String> {
    let item = &model.items[index];
    let delta = indent_delta(model, index)?;
    let start = source.line(item.marker.start, item.quotes)?.bytes.start;
    let end = source.item_end(item)?;
    let mut output = Builder::new(source, start..end)?;
    output.shift(
        start..end,
        &Layout {
            quotes: item.quotes,
            shift: Shift::In(delta),
            first_prefix: None,
        },
    )?;
    output.finish()
}

fn indent_delta(model: &Observation, index: usize) -> Result<usize, String> {
    let item = &model.items[index];
    let previous = model.items[..index]
        .iter()
        .rfind(|before| before.parent == item.parent && before.group == item.group)
        .ok_or("A Markdown item needs a previous sibling before indentation")?;
    previous
        .content_column
        .checked_sub(item.marker_column)
        .filter(|delta| *delta > 0)
        .ok_or_else(|| "Markdown item is already inside that sibling".into())
}

pub(super) fn outdent(
    model: &Observation,
    source: &Source<'_>,
    index: usize,
) -> Result<Edit, String> {
    let Outdent {
        start,
        end,
        parent_end,
        prefix,
        delta,
        inline,
        quotes,
    } = Outdent::new(model, source, index)?;
    let mut output = Builder::new(source, start..parent_end)?;
    if inline {
        output.insert(source.buffer.newline())?;
    }
    let moved_end = tail(&mut output, source, start..end, end..parent_end)?;
    output.shift(
        start..moved_end,
        &Layout {
            quotes,
            shift: Shift::Out(delta),
            first_prefix: inline.then_some(prefix.as_str()),
        },
    )?;
    output.finish()
}

struct Outdent {
    start: usize,
    end: usize,
    parent_end: usize,
    prefix: String,
    delta: usize,
    inline: bool,
    quotes: usize,
}

impl Outdent {
    fn new(model: &Observation, source: &Source<'_>, index: usize) -> Result<Self, String> {
        let item = &model.items[index];
        let parent = &model.items[item
            .parent
            .ok_or("Markdown item is already at the outer list level")?];
        let prefix = super::continuation::prefix(source, item)?;
        let (prefix, delta) = outdent_prefix(&prefix, parent)?;
        let line = source.line(item.marker.start, item.quotes)?;
        let inline = parent.line_start == line.bytes.start;
        let start = if inline {
            item.bytes.start
        } else {
            line.bytes.start
        };
        let end = source.item_end(item)?;
        let parent_end = source.item_end(parent)?;
        Ok(Self {
            start,
            end,
            parent_end,
            prefix,
            delta,
            inline,
            quotes: parent.quotes,
        })
    }
}

// Remove the selected parent's list indentation before any quote containers
// introduced inside it. Those quotes belong to retained child source.
fn outdent_prefix(prefix: &str, parent: &super::super::Item) -> Result<(String, usize), String> {
    use super::super::item::{columns, quote_end};
    let quote = quote_end(prefix, parent.quotes);
    let end = quote
        + prefix[quote..]
            .bytes()
            .take_while(|b| matches!(b, b' ' | b'\t'))
            .count();
    let column = columns(&prefix[..end]) - columns(&prefix[..quote]);
    let delta = column
        .checked_sub(parent.marker_column)
        .filter(|delta| *delta > 0)
        .ok_or("Markdown item indentation is ambiguous")?;
    let mut output = prefix[..quote].to_owned();
    output.push_str(&" ".repeat(parent.marker_column));
    output.push_str(&prefix[end..]);
    Ok((output, delta))
}

// Transfer the selected item's existing separator when an unterminated final
// sibling moves before it. EOF keeps its original final-newline identity.
fn tail(
    output: &mut Builder<'_, '_>,
    source: &Source<'_>,
    item: std::ops::Range<usize>,
    tail: std::ops::Range<usize>,
) -> Result<usize, String> {
    if tail.is_empty() {
        return Ok(item.end);
    }
    output.copy(tail.clone())?;
    let raw = source.text(tail)?;
    if separator(&raw) > 0 {
        return Ok(item.end);
    }
    let raw = source.text(item.clone())?;
    let length = separator(&raw);
    if length == 0 {
        return Err("Markdown sibling boundary has no separator".into());
    }
    output.copy(item.end - length..item.end)?;
    Ok(item.end - length)
}
