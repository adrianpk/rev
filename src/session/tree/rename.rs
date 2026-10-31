use crate::file::Rename;
use crate::input::Focus;

use super::super::{Action, Pending, Session};

impl Session {
    pub(super) fn tree_rename(&mut self) -> Action {
        if self.input.focus() != Focus::Editing {
            return self.message("a prompt owns input focus");
        }
        if !self.tree.focused() {
            return self.message("File tree is not focused");
        }
        let rename = match self.tree.prepare_rename() {
            Ok(rename) => rename,
            Err(error) => return self.error(&error.to_string()),
        };
        if let Err(error) = self.input.argument("Rename to: ", "") {
            return self.error(&error.to_string());
        }
        self.echo_kind = crate::display::Message::Normal;
        self.echo = format!("Rename {}", rename.source().display());
        self.pending = Some(Pending::TreeRename(rename));
        Action::Redraw
    }

    pub(in crate::session) fn tree_renamed(&mut self, rename: Rename, name: &str) -> Action {
        let renamed = match self.buffers.rename(rename, name) {
            Ok(renamed) => renamed,
            Err(error) => return self.error(&error.to_string()),
        };
        let mut notice = format!(
            "Renamed {} to {}",
            renamed.source().display(),
            renamed.target().display()
        );
        let mut severity = crate::display::Message::Normal;
        if let Some(warning) = renamed.warning() {
            severity = crate::display::Message::Warning;
            notice.push_str(&format!("; {warning}"));
        }
        if let Err(error) = self.tree.renamed(&renamed) {
            severity = crate::display::Message::Warning;
            notice.push_str(&format!("; tree refresh failed: {error}"));
        }
        if self.buffers.snapshot().settings().bookmarks.save
            == crate::config::BookmarkSave::OnChange
            && self.bookmarks().is_dirty()
            && let Err(error) = self.save_bookmarks()
        {
            severity = crate::display::Message::Warning;
            notice.push_str(&format!("; {error}; bookmark changes remain unsaved"));
        }
        let action = self.message(&notice);
        self.echo_kind = severity;
        action
    }
}
