mod prompt;
mod viewer;
mod visit;
use super::{Action, Pending, Session, outline::Request};
use crate::{
    buffer::edit::Point,
    command::{Command, OrgCommand as C},
    input::Focus,
    org::{
        agenda::SourceSelection,
        links::{Follow, Output, Resolve, Snapshot, Stored},
    },
};
pub(super) enum Prompt {
    Insert(prompt::Prompt),
    Viewer(viewer::Prompt),
}
use std::collections::VecDeque;
#[derive(Default)]
pub(super) struct State {
    pub(in crate::session) request: Option<Request>,
    stored: VecDeque<Stored>,
    marks: VecDeque<Point>,
    wrap: Option<(Point, C)>,
}
impl Session {
    pub(in crate::session) fn start_links(&mut self, request: Request) -> Result<(), String> {
        if self.capture.is_some() || self.refile.is_some() || self.links.request.is_some() {
            return Err("Finish or cancel the active source interaction before links".into());
        }
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        if request.command == C::InsertLink && buffer.is_readonly() {
            return Err("Link insertion source is read-only".into());
        }
        let policy = &self.buffers.snapshot().settings().org;
        let snapshot = Snapshot::new(buffer, request.origin.point, policy)?;
        let wrap = self.links.wrap == Some((request.origin.point, request.command));
        self.org
            .start_links(snapshot, request.command, wrap, policy.limits.job_seconds)?;
        self.links.request = Some(request);
        self.message("Checking Org links; C-g cancels");
        Ok(())
    }
    pub(in crate::session) fn finish_links(&mut self, result: Result<Output, String>) {
        let Some(request) = self.links.request.take() else {
            return;
        };
        if !self.org_source_current(&request)
            || self.input.focus() != Focus::Editing
            || self.pending.is_some()
        {
            self.error("Link source or view changed; nothing followed");
            return;
        }
        if let Err(error) = result.and_then(|output| self.link_output(output, request)) {
            self.error(&error);
        }
    }
    fn link_output(&mut self, output: Output, request: Request) -> Result<(), String> {
        match output {
            Output::Store(stored) => {
                self.retain_link(stored);
                self.message("Org link stored for insertion");
            }
            Output::Edit(edit) => return self.begin_link_edit(edit, request),
            Output::Move(byte) => {
                self.links.wrap = None;
                self.move_link_point(byte)?;
            }
            Output::Wrap => {
                self.links.wrap = Some((request.origin.point, request.command));
                self.message("No further link; repeat to wrap");
            }
            Output::Follow(target) => return self.resolve_link(target, request),
        }
        Ok(())
    }
    fn resolve_link(&mut self, target: Follow, request: Request) -> Result<(), String> {
        if target.external_resource() {
            return self.prompt_viewer(target, request);
        }
        let selection = self.link_sources()?;
        let work = Resolve::new(&self.buffers, request.origin.point.stamp, target, selection)?;
        let seconds = self.buffers.snapshot().settings().org.limits.job_seconds;
        self.org
            .start_link_navigation(request.origin.point.stamp, work, seconds)?;
        self.links.request = Some(request);
        self.message("Resolving checked Org destination; C-g cancels");
        Ok(())
    }
    fn link_sources(&self) -> Result<SourceSelection, String> {
        let mut selection = SourceSelection::configured(self.buffers.snapshot())?;
        selection.buffers = self
            .buffers
            .iter()
            .filter(|b| b.is_org())
            .map(|b| b.id())
            .collect();
        if let Some(sources) = &self.agenda_sources {
            selection
                .paths
                .extend(sources.documents.iter().filter_map(|s| s.path.clone()));
        }
        selection.paths.sort();
        selection.paths.dedup();
        Ok(selection)
    }
    fn retain_link(&mut self, stored: Stored) {
        self.links.stored.retain(|l| l.target != stored.target);
        self.links.stored.push_front(stored);
        self.links.stored.truncate(32);
    }
    pub(in crate::session) fn store_plain_link(&mut self) -> Action {
        let result = self
            .buffer()
            .map_err(|e| e.to_string())
            .and_then(|buffer| crate::org::capture::origin_link(buffer, self.view().point));
        match result {
            Ok(Some(link)) => {
                let target = link
                    .trim_start_matches("[[")
                    .trim_end_matches("]]")
                    .to_owned();
                self.retain_link(Stored {
                    target,
                    description: String::new(),
                });
                self.message("File link stored for insertion")
            }
            Ok(None) => self.message("Associate this source with a file before storing a link"),
            Err(error) => self.error(&error),
        }
    }
    pub(in crate::session) fn cancel_links(&mut self, command: Command) -> Option<Action> {
        let pending = matches!(self.pending, Some(Pending::Link(_)));
        if command != Command::KeyboardQuit || self.links.request.is_none() && !pending {
            return None;
        }
        self.org.cancel();
        self.links.request = None;
        self.pending = None;
        self.input.cancel();
        Some(self.message("Org link cancelled; source unchanged"))
    }
}

impl Session {
    pub(in crate::session) fn link_prompt_argument(
        &mut self,
        prompt: Prompt,
        draft: &str,
    ) -> Action {
        match prompt {
            Prompt::Insert(prompt) => self.link_argument(prompt, draft),
            Prompt::Viewer(prompt) => self.viewer_argument(prompt, draft),
        }
    }
    pub(in crate::session) fn links_source_changed(&self) -> bool {
        self.links
            .request
            .as_ref()
            .is_some_and(|r| !self.org_source_current(r))
    }
}

impl Session {
    pub(in crate::session) fn store_attachment_link(
        &mut self,
        target: String,
        description: String,
    ) {
        self.retain_link(Stored {
            target,
            description,
        });
    }
    pub(in crate::session) fn follow_attachment(
        &mut self,
        target: Follow,
        request: Request,
        attachment: crate::org::attachment::Selected,
    ) -> Result<(), String> {
        if target.external_resource() {
            self.prompt_selected_viewer(
                target,
                request,
                crate::org::links::Selection::Attachment {
                    attachment: Box::new(attachment),
                    native: false,
                },
            )
        } else {
            let policy = &self.buffers.snapshot().settings().org;
            let work = Resolve::new(
                &self.buffers,
                request.origin.point.stamp,
                target,
                self.link_sources()?,
            )?
            .selected(attachment);
            self.org.start_link_navigation(
                request.origin.point.stamp,
                work,
                policy.limits.job_seconds,
            )?;
            self.links.request = Some(request);
            Ok(())
        }
    }
}
