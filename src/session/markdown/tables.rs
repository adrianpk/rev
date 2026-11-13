use crate::buffer::edit::Point;
use crate::command::MarkdownCommand as C;
use crate::input::Focus;
use crate::window::ViewId;

use super::{Action, POLL, Session};
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub(in crate::session) struct Origin {
    view: ViewId,
    point: Point,
    generation: u64,
}

#[derive(Default)]
pub(super) struct State {
    job: crate::org::Job<Result<crate::markdown::Edit, String>>,
    origin: Option<Origin>,
    deadline: Option<Instant>,
}

impl State {
    pub(super) fn remaining(&self, now: Instant) -> Option<Duration> {
        self.deadline.map(|at| at.saturating_duration_since(now))
    }

    pub(super) fn cancel(&self) {
        self.job.cancel();
    }
}

impl Origin {
    fn current(self, session: &Session) -> bool {
        session.windows.selected_id() == self.view
            && session.view().point == self.point
            && session.buffer().is_ok_and(|buffer| {
                buffer.is_markdown()
                    && buffer.stamp() == self.point.stamp
                    && buffer.markdown_generation() == self.generation
                    && buffer.markdown_observation().is_some()
                    && !buffer.is_readonly()
            })
    }
}

impl Session {
    pub(in crate::session) fn markdown_table_active(&self) -> bool {
        self.buffer().is_ok_and(|buffer| {
            buffer.is_markdown()
                && buffer
                    .markdown_observation()
                    .is_some_and(|model| model.table_at(buffer, self.view().point))
        })
    }

    pub(super) fn markdown_table(&mut self, command: C) -> Option<Action> {
        if command == C::TableCreate {
            return Some(match self.markdown_table_prompt() {
                Ok(()) => self.message("Enter columns x body rows; C-g cancels"),
                Err(error) => self.error(&error),
            });
        }
        if !matches!(
            command,
            C::TableAlign
                | C::TableNextField
                | C::TablePreviousField
                | C::TableNextRow
                | C::TableInsertRow
                | C::TableDeleteRow
                | C::TableInsertColumn
                | C::TableDeleteColumn
                | C::Return
                | C::MetaReturn
        ) {
            return None;
        }
        match self.markdown_table_work() {
            Ok(Some(work)) => Some(self.start_markdown_table(work, command)),
            Ok(None) if matches!(command, C::Return | C::MetaReturn) => None,
            Ok(None) => Some(self.error("Point is not in an admitted Markdown table")),
            Err(error) => Some(self.error(&error)),
        }
    }

    fn markdown_table_origin(&self) -> Result<Origin, String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        self.view().cell(buffer).map_err(|e| e.to_string())?;
        if buffer.is_readonly() {
            return Err(crate::buffer::edit::Error::ReadOnly.to_string());
        }
        Ok(Origin {
            view: self.windows.selected_id(),
            point: self.view().point,
            generation: buffer.markdown_generation(),
        })
    }

    fn markdown_table_work(&self) -> Result<Option<crate::markdown::tables::Work>, String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        let Some(model) = buffer.markdown_observation() else {
            return Ok(None);
        };
        model.table_work(buffer, self.view().point)
    }

    fn markdown_table_prompt(&mut self) -> Result<(), String> {
        let origin = self.markdown_table_origin()?;
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        buffer
            .markdown_observation()
            .ok_or("Markdown structure is not current")?
            .blank_table_line(buffer, origin.point)?;
        self.input
            .argument("Markdown size (columns x body rows): ", "")
            .map_err(|e| e.to_string())?;
        self.pending = Some(super::super::Pending::MarkdownTable(origin));
        Ok(())
    }

    pub(in crate::session) fn markdown_table_argument(
        &mut self,
        origin: Origin,
        draft: &str,
    ) -> Action {
        if !origin.current(self) {
            return self.error("Markdown table source or view changed; source unchanged");
        }
        match self.create_markdown_table_work(origin, draft) {
            Ok(work) => self.start_markdown_table(work, C::TableCreate),
            Err(error) => self.error(&error),
        }
    }

    fn create_markdown_table_work(
        &self,
        origin: Origin,
        draft: &str,
    ) -> Result<crate::markdown::tables::Work, String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        buffer
            .markdown_observation()
            .ok_or("Markdown structure is not current")?
            .create_table(buffer, origin.point, draft)
    }

    fn start_markdown_table(&mut self, work: crate::markdown::tables::Work, command: C) -> Action {
        let origin = match self.markdown_table_origin() {
            Ok(origin) => origin,
            Err(error) => return self.error(&error),
        };
        match self
            .markdown
            .table
            .job
            .start(origin.point.stamp, move |cancel| work.run(command, cancel))
        {
            Ok(()) => {
                self.markdown.table.origin = Some(origin);
                self.markdown.table.deadline = Some(Instant::now() + POLL);
                self.message("Checking Markdown table; C-g cancels")
            }
            Err(error) => self.error(&error.to_string()),
        }
    }

    pub(super) fn poll_markdown_table(&mut self) -> bool {
        let Some(origin) = self.markdown.table.origin else {
            return false;
        };
        let live =
            origin.current(self) && self.input.focus() == Focus::Editing && self.pending.is_none();
        if !live {
            self.markdown.table.cancel();
        }
        let Some(result) = self.markdown.table.job.poll_owned() else {
            self.markdown.table.deadline = Some(Instant::now() + POLL);
            return false;
        };
        self.markdown.table.origin = None;
        self.markdown.table.deadline = None;
        if !live {
            self.error("Markdown table source or view changed; source unchanged");
            return true;
        }
        match result
            .map_err(|e| e.to_string())
            .and_then(|(_, plan)| plan)
            .and_then(|plan| self.publish_markdown(&plan))
        {
            Ok(()) => {
                self.message("");
            }
            Err(error) => {
                self.error(&error);
            }
        }
        true
    }
}
