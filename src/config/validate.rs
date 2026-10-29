use std::collections::BTreeMap;
use std::ops::RangeInclusive;

use crate::input::keymap;

use super::{
    Document, Keymaps, PartialGutterPalette, PartialOrgPalette, PartialPalette,
    PartialSyntaxPalette, Rgb,
};

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
    properties(&document.org.properties)?;
    todo(document)?;
    time(document)?;
    clock(document)?;
    timer(document)?;
    estimates(document)
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
    org_palette(&palette.org)?;
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

fn org_palette(palette: &PartialOrgPalette) -> Result<(), Failure> {
    let colors = [
        ("heading", &palette.heading),
        ("keyword", &palette.keyword),
        ("drawer", &palette.drawer),
        ("property", &palette.property),
        ("planning", &palette.planning),
        ("block", &palette.block),
        ("code", &palette.code),
        ("comment", &palette.comment),
        ("list", &palette.list),
        ("table", &palette.table),
        ("formula", &palette.formula),
        ("emphasis", &palette.emphasis),
        ("link", &palette.link),
        ("target", &palette.target),
        ("footnote", &palette.footnote),
        ("entity", &palette.entity),
        ("malformed", &palette.malformed),
        ("habit_clear", &palette.habit_clear),
        ("habit_clear_future", &palette.habit_clear_future),
        ("habit_ready", &palette.habit_ready),
        ("habit_ready_future", &palette.habit_ready_future),
        ("habit_alert", &palette.habit_alert),
        ("habit_alert_future", &palette.habit_alert_future),
        ("habit_overdue", &palette.habit_overdue),
        ("habit_overdue_future", &palette.habit_overdue_future),
    ];
    for (key, value) in colors {
        color(&format!("palette.org.{key}"), value.as_deref())?;
    }
    Ok(())
}

fn properties(partial: &super::PartialProperties) -> Result<(), Failure> {
    let Some(names) = &partial.inherit else {
        return Ok(());
    };
    if names.len() > 128 {
        return Err(Failure::new(
            "org.properties.inherit",
            "expected at most 128 property names".into(),
        ));
    }
    let mut unique = std::collections::BTreeSet::new();
    for name in names {
        let key = crate::org::property_name(name)
            .map_err(|e| Failure::new("org.properties.inherit", e))?;
        if name.ends_with('+') || crate::org::derived_property(&key) || !unique.insert(key) {
            return Err(Failure::new(
                "org.properties.inherit",
                "expected unique stored property names without + or derived fields".into(),
            ));
        }
    }
    Ok(())
}

fn todo(document: &Document) -> Result<(), Failure> {
    let mut policy = super::Todo::default();
    super::project::todo(&mut policy, &document.org.todo);
    crate::org::task::Workflow::configured(&policy)
        .map_err(|e| Failure::new("org.todo.sequence", e))?;
    if !policy.log_drawer.is_empty() && !crate::org::task::drawer_name(&policy.log_drawer) {
        return Err(Failure::new("org.todo.log_drawer", "Use an empty value or a drawer name of 1..64 ASCII letters, digits, _ or -; PROPERTIES and END are reserved".into()));
    }
    Ok(())
}

fn estimates(document: &Document) -> Result<(), Failure> {
    let priority = &document.org.priority;
    for (key, value) in [
        ("highest", &priority.highest),
        ("lowest", &priority.lowest),
        ("default", &priority.default),
    ] {
        if let Some(value) = value {
            crate::org::priority::Priority::new(value, value, value)
                .map_err(|e| Failure::new(&format!("org.priority.{key}"), e))?;
        }
    }
    let units = &document.org.duration.units;
    for (key, value) in [
        ("min", units.min),
        ("h", units.h),
        ("d", units.d),
        ("w", units.w),
        ("m", units.m),
        ("y", units.y),
    ] {
        if value.is_some_and(|v| !v.is_finite() || v <= 0.0) {
            return Err(Failure::new(
                &format!("org.duration.units.{key}"),
                "Duration units must be positive finite minutes".into(),
            ));
        }
    }
    Ok(())
}

