use super::*;
use crate::buffer::BufferId;
use crate::session::Pending;

pub(in crate::session) enum Departure {
    Quit,
    Context(crate::context::Generation),
    Roots(crate::context::Generation),
    Close(BufferId),
}
pub(in crate::session) enum Prompt {
    Departure(Departure),
    Recovery(Box<super::recovery::Recovery>),
    Stored(crate::org::io::ClockState),
}
impl Prompt {
    pub(in crate::session) fn root_departure(&self) -> Option<crate::context::Generation> {
        match self {
            Self::Departure(Departure::Roots(g)) => Some(*g),
            _ => None,
        }
    }
    pub(in crate::session) fn context_departure(&self) -> Option<crate::context::Generation> {
        match self {
            Self::Departure(Departure::Context(generation)) => Some(*generation),
            _ => None,
        }
    }
}
impl Session {
    pub(in crate::session) fn clock_departure(&mut self, departure: Departure) -> Action {
        match self.ask_clock(
            Prompt::Departure(departure),
            "Org active clock: ",
            "close = clock out, keep = detach and preserve open source; C-g cancels",
        ) {
            Ok(action) => action,
            Err(error) => self.error(&error),
        }
    }
    pub(super) fn ask_clock(
        &mut self,
        prompt: Prompt,
        label: &'static str,
        message: &str,
    ) -> Result<Action, String> {
        self.ask_clock_draft(prompt, label, message, "")
    }
    pub(super) fn ask_clock_draft(
        &mut self,
        prompt: Prompt,
        label: &'static str,
        message: &str,
        draft: &str,
    ) -> Result<Action, String> {
        self.input
            .argument(label, draft)
            .map_err(|e| e.to_string())?;
        self.pending = Some(Pending::Clock(Box::new(prompt)));
        Ok(self.message(message))
    }
    pub(in crate::session) fn clock_argument(&mut self, prompt: Prompt, draft: &str) -> Action {
        let result = match prompt {
            Prompt::Departure(departure) => self.depart_clock(departure, draft),
            Prompt::Recovery(recovery) => self.recover_clock(*recovery, draft),
            Prompt::Stored(record) => self.restore_stored_clock(record, draft),
        };
        match result {
            Ok(action) => action,
            Err(error) => self.error(&error),
        }
    }
    fn depart_clock(&mut self, departure: Departure, draft: &str) -> Result<Action, String> {
        match draft.trim() {
            "close" => self.finish_clock(OrgCommand::ClockOut)?,
            "keep" => {
                self.retained_clock = self.active_clock.take();
            }
            _ => {
                return self.ask_clock(
                    Prompt::Departure(departure),
                    "Org active clock: ",
                    "Enter close or keep; C-g cancels",
                );
            }
        }
        Ok(match departure {
            Departure::Quit => self.begin_quit(),
            Departure::Context(generation) => {
                self.begin_departure(crate::session::quit::Goal::Context(generation))
            }
            Departure::Roots(generation) => self.begin_workspace_sources(generation),
            Departure::Close(id) => self.kill_buffer(id),
        })
    }
}
