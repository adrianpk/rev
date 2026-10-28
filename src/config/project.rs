use super::{
    Document, Editor, Files, GutterPalette, History, KillRing, Language, Minibuffer, Mouse,
    OrgPalette, Palette, PartialEditor, PartialFiles, PartialGutter, PartialGutterPalette,
    PartialHistory, PartialKillRing, PartialMinibuffer, PartialMouse, PartialOrgPalette,
    PartialPalette, PartialScroll, PartialSyntax, PartialSyntaxPalette, PartialTree,
    PartialWindows, Rgb, Scroll, Settings, Syntax, SyntaxPalette, Tree, Windows,
};

// Projection runs only after validating every partial and the complete candidate.
// Numeric inputs then fit usize, and color literals are known to decode.
pub(super) fn settings(document: &Document) -> Settings {
    let mut settings = Settings::default();
    editor(&mut settings.editor, &document.editor);
    scroll(&mut settings.scroll, &document.scroll);
    windows(&mut settings.windows, &document.windows);
    gutter(&mut settings.gutter, &document.gutter);
    mouse(&mut settings.mouse, &document.mouse);
    minibuffer(&mut settings.minibuffer, &document.minibuffer);
    history(&mut settings.history, &document.history);
    kill_ring(&mut settings.kill_ring, &document.kill_ring);
    tree(&mut settings.tree, &document.tree);
    syntax(&mut settings.syntax, &document.syntax);
    files(&mut settings.files, &document.files);
    scalar(&mut settings.org.enabled, document.org.enabled);
    scalar(&mut settings.org.tags.inherit, document.org.tags.inherit);
    if let Some(names) = &document.org.properties.inherit {
        settings.org.properties.inherit.clone_from(names);
    }
    todo(&mut settings.org.todo, &document.org.todo);
    priority(&mut settings.org.priority, &document.org.priority);
    duration(
        &mut settings.org.duration.units,
        &document.org.duration.units,
    );
    time(&mut settings.org.time, &document.org.time);
    clock(&mut settings.org.clock, &document.org.clock);
    palette(&mut settings.palette, &document.palette);
    settings.language.clone_from(&document.language);
    settings.keys.clone_from(&document.keys);
    settings
}

pub(super) fn language(global: &Editor, language: Option<&Language>) -> Editor {
    let mut editor = global.clone();
    if let Some(language) = language {
        scalar(&mut editor.indent_style, language.indent_style);
        count(&mut editor.indent_width, language.indent_width);
        count(&mut editor.tab_width, language.tab_width);
    }
    editor
}

fn scalar<T: Copy>(base: &mut T, value: Option<T>) {
    if let Some(value) = value {
        *base = value;
    }
}

fn count(base: &mut usize, value: Option<i64>) {
    if let Some(value) = value {
        *base = value as usize;
    }
}

fn color(base: &mut Rgb, value: &Option<String>) {
    if let Some(value) = value {
        *base = Rgb::parse(value).expect("validated RGB literal");
    }
}

fn editor(base: &mut Editor, partial: &PartialEditor) {
    scalar(&mut base.indent_style, partial.indent_style);
    count(&mut base.indent_width, partial.indent_width);
    count(&mut base.tab_width, partial.tab_width);
}

fn scroll(base: &mut Scroll, partial: &PartialScroll) {
    count(&mut base.page_overlap, partial.page_overlap);
}

fn windows(base: &mut Windows, partial: &PartialWindows) {
    scalar(&mut base.split_ratio, partial.split_ratio);
}

