use std::io;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use super::{Action, Pending, Session};
use crate::buffer::reload::{Checked, Reload};

#[derive(Default)]
pub(super) struct Work {
    origin: Option<crate::file::watch::Source>,
    buffer: Option<crate::buffer::BufferId>,
    task: Option<JoinHandle<io::Result<Completed>>>,
    started: Option<Instant>,
    canceled: bool,
}

enum Completed {
    Read(Reload),
    Checked(Checked),
}

impl Work {
    fn start(
        &mut self,
        buffer: crate::buffer::BufferId,
        run: impl FnOnce() -> io::Result<Completed> + Send + 'static,
    ) -> io::Result<()> {
        if self.task.is_some() {
            return Err(io::Error::other("a reload worker is still active"));
        }
        self.task = Some(
            thread::Builder::new()
                .name("file-reload".into())
                .spawn(run)?,
        );
        self.started = Some(Instant::now());
        self.buffer = Some(buffer);
        self.canceled = false;
        Ok(())
    }
}

impl Session {
    pub(super) fn reload_pending_for(&self, id: crate::buffer::BufferId) -> bool {
        (self.reload.task.is_some() && self.reload.buffer == Some(id))
            || matches!(&self.pending, Some(Pending::Reload(reload)) if reload.buffer() == id)
    }
    pub(super) fn reload_idle(&self) -> bool {
        self.reload.task.is_none()
    }
    pub(super) fn clear_reload_origin(&mut self) {
        self.reload.origin = None;
    }
    pub(super) fn start_observed_reload(
        &mut self,
        request: crate::buffer::reload::Request,
        source: crate::file::watch::Source,
    ) -> io::Result<()> {
        self.reload
            .start(source.id, move || request.read().map(Completed::Read))?;
        self.reload.origin = Some(source);
        Ok(())
    }
    pub(super) fn revert_buffer(&mut self) -> Action {
        if self.reload.task.is_some() || matches!(self.pending, Some(Pending::Reload(_))) {
            return self.message("Reload is already pending; C-g cancels");
        }
        self.reload.origin = None;
        let request = self.buffer().map_err(io::Error::other).and_then(|buffer| {
            buffer.request_reload(
                self.buffers
                    .snapshot()
                    .settings()
                    .files
                    .external_change_max_bytes,
            )
        });
        let request = match request {
            Ok(request) => request,
            Err(error) => return self.error(&format!("reload: {error}")),
        };
        match self.reload.start(self.view().point.stamp.buffer, move || {
            request.read().map(Completed::Read)
        }) {
            Ok(()) => self.message("Checking visiting file for reload"),
            Err(error) => self.error(&format!("reload: {error}")),
        }
    }

    pub(super) fn reload_remaining(&self) -> Option<Duration> {
        self.reload
            .task
            .as_ref()
            .map(|_| Duration::from_millis(if self.reload.canceled { 100 } else { 10 }))
    }

    pub(super) fn expire_reload(&mut self) -> Action {
        let Some(task) = &self.reload.task else {
            return Action::Wait;
        };
        if !task.is_finished() {
            if !self.reload.canceled
                && self
                    .reload
                    .started
                    .is_some_and(|t| t.elapsed() > crate::file::reload::DEADLINE)
            {
                self.reload.canceled = true;
                if let Some(source) = self.reload.origin.clone() {
                    return self.observation_notice(
                        &source,
                        "reload deadline exceeded; late work cannot publish; use revert-buffer",
                    );
                }
                return self
                    .error("reload: five-second deadline exceeded; late work cannot publish");
            }
            return Action::Wait;
        }
        // is_finished makes this join nonblocking. Timed-out/canceled work owns
        // the only slot until it exits; no series of replacement threads forms.
        let result = self.reload.task.take().expect("finished worker").join();
        self.reload.started = None;
        if self.reload.canceled {
            self.reload.origin = None;
            return Action::Wait;
        }
        if self
            .reload
            .origin
            .as_ref()
            .is_some_and(|s| !self.current_observation(s))
        {
            self.reload.origin = None;
            return Action::Wait;
        }
        let result = match result {
            Ok(result) => result,
            Err(_) => Err(io::Error::other("reload worker failed")),
        };
        match result {
            Err(error) => {
                if let Some(source) = self.reload.origin.take() {
                    self.observation_notice(
                        &source,
                        &format!("{error}; text retained; use revert-buffer"),
                    )
                } else {
                    self.error(&format!("reload: {error}; buffer retained"))
                }
            }
            Ok(Completed::Read(reload)) => match self.reload.origin.clone() {
                Some(source) => self.observed_reload(&source, reload),
                None => self.reload_read(reload),
            },
            Ok(Completed::Checked(candidate)) => self.reload_publish(candidate),
        }
    }

