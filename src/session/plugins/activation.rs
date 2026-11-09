use super::super::Session;
use crate::{
    buffer::{BufferId, Stamp},
    plugins::lifecycle::{Service, State},
};
use rev_plugin_sdk::{identity::PackageId, methods::EventKind};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Bound::{Excluded, Unbounded},
    time::Duration,
};

const OBSERVATIONS: usize = 4096;
struct Observed {
    stamp: Stamp,
    language: String,
    epoch: u64,
}
struct Scan {
    cursor: Option<BufferId>,
    epoch: u64,
    closed: bool,
    count: usize,
}
#[derive(Default)]
pub(super) struct Activation {
    sources: BTreeMap<BufferId, Observed>,
    scan: Option<Scan>,
    epoch: u64,
    dirty: bool,
    triggers: BTreeSet<PackageId>,
    limited: bool,
}
impl Activation {
    pub fn remaining(&self) -> Option<Duration> {
        (self.dirty || self.scan.is_some() || !self.triggers.is_empty()).then_some(Duration::ZERO)
    }
}
impl Session {
    pub(in crate::session) fn startup_plugins(&mut self) {
        self.buffers
            .observe_events(self.plugins.host.event_observer());
        let snapshot = self.buffers.snapshot();
        if !snapshot.settings().plugins.enabled {
            return;
        }
        let mut diagnostic = None;
        for id in &snapshot.settings().plugins.startup {
            if let Some(package) = snapshot.plugins().package(id.as_str())
                && let Err(error) = self.plugins.host.activate(package)
            {
                diagnostic = Some(error);
            }
        }
        if let Some(error) = diagnostic {
            self.message(&error);
        }
    }

    /// Compare native in-memory identities only. No program, syntax, filesystem
    /// or process probe runs on input. A service scan publishes bounded triggers.
    pub(in crate::session) fn observe_plugins(&mut self) {
        if !self.buffers.snapshot().settings().plugins.enabled {
            return;
        }
        let mut count = 0;
        let changed = self
            .buffers
            .iter()
            .filter(|b| !b.is_transient() && b.terminal().is_none())
            .take(OBSERVATIONS)
            .any(|b| {
                count += 1;
                !self
                    .plugins
                    .activation
                    .sources
                    .get(&b.id())
                    .is_some_and(|old| {
                        old.stamp == b.stamp()
                            && old.language == b.language(self.buffers.snapshot())
                    })
            });
        if changed || count != self.plugins.activation.sources.len() {
            self.plugins.activation.dirty = true;
        }
    }

    pub(super) fn advance_plugin_activation(
        &mut self,
        service: &mut Service,
        notices: &mut Vec<String>,
    ) {
        if !self.buffers.snapshot().settings().plugins.enabled {
            return;
        }
        if self.plugins.activation.scan.is_none() && self.plugins.activation.dirty {
            let work = &mut self.plugins.activation;
            let Some(epoch) = work.epoch.checked_add(1) else {
                work.dirty = false;
                notices.push("Plugin activation observation exhausted".into());
                return;
            };
            work.epoch = epoch;
            work.dirty = false;
            work.scan = Some(Scan {
                cursor: None,
                epoch,
                closed: false,
                count: 0,
            });
        }
        while self.plugins.activation.scan.is_some() && service.take() {
            self.plugin_activation_item(notices);
        }
        while !self.plugins.activation.triggers.is_empty() && service.take() {
            let id = self
                .plugins
                .activation
                .triggers
                .pop_first()
                .expect("queued activation");
            let Some(package) = self.buffers.snapshot().plugins().package(id.as_str()) else {
                continue;
            };
            // Failed generations do not form automatic event-driven restart
            // loops. A later explicit command/activate/restart can retry them.
            if matches!(
                self.plugins.host.inspect(&id).state,
                State::Installed | State::Inactive
            ) && let Err(error) = self.plugins.host.activate(package)
            {
                notices.push(error);
            }
        }
    }

    fn plugin_activation_item(&mut self, notices: &mut Vec<String>) {
        let scan = self.plugins.activation.scan.as_ref().expect("active scan");
        let cursor = scan.cursor;
        let scan_epoch = scan.epoch;
        if scan.closed {
            let next = self
                .plugins
                .activation
                .sources
                .range((cursor.map_or(Unbounded, Excluded), Unbounded))
                .next()
                .map(|(id, source)| (*id, source.epoch));
            let Some((id, epoch)) = next else {
                self.plugins.activation.scan = None;
                return;
            };
            self.plugins
                .activation
                .scan
                .as_mut()
                .expect("active scan")
                .cursor = Some(id);
            if epoch != scan_epoch {
                self.plugins.activation.sources.remove(&id);
                self.plugin_trigger(EventKind::Closed, None);
            }
            return;
        }
        let next = self
            .buffers
            .iter()
            .filter(|b| !b.is_transient() && b.terminal().is_none())
            .find(|b| cursor.is_none_or(|id| b.id() > id));
        let Some(buffer) = next else {
            let scan = self.plugins.activation.scan.as_mut().expect("active scan");
            scan.closed = true;
            scan.cursor = None;
            return;
        };
        if scan.count == OBSERVATIONS {
            if !self.plugins.activation.limited {
                notices.push("Plugin activation observes at most 4096 native text buffers".into());
                self.plugins.activation.limited = true;
            }
            let scan = self.plugins.activation.scan.as_mut().expect("active scan");
            scan.closed = true;
            scan.cursor = None;
            return;
        }
        let id = buffer.id();
        let stamp = buffer.stamp();
        let language = buffer.language(self.buffers.snapshot()).to_owned();
        let old = self.plugins.activation.sources.get(&id);
        let event = if old.is_none() {
            Some(EventKind::Opened)
        } else if old.is_some_and(|old| old.stamp != stamp) {
            Some(EventKind::Changed)
        } else {
            None
        };
        let filetype = old.is_none_or(|old| old.language != language);
        let scan = self.plugins.activation.scan.as_mut().expect("active scan");
        scan.cursor = Some(id);
        scan.count += 1;
        self.plugins.activation.sources.insert(
            id,
            Observed {
                stamp,
                language: language.clone(),
                epoch: scan.epoch,
            },
        );
        if let Some(event) = event {
            self.plugin_trigger(event, filetype.then_some(language.as_str()));
        } else if filetype {
            self.plugin_trigger(EventKind::Opened, Some(&language));
        }
    }

    fn plugin_trigger(&mut self, event: EventKind, filetype: Option<&str>) {
        for package in self.buffers.snapshot().plugins().packages() {
            let activation = &package.manifest.activation;
            if activation.events.contains(&event)
                || filetype.is_some_and(|language| {
                    activation
                        .filetypes
                        .iter()
                        .any(|id| id.as_str() == language)
                })
            {
                self.plugins
                    .activation
                    .triggers
                    .insert(package.manifest.plugin.id.clone());
            }
        }
    }
}
