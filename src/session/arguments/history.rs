use std::time::Instant;

use crate::buffer::edit::{Error, Point};
use crate::command::{Command, History};
use crate::input::Invocation;
use crate::session::{Action, Session};

/// History is deliberately progressive: each existing native group is checked
/// and published independently. Cancellation retains already replayed groups.
pub(in crate::session) struct Replay {
    invocation: Invocation,
    expected: Point,
    remaining: usize,
    stopped: Option<Error>,
}

impl Session {
    pub(super) fn execute_argument_history(
        &mut self,
        invocation: Invocation,
        direction: History,
    ) -> Action {
        if invocation.argument.value() < 0 {
            return self.error("History requires a nonnegative argument");
        }
        if invocation.argument.value() == 0 {
            return self.message("");
        }
        self.command_boundary(Command::History(direction));
        self.finish_repeat_entry();
        if let Err(error) = self.stop_typing() {
            return self.error(&error.to_string());
        }
        self.layout.history = Some(Replay {
            invocation,
            expected: self.view().point,
            remaining: usize::try_from(invocation.argument.value())
                .expect("positive history argument"),
            stopped: None,
        });
        if let Some(action) = self.layout_argument_history() {
            return action;
        }
        self.publish_argument_history()
    }

    pub(in crate::session) fn prepare_argument_history(
        &mut self,
    ) -> Result<(), crate::view::Error> {
        let mut work = self
            .layout
            .history
            .take()
            .expect("history request owns progress");
        let origin = crate::input::InvocationOrigin {
            source: Some((self.windows.selected_id(), work.expected)),
            ..work.invocation.origin
        };
        if work.expected != self.view().point
            || !self.source_layout_focus()
            || !self.input.invocation_current(origin)
        {
            return Err(Error::ChangedPositions.into());
        }
        let Command::History(direction) = work.invocation.command else {
            unreachable!()
        };
        let started = Instant::now();
        for _ in 0..256 {
            if work.remaining == 0 {
                break;
            }
            match self.replay(direction) {
                Ok(()) => {
                    work.remaining -= 1;
                    work.expected = self.view().point;
                    self.sync_input_context();
                }
                Err(error) => {
                    work.stopped = Some(error);
                    work.remaining = 0;
                }
            }
            if started.elapsed() >= crate::session::events::INPUT_BATCH_TIME {
                break;
            }
        }
        let complete = work.remaining == 0;
        self.layout.history = Some(work);
        if complete {
            Ok(())
        } else {
            Err(crate::coordinates::Error::Pending.into())
        }
    }

    pub(in crate::session) fn publish_argument_history(&mut self) -> Action {
        let Some(work) = self.layout.history.take() else {
            return self.message("History argument retired");
        };
        match work.stopped {
            Some(error) => self.error(&error.to_string()),
            None => self.message(""),
        }
    }
}
