use std::collections::BTreeSet;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::PathBuf;

use super::super::{Bookmarks, Context, MAX_NAME_BYTES, MAX_PATH_BYTES};

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    version: u8,
    #[serde(default)]
    bookmarks: Vec<Entry>,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    name: String,
    path: Option<String>,
    path_bytes: Option<Vec<u8>>,
    position: usize,
    before: String,
    after: String,
    at_start: bool,
    at_end: bool,
}

pub(crate) struct Stored {
    pub name: String,
    pub path: PathBuf,
    pub context: Context,
}

pub(super) fn decode(bytes: &[u8], limit: usize) -> Result<Vec<Stored>, String> {
    let text =
        std::str::from_utf8(bytes).map_err(|e| format!("Bookmark store is not UTF-8: {e}"))?;
    let document: Document =
        toml::from_str(text).map_err(|e| format!("Invalid bookmark store: {e}"))?;
    if document.version != 1 {
        return Err("Unsupported bookmark store version".into());
    }
    if document.bookmarks.len() > limit {
        return Err(format!("Bookmark store exceeds {limit} entries"));
    }
    let mut names = BTreeSet::new();
    let mut records = Vec::with_capacity(document.bookmarks.len());
    for entry in document.bookmarks {
        if entry.name.is_empty()
            || entry.name.len() > MAX_NAME_BYTES
            || entry.name.chars().any(char::is_control)
        {
            return Err("Invalid bookmark name".into());
        }
        if !names.insert(entry.name.clone()) {
            return Err("Duplicate bookmark name".into());
        }
        let path = match (entry.path, entry.path_bytes) {
            (Some(path), None) => PathBuf::from(path),
            (None, Some(bytes)) => PathBuf::from(std::ffi::OsString::from_vec(bytes)),
            _ => return Err("Bookmark requires exactly one of path or path_bytes".into()),
        };
        if !path.is_absolute()
            || path.as_os_str().len() > MAX_PATH_BYTES
            || path.as_os_str().as_bytes().contains(&0)
        {
            return Err(
                "Bookmark path must be absolute, contain no NUL and use at most 4096 bytes".into(),
            );
        }
        let context = Context {
            position: entry.position,
            before: entry.before,
            after: entry.after,
            at_start: entry.at_start,
            at_end: entry.at_end,
        };
        validate_context(&context)?;
        records.push(Stored {
            name: entry.name,
            path,
            context,
        });
    }
    Ok(records)
}

fn validate_context(context: &Context) -> Result<(), String> {
    let before = context.before.chars().count();
    let after = context.after.chars().count();
    if before > 256
        || after > 256
        || before > context.position
        || context.at_start != (before == context.position)
        || before == 0 && !context.at_start
        || after == 0 && !context.at_end
        || context.position.checked_add(after).is_none()
        || i64::try_from(context.position).is_err()
    {
        return Err("Invalid bookmark context or position bounds".into());
    }
    Ok(())
}

pub(super) fn encode(bookmarks: &Bookmarks) -> Result<Vec<u8>, String> {
    let mut text = String::from("version = 1\n");
    for record in bookmarks.iter() {
        validate_context(record.context())?;
        text.push_str(&format!(
            "\n[[bookmarks]]\nname = {}\n",
            quoted(record.name())
        ));
        if let Some(path) = record.path().to_str() {
            text.push_str(&format!("path = {}\n", quoted(path)));
        } else {
            text.push_str("path_bytes = [");
            for (index, byte) in record.path().as_os_str().as_bytes().iter().enumerate() {
                if index != 0 {
                    text.push_str(", ");
                }
                text.push_str(&byte.to_string());
            }
            text.push_str("]\n");
        }
        let c = record.context();
        text.push_str(&format!(
            "position = {}\nbefore = {}\nafter = {}\nat_start = {}\nat_end = {}\n",
            c.position,
            quoted(&c.before),
            quoted(&c.after),
            c.at_start,
            c.at_end
        ));
        if text.len() > super::LIMIT {
            return Err("Bookmark store exceeds 16 MiB".into());
        }
    }
    Ok(text.into_bytes())
}

fn quoted(value: &str) -> String {
    let mut text = String::from("\"");
    for c in value.chars() {
        match c {
            '"' => text.push_str("\\\""),
            '\\' => text.push_str("\\\\"),
            c if c.is_control() => text.push_str(&format!("\\u{:04X}", u32::from(c))),
            c => text.push(c),
        }
    }
    text.push('"');
    text
}
