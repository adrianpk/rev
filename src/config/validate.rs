use std::collections::BTreeMap;
use std::ops::RangeInclusive;

use crate::input::keymap;

use super::{Document, Keymaps, PartialGutterPalette, PartialPalette, PartialSyntaxPalette, Rgb};

pub(super) enum Target {
    Key,
    Value,
}

pub(super) struct Failure {
    pub key: Vec<String>,
    pub message: String,
    pub target: Target,
}

impl Failure {
    fn new(key: &str, message: String) -> Self {
        Self {
            key: key.split('.').map(str::to_owned).collect(),
            message,
            target: Target::Value,
        }
    }

    fn at(key: &[&str], message: &str, target: Target) -> Self {
        Self {
            key: key.iter().map(|key| (*key).to_owned()).collect(),
            message: message.to_owned(),
            target,
        }
    }
}

pub(super) fn document(document: &Document) -> Result<keymap::Effective, Failure> {
    integer("version", document.version, 1..=1)?;
    editing(document)?;
    retention(document)?;
    gutter(document)?;
    float(
        "windows.split_ratio",
        document.windows.split_ratio,
        0.1..=0.9,
    )?;
    float("tree.max_fraction", document.tree.max_fraction, 0.1..=0.5)?;
    palette(&document.palette)?;
    languages(document)?;
    associations(&document.files.associations)?;
    bindings(&document.keys)
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
        ("mouse.scroll_lines", document.mouse.scroll_lines, 1..=100),
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
    gutter_palette(&palette.gutter)?;
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

fn languages(document: &Document) -> Result<(), Failure> {
    for (id, language) in &document.language {
        if !language_id(id) {
            return Err(Failure::at(
                &["language", id],
                "expected lowercase language ID starting with a letter",
                Target::Key,
            ));
        }
        integer(
            &format!("language.{id}.indent_width"),
            language.indent_width,
            1..=32,
        )?;
        integer(
            &format!("language.{id}.tab_width"),
            language.tab_width,
            1..=32,
        )?;
    }
    Ok(())
}

fn language_id(id: &str) -> bool {
    id.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn associations(associations: &BTreeMap<String, String>) -> Result<(), Failure> {
    for (pattern, id) in associations {
        let key = ["files", "associations", pattern];
        if !association_pattern(pattern) {
            return Err(Failure::at(
                &key,
                "expected literal basename or leading *.suffix without other wildcards or path separators",
                Target::Key,
            ));
        }
        if !language_id(id) {
            return Err(Failure::at(
                &key,
                "expected lowercase language ID starting with a letter",
                Target::Value,
            ));
        }
    }
    Ok(())
}

fn association_pattern(pattern: &str) -> bool {
    let literal = pattern.strip_prefix("*.").unwrap_or(pattern);
    !literal.is_empty()
        && !literal
            .chars()
            .any(|character| character.is_control() || "*?[]/\\".contains(character))
}

fn bindings(keys: &Keymaps) -> Result<keymap::Effective, Failure> {
    keymap::compile(keys).map_err(|error| Failure {
        key: vec!["keys".into(), error.context.name().into(), error.sequence],
        message: error.message,
        target: match error.site {
            keymap::Site::Key => Target::Key,
            keymap::Site::Value => Target::Value,
        },
    })
}

fn gutter(document: &Document) -> Result<(), Failure> {
    for (key, value, range) in [
        (
            "gutter.minimum_digits",
            document.gutter.minimum_digits,
            1..=20,
        ),
        ("gutter.left_padding", document.gutter.left_padding, 0..=4),
        ("gutter.right_padding", document.gutter.right_padding, 0..=4),
    ] {
        integer(key, value, range)?;
    }
    Ok(())
}

fn gutter_palette(palette: &PartialGutterPalette) -> Result<(), Failure> {
    for (key, value) in [
        ("background", &palette.background),
        ("foreground", &palette.foreground),
        ("current_line", &palette.current_line),
        ("separator", &palette.separator),
    ] {
        color(&format!("palette.gutter.{key}"), value.as_deref())?;
    }
    Ok(())
}
