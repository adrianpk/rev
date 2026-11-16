mod pulse;
mod rendering;

use crate::command::Command;
use crate::config::PairHighlight;
use crate::session::{Action, Pending, Session};
use crate::window::ViewId;

impl Session {
    pub(in crate::session) fn pair_highlight_command(&mut self, command: Command) -> Action {
        if !self.pair_presentation_focus() {
            return self.message("Pair presentation requires a source view");
        }
        if command == Command::PairHighlightMode {
            let defaults = self.buffer().expect("selected source buffer").pair_policy();
            self.windows.selected_mut().pairs.toggle(defaults);
            return self.pair_highlight_feedback();
        }
        if let Err(error) = self
            .input
            .argument("Pair highlight style (depth/match/pulse/off): ", "")
        {
            return self.error(&error.to_string());
        }
        self.pending = Some(Pending::PairHighlight(self.windows.selected_id()));
        self.message("")
    }

    pub(in crate::session) fn pair_highlight_argument(
        &mut self,
        view: ViewId,
        value: &str,
    ) -> Action {
        if self.windows.selected_id() != view || !self.pair_presentation_focus() {
            return self.error("Pair presentation view changed");
        }
        let Some(style) = PairHighlight::parse(value.trim()) else {
            return self.error("Choose depth, match, pulse or off");
        };
        let defaults = self.buffer().expect("selected source buffer").pair_policy();
        self.windows.selected_mut().pairs.choose(style, defaults);
        self.pair_highlight_feedback()
    }

    fn pair_presentation_focus(&self) -> bool {
        self.source_layout_focus()
            && !self.terminals.focused
            && !self.git_visible()
            && !self.capture_blocks_input()
            && !self.context_storage.locked()
            && self.pending.is_none()
            && !self.input.editing_composition()
    }

    fn pair_highlight_feedback(&mut self) -> Action {
        let defaults = self.buffer().expect("selected source buffer").pair_policy();
        let style = self.view().pairs.style(defaults);
        self.message(&format!("Pair highlight style: {}", style.name()))
    }
}
