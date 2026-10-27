use std::ops::RangeInclusive;

use super::{Document, PartialPalette, PartialSyntaxPalette, Rgb};

pub(super) struct Failure {
    pub key: Vec<String>,
    pub message: String,
}

impl Failure {
    fn new(key: &str, message: String) -> Self {
        Self {
            key: key.split('.').map(str::to_owned).collect(),
            message,
        }
    }
}

pub(super) fn document(document: &Document) -> Result<(), Failure> {
    integer("version", document.version, 1..=1)?;
    editing(document)?;
    retention(document)?;
    float(
        "windows.split_ratio",
        document.windows.split_ratio,
        0.1..=0.9,
    )?;
    float("tree.max_fraction", document.tree.max_fraction, 0.1..=0.5)?;
    palette(&document.palette)
}

fn editing(document: &Document) -> Result<(), Failure> {
    let fields = [
        ("editor.indent_width", document.editor.indent_width, 1..=32),
        ("editor.tab_width", document.editor.tab_width, 1..=32),
        (
            "scroll.page_overlap",
            document.scroll.page_overlap,
            0..=1000,
        ),
        (
            "minibuffer.candidate_limit",
            document.minibuffer.candidate_limit,
            1..=32,
        ),
        ("tree.width", document.tree.width, 1..=512),
        (
            "syntax.max_bytes",
            document.syntax.max_bytes,
            0..=1_073_741_824,
        ),
    ];
    for (key, value, range) in fields {
        integer(key, value, range)?;
    }
    Ok(())
}

fn retention(document: &Document) -> Result<(), Failure> {
    let fields = [
        (
            "minibuffer.history_limit",
            document.minibuffer.history_limit,
            1..=4096,
        ),
        (
            "minibuffer.input_limit",
            document.minibuffer.input_limit,
            1..=65536,
        ),
        (
            "history.group_limit",
            document.history.group_limit,
            1..=65536,
        ),
        (
            "history.payload_target_bytes",
            document.history.payload_target_bytes,
            1..=1_073_741_824,
        ),
        ("kill_ring.capacity", document.kill_ring.capacity, 1..=4096),
    ];
    for (key, value, range) in fields {
        integer(key, value, range)?;
    }
    Ok(())
}

fn integer(key: &str, value: Option<i64>, range: RangeInclusive<i64>) -> Result<(), Failure> {
    if value.is_some_and(|value| !range.contains(&value)) {
        return Err(Failure::new(key, format!("expected integer in {range:?}")));
    }
    Ok(())
}

fn float(key: &str, value: Option<f64>, range: RangeInclusive<f64>) -> Result<(), Failure> {
    if value.is_some_and(|value| !value.is_finite() || !range.contains(&value)) {
        return Err(Failure::new(
            key,
            format!("expected finite float in {range:?}"),
        ));
    }
    Ok(())
}

fn palette(palette: &PartialPalette) -> Result<(), Failure> {
    let colors = [
        ("background", &palette.background),
        ("foreground", &palette.foreground),
        ("cursor", &palette.cursor),
        ("selection", &palette.selection),
        ("selection_text", &palette.selection_text),
        ("search", &palette.search),
        ("search_text", &palette.search_text),
        ("status", &palette.status),
        ("status_text", &palette.status_text),
        ("status_inactive", &palette.status_inactive),
        ("border", &palette.border),
        ("border_active", &palette.border_active),
        ("tree", &palette.tree),
        ("tree_selected", &palette.tree_selected),
        ("tree_selected_text", &palette.tree_selected_text),
        ("message", &palette.message),
        ("warning", &palette.warning),
        ("error", &palette.error),
    ];
    for (key, value) in colors {
        color(&format!("palette.{key}"), value.as_deref())?;
    }
    syntax_palette(&palette.syntax)
}

fn syntax_palette(palette: &PartialSyntaxPalette) -> Result<(), Failure> {
    let colors = [
        ("keyword", &palette.keyword),
        ("string", &palette.string),
        ("comment", &palette.comment),
        ("type", &palette.r#type),
        ("function", &palette.function),
        ("number", &palette.number),
        ("punctuation", &palette.punctuation),
    ];
    for (key, value) in colors {
        color(&format!("palette.syntax.{key}"), value.as_deref())?;
    }
    Ok(())
}

fn color(key: &str, value: Option<&str>) -> Result<(), Failure> {
    if value.is_some_and(|value| Rgb::parse(value).is_none()) {
        return Err(Failure::new(
            key,
            "expected six-digit RGB color #RRGGBB".into(),
        ));
    }
    Ok(())
}
