mod attachment;
mod block;
mod document;
mod exports;
mod links;
mod publishing;
mod refile;
mod tables;
use std::time::{Duration, Instant};

use crate::buffer::{BufferId, Stamp};
use crate::org::{ColorScan, Coloring, Job, JobError, Limits};

use super::{Action, Session};

const POLL: Duration = Duration::from_millis(10);

/// One temporary worker for the session, including shared-buffer views. The
/// native event owner wakes only while work is owned, and never joins live work.
#[derive(Default)]
pub(super) struct Work {
    job: Job<Observation>,
    pub(super) sources_pending: Option<crate::org::agenda::SourceSelection>,
    pub(super) sources_running: bool,
    capture_running: bool,
    block_running: bool,
    refile_running: bool,
    links_running: bool,
    document_running: bool,
    table_running: bool,
    pub(super) table_effect: bool,
    pub(super) export_running: bool,
    pub(super) publish_running: bool,
    pub(super) export_effect: bool,
    attachment_running: bool,
    capture_finalizing: bool,
    pub(super) report: Option<super::outline::Request>,
    pub(super) request: Option<super::outline::Request>,
    source: Option<Stamp>,
    deadline: Option<Instant>,
}

enum Observation {
    SourceBlock(Result<crate::org::block::Definition, String>),
    Publish(Result<crate::org::publish::Output, String>),
    Export(Result<crate::org::export::output::Output, String>),
    Table(Result<crate::org::table::Output, String>),
    Document(Result<crate::org::document::Output, String>),
    AttachmentPlan(Result<crate::org::attachment::Plan, String>),
    AttachmentListing(Result<crate::org::attachment::Listing, String>),
    AttachmentEffect(crate::org::attachment::FileOutcome),
    Links(Result<crate::org::links::Output, String>),
    LinkViewer(Result<crate::org::io::process::Outcome, String>),
    LinkNavigation(Result<crate::org::links::Navigation, String>),
    RefileTargets(Result<crate::org::refile::Targets, String>),
    RefilePlan(Result<crate::org::refile::Plan, String>),
    Color(Coloring),
    Capture(Result<crate::org::capture::Resolved, String>),
    CaptureFinalize(Result<crate::org::capture::Insertion, String>),
    Sources(Result<super::agenda::Output, String>),
    Report(Result<super::outline::reports::Output, String>),
}
impl Work {
    pub(super) fn cancel(&mut self) {
        self.request = None;
        self.report = None;
        self.sources_pending = None;
        self.job.cancel();
    }

