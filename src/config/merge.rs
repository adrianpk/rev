use std::collections::BTreeMap;

use super::{
    Document, Keymaps, Language, PartialEditor, PartialFiles, PartialGutter, PartialGutterPalette,
    PartialHistory, PartialKillRing, PartialMinibuffer, PartialMouse, PartialOrgPalette,
    PartialPalette, PartialScroll, PartialSyntax, PartialSyntaxPalette, PartialTree,
    PartialWindows,
};

#[derive(Default)]
struct Merge {
    paths: Vec<Vec<String>>,
}

impl Document {
    /// Merge a later partial layer by leaf and return the paths explicitly set
    /// by that layer, for source provenance. Absence inherits, arrays replace
    /// even when empty, and maps merge by key. Include arrays remain source
    /// metadata; this operation never discovers or loads their paths.
    pub fn overlay(&mut self, later: &Self) -> Vec<Vec<String>> {
        let mut merge = Merge::default();
        merge.leaf(&["version"], &mut self.version, &later.version);
        merge.leaf(&["include"], &mut self.include, &later.include);
        merge.editor(&mut self.editor, &later.editor);
        merge.scroll(&mut self.scroll, &later.scroll);
        merge.windows(&mut self.windows, &later.windows);
        merge.gutter(&mut self.gutter, &later.gutter);
        merge.mouse(&mut self.mouse, &later.mouse);
        merge.minibuffer(&mut self.minibuffer, &later.minibuffer);
        merge.history(&mut self.history, &later.history);
        merge.kill_ring(&mut self.kill_ring, &later.kill_ring);
        merge.tree(&mut self.tree, &later.tree);
        merge.syntax(&mut self.syntax, &later.syntax);
        merge.leaf(
            &["org", "enabled"],
            &mut self.org.enabled,
            &later.org.enabled,
        );
        merge.leaf(
            &["org", "properties", "inherit"],
            &mut self.org.properties.inherit,
            &later.org.properties.inherit,
        );
        merge.leaf(
            &["org", "tags", "inherit"],
            &mut self.org.tags.inherit,
            &later.org.tags.inherit,
        );
        merge.todo(&mut self.org.todo, &later.org.todo);
        merge.priority(&mut self.org.priority, &later.org.priority);
        merge.duration(&mut self.org.duration.units, &later.org.duration.units);
        merge.time(&mut self.org.time, &later.org.time);
        merge.files(&mut self.files, &later.files);
        merge.palette(&mut self.palette, &later.palette);
        merge.languages(&mut self.language, &later.language);
        merge.keys(&mut self.keys, &later.keys);
        merge.paths
    }
}

impl Merge {
    fn priority(&mut self, base: &mut super::PartialPriority, later: &super::PartialPriority) {
        self.leaf(
            &["org", "priority", "highest"],
            &mut base.highest,
            &later.highest,
        );
        self.leaf(
            &["org", "priority", "lowest"],
            &mut base.lowest,
            &later.lowest,
        );
        self.leaf(
            &["org", "priority", "default"],
            &mut base.default,
            &later.default,
        );
    }
    fn duration(
        &mut self,
        base: &mut super::PartialDurationUnits,
        later: &super::PartialDurationUnits,
    ) {
        self.leaf(
            &["org", "duration", "units", "min"],
            &mut base.min,
            &later.min,
        );
        self.leaf(&["org", "duration", "units", "h"], &mut base.h, &later.h);
        self.leaf(&["org", "duration", "units", "d"], &mut base.d, &later.d);
        self.leaf(&["org", "duration", "units", "w"], &mut base.w, &later.w);
        self.leaf(&["org", "duration", "units", "m"], &mut base.m, &later.m);
        self.leaf(&["org", "duration", "units", "y"], &mut base.y, &later.y);
    }
    fn todo(&mut self, base: &mut super::PartialTodo, later: &super::PartialTodo) {
        self.leaf(
            &["org", "todo", "sequence"],
            &mut base.sequence,
            &later.sequence,
        );
        self.leaf(
            &["org", "todo", "finished"],
            &mut base.finished,
            &later.finished,
        );
        self.leaf(
            &["org", "todo", "log_done"],
            &mut base.log_done,
            &later.log_done,
        );
        self.leaf(
            &["org", "todo", "log_drawer"],
            &mut base.log_drawer,
            &later.log_drawer,
        );
        self.leaf(
            &["org", "todo", "block_children"],
            &mut base.block_children,
            &later.block_children,
        );
    }
    fn mouse(&mut self, base: &mut PartialMouse, later: &PartialMouse) {
        self.leaf(&["mouse", "enabled"], &mut base.enabled, &later.enabled);
        self.leaf(
            &["mouse", "scroll_lines"],
            &mut base.scroll_lines,
            &later.scroll_lines,
        );
    }

