use super::{Action, Pending, Session};
use crate::{buffer::Stamp, input::Focus, shell::Operation};
use std::{path::PathBuf, time::Duration};

pub(super) struct Request {
    source: crate::buffer::edit::Point,
    view: crate::window::ViewId,
    directory: PathBuf,
}

pub(super) struct Work {
    launch: PathBuf,
    startup_shell: String,
    operation: Option<Operation>,
    request: Option<Request>,
    output: Option<Stamp>,
    diagnostics: Option<Stamp>,
}
impl Work {
    pub(super) fn new(launch: PathBuf) -> Self {
        let startup_shell = std::env::var("SHELL")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "/bin/sh".into());
        Self {
            launch,
            startup_shell,
            operation: None,
            request: None,
            output: None,
            diagnostics: None,
        }
    }
    pub(super) fn stop(&mut self) {
        self.operation = None;
        self.request = None;
    }
}

impl Session {
    pub(super) fn shell_command(&mut self) -> Action {
        if self.shell.operation.is_some() {
            return self.error(
                "A shell command is already active or stopping; shell-command-cancel cancels it",
            );
        }
        if self.input.focus() != Focus::Editing
            || self.pending.is_some()
            || self.tree.focused()
            || self.windows.retained_view_active()
        {
            return self.error("Editing must own shell command invocation");
        }
        let buffer = match self.buffer() {
            Ok(b) => b,
            Err(e) => return self.error(&e.to_string()),
        };
        let directory = buffer
            .file()
            .and_then(|file| file.path().parent())
            .or_else(|| self.tree.chosen_root())
            .unwrap_or(&self.shell.launch)
            .to_owned();
        let request = Request {
            source: self.view().point,
            view: self.windows.selected_id(),
            directory,
        };
        if let Err(e) = self.input.argument("Shell command: ", "") {
            return self.error(&e.to_string());
        }
        self.pending = Some(Pending::Shell(request));
        self.message("Enter a shell command; no document text is sent; C-g cancels")
    }
    pub(super) fn shell_argument(&mut self, request: Request, command: &str) -> Action {
        if self.shell.operation.is_some() {
            return self.error("A shell command is already active or stopping");
        }
        if self.buffers.get(request.source.stamp.buffer).is_err() {
            return self.error("Shell command source was closed");
        }
        let policy = &self.buffers.snapshot().settings().shell;
        match Operation::start(
            policy,
            &self.shell.startup_shell,
            request.directory.clone(),
            command,
        ) {
            Ok(operation) => {
                self.shell.operation = Some(operation);
                self.shell.request = Some(request);
                self.message("Shell command running; shell-command-cancel or source C-g cancels")
            }
            Err(error) => self.error(&error),
        }
    }
    pub(super) fn shell_remaining(&self) -> Option<Duration> {
        self.shell
            .operation
            .as_ref()
            .map(|_| Duration::from_millis(10))
    }
    pub(super) fn shell_cancel(&mut self) -> Action {
        if let Some(operation) = &self.shell.operation {
            operation.cancel();
            self.message("Shell command cancellation requested; stopping owned process group")
        } else {
            self.message("No shell command is active")
        }
    }
    pub(super) fn cancel_shell_input(&mut self) -> Option<Action> {
        if matches!(self.pending, Some(Pending::Shell(_))) {
            self.pending = None;
            self.input.cancel();
            return Some(self.message("Shell command prompt canceled; source unchanged"));
        }
        if self.pending.is_none()
            && self.input.focus() == Focus::Editing
            && !self.tree.focused()
            && self
                .shell
                .request
                .as_ref()
                .is_some_and(|r| r.source.stamp.buffer == self.view().point.stamp.buffer)
            && self.shell.operation.is_some()
        {
            return Some(self.shell_cancel());
        }
        None
    }
    pub(super) fn expire_shell(&mut self) -> Action {
        let Some(outcome) = self.shell.operation.as_mut().and_then(Operation::poll) else {
            return Action::Wait;
        };
        self.shell.operation = None;
        let request = self.shell.request.take().expect("owned shell request");
        let current = self.pending.is_none()
            && self.input.focus() == Focus::Editing
            && !self.tree.focused()
            && self.windows.selected_id() == request.view
            && self.view().point == request.source;
        let message = match outcome {
            Err(error) => self.shell_failure(&error),
            Ok(outcome) => {
                match (
                    crate::shell::text(outcome.stdout),
                    crate::shell::text(outcome.stderr),
                ) {
                    (Ok(stdout), Ok(stderr)) => self.shell_result(&stdout, &stderr, outcome.status),
                    (Err(error), _) | (_, Err(error)) => self.shell_failure(&error),
                }
            }
        };
        // Completion retains output without taking focus or replacing another
        // prompt's echo. The explicit output command visits the retained identity.
        if current {
            self.message(&message)
        } else {
            Action::Redraw
        }
    }
    fn shell_failure(&mut self, error: &str) -> String {
        if let Err(failure) = self.retain_shell_output(None, Some(error)) {
            return format!("Shell command failed: {error}; retain diagnostics: {failure}");
        }
        format!("Shell command failed: {error}; shell-command-output opens diagnostics")
    }
    fn shell_result(
        &mut self,
        stdout: &str,
        stderr: &str,
        status: std::process::ExitStatus,
    ) -> String {
        let diagnostic = if status.success() && stderr.is_empty() {
            None
        } else {
            Some(format!("Shell command {status}\n{stderr}"))
        };
        if let Err(error) = self.retain_shell_output(Some(stdout), diagnostic.as_deref()) {
            return format!("Shell result could not be retained: {error}");
        }
        format!(
            "Shell command {status}; shell-command-output opens retained text{}",
            if diagnostic.is_some() {
                "; diagnostics in *Shell Diagnostics*"
            } else {
                ""
            }
        )
    }
    fn shell_buffer(&mut self, name: &str, text: Option<&str>) -> Result<Option<Stamp>, String> {
        let Some(text) = text else {
            return Ok(None);
        };
        let id = self.buffers.result(name, text).map_err(|e| e.to_string())?;
        Ok(Some(
            self.buffers.get(id).map_err(|e| e.to_string())?.stamp(),
        ))
    }
    fn retain_shell_output(
        &mut self,
        stdout: Option<&str>,
        diagnostic: Option<&str>,
    ) -> Result<(), String> {
        let output = self.shell_buffer("*Shell Output*", stdout)?;
        let diagnostics = self.shell_buffer("*Shell Diagnostics*", diagnostic)?;
        let old = [self.shell.output.take(), self.shell.diagnostics.take()];
        self.shell.output = output;
        self.shell.diagnostics = diagnostics;
        // A viewed, edited or re-owned result belongs to the user. Only retire
        // a still-private, unchanged readonly buffer; never overwrite it by name.
        for stamp in old.into_iter().flatten() {
            if self
                .windows
                .iter()
                .any(|(_, v)| v.point.stamp.buffer == stamp.buffer)
            {
                continue;
            }
            if self
                .buffers
                .get(stamp.buffer)
                .is_ok_and(|b| b.stamp() == stamp && b.is_readonly())
                && let Ok(kill) = self.buffers.prepare_kill(stamp.buffer)
            {
                self.buffers.kill(kill).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }
    pub(super) fn shell_output(&mut self) -> Action {
        let Some(stamp) = self.shell.output.or(self.shell.diagnostics) else {
            return self.message("No retained shell result is available");
        };
        if !self
            .buffers
            .get(stamp.buffer)
            .is_ok_and(|b| b.stamp() == stamp && b.is_readonly())
        {
            return self.message("Retained shell result is unavailable or its ownership changed");
        }
        self.activate(stamp.buffer)
    }
}
