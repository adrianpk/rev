use std::time::{Duration, Instant};

use crate::buffer::{BufferId, Stamp};
use crate::org::{ColorScan, Coloring, Job, JobError, Limits};

use super::{Action, Session};

const POLL: Duration = Duration::from_millis(10);

/// One temporary worker for the session, including shared-buffer views. The
/// native event owner wakes only while work is owned, and never joins live work.
#[derive(Default)]
pub(super) struct Work {
    job: Job<Coloring>,
    pub(super) request: Option<super::outline::Request>,
    source: Option<Stamp>,
    deadline: Option<Instant>,
}

impl Work {
    pub(super) fn cancel(&mut self) {
        self.request = None;
        self.job.cancel();
    }

    pub(super) fn cancel_request(&mut self) {
        if self.request.take().is_some() {
            self.job.cancel();
        }
    }
    pub(super) fn remaining(&self, now: Instant) -> Option<Duration> {
        self.deadline
            .map(|deadline| deadline.saturating_duration_since(now))
    }
}

impl Session {
    pub(super) fn refresh_org(&mut self) {
        self.poll_org();
        self.finish_org_request();
        if self.org.job.is_active() {
            return;
        }
        if let Some(request) = &self.org.request {
            let id = request.origin.point.stamp.buffer;
            self.start_org(id);
            return;
        }
        let Some(id) = self.visible_org().into_iter().find(|id| {
            self.buffers
                .get(*id)
                .is_ok_and(|buffer| buffer.needs_org_colors())
        }) else {
            return;
        };
        self.start_org(id);
    }

    fn visible_org(&self) -> Vec<BufferId> {
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

    fn start_org(&mut self, id: BufferId) {
        let Ok(buffer) = self.buffers.get_mut(id) else {
            return;
        };
        let scan = match buffer.take_org_index() {
            Some(index) => ColorScan::resume(index, buffer),
            None => ColorScan::new(buffer, Limits::default()),
        };
        buffer.attempted_org_colors();
        let Ok(scan) = scan else { return };
        let stamp = buffer.stamp();
        match self.org.job.start(stamp, move |cancel| scan.run(cancel)) {
            Ok(()) => {
                self.org.source = Some(stamp);
                self.org.deadline = Some(Instant::now() + POLL);
            }
            Err(error) => {
                self.org.request = None;
                self.error(&error.to_string());
            }
        }
    }

    fn org_source_live(&self, source: Stamp) -> bool {
        self.buffers.get(source.buffer).is_ok_and(|buffer| {
            buffer.stamp() == source
                && buffer.is_org()
                && (buffer.org_colors_admitted()
                    || self
                        .org
                        .request
                        .as_ref()
                        .is_some_and(|r| r.origin.point.stamp == source))
        }) && self.visible_org().contains(&source.buffer)
    }

    fn poll_org(&mut self) -> bool {
        let Some(source) = self.org.source else {
            return false;
        };
        if !self.org_source_live(source) {
            self.org.job.cancel();
        }
        // A removed buffer is polled against the live selected buffer solely
        // to reap ownership; the job rejects the foreign source before publish.
        let buffer = self
            .buffers
            .get(source.buffer)
            .or_else(|_| self.buffers.get(self.windows.selected().point.stamp.buffer));
        let Ok(buffer) = buffer else { return false };
        let Some(result) = self.org.job.poll(buffer) else {
            return false;
        };
        self.org.source = None;
        self.org.deadline = None;
        match result {
            Ok(coloring) => {
                self.publish_org(source.buffer, coloring);
                self.finish_org_request();
            }
            Err(JobError::Cancelled) => {
                self.finish_org_request();
            }
            Err(_) => {
                self.org.request = None;
            }
        }
        true
    }

    fn publish_org(&mut self, id: BufferId, coloring: Coloring) {
        if let Ok(buffer) = self.buffers.get_mut(id) {
            let _ = buffer.install_org_coloring(coloring);
        }
    }

    pub(super) fn expire_org(&mut self, now: Instant) -> Action {
        if !self.org.deadline.is_some_and(|deadline| now >= deadline) {
            return Action::Wait;
        }
        if self.poll_org() {
            return Action::Redraw;
        }
        self.org.deadline = Some(now + POLL);
        Action::Wait
    }
}
