use std::collections::BTreeSet;
use std::io;
use std::path::PathBuf;

use crate::context::{Root, RootId, WorkspaceId};
use crate::view::Size;

use super::{Entry, Tree};

impl Tree {
    /// Ordered peer roots start collapsed. No root is substituted for an empty
    /// or unavailable membership; painting and hits use retained observations.
    pub(crate) fn opening_workspace(&self, id: WorkspaceId, roots: &[Root], size: Size) -> Self {
        let mut tree = Self {
            workspace: Some((id, roots.to_vec())),
            visible: true,
            explicit_root: true,
            initialized: true,
            policy: self.policy.clone(),
            nerd_fonts: self.nerd_fonts,
            entries: roots.iter().map(workspace_entry).collect(),
            ..Default::default()
        };
        tree.focused = tree.width(size) > 0;
        tree.focus_pending = !tree.focused;
        tree
    }
    /// Called after durable membership publication. Keep each retained route's
    /// loaded preorder and expansion; new members start as independent roots.
    pub(crate) fn workspace_roots(&mut self, roots: &[Root]) {
        let Some((owner, _)) = &self.workspace else {
            return;
        };
        let owner = owner.clone();
        self.follow_dirty = true;
        let selected = self.selected().map(|e| (e.root_id.clone(), e.path.clone()));
        let previous = std::mem::take(&mut self.entries);
        for root in roots {
            let retained = previous
                .iter()
                .filter(|e| e.root_id.as_ref() == Some(&root.id));
            if previous.iter().any(|e| {
                e.depth == 0
                    && e.root_id.as_ref() == Some(&root.id)
                    && e.path == root.path.as_path()
            }) {
                self.entries.extend(retained.cloned());
            } else {
                self.entries.push(workspace_entry(root));
            }
        }
        self.workspace = Some((owner, roots.to_vec()));
        self.selected = selected
            .and_then(|(id, path)| {
                self.entries
                    .iter()
                    .position(|e| e.root_id == id && e.path == path)
            })
            .unwrap_or(0);
        self.top = self.top.min(self.selected);
    }
    pub(super) fn refresh_workspace(&mut self) -> io::Result<()> {
        let Some((_, roots)) = &self.workspace else {
            return Ok(());
        };
        let selected = self.selected().map(|e| (e.root_id.clone(), e.path.clone()));
        let expanded: BTreeSet<_> = self
            .entries
            .iter()
            .filter(|e| e.expanded)
            .map(|e| (e.root_id.clone(), e.path.clone()))
            .collect();
        self.entries = roots.iter().map(workspace_entry).collect();
        self.selected = 0;
        self.top = 0;
        let mut first_error = None;
        let mut index = 0;
        while index < self.entries.len() {
            let entry = &self.entries[index];
            if expanded.contains(&(entry.root_id.clone(), entry.path.clone())) {
                match entry.children(&self.policy) {
                    Ok(children) => {
                        self.entries[index].expanded = true;
                        self.entries[index].incomplete = children.incomplete;
                        self.entries.splice(index + 1..index + 1, children.entries);
                    }
                    Err(error) => {
                        first_error.get_or_insert(error);
                    }
                }
            }
            index += 1;
        }
        self.selected = selected
            .and_then(|(id, path): (Option<RootId>, PathBuf)| {
                path.ancestors().find_map(|p| {
                    self.entries
                        .iter()
                        .position(|e| e.root_id == id && e.path == p)
                })
            })
            .unwrap_or(0);
        first_error.map_or(Ok(()), Err)
    }
}
fn workspace_entry(root: &Root) -> Entry {
    let mut entry = Entry::root(root.path.as_path().to_owned());
    entry.root_id = Some(root.id.clone());
    entry.root_label = Some(format!(
        "{} — {}",
        root.label.as_str(),
        root.path.as_path().display()
    ));
    entry.unavailable = !std::fs::symlink_metadata(root.path.as_path()).is_ok_and(|m| m.is_dir());
    entry
}
