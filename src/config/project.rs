use super::{
    Document, Editor, Files, History, KillRing, Language, Minibuffer, Palette, PartialEditor,
    PartialFiles, PartialHistory, PartialKillRing, PartialMinibuffer, PartialPalette,
    PartialScroll, PartialSyntax, PartialSyntaxPalette, PartialTree, PartialWindows, Rgb, Scroll,
    Settings, Syntax, SyntaxPalette, Tree, Windows,
};

// Projection runs only after validating every partial and the complete candidate.
// Numeric inputs then fit usize, and color literals are known to decode.
pub(super) fn settings(document: &Document) -> Settings {
    let mut settings = Settings::default();
    editor(&mut settings.editor, &document.editor);
    scroll(&mut settings.scroll, &document.scroll);
    windows(&mut settings.windows, &document.windows);
    minibuffer(&mut settings.minibuffer, &document.minibuffer);
    history(&mut settings.history, &document.history);
    kill_ring(&mut settings.kill_ring, &document.kill_ring);
    tree(&mut settings.tree, &document.tree);
    syntax(&mut settings.syntax, &document.syntax);
    files(&mut settings.files, &document.files);
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
    syntax_palette(&mut base.syntax, &partial.syntax);
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
