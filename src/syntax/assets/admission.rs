use super::{Assets, Entry};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use tree_sitter::{Language, Parser, Query};

const QUERY_BYTES: u64 = 1024 * 1024;

pub(super) fn load(entry: &Entry) -> Result<Arc<Assets>, String> {
    let registration = &entry.registration;
    if !crate::config::external_symbol(&registration.symbol) {
        return Err("expected tree_sitter_ symbol with ASCII identifier suffix".into());
    }
    let parser_path = path(entry, &registration.parser)?;
    let highlights = query_source(entry, &registration.highlights)?;
    let completions = query_source(entry, &registration.completions)?;
    // The upstream safe API loads only this library and symbol, never compiles
    // or discovers anything, and intentionally retains the mapping forever.
    let language = tree_sitter_loader::Loader::load_language(&parser_path, &registration.symbol)
        .map_err(|error| format!("load external parser: {error}"))?;
    Parser::new()
        .set_language(&language)
        .map_err(|error| format!("external parser ABI: {error}"))?;
    let highlights = query(&language, &highlights, false)?;
    let completions = query(&language, &completions, true)?;
    Ok(Arc::new(Assets {
        language,
        highlights: Arc::new(highlights),
        completions: Arc::new(completions),
    }))
}

fn path(entry: &Entry, value: &str) -> Result<PathBuf, String> {
    let path = Path::new(value);
    let resolved = if path.is_absolute() {
        path.to_owned()
    } else {
        if path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
        {
            return Err("relative grammar path escapes rev/grammars".into());
        }
        entry
            .root
            .as_ref()
            .ok_or("cannot resolve grammar data root")?
            .join(path)
    };
    let canonical = fs::canonicalize(&resolved)
        .map_err(|error| format!("grammar asset {}: {error}", resolved.display()))?;
    if !fs::metadata(&canonical)
        .map_err(|error| error.to_string())?
        .is_file()
    {
        return Err("expected regular grammar asset file".into());
    }
    if !path.is_absolute() {
        let root = fs::canonicalize(entry.root.as_ref().expect("resolved root"))
            .map_err(|error| error.to_string())?;
        if !canonical.starts_with(root) {
            return Err("relative grammar path escapes rev/grammars".into());
        }
    }
    Ok(canonical)
}

fn query_source(entry: &Entry, value: &str) -> Result<String, String> {
    let path = path(entry, value)?;
    let file = File::open(path).map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    file.take(QUERY_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.is_empty() || bytes.len() as u64 > QUERY_BYTES {
        return Err("grammar query must contain 1..=1048576 UTF-8 bytes".into());
    }
    String::from_utf8(bytes).map_err(|_| "grammar query must be UTF-8".into())
}

fn query(language: &Language, source: &str, completion: bool) -> Result<Query, String> {
    let query = Query::new(language, source).map_err(|error| format!("external query: {error}"))?;
    if query.capture_names().is_empty()
        || !query
            .capture_names()
            .iter()
            .all(|name| capture(name, completion))
    {
        return Err("external query uses an unsupported capture profile".into());
    }
    if (0..query.pattern_count()).any(|index| {
        !query.general_predicates(index).is_empty()
            || !query.property_settings(index).is_empty()
            || !query.property_predicates(index).is_empty()
    }) {
        return Err("external query uses an unsupported predicate".into());
    }
    if completion
        && !query
            .capture_names()
            .iter()
            .any(|name| super::super::completion::category(name).is_some())
    {
        return Err("external completion query requires a declaration capture".into());
    }
    Ok(query)
}

fn capture(name: &str, completion: bool) -> bool {
    if completion {
        [
            "function",
            "type",
            "module",
            "constant",
            "binding",
            "parameter",
            "field",
            "scope",
            "comment",
            "string",
        ]
        .contains(&name)
    } else {
        super::super::Style::named(name).is_some()
    }
}
