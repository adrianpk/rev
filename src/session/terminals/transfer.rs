use super::{Action, Session};
use crate::{
    command::TerminalCommand,
    term::{Paste, SessionId},
};

pub(super) struct Yank {
    id: SessionId,
    paste: Paste,
    generation: Option<u64>,
    rotation: usize,
}

impl Session {
    pub(super) fn terminal_transfer_command(&mut self, command: TerminalCommand) -> Option<Action> {
        Some(match command {
            TerminalCommand::Yank => self.terminal_yank(false),
            TerminalCommand::YankPop => self.terminal_yank(true),
            TerminalCommand::ClearScrollback => self.terminal_clear_scrollback(),
            TerminalCommand::BeginningOfLine => self.terminal_copy_line(false),
            TerminalCommand::EndOfLine => self.terminal_copy_line(true),
            TerminalCommand::ResetCursorPoint => self.terminal_copy_cursor(),
            _ => return self.terminal_control_command(command),
        })
    }
    pub(super) fn terminal_yank(&mut self, rotate: bool) -> Action {
        if self.terminal_copy_active() {
            return self.error("Exit terminal copy mode before sending input");
        }
        match self.paste_retained(rotate) {
            Ok(()) => {
                self.resume_terminal_input_tail();
                self.message("Terminal retained text pasted")
            }
            Err(error) => self.error(&error),
        }
    }
    fn paste_retained(&mut self, rotate: bool) -> Result<(), String> {
        let id = self.terminal_input_id()?;
        let (rotation, previous) = self.terminal_yank_rotation(id, rotate)?;
        let text = self
            .kills
            .iter()
            .nth(rotation)
            .ok_or("Kill ring is empty")?;
        // Bound the owned candidate before flattening a potentially large rope.
        let limit = self
            .buffers
            .snapshot()
            .settings()
            .terminal
            .max_pending_input_bytes;
        if text.len_bytes() > limit {
            return Err("Terminal retained text exceeds the input queue limit".into());
        }
        let text = text.to_string();
        let paste = self
            .terminals
            .sessions
            .paste(id, &text, previous)
            .map_err(|error| error.to_string())?;
        self.terminals.yank = Some(Yank {
            id,
            paste,
            rotation,
            generation: self.kills.generation(),
        });
        Ok(())
    }
    fn terminal_yank_rotation(
        &self,
        id: SessionId,
        rotate: bool,
    ) -> Result<(usize, Option<Paste>), String> {
        if !rotate {
            return Ok((0, None));
        }
        let yank = self
            .terminals
            .yank
            .as_ref()
            .ok_or("No current terminal yank to replace")?;
        if yank.id != id || yank.generation.is_none() || yank.generation != self.kills.generation()
        {
            return Err("Terminal yank-pop is no longer current".into());
        }
        let rotation = self
            .kills
            .older(yank.rotation)
            .ok_or("Kill ring is empty")?
            .0;
        Ok((rotation, Some(yank.paste)))
    }
    pub(super) fn terminal_paste_text(&mut self, text: &str) -> Action {
        let result = self.terminal_input_id().and_then(|id| {
            self.terminals
                .sessions
                .paste(id, text, None)
                .map_err(|error| error.to_string())
        });
        match result {
            Ok(_) => {
                self.terminals.yank = None;
                self.resume_terminal_input_tail();
                Action::Redraw
            }
            Err(error) => self.error(&error),
        }
    }
}