    fn reload_read(&mut self, reload: Reload) -> Action {
        let buffer = match self.buffers.get(reload.buffer()) {
            Ok(buffer) => buffer,
            Err(error) => return self.error(&format!("reload: {error}")),
        };
        if let Err(error) = reload.current(buffer) {
            return self.error(&format!("reload: {error}; buffer retained"));
        }
        if reload.needs_confirmation() {
            if self.pending.is_some()
                || self.input.focus() != crate::input::Focus::Editing
                || self.view().point.stamp.buffer != reload.buffer()
            {
                return self.message("Reload needs discard consent; invoke revert-buffer again");
            }
            let question = format!(
                "Discard unsaved contents of {} and reload? [y/n]",
                buffer.name()
            );
            if let Err(error) = self
                .input
                .decision("Discard unsaved contents and reload? [y/n]")
            {
                return self.error(&error.to_string());
            }
            self.pending = Some(Pending::Reload(Box::new(reload)));
            return self.message(&question);
        }
        self.reload_answer(reload, true)
    }

    pub(super) fn reload_answer(&mut self, reload: Reload, accepted: bool) -> Action {
        if !accepted {
            return self.message("Reload canceled; buffer text retained");
        }
        match self.reload.start(reload.buffer(), move || {
            reload.verify(true).map(Completed::Checked)
        }) {
            Ok(()) => self.message("Rechecking reload source"),
            Err(error) => self.error(&format!("reload: {error}")),
        }
    }

    fn reload_publish(&mut self, candidate: Checked) -> Action {
        let origin = self.reload.origin.take();
        let result = (|| {
            let buffer = self
                .buffers
                .get_mut(candidate.buffer())
                .map_err(io::Error::other)?;
            let points = self
                .windows
                .capture_points(buffer)
                .map_err(io::Error::other)?;
            let mut publication = buffer.prepare_reload(candidate, &points.values)?;
            if let Some(edit) = &mut publication.edit {
                edit.set_origin(self.windows.selected_id());
            }
            let label = buffer.name().to_owned();
            let changed = self.windows.reload(buffer, publication, points)?;
            Ok::<_, io::Error>((label, changed))
        })();
        match result {
            Ok((label, changed)) => {
                if changed {
                    self.invalidate_completion();
                    self.published_repeat_boundary();
                    self.activate_lsp();
                }
                // Publish association/baseline updates before native input can
                // block again, including explicit recovery of a retired watch.
                self.observe_files();
                self.message(&format!("Reloaded {label}"))
            }
            Err(error) => {
                if let Some(source) = origin {
                    self.observation_notice(
                        &source,
                        &format!("{error}; text retained; use revert-buffer"),
                    )
                } else {
                    self.error(&format!("reload: {error}; buffer retained"))
                }
            }
        }
    }

    pub(super) fn cancel_reload(&mut self) -> bool {
        let pending = matches!(self.pending, Some(Pending::Reload(_)));
        if pending {
            self.pending = None;
        }
        let active = self.reload.task.is_some() && !self.reload.canceled;
        if active {
            self.reload.canceled = true;
            if let Some(source) = self.reload.origin.clone() {
                self.observation_notice(&source, "external reload canceled; use revert-buffer");
            }
        }
        pending || active
    }
}
