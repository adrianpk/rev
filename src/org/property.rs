mod edit;
mod model;
mod resolve;

use super::{Cancellation, Outline, Structure};
use crate::buffer::{Buffer, edit::Replacement};
use crate::coordinates::{Byte, Coordinates, Scalar};
use std::ops::Range;

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Properties {
    complete: bool,
    entries: Vec<Entry>,
    records: Vec<Record>,
    contexts: Vec<Range<usize>>,
}
#[derive(Debug, PartialEq, Eq)]
struct Entry {
    start: usize,
    end: usize,
    anchor: usize,
    parent: Option<usize>,
    invalid: bool,
    drawer: Option<Drawer>,
    records: Vec<usize>,
}
#[derive(Debug, PartialEq, Eq)]
struct Drawer {
    source: Range<usize>,
    closing: usize,
    indent: String,
}
#[derive(Debug, PartialEq, Eq)]
struct Record {
    key: String,
    additive: bool,
    source: Range<usize>,
    value: Range<usize>,
    owner: usize,
    directive: bool,
}

pub(crate) struct PropertyValue {
    pub(crate) text: String,
    pub(crate) undefined: bool,
    pub(crate) records: Vec<usize>,
}

pub(crate) fn property_name(name: &str) -> Result<String, String> {
    let base = name.strip_suffix('+').unwrap_or(name);
    if base.is_empty()
        || name.len() > 128
        || base
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || ":+".contains(c))
    {
        return Err(
            "Org property name must be 1..128 bytes without whitespace, colons or interior +"
                .into(),
        );
    }
    if base.eq_ignore_ascii_case("END") || base.eq_ignore_ascii_case("PROPERTIES") {
        return Err("Org drawer delimiters are not property names".into());
    }
    Ok(name.to_ascii_uppercase())
}
pub(crate) fn derived_property(key: &str) -> bool {
    matches!(
        key.trim_end_matches('+'),
        "ITEM"
            | "TODO"
            | "PRIORITY"
            | "TAGS"
            | "ALLTAGS"
            | "FILE"
            | "BLOCKED"
            | "CLOCKSUM"
            | "CLOCKSUM_T"
    )
}
impl Outline {
    pub(crate) fn property_target(&self, buffer: &Buffer, point: Scalar) -> Result<usize, String> {
        self.complete().map_err(str::to_owned)?;
        super::source::current(buffer, self.stamp).map_err(|e| e.to_string())?;
        if !self.properties.complete {
            return Err("Org property observation is incomplete".into());
        }
        let byte = Coordinates::new(buffer)
            .map_err(|e| e.to_string())?
            .to_byte(point)
            .map_err(|e| e.to_string())?
            .0;
        let admitted = self.properties.admits(byte, byte == buffer.len_bytes());
        if !admitted && self.unavailable(byte, byte == buffer.len_bytes(), byte > 0) {
            return Err("Org property operation is unavailable in this source context".into());
        }
        let entry = self
            .properties
            .entries
            .partition_point(|entry| entry.start <= byte)
            .saturating_sub(1);
        if self.properties.entries[entry].invalid {
            return Err("Malformed Org property metadata; correct source before editing".into());
        }
        Ok(entry)
    }
    pub(crate) fn property_edit(
        &self,
        buffer: &Buffer,
        entry: usize,
        key: &str,
        value: Option<&str>,
    ) -> Result<Vec<Replacement>, String> {
        self.properties.edit(buffer, entry, key, value)
    }
    pub(crate) fn property_value(
        &self,
        buffer: &Buffer,
        entry: usize,
        key: &str,
        inherit: &[String],
    ) -> Result<PropertyValue, String> {
        self.properties.resolve(buffer, entry, key, inherit)
    }
    pub(crate) fn property_origin(
        &self,
        buffer: &Buffer,
        record: usize,
        entry: usize,
    ) -> Result<String, String> {
        let property = &self.properties.records[record];
        let position = Coordinates::new(buffer)
            .map_err(|e| e.to_string())?
            .from_byte(Byte(property.source.start))
            .map_err(|e| e.to_string())?;
        let row = Coordinates::new(buffer)
            .map_err(|e| e.to_string())?
            .to_line(position)
            .map_err(|e| e.to_string())?
            .row
            + 1;
        let kind = if property.directive {
            "file directive"
        } else if property.owner == entry {
            "local"
        } else if property.owner == 0 {
            "file drawer"
        } else {
            "inherited"
        };
        Ok(format!(
            "{kind} {}{} at line {row}",
            property.key,
            if property.additive { "+" } else { "" }
        ))
    }
    pub(crate) fn property_owner_edit(
        &self,
        buffer: &Buffer,
        record: usize,
        value: &str,
    ) -> Result<Vec<Replacement>, String> {
        self.properties
            .replace(buffer, record, value)
            .map(|edit| vec![edit])
    }
}
fn text(buffer: &Buffer, range: Range<usize>) -> Result<String, String> {
    let span = buffer.byte_span(range).map_err(|e| e.to_string())?;
    Ok(buffer.slice(&span).map_err(|e| e.to_string())?.to_string())
}
fn replacement(buffer: &Buffer, range: Range<usize>, text: String) -> Result<Replacement, String> {
    let source = buffer.byte_span(range).map_err(|e| e.to_string())?;
    Ok(Replacement { source, text })
}

impl Properties {
    fn admits(&self, byte: usize, at_eof: bool) -> bool {
        self.contexts
            .iter()
            .any(|r| r.contains(&byte) || at_eof && r.end == byte)
    }
}
