use super::validate::{Failure, Target};
use super::{Document, External};

pub(super) fn registrations(document: &Document) -> Result<(), Failure> {
    if document
        .language
        .values()
        .filter(|language| language.external.is_some())
        .count()
        > 32
    {
        return Err(Failure::at(
            &["language"],
            "external registrations exceed 32",
            Target::Key,
        ));
    }
    Ok(())
}

pub(crate) fn symbol(symbol: &str) -> bool {
    symbol.strip_prefix("tree_sitter_").is_some_and(|suffix| {
        !suffix.is_empty()
            && symbol.len() <= 128
            && suffix
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    })
}

pub(super) fn validate(id: &str, external: Option<&External>) -> Result<(), Failure> {
    let Some(external) = external else {
        return Ok(());
    };
    if !symbol(&external.symbol) {
        return Err(Failure::at(
            &["language", id, "external", "symbol"],
            "expected tree_sitter_ symbol with ASCII identifier suffix",
            Target::Key,
        ));
    }
    for (key, path) in [
        ("parser", &external.parser),
        ("highlights", &external.highlights),
        ("completions", &external.completions),
    ] {
        if path.is_empty() || path.contains('\0') || path.contains("://") {
            return Err(Failure::at(
                &["language", id, "external", key],
                "expected a nonempty local asset path",
                Target::Key,
            ));
        }
    }
    Ok(())
}
