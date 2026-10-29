//! Explicit native export prompts and checked file publication.
use super::{Action, Pending, Session, outline::Request};
use crate::{
    command::{Command, OrgCommand},
    config::ExportBackend,
    coordinates::{Byte, Coordinates},
    file::{self, Location, Publication},
    org::{
        Source,
        export::{
            self, Options, Scope,
            output::{Output, Prepared, Published, Work},
        },
    },
};
use std::path::{Path, PathBuf};
mod arguments;
mod compiler;
pub(super) struct Prompt {
    request: Request,
    backend: ExportBackend,
    stage: Stage,
}
enum Stage {
    Scope,
    Options(Scope),
    Path(Scope, Option<Options>),
    Confirm(Box<Prepared>),
    Compiler(Box<Prepared>, export::compiler::Compiler),
}
impl Session {
    pub(super) fn begin_export(&mut self, command: OrgCommand, backend: ExportBackend) -> Action {
        let request = Request {
            view: self.windows.selected_id(),
            origin: self.view().clone(),
            command,
        };
        match self.export_prompt(
            Prompt {
                request,
                backend,
                stage: Stage::Scope,
            },
            "Export scope (buffer/subtree/region; empty=buffer): ",
        ) {
            Ok(()) => Action::Redraw,
            Err(e) => self.error(&e),
        }
    }
    fn export_prompt(&mut self, prompt: Prompt, label: &str) -> Result<(), String> {
        self.input.argument(label, "").map_err(|e| e.to_string())?;
        self.pending = Some(Pending::Export(Box::new(prompt)));
        self.message("Enter the export selection; C-g cancels");
        Ok(())
    }
    fn launch_export(&mut self, request: Request, work: Work) -> Result<(), String> {
        if let Work::Publish(publication) = &work {
            self.export_target_available(publication.target())?;
        }
        let seconds = self.buffers.snapshot().settings().org.limits.job_seconds;
        self.org
            .start_export(request.origin.point.stamp, work, seconds)?;
        self.export_request = Some(request);
        self.message("Org export running; C-g cancels");
        Ok(())
    }
    pub(super) fn finish_export(&mut self, result: Result<Output, String>) {
        let Some(request) = self.export_request.take() else {
            return;
        };
        if let Ok(Output::Published(published)) = result {
            self.export_result(published);
            return;
        }
        if !self.org_source_current(&request)
            || self.pending.is_some()
            || self.input.focus() != crate::input::Focus::Editing
        {
            self.error("Export source or view changed; destination unchanged");
            return;
        }
        let result = result.and_then(|output| match output {
            Output::Prepared(prepared) => self.review_export(request, prepared),
            _ => unreachable!("publication handled"),
        });
        if let Err(e) = result {
            self.error(&e);
        }
    }
    fn review_export(&mut self, request: Request, prepared: Box<Prepared>) -> Result<(), String> {
        prepared.snapshot.buffers_current(&self.buffers)?;
        self.export_target_available(&prepared.target)?;
        if request.command.export_pdf() && prepared.binary.is_none() {
            return self.confirm_compiler(request, prepared);
        }
        if !prepared.target.is_present() {
            return self.publish_export(request, *prepared);
        }
        let label = format!(
            "Replace export {}? yes/no: ",
            prepared.target.path().display()
        );
        let backend = prepared.snapshot.request().backend;
        self.export_prompt(
            Prompt {
                request,
                backend,
                stage: Stage::Confirm(prepared),
            },
            &label,
        )
    }
    fn publish_export(&mut self, request: Request, prepared: Prepared) -> Result<(), String> {
        prepared.snapshot.buffers_current(&self.buffers)?;
        self.export_target_available(&prepared.target)?;
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        let publication = prepared.publication(buffer);
        self.launch_export(request, Work::Publish(Box::new(publication)))
    }
    fn export_target_available(&self, target: &Location) -> Result<(), String> {
        let identity = target.observation().map(|o| (o.device, o.inode));
        for file in self.buffers.iter().filter_map(|b| b.file()) {
            let same =
                identity.is_some() && file.observation().map(|o| (o.device, o.inode)) == identity;
            if file.path() == target.path() || same {
                return Err(
                    "Export destination is owned by a live buffer; choose another path".into(),
                );
            }
        }
        Ok(())
    }
    fn export_result(&mut self, published: Published) {
        let path = published.outcome.target().display();
        let error = published
            .outcome
            .error()
            .map(ToString::to_string)
            .unwrap_or_default();
        let message = match published.outcome.publication() {
            Publication::Complete => format!("Exported Org document to {path}"),
            Publication::Uncertain => {
                format!("Export output published to {path}; completion unresolved: {error}")
            }
            Publication::NotPublished => format!("Export output not published to {path}: {error}"),
        };
        let detail = if published.diagnostics.is_empty() {
            String::new()
        } else {
            format!(
                "; {} diagnostics: {}",
                published.diagnostics.len(),
                published
                    .diagnostics
                    .iter()
                    .take(4)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("; ")
            )
        };
        self.message(&format!("{message}{detail}"));
    }
    pub(super) fn cancel_export(&mut self, command: Command) -> Option<Action> {
        if command != Command::KeyboardQuit {
            return None;
        }
        let prompt = matches!(self.pending, Some(Pending::Export(_)));
        if self.org.export_effect {
            self.org.cancel_export();
            return Some(self.message("Export cancellation requested; awaiting file outcome"));
        }
        if self.export_request.take().is_none() && !prompt {
            return None;
        }
        self.org.cancel_export();
        self.pending = None;
        self.input.cancel();
        Some(self.message("Org export cancelled; destination unchanged"))
    }
}