    fn leaf<T: Clone>(&mut self, path: &[&str], base: &mut Option<T>, later: &Option<T>) {
        if later.is_some() {
            base.clone_from(later);
            self.paths
                .push(path.iter().map(|part| (*part).to_owned()).collect());
        }
    }

    fn table(
        &mut self,
        prefix: &[&str],
        base: &mut BTreeMap<String, String>,
        later: &BTreeMap<String, String>,
    ) {
        for (key, value) in later {
            base.insert(key.clone(), value.clone());
            let mut path: Vec<String> = prefix.iter().map(|part| (*part).to_owned()).collect();
            path.push(key.clone());
            self.paths.push(path);
        }
    }

    fn editor(&mut self, base: &mut PartialEditor, later: &PartialEditor) {
        self.leaf(
            &["editor", "indent_style"],
            &mut base.indent_style,
            &later.indent_style,
        );
        self.leaf(
            &["editor", "indent_width"],
            &mut base.indent_width,
            &later.indent_width,
        );
        self.leaf(
            &["editor", "tab_width"],
            &mut base.tab_width,
            &later.tab_width,
        );
    }

    fn scroll(&mut self, base: &mut PartialScroll, later: &PartialScroll) {
        self.leaf(
            &["scroll", "page_overlap"],
            &mut base.page_overlap,
            &later.page_overlap,
        );
    }

    fn windows(&mut self, base: &mut PartialWindows, later: &PartialWindows) {
        self.leaf(
            &["windows", "split_ratio"],
            &mut base.split_ratio,
            &later.split_ratio,
        );
    }

