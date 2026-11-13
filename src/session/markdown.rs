pub(super) mod tables;

use std::time::{Duration, Instant};

use crate::buffer::{BufferId, Stamp};
use crate::markdown::Observation;

use super::{Action, Session};

const POLL: Duration = Duration::from_millis(10);

/// Reuse the existing temporary native job lifecycle. Markdown owns its slot,
/// cancellation, source generation and publication independently of Org work.
#[derive(Default)]
pub(super) struct Work {
    job: crate::org::Job<Option<Observation>>,
    table: tables::State,
    source: Option<(Stamp, u64)>,
    deadline: Option<Instant>,
}

impl Work {
    pub(super) fn remaining(&self, now: Instant) -> Option<Duration> {
        [
            self.deadline
                .map(|value| value.saturating_duration_since(now)),
            self.table.remaining(now),
        ]
        .into_iter()
        .flatten()
        .min()
    }

    pub(super) fn cancel(&self) {
        self.job.cancel();
        self.table.cancel();
    }
}

impl Session {
    pub(super) fn refresh_markdown(&mut self) {
        self.poll_markdown_table();
        self.poll_markdown();
        if self.markdown.job.is_active() {
            return;
        }
        for id in self.visible_markdown() {
            if self.start_markdown(id) {
                break;
            }
        }
    }

    fn visible_markdown(&self) -> Vec<BufferId> {
        let Some(geometry) = self.windows.geometry(self.editing_size()) else {
            return Vec::new();
        };
        geometry
            .panes()
            .iter()
            .filter_map(|(id, _)| {
                self.windows
                    .get(*id)
                    .ok()
                    .map(|view| view.point.stamp.buffer)
            })
            .collect()
    }

    fn start_markdown(&mut self, id: BufferId) -> bool {
        let Ok(buffer) = self.buffers.get_mut(id) else {
            return false;
        };
        let Some(work) = buffer.markdown_work() else {
            return false;
        };
        let source = (buffer.stamp(), buffer.markdown_generation());
        match self
            .markdown
            .job
            .start(source.0, move |cancel| work.run(cancel))
        {
            Ok(()) => {
                self.markdown.source = Some(source);
                self.markdown.deadline = Some(Instant::now() + POLL);
            }
            Err(error) => {
                self.error(&error.to_string());
            }
        }
        true
    }

    fn markdown_source_live(&self, source: (Stamp, u64)) -> bool {
        self.buffers.get(source.0.buffer).is_ok_and(|buffer| {
            buffer.stamp() == source.0
                && buffer.markdown_generation() == source.1
                && buffer.markdown_admitted()
        }) && self.visible_markdown().contains(&source.0.buffer)
    }

    fn poll_markdown(&mut self) -> bool {
        let Some(source) = self.markdown.source else {
            return false;
        };
        let live = self.markdown_source_live(source);
        if !live {
            self.markdown.cancel();
        }
        let Some(result) = self.markdown.job.poll_owned() else {
            return false;
        };
        self.markdown.source = None;
        self.markdown.deadline = None;
        if live
            && let Ok((_, Some(observation))) = result
            && let Ok(buffer) = self.buffers.get_mut(source.0.buffer)
        {
            buffer.install_markdown(observation);
        }
        true
    }

    pub(super) fn expire_markdown(&mut self, now: Instant) -> Action {
        if self.poll_markdown_table() {
            return Action::Redraw;
        }
        if !self
            .markdown
            .deadline
            .is_some_and(|deadline| now >= deadline)
        {
            return Action::Wait;
        }
        if self.poll_markdown() {
            return Action::Redraw;
        }
        self.markdown.deadline = Some(now + POLL);
        Action::Wait
    }
}

impl Session {
    pub(super) fn markdown_command(
        &mut self,
        command: crate::command::MarkdownCommand,
    ) -> super::Action {
        if self.input.focus() != crate::input::Focus::Editing {
            return self.message("a prompt owns input focus");
        }
        if !self.buffer().is_ok_and(|buffer| buffer.is_markdown()) {
            return self.message("Native Markdown mode is not active");
        }
        if !matches!(
            command,
            crate::command::MarkdownCommand::Pretty
                | crate::command::MarkdownCommand::ResetPresentation
        ) {
            return self.markdown_edit(command);
        }
        let view = self.windows.selected_mut();
        match command {
            crate::command::MarkdownCommand::Pretty => view.toggle_markdown_presentation(),
            crate::command::MarkdownCommand::ResetPresentation => {
                view.reset_markdown_presentation()
            }
            _ => unreachable!("source commands have a separate plan owner"),
        }
        if view.pretty_markdown() && !self.markdown.job.is_active() {
            let buffer = view.point.stamp.buffer;
            if let Ok(buffer) = self.buffers.get_mut(buffer) {
                buffer.request_markdown();
            }
        }
        self.message(if self.view().pretty_markdown() {
            "Markdown: pretty"
        } else {
            "Markdown: source"
        })
    }
}

impl Session {
    fn markdown_edit(&mut self, command: crate::command::MarkdownCommand) -> Action {
        if let Some(action) = self.markdown_table(command) {
            return action;
        }
        match self.markdown_plan(command) {
            Ok(Some(plan)) => match self.publish_markdown(&plan) {
                Ok(()) => self.message(""),
                Err(error) => self.error(&error),
            },
            Ok(None)
                if matches!(
                    command,
                    crate::command::MarkdownCommand::Return
                        | crate::command::MarkdownCommand::MetaReturn
                ) =>
            {
                self.edit_command(crate::command::Command::Newline)
            }
            Ok(None) => self.error("Point is not in a Markdown list item"),
            Err(error) => self.error(&error),
        }
    }

    fn markdown_plan(
        &self,
        command: crate::command::MarkdownCommand,
    ) -> Result<Option<crate::markdown::Edit>, String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        self.view().cell(buffer).map_err(|e| e.to_string())?;
        if buffer.is_readonly() {
            return Err(crate::buffer::edit::Error::ReadOnly.to_string());
        }
        let Some(model) = buffer.markdown_observation() else {
            return if matches!(
                command,
                crate::command::MarkdownCommand::Return
                    | crate::command::MarkdownCommand::MetaReturn
            ) {
                Ok(None)
            } else {
                Err("Markdown structure is not current".into())
            };
        };
        model.list_edit(buffer, command, self.view().point.position)
    }

    fn publish_markdown(&mut self, plan: &crate::markdown::Edit) -> Result<(), String> {
        use crate::buffer::edit::Cursor;
        if plan.edits.is_empty() {
            if let Some(position) = plan.cursor {
                let point = crate::buffer::edit::Point {
                    stamp: self.buffer().map_err(|e| e.to_string())?.stamp(),
                    position,
                };
                self.place_point(point).map_err(|e| e.to_string())?;
            }
            return Ok(());
        }
        let buffer = self
            .buffers
            .get_mut(self.windows.selected().point.stamp.buffer)
            .map_err(|e| e.to_string())?;
        let points = self.windows.points(buffer).map_err(|e| e.to_string())?;
        let cursor = plan.cursor.map(|position| Cursor {
            index: points.initiator,
            position,
        });
        let mut prepared = buffer
            .prepare_retained(&plan.edits, &points.values, cursor, &plan.retained)
            .map_err(|e| e.to_string())?;
        prepared
            .retain_initiator(points.initiator)
            .map_err(|e| e.to_string())?;
        prepared.set_origin(self.windows.selected_id());
        self.windows
            .publish(buffer, prepared, points)
            .map_err(|e| e.to_string())?;
        self.windows.selected_mut().reset_navigation();
        Ok(())
    }
}
