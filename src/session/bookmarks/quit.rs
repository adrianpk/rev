use super::*;

impl Session {
    /// Source save/discard decisions have all finished. Retain the live session
    /// on storage failure; discarding permits this exit only, as for sources.
    pub(in crate::session) fn finish_bookmark_quit(&mut self) -> Action {
        let automatic = self.buffers.snapshot().settings().bookmarks.save
            != crate::config::BookmarkSave::Manual;
        if automatic
            && self.bookmarks().is_dirty()
            && (self.bookmark_storage.path.is_some() || self.bookmark_storage.error.is_some())
            && let Err(error) = self.save_bookmarks()
        {
            return self.bookmark_quit_prompt(&error);
        }
        self.end_timer_session();
        Action::Exit
    }
    fn bookmark_quit_prompt(&mut self, notice: &str) -> Action {
        if let Err(error) = self.input.names_argument(
            "Quit bookmarks (retry/discard/cancel): ",
            "",
            vec!["retry".into(), "discard".into(), "cancel".into()],
        ) {
            return self.error(&error.to_string());
        }
        self.pending = Some(Pending::BookmarkQuit);
        self.error(notice)
    }
    pub(in crate::session) fn bookmark_quit_answer(&mut self, answer: &str) -> Action {
        match answer {
            "retry" => self.finish_bookmark_quit(),
            "discard" => {
                self.end_timer_session();
                Action::Exit
            }
            "cancel" => self.message("Quit canceled; unsaved bookmarks retained"),
            _ => self.bookmark_quit_prompt("Choose retry, discard or cancel; C-g cancels"),
        }
    }
}