pub(super) fn candidate(value: &Document) -> Result<keymap::Effective, Failure> {
    let keymaps = document(value)?;
    let mut priority = super::Priority::default();
    super::project::priority(&mut priority, &value.org.priority);
    crate::org::priority::Priority::configured(&priority)
        .map_err(|e| Failure::new("org.priority.default", e))?;
    let policy = super::project::settings(value).org;
    for view in &policy.agenda.views {
        for block in &view.blocks {
            crate::org::agenda::AgendaQuery::block(block, &policy.limits)
                .map_err(|e| Failure::new("org.agenda.views", e))?;
        }
    }
    Ok(keymaps)
}

fn time(document: &Document) -> Result<(), Failure> {
    habit(&document.org.habit)?;
    agenda(&document.org)?;
    integer(
        "org.time.extend_today_until",
        document.org.time.extend_today_until,
        0..=23,
    )?;
    if let Some(zone) = &document.org.time.zone {
        crate::org::time::ZoneSelection::parse(zone)
            .map_err(|e| Failure::new("org.time.zone", e))?;
    }
    Ok(())
}

fn clock(document: &Document) -> Result<(), Failure> {
    integer(
        "org.clock.history_limit",
        document.org.clock.history_limit,
        0..=256,
    )?;
    if let Some(drawer) = &document.org.clock.drawer
        && !drawer.is_empty()
        && !crate::org::task::drawer_name(drawer)
    {
        return Err(Failure::new("org.clock.drawer", "Use an empty value or a drawer name of 1..64 ASCII letters, digits, _ or -; PROPERTIES and END are reserved".into()));
    }
    Ok(())
}

fn timer(document: &Document) -> Result<(), Failure> {
    if let Some(value) = &document.org.timer.default {
        crate::org::time::TimerDuration::parse(value)
            .map_err(|e| Failure::new("org.timer.default", e))?;
    }
    Ok(())
}

fn habit(partial: &super::PartialHabit) -> Result<(), Failure> {
    integer("org.habit.preceding_days", partial.preceding_days, 0..=366)?;
    integer("org.habit.following_days", partial.following_days, 0..=366)?;
    integer("org.habit.graph_column", partial.graph_column, 0..=4096)?;
    Ok(())
}

fn agenda(org: &super::PartialOrg) -> Result<(), Failure> {
    integer("org.agenda.week_start", org.agenda.week_start, 1..=7)?;
    integer("org.limits.agenda_rows", org.limits.agenda_rows, 1..=100000)?;
    query_limits(&org.limits)?;
    agenda_views(&org.agenda)?;
    let limits = &org.limits;
    integer("org.limits.agenda_files", limits.agenda_files, 1..=4096)?;
    integer(
        "org.limits.agenda_file_bytes",
        limits.agenda_file_bytes,
        1..=33554432,
    )?;
    integer(
        "org.limits.agenda_total_bytes",
        limits.agenda_total_bytes,
        1..=268435456,
    )?;
    integer("org.limits.job_seconds", limits.job_seconds, 1..=600)?;
    agenda_paths(&org.agenda)?;
    Ok(())
}

fn query_limits(limits: &super::PartialOrgLimits) -> Result<(), Failure> {
    integer("org.limits.query_bytes", limits.query_bytes, 1..=65536)?;
    integer("org.limits.query_depth", limits.query_depth, 1..=128)
}

fn agenda_views(agenda: &super::PartialAgenda) -> Result<(), Failure> {
    if let Some(views) = &agenda.views {
        super::agenda::views(views).map_err(|e| Failure::new("org.agenda.views", e))?;
    }
    Ok(())
}

fn agenda_paths(agenda: &super::PartialAgenda) -> Result<(), Failure> {
    if let Some(paths) = &agenda.sources {
        crate::org::agenda::SourceSelection::paths(paths)
            .map_err(|e| Failure::new("org.agenda.sources", e))?;
    }
    Ok(())
}
