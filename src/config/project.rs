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
    if let Some(value) = document.macros.ring_limit {
        settings.macros.ring_limit = usize::try_from(value).expect("validated configuration count");
    }
    if let Some(value) = document.macros.execute_before_append {
        settings.macros.execute_before_append = value;
    }
    tree(&mut settings.tree, &document.tree);
    syntax(&mut settings.syntax, &document.syntax);
    files(&mut settings.files, &document.files);
    bookmarks(&mut settings.bookmarks, &document.bookmarks);
    completion(&mut settings.completion, &document.completion);
    settings.lsp = document.lsp.project();
    settings.ai = document.ai.project();
    settings.shell = document.shell.project();
    settings.git = document.git.project();
    settings.terminal = document.terminal.project();
    settings.plugins = document.plugins.project();
    org(&mut settings.org, &document.org);
    palette(&mut settings.palette, &document.palette);
    settings.language.clone_from(&document.language);
    settings.keys.clone_from(&document.keys);
    settings
}

fn org(settings: &mut super::Org, partial: &super::PartialOrg) {
    scalar(&mut settings.enabled, partial.enabled);
    scalar(&mut settings.indent, partial.indent);
    scalar(&mut settings.startup_folded, partial.startup_folded);
    headings(&mut settings.headings, &partial.headings);
    scalar(&mut settings.tags.inherit, partial.tags.inherit);
    if let Some(names) = &partial.properties.inherit {
        settings.properties.inherit.clone_from(names);
    }
    todo(&mut settings.todo, &partial.todo);
    priority(&mut settings.priority, &partial.priority);
    duration(&mut settings.duration.units, &partial.duration.units);
    time(&mut settings.time, &partial.time);
    habit(&mut settings.habit, &partial.habit);
    agenda(settings, partial);
    tables(settings, partial);
    export(settings, partial);
    source(&mut settings.source, &partial.source);
    feeds(settings, partial);
    scalar(&mut settings.preview.enabled, partial.preview.enabled);
    count(&mut settings.preview.max_bytes, partial.preview.max_bytes);
    if let Some(value) = &partial.publish.projects {
        settings.publish.projects.clone_from(value);
    }
    if let Some(value) = &partial.capture.templates {
        settings.capture.templates.clone_from(value);
    }
    if let Some(value) = &partial.capture.default_file {
        settings.capture.default_file.clone_from(value);
    }
    if let Some(value) = &partial.refile.targets {
        settings.refile.targets.clone_from(value);
    }
    if let Some(value) = &partial.archive.location {
        settings.archive.location.clone_from(value);
    }
    if let Some(value) = &partial.attachments.directory {
        settings.attachments.directory.clone_from(value);
    }
    tools(&mut settings.tools, &partial.tools);
    clock(&mut settings.clock, &partial.clock);
    if let Some(value) = &partial.timer.default {
        settings.timer.default.clone_from(value);
    }
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
        *base = usize::try_from(value).expect("validated configuration count");
    }
}

fn color(base: &mut Rgb, value: &Option<String>) {
    if let Some(value) = value {
        *base = Rgb::parse(value).expect("validated RGB literal");
    }
}

fn editor(base: &mut Editor, partial: &PartialEditor) {
    scalar(&mut base.interaction_profile, partial.interaction_profile);
    scalar(&mut base.soft_wrap, partial.soft_wrap);
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
    scalar(&mut base.follow_active_buffer, partial.follow_active_buffer);
    scalar(&mut base.decorated, partial.decorated);
    scalar(&mut base.auto_width, partial.auto_width);
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
    scalar(&mut base.external_changes, partial.external_changes);
    count(
        &mut base.external_change_debounce_ms,
        partial.external_change_debounce_ms,
    );
    count(
        &mut base.external_change_max_bytes,
        partial.external_change_max_bytes,
    );
    count(
        &mut base.external_change_max_buffers,
        partial.external_change_max_buffers,
    );
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
    git_palette(&mut base.git, &partial.git);
    tree_palette(&mut base.tree_entries, &partial.tree_entries);
}

