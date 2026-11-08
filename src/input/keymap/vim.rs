use std::collections::BTreeMap;
use std::sync::LazyLock;

use crate::command::{Binding, Command};
use crate::config::{InteractionProfile, Keymaps};

use super::{Context, Effective, Error, Map, Sequence, Site};

const STATES: [Context; 6] = [
    Context::VimNormal,
    Context::VimInsert,
    Context::VimReplace,
    Context::VimVisualCharacter,
    Context::VimVisualLine,
    Context::VimVisualBlock,
];

pub(super) fn normal() -> &'static [(&'static str, &'static str)] {
    static NORMAL: LazyLock<Vec<(&str, &str)>> = LazyLock::new(|| {
        let mut bindings: Vec<_> = super::defaults::EDITOR
            .iter()
            .copied()
            .filter(|(key, _)| {
                key.starts_with("M-")
                    || key.starts_with("C-x ")
                    || key.starts_with("C-c ")
                    || matches!(*key, "C-g" | "C-l" | "C-_" | "F3" | "F4")
            })
            .collect();
        bindings.extend([
            ("i", "modal-enter-insert"),
            ("a", "modal-enter-insert-after"),
            ("I", "modal-enter-insert-line-first"),
            ("A", "modal-enter-insert-line-end"),
            ("o", "modal-open-line-below"),
            ("O", "modal-open-line-above"),
        ]);
        bindings.extend([
            ("v", "modal-enter-visual-character"),
            ("V", "modal-enter-visual-line"),
            ("C-v", "modal-enter-visual-block"),
            ("R", "modal-enter-replace"),
            (".", "modal-repeat-last-change"),
            ("C-o", "modal-jump-backward"),
            ("C-i", "modal-jump-forward"),
        ]);
        bindings.extend(PANES);
        bindings.extend(super::vim_navigation::BINDINGS);
        bindings.extend(super::vim_editing::BINDINGS);
        bindings
    });
    &NORMAL
}

pub(super) fn insert() -> &'static [(&'static str, &'static str)] {
    static INSERT: LazyLock<Vec<(&str, &str)>> = LazyLock::new(|| {
        let mut bindings = super::defaults::EDITOR.to_vec();
        bindings.retain(|(key, _)| *key != "C-o");
        bindings.push(("C-o", "modal-one-normal-command"));
        bindings
    });
    &INSERT
}

pub(super) fn replace() -> &'static [(&'static str, &'static str)] {
    super::defaults::EDITOR
}

pub(super) fn visual(block: bool) -> &'static [(&'static str, &'static str)] {
    static LINEAR: LazyLock<Vec<(&str, &str)>> = LazyLock::new(|| visual_bindings(false));
    static BLOCK: LazyLock<Vec<(&str, &str)>> = LazyLock::new(|| visual_bindings(true));
    if block { &BLOCK } else { &LINEAR }
}

fn visual_bindings(block: bool) -> Vec<(&'static str, &'static str)> {
    let mut bindings: Vec<_> = normal()
        .iter()
        .copied()
        .filter(|(key, _)| {
            key.starts_with("M-")
                || key.starts_with("C-x ")
                || key.starts_with("C-c ")
                || matches!(*key, "C-g" | "C-l" | "C-_" | "F3" | "F4")
        })
        .collect();
    bindings.extend(PANES);
    bindings.extend(super::vim_navigation::BINDINGS);
    bindings.extend([
        ("v", "modal-enter-visual-character"),
        ("V", "modal-enter-visual-line"),
        ("C-v", "modal-enter-visual-block"),
        ("o", "exchange-point-and-mark"),
        ("d", "modal-delete"),
        ("c", "modal-change"),
        ("y", "modal-copy"),
    ]);
    if !block {
        bindings.extend([
            (">", "modal-indent"),
            ("<", "modal-outdent"),
            ("i", "modal-select-inner-object"),
            ("a", "modal-select-around-object"),
        ]);
    }
    bindings
}

pub(super) fn maps(
    keys: &Keymaps,
    commands: Option<&BTreeMap<String, crate::config::AiCommand>>,
) -> Result<BTreeMap<Context, Map>, Error> {
    STATES
        .into_iter()
        .map(|state| {
            let map = Map::compile(state, keys.map(state), commands)?;
            recovery(&map, state)?;
            Ok((state, map))
        })
        .collect()
}

pub(super) fn org_maps(
    keys: &Keymaps,
    maps: &BTreeMap<Context, Map>,
    profile: InteractionProfile,
    commands: Option<&BTreeMap<String, crate::config::AiCommand>>,
) -> Result<BTreeMap<(Context, Context), Map>, Error> {
    let mut result = BTreeMap::new();
    if profile != InteractionProfile::Modal {
        return Ok(result);
    }
    for state in [
        Context::VimNormal,
        Context::VimInsert,
        Context::VimReplace,
        Context::VimVisualCharacter,
        Context::VimVisualLine,
        Context::VimVisualBlock,
    ] {
        for local in [Context::Org, Context::OrgCapture, Context::OrgSourceEdit] {
            let map = Map::overlay(&maps[&state], local, keys.map(local), commands)?;
            recovery(&map, local)?;
            result.insert((state, local), map);
        }
    }
    Ok(result)
}

fn recovery(map: &Map, context: Context) -> Result<(), Error> {
    // Empty maps reserve future states without advertising unsupported bindings.
    if !matches!(
        context,
        Context::VimNormal
            | Context::VimInsert
            | Context::VimReplace
            | Context::VimVisualCharacter
            | Context::VimVisualLine
            | Context::VimVisualBlock
            | Context::Org
            | Context::OrgCapture
            | Context::OrgSourceEdit
    ) {
        return Ok(());
    }
    let sequence = Sequence::parse("M-x").expect("canonical recovery chord");
    if map.lookup(&sequence) == Some(Binding::Command(Command::ExecuteExtended)) {
        return Ok(());
    }
    Err(Error::new(
        context,
        "M-x",
        Site::Value,
        "M-x is reserved for execute-extended-command in modal editing",
    ))
}

impl Effective {
    pub(crate) fn vim_map(&self, state: Context, local: Option<Context>) -> &Map {
        local
            .and_then(|local| self.vim_org.get(&(state, local)))
            .unwrap_or_else(|| self.map(state))
    }
}

const PANES: &[(&str, &str)] = &[
    (":", "execute-extended-command"),
    ("C-w h", "windmove-left"),
    ("C-w j", "windmove-down"),
    ("C-w k", "windmove-up"),
    ("C-w l", "windmove-right"),
    ("C-w s", "split-window-below"),
    ("C-w v", "split-window-right"),
    ("C-w c", "delete-window"),
    ("C-w o", "delete-other-windows"),
    ("C-w =", "balance-windows"),
];
