use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;

use super::{Action, Pending, Session};
use crate::buffer::{BufferId, reload::Reload};
use crate::file::watch::{BATCH, Changed, Disk, QUEUE, Source, Wake, Watch};

#[derive(Default)]
pub(super) struct Work {
    watch: Option<Watch>,
    pub(super) wake: Arc<Wake>,
    sources: Vec<Source>,
    next: u64,
    pending: VecDeque<Changed>,
    notices: BTreeMap<BufferId, Disk>,
    limited: bool,
}
impl Work {
    pub(super) fn with_wake(wake: Arc<Wake>) -> Self {
        Self {
            wake,
            ..Self::default()
        }
    }

    fn generation(&mut self, id: BufferId, file: &crate::file::Location) -> Option<u64> {
        let prior = self
            .sources
            .binary_search_by_key(&id, |s| s.id)
            .ok()
            .map(|index| &self.sources[index])
            .filter(|s| s.path == file.path() && s.baseline.as_ref() == file.observation());
        match prior {
            Some(prior) => Some(prior.generation),
            None => {
                let next = self.next.checked_add(1)?;
                self.next = next;
                Some(next)
            }
        }
    }
}

impl Session {
    /// Reconcile only in-memory file associations. Registration, metadata and
    /// notification consumption belong to the IO worker.
    pub(super) fn observe_files(&mut self) {
        self.observe_plugins();
        let limit = self
            .buffers
            .snapshot()
            .settings()
            .files
            .external_change_max_buffers;
        if let Some(limited) = self.unchanged_observations(limit) {
            self.observation_limit(limited);
            return;
        }
        let Some((sources, limited)) = self.prepare_observation_sources(limit) else {
            self.message("file observation generation exhausted; use revert-buffer");
            return;
        };
        self.observation_limit(limited);
        if sources == self.observation.sources {
            return;
        }
        self.retire_observation_sources(&sources);
        self.update_observation_watch(&sources);
        self.observation.sources = sources;
    }

    fn unchanged_observations(&self, limit: usize) -> Option<bool> {
        // The common typing/navigation path compares borrowed evidence only.
        // Source vectors follow the registry's stable BufferId order.
        let mut count = 0;
        let mut unchanged = true;
        let mut over_limit = false;
        for buffer in self.buffers.iter().filter(|b| !b.is_transient()) {
            let Some(file) = buffer.file() else {
                continue;
            };
            if count == limit {
                over_limit = true;
                break;
            }
            unchanged &= self.observation.sources.get(count).is_some_and(|s| {
                s.id == buffer.id()
                    && s.path == file.path()
                    && s.baseline.as_ref() == file.observation()
            });
            count += 1;
        }
        (unchanged && count == self.observation.sources.len()).then_some(over_limit)
    }

    fn prepare_observation_sources(&mut self, limit: usize) -> Option<(Vec<Source>, bool)> {
        let mut sources = Vec::new();
        let mut limited = false;
        for buffer in self.buffers.iter().filter(|b| !b.is_transient()) {
            let Some(file) = buffer.file() else {
                continue;
            };
            if sources.len() == limit {
                limited = true;
                break;
            }
            let generation = self.observation.generation(buffer.id(), file)?;
            sources.push(Source {
                id: buffer.id(),
                generation,
                path: file.path().into(),
                baseline: file.observation().cloned(),
            });
        }
        Some((sources, limited))
    }

    fn observation_limit(&mut self, limited: bool) {
        if limited && !self.observation.limited {
            self.message(
                "File observation buffer limit reached; use revert-buffer for unobserved files",
            );
        }
        self.observation.limited = limited;
    }

    fn retire_observation_sources(&mut self, sources: &[Source]) {
        let retired: Vec<_> = self
            .observation
            .sources
            .iter()
            .filter(|old| {
                !sources
                    .iter()
                    .any(|new| new.id == old.id && new.generation == old.generation)
            })
            .map(|old| old.id)
            .collect();
        for id in retired {
            self.observation.notices.remove(&id);
            if let Ok(buffer) = self.buffers.get_mut(id) {
                buffer.set_external_notice(None);
            }
        }
        self.observation.pending.retain(|pending| {
            sources
                .iter()
                .any(|s| s.id == pending.source.id && s.generation == pending.source.generation)
        });
        self.observation
            .notices
            .retain(|id, _| sources.iter().any(|s| s.id == *id));
    }

    fn update_observation_watch(&mut self, sources: &[Source]) {
        if let Some(watch) = &self.observation.watch {
            watch.update(sources.to_vec());
        } else if !sources.is_empty() {
            match Watch::start(
                sources.to_vec(),
                self.buffers
                    .snapshot()
                    .settings()
                    .files
                    .external_change_debounce_ms,
                Arc::clone(&self.observation.wake),
            ) {
                Ok(watch) => self.observation.watch = Some(watch),
                Err(error) => {
                    self.message(&format!(
                        "file observation unavailable: {error}; use revert-buffer"
                    ));
                }
            }
        }
    }

    pub(super) fn current_observation(&self, source: &Source) -> bool {
        self.observation.sources.iter().any(|s| {
            s.id == source.id && s.generation == source.generation && s.path == source.path
        })
    }

    pub(super) fn observation_pending_for(&self, id: BufferId) -> bool {
        self.observation
            .pending
            .iter()
            .any(|event| event.source.id == id)
            || self
                .observation
                .watch
                .as_ref()
                .is_some_and(|watch| watch.pending_for(id))
            || self
                .buffers
                .get(id)
                .is_ok_and(|buffer| buffer.external_change_notice().is_some())
    }