    pub(super) fn cancel_request(&mut self) {
        let report = self.report.take();
        if self.request.take().is_some() || report.is_some() {
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
        if self.org.job.is_active() || self.capture_blocks_input() {
            return;
        }
        if self.launch_agenda_sources() {
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
        match self
            .org
            .job
            .start(stamp, move |cancel| Observation::Color(scan.run(cancel)))
        {
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

    fn poll_org_files(&mut self) -> Option<bool> {
        if self.org.block_running {
            return Some(self.poll_block());
        }
        self.poll_org_outputs()
            .or_else(|| self.poll_org_interactions())
    }
    fn poll_org_outputs(&mut self) -> Option<bool> {
        if self.org.publish_running {
            return Some(self.poll_publishing());
        }
        if self.org.export_running {
            return Some(self.poll_export());
        }
        if self.org.table_running {
            return Some(self.poll_table());
        }
        if self.org.document_running {
            return Some(self.poll_document());
        }
        None
    }
    fn poll_org_interactions(&mut self) -> Option<bool> {
        if self.org.attachment_running {
            return Some(self.poll_attachment());
        }
        if self.org.links_running {
            return Some(self.poll_links());
        }
        if self.org.refile_running {
            return Some(self.poll_refile_job());
        }
        if self.org.capture_running {
            return Some(self.poll_capture());
        }
        if self.org.sources_running {
            return Some(self.poll_agenda_sources());
        }
        None
    }
    fn poll_org(&mut self) -> bool {
        if let Some(result) = self.poll_org_files() {
            return result;
        }
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
            Ok(Observation::Color(coloring)) => {
                self.publish_org(source.buffer, coloring);
                self.finish_org_request();
            }
            Ok(Observation::Report(result)) => self.finish_clock_report(result),
            Ok(_) => unreachable!("source discovery owns separate publication checks"),
            Err(JobError::Cancelled) => {
                self.org.report = None;
                self.finish_org_request();
            }
            Err(_) => {
                self.org.report = None;
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

impl Work {
    pub(super) fn busy(&self) -> bool {
        self.job.is_active()
    }
    pub(super) fn start_report(
        &mut self,
        request: super::outline::Request,
        work: super::outline::reports::Work,
        operation: crate::org::time::ZonedOperation,
    ) -> Result<(), String> {
        let stamp = request.origin.point.stamp;
        self.job
            .start(stamp, move |cancel| {
                Observation::Report(work.run(operation.zone(), cancel))
            })
            .map_err(|e| e.to_string())?;
        self.source = Some(stamp);
        self.report = Some(request);
        self.deadline = Some(Instant::now() + POLL);
        Ok(())
    }
}

impl Work {
    pub(super) fn start_sources(
        &mut self,
        stamp: Stamp,
        work: super::agenda::Work,
    ) -> Result<(), String> {
        self.job
            .start(stamp, move |cancel| Observation::Sources(work.run(cancel)))
            .map_err(|e| e.to_string())?;
        self.sources_running = true;
        self.source = Some(stamp);
        self.deadline = Some(Instant::now() + POLL);
        Ok(())
    }
}
impl Session {
    fn poll_agenda_sources(&mut self) -> bool {
        let Some(result) = self.org.job.poll_owned() else {
            return false;
        };
        self.org.source = None;
        self.org.deadline = None;
        self.org.sources_running = false;
        match result {
            Ok((_, Observation::Sources(result))) => self.finish_agenda_sources(result),
            Err(JobError::Cancelled) => {}
            Err(error) => {
                self.error(&error.to_string());
            }
            _ => unreachable!("source discovery owns this publication"),
        }
        true
    }
}

impl Work {
    pub(super) fn start_capture(
        &mut self,
        stamp: Stamp,
        work: crate::org::capture::Discovery,
    ) -> Result<(), String> {
        self.job
            .start(stamp, move |cancel| Observation::Capture(work.run(cancel)))
            .map_err(|e| e.to_string())?;
        self.capture_finalizing = false;
        self.capture_running = true;
        self.source = Some(stamp);
        self.deadline = Some(Instant::now() + POLL);
        Ok(())
    }
    pub(super) fn cancel_capture(&mut self) {
        if self.capture_running {
            self.job.cancel();
        }
    }
}
impl Session {
    fn poll_capture(&mut self) -> bool {
        let Some(result) = self.org.job.poll_owned() else {
            return false;
        };
        let finalizing = self.org.capture_finalizing;
        self.org.capture_finalizing = false;
        self.org.capture_running = false;
        self.org.source = None;
        self.org.deadline = None;
        match result {
            Ok((_, Observation::Capture(result))) => self.finish_capture(result),
            Ok((_, Observation::CaptureFinalize(result))) => self.finish_capture_finalize(result),
            Err(JobError::Cancelled) => {}
            Err(error) if finalizing => self.finish_capture_finalize(Err(error.to_string())),
            Err(error) => self.finish_capture(Err(error.to_string())),
            _ => unreachable!("capture discovery owns this publication"),
        }
        true
    }
}

impl Work {
    pub(super) fn start_capture_finalize(
        &mut self,
        stamp: Stamp,
        work: crate::org::capture::Finalize,
    ) -> Result<(), String> {
        self.job
            .start(stamp, move |cancel| {
                Observation::CaptureFinalize(work.run(cancel))
            })
            .map_err(|e| e.to_string())?;
        self.capture_finalizing = true;
        self.capture_running = true;
        self.source = Some(stamp);
        self.deadline = Some(Instant::now() + POLL);
        Ok(())
    }
}
