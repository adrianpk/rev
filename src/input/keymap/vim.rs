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
                    || matches!(*key, "C-g" | "C-l" | "C-_")
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
        bindings.extend(super::vim_navigation::BINDINGS);
        bindings
    });
    &NORMAL
}

pub(super) fn maps(keys: &Keymaps) -> Result<BTreeMap<Context, Map>, Error> {
    STATES
        .into_iter()
        .map(|state| {
            let map = Map::compile(state, keys.map(state))?;
            recovery(&map, state)?;
            Ok((state, map))
        })
        .collect()
}

pub(super) fn org_maps(
    keys: &Keymaps,
    maps: &BTreeMap<Context, Map>,
    profile: InteractionProfile,
) -> Result<BTreeMap<(Context, Context), Map>, Error> {
    let mut result = BTreeMap::new();
    if profile != InteractionProfile::Vim {
        return Ok(result);
    }
    for state in [Context::VimNormal, Context::VimInsert] {
        for local in [Context::Org, Context::OrgCapture, Context::OrgSourceEdit] {
            let map = Map::overlay(&maps[&state], local, keys.map(local))?;
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
        "M-x is reserved for execute-extended-command in Vim editing",
    ))
}

impl Effective {
    pub(crate) fn vim_map(&self, state: Context, local: Option<Context>) -> &Map {
        local
            .and_then(|local| self.vim_org.get(&(state, local)))
            .unwrap_or_else(|| self.map(state))
    }
}
