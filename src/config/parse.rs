use std::collections::BTreeMap;
use std::ops::Range;
use std::path::Path;

use serde::Deserialize;
use toml::de::{DeTable, Deserializer};

use super::{Diagnostic, Document, validate};

struct Location {
    key: Range<usize>,
    value: Range<usize>,
}

#[derive(Default)]
struct Locations(BTreeMap<Vec<String>, Location>);

/// Parse and validate one UTF-8 TOML 1.1 source without IO or editor mutation.
/// Omitted schema version means 1. Successful records retain partial presence;
/// this operation neither resolves includes nor fills defaults into input.
pub fn parse(source: &Path, bytes: &[u8]) -> Result<Document, Diagnostic> {
    let text = std::str::from_utf8(bytes).map_err(|error| invalid_utf8(source, bytes, error))?;
    let root = DeTable::parse(text).map_err(|error| toml_error(source, text, error))?;
    let mut locations = Locations::default();
    locations.collect(root.get_ref(), &[]);
    let document = Document::deserialize(Deserializer::from(root)).map_err(|error| {
        let mut diagnostic = toml_error(source, text, error);
        diagnostic.key = locations.containing(diagnostic.span.as_ref());
        diagnostic
    })?;
    validate::document(&document).map_err(|failure| {
        let span = locations
            .0
            .get(&failure.key)
            .map(|location| location.value.clone());
        let mut diagnostic = Diagnostic::new(source, failure.message).locate(text, span);
        diagnostic.key = failure.key;
        diagnostic
    })?;
    Ok(document)
}

fn invalid_utf8(source: &Path, bytes: &[u8], error: std::str::Utf8Error) -> Diagnostic {
    let start = error.valid_up_to();
    let end = start + error.error_len().unwrap_or(bytes.len() - start);
    let prefix = std::str::from_utf8(&bytes[..start]).expect("UTF-8 decoder's valid prefix");
    Diagnostic::new(source, "configuration must be UTF-8".into()).locate(prefix, Some(start..end))
}

fn toml_error(source: &Path, text: &str, error: toml::de::Error) -> Diagnostic {
    Diagnostic::new(source, error.message().to_owned()).locate(text, error.span())
}

impl Locations {
    fn collect(&mut self, table: &DeTable<'_>, parent: &[String]) {
        for (key, value) in table {
            let mut path = parent.to_vec();
            path.push(key.get_ref().to_string());
            self.0.insert(
                path.clone(),
                Location {
                    key: key.span(),
                    value: value.span(),
                },
            );
            self.children(value.get_ref().as_table(), &path);
        }
    }

    fn children(&mut self, table: Option<&DeTable<'_>>, path: &[String]) {
        // Accepted leaves have at most three components, including dynamic map
        // keys. Stop before walking arbitrary rejected user-defined subtrees.
        if path.len() < 3
            && let Some(table) = table
        {
            self.collect(table, path);
        }
    }

    fn containing(&self, span: Option<&Range<usize>>) -> Vec<String> {
        let Some(span) = span else { return Vec::new() };
        self.0
            .iter()
            .filter(|(_, location)| {
                contains(&location.key, span) || contains(&location.value, span)
            })
            .max_by_key(|(path, _)| path.len())
            .map(|(path, _)| path.clone())
            .unwrap_or_default()
    }
}

fn contains(outer: &Range<usize>, inner: &Range<usize>) -> bool {
    outer.start <= inner.start && inner.end <= outer.end
}
