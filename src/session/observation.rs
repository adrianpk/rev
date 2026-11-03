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

impl Session {
    /// Reconcile only in-memory file associations. Registration, metadata and
    /// notification consumption belong to the IO worker.
    pub(super) fn observe_files(&mut self) {
        let limit = self
            .buffers
            .snapshot()
            .settings()
            .files
            .external_change_max_buffers;
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
        if unchanged && count == self.observation.sources.len() {
            if over_limit && !self.observation.limited {
                self.message(
                    "File observation buffer limit reached; use revert-buffer for unobserved files",
                );
            }
            self.observation.limited = over_limit;
            return;
        }
        let mut sources = Vec::new();
        let mut limited = false;
        let mut exhausted = false;
        for buffer in self.buffers.iter().filter(|b| !b.is_transient()) {
            let Some(file) = buffer.file() else {
                continue;
            };
            if sources.len() == limit {
                limited = true;
                break;
            }
            let prior = self
                .observation
                .sources
                .binary_search_by_key(&buffer.id(), |s| s.id)
                .ok()
                .map(|index| &self.observation.sources[index])
                .filter(|s| s.path == file.path() && s.baseline.as_ref() == file.observation());
            let generation = match prior {
                Some(prior) => prior.generation,
                None => {
                    let Some(next) = self.observation.next.checked_add(1) else {
                        exhausted = true;
                        break;
                    };
                    self.observation.next = next;
                    next
                }
            };
            sources.push(Source {
                id: buffer.id(),
                generation,
                path: file.path().into(),
                baseline: file.observation().cloned(),
            });
        }
        if exhausted {
            self.message("file observation generation exhausted; use revert-buffer");
            return;
        }
        if limited && !self.observation.limited {
            self.message(
                "File observation buffer limit reached; use revert-buffer for unobserved files",
            );
        }
        self.observation.limited = limited;
        if sources == self.observation.sources {
            return;
        }
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
        if let Some(watch) = &self.observation.watch {
            watch.update(sources.clone());
        } else if !sources.is_empty() {
            match Watch::start(
                sources.clone(),
                self.buffers
                    .snapshot()
                    .settings()
                    .files
                    .external_change_debounce_ms,
                self.observation.wake.clone(),
            ) {
                Ok(watch) => self.observation.watch = Some(watch),
                Err(error) => {
                    self.message(&format!(
                        "file observation unavailable: {error}; use revert-buffer"
                    ));
                }
            }
        }
        self.observation.sources = sources;
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
        let mut action = Action::Wait;
        if self.observation_ready()
            && let Some(watch) = &self.observation.watch
        {
            let (events, diagnostic) = watch.take();
            if let Some(diagnostic) = diagnostic {
                action = self.message(&diagnostic);
            }
            for event in events {
                if self.current_observation(&event.source) {
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
            }
        }
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
                let request = match self.buffers.get(changed.source.id) {
                    Ok(buffer) => {
                        // An editor-owned save/reload already acknowledged this
                        // exact completed observation; do not manufacture a loop.
                        if buffer.file().and_then(|f| f.observation()) == Some(&observed) {
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
                match request
                    .and_then(|request| self.start_observed_reload(request, changed.source.clone()))
                {
                    Ok(()) => action,
                    Err(error) => self.observation_notice(
                        &changed.source,
                        &format!("{error}; use revert-buffer"),
                    ),
                }
            }
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

    pub(super) fn observed_reload(&mut self, source: Source, reload: Reload) -> Action {
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
                    &source,
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
            &source,
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
