//! Effective plugin maps share native dispatch and hint probing.
use super::{Input, OrgContext};
use crate::input::keymap::{
    Context, Map,
    contributions::{CONTEXTS, Contribution, overlaps},
};
use rev_plugin_sdk::methods::ResolutionStatus;
use std::collections::BTreeMap;
#[derive(Clone, Debug, Default)]
pub(crate) struct Keymaps {
    pub(super) values: Vec<Contribution>,
    maps: BTreeMap<Context, Map>,
    statuses: BTreeMap<Context, Vec<ResolutionStatus>>,
    macros: BTreeMap<crate::input::keymap::Sequence, crate::command::Command>,
    org: bool,
    markdown: bool,
}
impl Input {
    pub(crate) fn plugin_keymaps(&mut self, values: Vec<Contribution>) {
        let org = self.workspace_org == Some(OrgContext::Editing);
        if self.plugin_keys.values == values
            && self.plugin_keys.markdown == self.workspace_markdown
            && self.plugin_keys.org == org
            && self.plugin_keys.macros == self.macro_bindings
        {
            return;
        }
        let mut keys = Keymaps {
            values,
            macros: self.macro_bindings.clone(),
            org,
            markdown: self.workspace_markdown,
            ..Keymaps::default()
        };
        if !keys.values.is_empty() {
            for context in CONTEXTS {
                let base = self.plugin_binding_native(context, org);
                let values: Vec<_> = keys
                    .values
                    .iter()
                    .filter(|v| v.context == context)
                    .cloned()
                    .collect();
                let (map, statuses) = base.contributions(&values);
                keys.maps.insert(context, map);
                keys.statuses.insert(context, statuses);
            }
        }
        self.plugin_keys = keys;
        if self.focus() == super::Focus::Editing {
            self.clear_prefix();
        }
        self.retire_command_hints();
    }
    pub(crate) fn plugin_binding_status(&self, value: &Contribution) -> ResolutionStatus {
        let index = self
            .plugin_keys
            .values
            .iter()
            .filter(|v| v.context == value.context)
            .position(|v| v == value);
        index
            .and_then(|i| {
                self.plugin_keys
                    .statuses
                    .get(&value.context)
                    .and_then(|s| s.get(i))
            })
            .copied()
            .unwrap_or(ResolutionStatus::Shadowed)
    }
    pub(crate) fn plugin_binding_map(&self, context: Context) -> &Map {
        self.plugin_keys
            .maps
            .get(&context)
            .unwrap_or_else(|| self.keymaps.map(context))
    }
    pub(crate) fn plugin_binding_native(&self, context: Context, org: bool) -> Map {
        let base = if self.workspace_markdown && !org {
            self.markdown_binding_native(context)
        } else if org && context != Context::Editor {
            self.keymaps.vim_map(context, Some(Context::Org))
        } else if org {
            self.keymaps.map(Context::Org)
        } else {
            self.keymaps.map(context)
        };
        base.with_macros(&self.macro_bindings)
    }
    fn markdown_binding_native(&self, context: Context) -> &Map {
        if context == Context::Editor {
            self.keymaps.map(Context::Markdown)
        } else {
            self.keymaps.vim_map(context, Some(Context::Markdown))
        }
    }
    pub(crate) fn admit_plugin_binding(
        &self,
        context: Context,
        sequence: &crate::input::keymap::Sequence,
    ) -> Result<(), String> {
        self.keymaps.admit_plugin(context, sequence)?;
        self.plugin_binding_native(context, false)
            .admit_plugin(sequence)?;
        self.plugin_binding_native(context, true)
            .admit_plugin(sequence)
    }
    pub(super) fn plugin_active_map(&self) -> Option<&Map> {
        if !self.macro_bindings_active()
            || matches!(self.workspace_org, Some(OrgContext::SourceEdit))
        {
            return None;
        }
        let context = if self.modal_editing() {
            self.modal_context()
        } else {
            Context::Editor
        };
        self.plugin_keys.maps.get(&context)
    }
    pub(super) fn plugin_local_key(&self, sequence: &crate::input::keymap::Sequence) -> bool {
        if self.plugin_active_map().is_none() {
            return false;
        }
        let context = if self.modal_editing() {
            self.modal_context()
        } else {
            Context::Editor
        };
        self.plugin_keys.values.iter().any(|v| {
            v.context == context
                && v.local
                && overlaps(sequence, &v.sequence)
                && self.plugin_binding_status(v) == ResolutionStatus::Effective
        })
    }
}
