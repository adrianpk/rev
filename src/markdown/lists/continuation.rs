use crate::buffer::edit::Replacement;
use crate::coordinates::Scalar;

use super::super::{Item, Marker, Observation};
use super::{
    Edit,
    source::{Source, separator},
};

pub(super) fn continue_item(
    model: &Observation,
    source: &Source<'_>,
    index: usize,
) -> Result<Edit, String> {
    let item = &model.items[index];
    if !item.nonempty {
        return exit(model, source, index);
    }
    let anchor = source.item_end(item)?;
    let prefix = sibling_prefix(source, item)?;
    let mut text = String::new();
    let ended = separator(&source.text(item.bytes.start..anchor)?) > 0;
    if !ended {
        text.push_str(source.buffer.newline());
    }
    text.push_str(&prefix);
    let cursor = Scalar(source.scalar(anchor)?.0 + text.chars().count());
    if ended {
        text.push_str(source.buffer.newline());
    }
    Ok(Edit {
        edits: vec![Replacement {
            source: source.span(anchor..anchor)?,
            text,
        }],
        retained: Vec::new(),
        cursor: Some(cursor),
    })
}

fn sibling_prefix(source: &Source<'_>, item: &Item) -> Result<String, String> {
    let mut text = prefix(source, item)?;
    text.push_str(&marker(item)?);
    let gap = source.text(item.marker.end..item.content)?;
    text.push_str(if gap.is_empty() { " " } else { &gap });
    Ok(text)
}

fn exit(model: &Observation, source: &Source<'_>, index: usize) -> Result<Edit, String> {
    let item = &model.items[index];
    if item.parent.is_some() {
        return super::shift::outdent(model, source, index);
    }
    let line = source.line(item.marker.start, item.quotes)?;
    Ok(Edit {
        edits: vec![Replacement {
            source: source.span(item.marker.start..line.body_end)?,
            text: String::new(),
        }],
        retained: Vec::new(),
        cursor: Some(source.scalar(item.marker.start)?),
    })
}

pub(super) fn prefix(source: &Source<'_>, item: &Item) -> Result<String, String> {
    let raw = source.text(item.line_start..item.marker.start)?;
    Ok(raw
        .chars()
        .map(|c| if matches!(c, '\t' | '>') { c } else { ' ' })
        .collect())
}

fn marker(item: &Item) -> Result<String, String> {
    match item.form {
        Marker::Bullet(byte) => Ok(char::from(byte).to_string()),
        Marker::Ordered {
            number,
            delimiter,
            width,
        } => {
            let next = number
                .checked_add(1)
                .filter(|n| *n <= 999_999_999)
                .ok_or("Markdown ordered list number is exhausted")?;
            Ok(format!("{next:0width$}{}", char::from(delimiter)))
        }
    }
}