    pub(super) fn observation_ready(&self) -> bool {
        self.observation.wake.ready() && self.observation.pending.len() <= QUEUE - BATCH
    }

    pub(super) fn observation_remaining(&self) -> Option<Duration> {
        (self.observation_ready()
            || (!self.observation.pending.is_empty()
                && self.reload_idle()
                && !matches!(self.pending, Some(Pending::Reload(_)))))
        .then_some(Duration::ZERO)
    }

    pub(super) fn expire_observation(&mut self) -> Action {
        let action = self.take_observation_events();
        if !self.reload_idle() || matches!(self.pending, Some(Pending::Reload(_))) {
            return action;
        }
        let Some(changed) = self.observation.pending.pop_front() else {
            return action;
        };
        if !self.current_observation(&changed.source) {
            return action;
        }
        let fresh = self
            .observation
            .notices
            .insert(changed.source.id, changed.disk.clone())
            != Some(changed.disk.clone());
        if fresh && let Ok(buffer) = self.buffers.get_mut(changed.source.id) {
            buffer.set_external_notice(None);
        }
        match changed.disk {
            Disk::Unavailable(error) => self.observation_notice(&changed.source, &error),
            Disk::Image(None) => self.observation_notice(
                &changed.source,
                "file is missing; text retained; use revert-buffer after recreation",
            ),
            Disk::Image(Some(observed)) => {
                self.reload_observed_image(&changed.source, &observed, action)
            }
        }
    }

    fn take_observation_events(&mut self) -> Action {
        let mut action = Action::Wait;
        if self.observation_ready()
            && let Some(watch) = &self.observation.watch
        {
            let (events, diagnostic) = watch.take();
            if let Some(diagnostic) = diagnostic {
                action = self.message(&diagnostic);
            }
            for event in events {
                self.queue_observation_event(event);
            }
        }
        action
    }

    fn queue_observation_event(&mut self, event: Changed) {
        if !self.current_observation(&event.source) {
            return;
        }
        if let Some(queued) = self
            .observation
            .pending
            .iter_mut()
            .find(|e| e.source.id == event.source.id)
        {
            *queued = event;
        } else {
            self.observation.pending.push_back(event);
        }
    }

    fn reload_observed_image(
        &mut self,
        source: &Source,
        observed: &crate::file::Observation,
        action: Action,
    ) -> Action {
        let request = match self.buffers.get(source.id) {
            Ok(buffer) => {
                // An editor-owned save/reload already acknowledged this
                // exact completed observation; do not manufacture a loop.
                if buffer.file().and_then(|f| f.observation()) == Some(observed) {
                    return action;
                }
                buffer.request_reload(
                    self.buffers
                        .snapshot()
                        .settings()
                        .files
                        .external_change_max_bytes,
                )
            }
            Err(_) => return action,
        };
        match request.and_then(|request| self.start_observed_reload(request, source.clone())) {
            Ok(()) => action,
            Err(error) => self.observation_notice(source, &format!("{error}; use revert-buffer")),
        }
    }

    pub(super) fn observation_notice(&mut self, source: &Source, reason: &str) -> Action {
        if !self.current_observation(source) {
            return Action::Wait;
        }
        let Ok(buffer) = self.buffers.get_mut(source.id) else {
            return Action::Wait;
        };
        let notice = format!("{}: {reason}", buffer.name());
        if buffer.external_change_notice() == Some(notice.as_str()) {
            return Action::Wait;
        }
        buffer.set_external_notice(Some(notice.clone()));
        // Echo and per-buffer status are passive; no prompt or input focus moves.
        self.message(&notice)
    }

    pub(super) fn observed_reload(&mut self, source: &Source, reload: Reload) -> Action {
        let admitted = self
            .buffers
            .get(source.id)
            .map_err(std::io::Error::other)
            .and_then(|buffer| {
                reload
                    .current(buffer)
                    .map(|()| (reload.matches(buffer), buffer.is_modified()))
            });
        let (same, dirty) = match admitted {
            Ok(result) => result,
            Err(error) => {
                self.clear_reload_origin();
                return self.observation_notice(
                    source,
                    &format!("local source changed: {error}; use revert-buffer"),
                );
            }
        };
        if same
            || (!dirty
                && self.buffers.snapshot().settings().files.external_changes
                    == crate::config::ExternalChanges::Reload)
        {
            // Identical complete bytes cannot discard local text. The final
            // native state/disk checks still fence typing and publication races.
            return self.reload_answer(reload, true);
        }
        self.clear_reload_origin();
        self.observation_notice(
            source,
            if dirty {
                "external change conflicts with unsaved text; use revert-buffer"
            } else {
                "external change pending; use revert-buffer"
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;

    // A completed native save must retire older queued notifications, including
    // missing/unavailable results that cannot be ordered by their file metadata.
    #[test]
    fn publication_retires_notices() {
        let fixture = fixture::Fixture::new();
        let path = fixture.write("source", "text");
        let mut session = Session::empty(crate::view::Size {
            width: 80,
            height: 24,
        })
        .unwrap();
        session.load(&path).unwrap();
        let old = session.observation.sources[0].clone();
        assert!(session.buffers.save(old.id).unwrap().is_complete());
        session.observe_files();
        assert_eq!(
            session.observation_notice(&old, "late missing observation; use revert-buffer"),
            Action::Wait
        );
        assert!(session.buffer().unwrap().external_change_notice().is_none());
        assert!(!session.buffer().unwrap().is_modified());
    }
}
