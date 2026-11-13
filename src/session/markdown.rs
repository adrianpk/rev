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
    source: Option<(Stamp, u64)>,
    deadline: Option<Instant>,
}

impl Work {
    pub(super) fn remaining(&self, now: Instant) -> Option<Duration> {
        self.deadline
            .map(|value| value.saturating_duration_since(now))
    }

    pub(super) fn cancel(&self) {
        self.job.cancel();
    }
}

impl Session {
    pub(super) fn refresh_markdown(&mut self) {
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
        let view = self.windows.selected_mut();
        match command {
            crate::command::MarkdownCommand::Pretty => view.toggle_markdown_presentation(),
            crate::command::MarkdownCommand::ResetPresentation => {
                view.reset_markdown_presentation()
            }
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