    fn minibuffer(&mut self, base: &mut PartialMinibuffer, later: &PartialMinibuffer) {
        self.leaf(
            &["minibuffer", "candidate_limit"],
            &mut base.candidate_limit,
            &later.candidate_limit,
        );
        self.leaf(&["minibuffer", "match"], &mut base.r#match, &later.r#match);
        self.leaf(
            &["minibuffer", "history_limit"],
            &mut base.history_limit,
            &later.history_limit,
        );
        self.leaf(
            &["minibuffer", "input_limit"],
            &mut base.input_limit,
            &later.input_limit,
        );
    }

    fn history(&mut self, base: &mut PartialHistory, later: &PartialHistory) {
        self.leaf(
            &["history", "group_limit"],
            &mut base.group_limit,
            &later.group_limit,
        );
        self.leaf(
            &["history", "payload_target_bytes"],
            &mut base.payload_target_bytes,
            &later.payload_target_bytes,
        );
    }

    fn kill_ring(&mut self, base: &mut PartialKillRing, later: &PartialKillRing) {
        self.leaf(
            &["kill_ring", "capacity"],
            &mut base.capacity,
            &later.capacity,
        );
        self.leaf(
            &["kill_ring", "coalesce"],
            &mut base.coalesce,
            &later.coalesce,
        );
    }

    fn tree(&mut self, base: &mut PartialTree, later: &PartialTree) {
        self.leaf(&["tree", "visible"], &mut base.visible, &later.visible);
        self.leaf(&["tree", "width"], &mut base.width, &later.width);
        self.leaf(
            &["tree", "max_fraction"],
            &mut base.max_fraction,
            &later.max_fraction,
        );
        self.leaf(&["tree", "root"], &mut base.root, &later.root);
        self.leaf(
            &["tree", "show_hidden"],
            &mut base.show_hidden,
            &later.show_hidden,
        );
        self.leaf(
            &["tree", "directories_first"],
            &mut base.directories_first,
            &later.directories_first,
        );
    }

    fn syntax(&mut self, base: &mut PartialSyntax, later: &PartialSyntax) {
        self.leaf(&["syntax", "enabled"], &mut base.enabled, &later.enabled);
        self.leaf(
            &["syntax", "max_bytes"],
            &mut base.max_bytes,
            &later.max_bytes,
        );
    }

    fn files(&mut self, base: &mut PartialFiles, later: &PartialFiles) {
        self.leaf(&["files", "newline"], &mut base.newline, &later.newline);
        self.leaf(&["files", "bom"], &mut base.bom, &later.bom);
        self.table(
            &["files", "associations"],
            &mut base.associations,
            &later.associations,
        );
    }

    fn palette(&mut self, base: &mut PartialPalette, later: &PartialPalette) {
        self.leaf(
            &["palette", "background"],
            &mut base.background,
            &later.background,
        );
        self.leaf(
            &["palette", "foreground"],
            &mut base.foreground,
            &later.foreground,
        );
        self.leaf(&["palette", "cursor"], &mut base.cursor, &later.cursor);
        self.leaf(
            &["palette", "selection"],
            &mut base.selection,
            &later.selection,
        );
        self.leaf(
            &["palette", "selection_text"],
            &mut base.selection_text,
            &later.selection_text,
        );
        self.leaf(&["palette", "search"], &mut base.search, &later.search);
        self.leaf(
            &["palette", "search_text"],
            &mut base.search_text,
            &later.search_text,
        );
        self.leaf(&["palette", "status"], &mut base.status, &later.status);
        self.leaf(
            &["palette", "status_text"],
            &mut base.status_text,
            &later.status_text,
        );
        self.leaf(
            &["palette", "status_inactive"],
            &mut base.status_inactive,
            &later.status_inactive,
        );
        self.leaf(&["palette", "border"], &mut base.border, &later.border);
        self.leaf(
            &["palette", "border_active"],
            &mut base.border_active,
            &later.border_active,
        );
        self.leaf(&["palette", "tree"], &mut base.tree, &later.tree);
        self.leaf(
            &["palette", "tree_selected"],
            &mut base.tree_selected,
            &later.tree_selected,
        );
        self.leaf(
            &["palette", "tree_selected_text"],
            &mut base.tree_selected_text,
            &later.tree_selected_text,
        );
        self.leaf(&["palette", "message"], &mut base.message, &later.message);
        self.leaf(&["palette", "warning"], &mut base.warning, &later.warning);
        self.leaf(&["palette", "error"], &mut base.error, &later.error);
        self.org_palette(&mut base.org, &later.org);
        self.syntax_palette(&mut base.syntax, &later.syntax);
        self.gutter_palette(&mut base.gutter, &later.gutter);
    }

    fn syntax_palette(&mut self, base: &mut PartialSyntaxPalette, later: &PartialSyntaxPalette) {
        self.leaf(
            &["palette", "syntax", "keyword"],
            &mut base.keyword,
            &later.keyword,
        );
        self.leaf(
            &["palette", "syntax", "string"],
            &mut base.string,
            &later.string,
        );
        self.leaf(
            &["palette", "syntax", "comment"],
            &mut base.comment,
            &later.comment,
        );
        self.leaf(
            &["palette", "syntax", "type"],
            &mut base.r#type,
            &later.r#type,
        );
        self.leaf(
            &["palette", "syntax", "function"],
            &mut base.function,
            &later.function,
        );
        self.leaf(
            &["palette", "syntax", "number"],
            &mut base.number,
            &later.number,
        );
        self.leaf(
            &["palette", "syntax", "punctuation"],
            &mut base.punctuation,
            &later.punctuation,
        );
    }

    fn gutter(&mut self, base: &mut PartialGutter, later: &PartialGutter) {
        self.leaf(&["gutter", "visible"], &mut base.visible, &later.visible);
        self.leaf(&["gutter", "mode"], &mut base.mode, &later.mode);
        self.leaf(
            &["gutter", "current_line"],
            &mut base.current_line,
            &later.current_line,
        );
        self.leaf(&["gutter", "side"], &mut base.side, &later.side);
        self.leaf(
            &["gutter", "minimum_digits"],
            &mut base.minimum_digits,
            &later.minimum_digits,
        );
        self.leaf(
            &["gutter", "left_padding"],
            &mut base.left_padding,
            &later.left_padding,
        );
        self.leaf(
            &["gutter", "right_padding"],
            &mut base.right_padding,
            &later.right_padding,
        );
    }

    fn gutter_palette(&mut self, base: &mut PartialGutterPalette, later: &PartialGutterPalette) {
        self.leaf(
            &["palette", "gutter", "background"],
            &mut base.background,
            &later.background,
        );
        self.leaf(
            &["palette", "gutter", "foreground"],
            &mut base.foreground,
            &later.foreground,
        );
        self.leaf(
            &["palette", "gutter", "current_line"],
            &mut base.current_line,
            &later.current_line,
        );
        self.leaf(
            &["palette", "gutter", "separator"],
            &mut base.separator,
            &later.separator,
        );
    }

    fn languages(
        &mut self,
        base: &mut BTreeMap<String, Language>,
        later: &BTreeMap<String, Language>,
    ) {
        for (id, language) in later {
            self.language(id, base.entry(id.clone()).or_default(), language);
        }
    }

    fn language(&mut self, id: &str, base: &mut Language, later: &Language) {
        self.leaf(
            &["language", id, "indent_style"],
            &mut base.indent_style,
            &later.indent_style,
        );
        self.leaf(
            &["language", id, "indent_width"],
            &mut base.indent_width,
            &later.indent_width,
        );
        self.leaf(
            &["language", id, "tab_width"],
            &mut base.tab_width,
            &later.tab_width,
        );
        self.leaf(
            &["language", id, "grammar"],
            &mut base.grammar,
            &later.grammar,
        );
    }

    fn keys(&mut self, base: &mut Keymaps, later: &Keymaps) {
        for context in crate::input::keymap::Context::ALL {
            self.binding_table(
                &["keys", context.name()],
                base.map_mut(context),
                later.map(context),
            );
        }
    }
}

impl Merge {
    fn binding_table(
        &mut self,
        prefix: &[&str],
        base: &mut BTreeMap<String, String>,
        later: &BTreeMap<String, String>,
    ) {
        // A later exact normalized chord replaces its earlier spelling as well
        // as its value. Same-layer aliases are rejected before layer merging.
        use crate::input::keymap::Sequence;
        let replaced: std::collections::BTreeSet<_> = later
            .keys()
            .filter_map(|key| Sequence::parse(key).ok())
            .collect();
        base.retain(|key, _| {
            !Sequence::parse(key)
                .ok()
                .is_some_and(|key| replaced.contains(&key))
        });
        self.table(prefix, base, later);
    }
}

impl Merge {
    fn org_palette(&mut self, base: &mut PartialOrgPalette, later: &PartialOrgPalette) {
        self.leaf(
            &["palette", "org", "heading"],
            &mut base.heading,
            &later.heading,
        );
        self.leaf(
            &["palette", "org", "keyword"],
            &mut base.keyword,
            &later.keyword,
        );
        self.leaf(
            &["palette", "org", "drawer"],
            &mut base.drawer,
            &later.drawer,
        );
        self.leaf(
            &["palette", "org", "property"],
            &mut base.property,
            &later.property,
        );
        self.leaf(
            &["palette", "org", "planning"],
            &mut base.planning,
            &later.planning,
        );
        self.leaf(&["palette", "org", "block"], &mut base.block, &later.block);
        self.leaf(&["palette", "org", "code"], &mut base.code, &later.code);
        self.leaf(
            &["palette", "org", "comment"],
            &mut base.comment,
            &later.comment,
        );
        self.leaf(&["palette", "org", "list"], &mut base.list, &later.list);
        self.leaf(&["palette", "org", "table"], &mut base.table, &later.table);
        self.leaf(
            &["palette", "org", "formula"],
            &mut base.formula,
            &later.formula,
        );
        self.leaf(
            &["palette", "org", "emphasis"],
            &mut base.emphasis,
            &later.emphasis,
        );
        self.leaf(&["palette", "org", "link"], &mut base.link, &later.link);
        self.leaf(
            &["palette", "org", "target"],
            &mut base.target,
            &later.target,
        );
        self.leaf(
            &["palette", "org", "footnote"],
            &mut base.footnote,
            &later.footnote,
        );
        self.leaf(
            &["palette", "org", "entity"],
            &mut base.entity,
            &later.entity,
        );
        self.leaf(
            &["palette", "org", "malformed"],
            &mut base.malformed,
            &later.malformed,
        );
    }
}

impl Merge {
    fn time(&mut self, base: &mut super::PartialTime, later: &super::PartialTime) {
        self.leaf(&["org", "time", "zone"], &mut base.zone, &later.zone);
        self.leaf(
            &["org", "time", "ambiguous"],
            &mut base.ambiguous,
            &later.ambiguous,
        );
        self.leaf(
            &["org", "time", "nonexistent"],
            &mut base.nonexistent,
            &later.nonexistent,
        );
    }
}