fn minibuffer(base: &mut Minibuffer, partial: &PartialMinibuffer) {
    count(&mut base.candidate_limit, partial.candidate_limit);
    scalar(&mut base.r#match, partial.r#match);
    count(&mut base.history_limit, partial.history_limit);
    count(&mut base.input_limit, partial.input_limit);
}

fn history(base: &mut History, partial: &PartialHistory) {
    count(&mut base.group_limit, partial.group_limit);
    count(&mut base.payload_target_bytes, partial.payload_target_bytes);
}

fn kill_ring(base: &mut KillRing, partial: &PartialKillRing) {
    count(&mut base.capacity, partial.capacity);
    scalar(&mut base.coalesce, partial.coalesce);
}

fn tree(base: &mut Tree, partial: &PartialTree) {
    scalar(&mut base.visible, partial.visible);
    count(&mut base.width, partial.width);
    scalar(&mut base.max_fraction, partial.max_fraction);
    scalar(&mut base.root, partial.root);
    scalar(&mut base.show_hidden, partial.show_hidden);
    scalar(&mut base.directories_first, partial.directories_first);
}

fn syntax(base: &mut Syntax, partial: &PartialSyntax) {
    scalar(&mut base.enabled, partial.enabled);
    count(&mut base.max_bytes, partial.max_bytes);
}

fn files(base: &mut Files, partial: &PartialFiles) {
    scalar(&mut base.newline, partial.newline);
    scalar(&mut base.bom, partial.bom);
    base.associations.clone_from(&partial.associations);
}

fn palette(base: &mut Palette, partial: &PartialPalette) {
    color(&mut base.background, &partial.background);
    color(&mut base.foreground, &partial.foreground);
    color(&mut base.cursor, &partial.cursor);
    color(&mut base.selection, &partial.selection);
    color(&mut base.selection_text, &partial.selection_text);
    color(&mut base.search, &partial.search);
    color(&mut base.search_text, &partial.search_text);
    color(&mut base.status, &partial.status);
    color(&mut base.status_text, &partial.status_text);
    color(&mut base.status_inactive, &partial.status_inactive);
    color(&mut base.border, &partial.border);
    color(&mut base.border_active, &partial.border_active);
    color(&mut base.tree, &partial.tree);
    color(&mut base.tree_selected, &partial.tree_selected);
    color(&mut base.tree_selected_text, &partial.tree_selected_text);
    color(&mut base.message, &partial.message);
    color(&mut base.warning, &partial.warning);
    color(&mut base.error, &partial.error);
    org_palette(&mut base.org, &partial.org);
    syntax_palette(&mut base.syntax, &partial.syntax);
    gutter_palette(&mut base.gutter, &partial.gutter);
}

fn syntax_palette(base: &mut SyntaxPalette, partial: &PartialSyntaxPalette) {
    color(&mut base.keyword, &partial.keyword);
    color(&mut base.string, &partial.string);
    color(&mut base.comment, &partial.comment);
    color(&mut base.r#type, &partial.r#type);
    color(&mut base.function, &partial.function);
    color(&mut base.number, &partial.number);
    color(&mut base.punctuation, &partial.punctuation);
}

fn gutter(base: &mut crate::gutter::Settings, partial: &PartialGutter) {
    scalar(&mut base.visible, partial.visible);
    scalar(&mut base.mode, partial.mode);
    scalar(&mut base.current_line, partial.current_line);
    scalar(&mut base.side, partial.side);
    let defaults = base.format;
    base.format = crate::gutter::Format::new(
        partial
            .minimum_digits
            .map_or(defaults.minimum_digits(), |n| n as u8),
        partial
            .left_padding
            .map_or(defaults.left_padding(), |n| n as u8),
        partial
            .right_padding
            .map_or(defaults.right_padding(), |n| n as u8),
    )
    .expect("validated gutter format");
}

fn gutter_palette(base: &mut GutterPalette, partial: &PartialGutterPalette) {
    color(&mut base.background, &partial.background);
    color(&mut base.foreground, &partial.foreground);
    color(&mut base.current_line, &partial.current_line);
    color(&mut base.separator, &partial.separator);
}

fn mouse(base: &mut Mouse, partial: &PartialMouse) {
    scalar(&mut base.enabled, partial.enabled);
    count(&mut base.scroll_lines, partial.scroll_lines);
}

fn org_palette(base: &mut OrgPalette, partial: &PartialOrgPalette) {
    color(&mut base.heading, &partial.heading);
    color(&mut base.keyword, &partial.keyword);
    color(&mut base.drawer, &partial.drawer);
    color(&mut base.property, &partial.property);
    color(&mut base.planning, &partial.planning);
    color(&mut base.block, &partial.block);
    color(&mut base.code, &partial.code);
    color(&mut base.comment, &partial.comment);
    color(&mut base.list, &partial.list);
    color(&mut base.table, &partial.table);
    color(&mut base.formula, &partial.formula);
    color(&mut base.emphasis, &partial.emphasis);
    color(&mut base.link, &partial.link);
    color(&mut base.target, &partial.target);
    color(&mut base.footnote, &partial.footnote);
    color(&mut base.entity, &partial.entity);
    color(&mut base.malformed, &partial.malformed);
}

pub(super) fn todo(base: &mut super::Todo, partial: &super::PartialTodo) {
    if let Some(value) = &partial.sequence {
        base.sequence.clone_from(value);
    }
    if let Some(value) = &partial.finished {
        base.finished.clone_from(value);
    }
    if let Some(value) = &partial.log_drawer {
        base.log_drawer.clone_from(value);
    }
    scalar(&mut base.log_done, partial.log_done);
    scalar(&mut base.log_repeat, partial.log_repeat);
    scalar(&mut base.block_children, partial.block_children);
}

pub(super) fn priority(base: &mut super::Priority, partial: &super::PartialPriority) {
    if let Some(value) = &partial.highest {
        base.highest.clone_from(value);
    }
    if let Some(value) = &partial.lowest {
        base.lowest.clone_from(value);
    }
    if let Some(value) = &partial.default {
        base.default.clone_from(value);
    }
}
pub(super) fn duration(base: &mut super::DurationUnits, partial: &super::PartialDurationUnits) {
    scalar(&mut base.min, partial.min);
    scalar(&mut base.h, partial.h);
    scalar(&mut base.d, partial.d);
    scalar(&mut base.w, partial.w);
    scalar(&mut base.m, partial.m);
    scalar(&mut base.y, partial.y);
}

pub(super) fn time(base: &mut super::Time, partial: &super::PartialTime) {
    if let Some(zone) = &partial.zone {
        base.zone.clone_from(zone);
    }
    scalar(&mut base.ambiguous, partial.ambiguous);
    scalar(&mut base.nonexistent, partial.nonexistent);
}

pub(super) fn clock(base: &mut super::Clock, partial: &super::PartialClock) {
    scalar(&mut base.out_when_done, partial.out_when_done);
    if let Some(value) = &partial.drawer {
        base.drawer.clone_from(value);
    }
    scalar(&mut base.remove_zero, partial.remove_zero);
}