fn git_palette(base: &mut super::GitPalette, partial: &super::PartialGitPalette) {
    color(&mut base.heading, &partial.heading);
    color(&mut base.metadata, &partial.metadata);
    color(&mut base.untracked, &partial.untracked);
    color(&mut base.unstaged, &partial.unstaged);
    color(&mut base.staged, &partial.staged);
    color(&mut base.conflict, &partial.conflict);
    color(&mut base.added, &partial.added);
    color(&mut base.removed, &partial.removed);
    color(&mut base.hunk, &partial.hunk);
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
    scalar(&mut base.nerd_fonts, partial.nerd_fonts);
    scalar(&mut base.mode, partial.mode);
    scalar(&mut base.current_line, partial.current_line);
    scalar(&mut base.side, partial.side);
    let defaults = base.format;
    base.format = crate::gutter::Format::new(
        partial
            .minimum_digits
            .map_or(defaults.minimum_digits(), |n| {
                u8::try_from(n).expect("validated gutter count")
            }),
        partial.left_padding.map_or(defaults.left_padding(), |n| {
            u8::try_from(n).expect("validated gutter count")
        }),
        partial.right_padding.map_or(defaults.right_padding(), |n| {
            u8::try_from(n).expect("validated gutter count")
        }),
    )
    .expect("validated gutter format");
}

fn gutter_palette(base: &mut GutterPalette, partial: &PartialGutterPalette) {
    color(&mut base.bookmark, &partial.bookmark);
    color(&mut base.background, &partial.background);
    color(&mut base.foreground, &partial.foreground);
    color(&mut base.current_line, &partial.current_line);
    color(&mut base.separator, &partial.separator);
}

fn mouse(base: &mut Mouse, partial: &PartialMouse) {
    scalar(&mut base.enabled, partial.enabled);
    count(&mut base.scroll_lines, partial.scroll_lines);
    scalar(
        &mut base.tree_activate_on_click,
        partial.tree_activate_on_click,
    );
}

fn org_palette(base: &mut OrgPalette, partial: &PartialOrgPalette) {
    color(&mut base.heading, &partial.heading);
    color(&mut base.heading_2, &partial.heading_2);
    color(&mut base.heading_3, &partial.heading_3);
    color(&mut base.heading_4, &partial.heading_4);
    color(&mut base.heading_5, &partial.heading_5);
    color(&mut base.heading_6, &partial.heading_6);
    color(&mut base.heading_7, &partial.heading_7);
    color(&mut base.heading_8, &partial.heading_8);
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
    color(&mut base.habit_clear, &partial.habit_clear);
    color(&mut base.habit_clear_future, &partial.habit_clear_future);
    color(&mut base.habit_ready, &partial.habit_ready);
    color(&mut base.habit_ready_future, &partial.habit_ready_future);
    color(&mut base.habit_alert, &partial.habit_alert);
    color(&mut base.habit_alert_future, &partial.habit_alert_future);
    color(&mut base.habit_overdue, &partial.habit_overdue);
    color(
        &mut base.habit_overdue_future,
        &partial.habit_overdue_future,
    );
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
    count(&mut base.extend_today_until, partial.extend_today_until);
    scalar(&mut base.ambiguous, partial.ambiguous);
    scalar(&mut base.nonexistent, partial.nonexistent);
}

pub(super) fn clock(base: &mut super::Clock, partial: &super::PartialClock) {
    scalar(&mut base.persist, partial.persist);
    count(&mut base.history_limit, partial.history_limit);
    scalar(&mut base.out_when_done, partial.out_when_done);
    if let Some(value) = &partial.drawer {
        base.drawer.clone_from(value);
    }
    scalar(&mut base.remove_zero, partial.remove_zero);
}

fn tables(base: &mut super::Org, partial: &super::PartialOrg) {
    count(
        &mut base.tables.significant_digits,
        partial.tables.significant_digits,
    );
    count(&mut base.limits.formula_cells, partial.limits.formula_cells);
    count(&mut base.limits.formula_steps, partial.limits.formula_steps);
    count(&mut base.limits.formula_depth, partial.limits.formula_depth);
    count(
        &mut base.limits.formula_decimal_places,
        partial.limits.formula_decimal_places,
    );
}
fn habit(base: &mut super::Habit, partial: &super::PartialHabit) {
    count(&mut base.preceding_days, partial.preceding_days);
    count(&mut base.following_days, partial.following_days);
    count(&mut base.graph_column, partial.graph_column);
    scalar(&mut base.today_only, partial.today_only);
    scalar(&mut base.show_all_today, partial.show_all_today);
    scalar(&mut base.done_always_green, partial.done_always_green);
}

