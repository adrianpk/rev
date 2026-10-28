use std::collections::BTreeMap;
use std::sync::LazyLock;

use super::{Context, Sequence};

pub(super) fn org_complete() -> &'static [(&'static str, &'static str)] {
    static COMPLETE: LazyLock<Vec<(&str, &str)>> = LazyLock::new(|| {
        EDITOR
            .iter()
            .chain(ORG)
            .map(|entry| {
                let key = Sequence::parse(entry.0).expect("audited built-in sequence");
                (key, *entry)
            })
            .collect::<BTreeMap<_, _>>()
            .into_values()
            .collect()
    });
    &COMPLETE
}

pub(super) fn bindings(context: Context) -> &'static [(&'static str, &'static str)] {
    match context {
        Context::Editor => EDITOR,
        Context::Minibuffer => MINIBUFFER,
        Context::Tree => TREE,
        Context::Org => ORG,
        Context::OrgAgenda => ORG_AGENDA,
        Context::OrgCapture => ORG_CAPTURE,
        Context::OrgCalendar => ORG_CALENDAR,
        Context::OrgSourceEdit => ORG_SOURCE_EDIT,
    }
}

const EDITOR: &[(&str, &str)] = &[
    ("C-g", "keyboard-quit"),
    ("M-x", "execute-extended-command"),
    ("M-g g", "goto-line"),
    ("M-g M-g", "goto-line"),
    ("RET", "newline"),
    ("TAB", "insert-tab"),
    ("Backspace", "backward-delete-char"),
    ("Delete", "delete-char"),
    ("C-d", "delete-char"),
    ("Left", "backward-char"),
    ("C-b", "backward-char"),
    ("Right", "forward-char"),
    ("C-f", "forward-char"),
    ("Home", "move-beginning-of-line"),
    ("C-a", "move-beginning-of-line"),
    ("End", "move-end-of-line"),
    ("C-e", "move-end-of-line"),
    ("Up", "previous-line"),
    ("C-p", "previous-line"),
    ("Down", "next-line"),
    ("C-n", "next-line"),
    ("PageUp", "scroll-down-command"),
    ("M-v", "scroll-down-command"),
    ("PageDown", "scroll-up-command"),
    ("C-v", "scroll-up-command"),
    ("C-l", "recenter-top-bottom"),
    ("C-_", "undo"),
    ("C-s", "isearch-forward"),
    ("C-r", "isearch-backward"),
    ("C-SPC", "set-mark-command"),
    ("C-k", "kill-line"),
    ("C-w", "kill-region"),
    ("C-y", "yank"),
    ("M-%", "query-replace"),
    ("M-d", "kill-word"),
    ("M-DEL", "backward-kill-word"),
    ("M-w", "kill-ring-save"),
    ("M-y", "yank-pop"),
    ("M-f", "forward-word"),
    ("M-b", "backward-word"),
    ("M-<", "beginning-of-buffer"),
    ("M->", "end-of-buffer"),
    ("C-x C-x", "exchange-point-and-mark"),
    ("C-x C-s", "save-buffer"),
    ("C-x C-w", "write-file"),
    ("C-x C-f", "find-file"),
    ("C-x C-c", "save-buffers-kill-terminal"),
    ("C-x C-b", "list-buffers"),
    ("C-x SPC", "rectangle-mark-mode"),
    ("C-x b", "switch-to-buffer"),
    ("C-x k", "kill-buffer"),
    ("C-x u", "undo"),
    ("C-x 2", "split-window-below"),
    ("C-x 3", "split-window-right"),
    ("C-x 0", "delete-window"),
    ("C-x 1", "delete-other-windows"),
    ("C-x o", "other-window"),
    ("C-x r M-w", "copy-rectangle-as-kill"),
    ("C-x r k", "kill-rectangle"),
    ("C-x r y", "yank-rectangle"),
    ("C-x t t", "toggle-file-tree"),
    ("C-x t f", "focus-file-tree"),
    ("C-x t r", "tree-refresh"),
    ("C-x t D", "tree-delete"),
    ("C-x t R", "tree-rename"),
    ("C-x t o", "tree-open"),
    ("C-x t n", "tree-create-file"),
    ("C-x t d", "tree-create-directory"),
];

const MINIBUFFER: &[(&str, &str)] = &[
    ("C-g", "keyboard-quit"),
    ("RET", "minibuffer-accept"),
    ("TAB", "minibuffer-complete"),
    ("Backspace", "backward-delete-char"),
    ("Delete", "delete-char"),
    ("C-d", "delete-char"),
    ("Left", "backward-char"),
    ("C-b", "backward-char"),
    ("Right", "forward-char"),
    ("C-f", "forward-char"),
    ("Home", "move-beginning-of-line"),
    ("C-a", "move-beginning-of-line"),
    ("End", "move-end-of-line"),
    ("C-e", "move-end-of-line"),
    ("Up", "minibuffer-previous"),
    ("C-p", "minibuffer-previous"),
    ("Down", "minibuffer-next"),
    ("C-n", "minibuffer-next"),
    ("M-p", "minibuffer-history-previous"),
    ("M-n", "minibuffer-history-next"),
    ("C-s", "isearch-forward"),
    ("C-r", "isearch-backward"),
];

const TREE: &[(&str, &str)] = &[
    ("C-g", "keyboard-quit"),
    ("TAB", "insert-tab"),
    ("M-x", "execute-extended-command"),
    ("Up", "tree-previous-line"),
    ("C-p", "tree-previous-line"),
    ("Down", "tree-next-line"),
    ("C-n", "tree-next-line"),
    ("Right", "tree-expand"),
    ("C-f", "tree-expand"),
    ("Left", "tree-collapse"),
    ("C-b", "tree-collapse"),
    ("RET", "tree-open"),
    ("C-x C-c", "save-buffers-kill-terminal"),
    ("C-x t t", "toggle-file-tree"),
    ("C-x t f", "focus-file-tree"),
    ("C-x t r", "tree-refresh"),
    ("C-x t D", "tree-delete"),
    ("C-x t R", "tree-rename"),
    ("C-x t o", "tree-open"),
    ("C-x t n", "tree-create-file"),
    ("C-x t d", "tree-create-directory"),
];

// Editing roles inherit the admitted editor map before these local entries.
const ORG: &[(&str, &str)] = &[
    ("TAB", "org-cycle"),        // The exact heading/drawer at point.
    ("BackTab", "org-shifttab"), // The whole view.
];
const ORG_CAPTURE: &[(&str, &str)] = &[];
const ORG_SOURCE_EDIT: &[(&str, &str)] = &[];

// Non-text workspace owners admit only implemented navigation/cancel targets.
const ORG_AGENDA: &[(&str, &str)] = &[
    ("C-g", "keyboard-quit"),
    ("M-x", "execute-extended-command"),
];
const ORG_CALENDAR: &[(&str, &str)] = &[
    ("C-g", "keyboard-quit"),
    ("M-x", "execute-extended-command"),
];