fn agenda(base: &mut super::Org, partial: &super::PartialOrg) {
    scalar(&mut base.agenda.span, partial.agenda.span);
    scalar(
        &mut base.agenda.include_archived,
        partial.agenda.include_archived,
    );
    if let Some(views) = &partial.agenda.views {
        base.agenda.views.clone_from(views);
    }
    count(&mut base.agenda.week_start, partial.agenda.week_start);
    count(&mut base.limits.agenda_rows, partial.limits.agenda_rows);
    count(&mut base.limits.query_bytes, partial.limits.query_bytes);
    count(&mut base.limits.query_depth, partial.limits.query_depth);
    if let Some(paths) = &partial.agenda.sources {
        base.agenda.sources.clone_from(paths);
    }
    let target = &mut base.limits;
    let limits = &partial.limits;
    count(&mut target.agenda_files, limits.agenda_files);
    count(&mut target.agenda_file_bytes, limits.agenda_file_bytes);
    count(&mut target.agenda_total_bytes, limits.agenda_total_bytes);
    count(&mut target.job_seconds, limits.job_seconds);
    count(&mut target.job_output_bytes, limits.job_output_bytes);
}

fn export(base: &mut super::Org, partial: &super::PartialOrg) {
    scalar(
        &mut base.export.default_format,
        partial.export.default_format,
    );
    if let Some(backends) = &partial.export.backends {
        base.export.backends.clone_from(backends);
    }
    count(&mut base.limits.include_depth, partial.limits.include_depth);
    count(&mut base.limits.include_files, partial.limits.include_files);
    count(
        &mut base.limits.expansion_bytes,
        partial.limits.expansion_bytes,
    );
    count(&mut base.limits.export_bytes, partial.limits.export_bytes);
}

fn tools(base: &mut super::Tools, partial: &super::PartialTools) {
    if let Some(value) = &partial.upload {
        base.upload.clone_from(value);
    }
    if let Some(value) = &partial.viewer {
        base.viewer.clone_from(value);
    }
    if let Some(value) = &partial.pdf {
        base.pdf.clone_from(value);
    }
}

fn source(base: &mut super::SourcePrograms, partial: &super::PartialSourcePrograms) {
    if let Some(files) = &partial.library_files {
        base.library_files.clone_from(files);
    }
    scalar(&mut base.confirm_execution, partial.confirm_execution);
    if let Some(programs) = &partial.programs {
        base.programs.clone_from(programs);
    }
}

fn feeds(base: &mut super::Org, partial: &super::PartialOrg) {
    if let Some(sources) = &partial.feeds.sources {
        base.feeds.sources.clone_from(sources);
    }
    if let Some(argv) = &partial.tools.fetch {
        base.tools.fetch.clone_from(argv);
    }
    count(&mut base.limits.feed_bytes, partial.limits.feed_bytes);
    count(&mut base.limits.feed_items, partial.limits.feed_items);
}

fn bookmarks(base: &mut super::Bookmarks, partial: &super::PartialBookmarks) {
    if let Some(path) = &partial.file {
        base.file = Some(path.clone());
    }
    scalar(&mut base.save, partial.save);
    scalar(&mut base.markers, partial.markers);
    count(&mut base.max_entries, partial.max_entries);
    count(&mut base.context_chars, partial.context_chars);
    count(&mut base.search_chars, partial.search_chars);
}

fn completion(base: &mut super::Completion, partial: &super::PartialCompletion) {
    scalar(&mut base.enabled, partial.enabled);
    scalar(&mut base.automatic, partial.automatic);
    count(&mut base.delay_ms, partial.delay_ms);
    count(&mut base.min_prefix, partial.min_prefix);
    scalar(&mut base.in_comments, partial.in_comments);
    scalar(&mut base.in_strings, partial.in_strings);
    count(&mut base.max_candidates, partial.max_candidates);
    count(&mut base.popup_rows, partial.popup_rows);
}

fn headings(base: &mut super::Headings, partial: &super::PartialHeadings) {
    scalar(&mut base.pretty, partial.pretty);
    scalar(&mut base.hide_leading_stars, partial.hide_leading_stars);
    scalar(&mut base.cycle_bullets, partial.cycle_bullets);
    if let Some(bullets) = &partial.bullets {
        base.bullets.clone_from(bullets);
    }
}

fn tree_palette(base: &mut super::TreePalette, partial: &super::PartialTreePalette) {
    color(&mut base.root, &partial.root);
    color(&mut base.directory, &partial.directory);
    color(&mut base.source, &partial.source);
    color(&mut base.document, &partial.document);
    color(&mut base.configuration, &partial.configuration);
    color(&mut base.file, &partial.file);
    color(&mut base.symlink, &partial.symlink);
}
