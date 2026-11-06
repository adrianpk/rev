use std::collections::{BTreeMap, VecDeque};

use super::Command;

const WINDOW: usize = 256;
const RECENT: usize = 4;
const FREQUENT: usize = 4;

/// Accepted named invocations, oldest first. Repetitions retain frequency;
/// distinct prompt-history navigation has a separate owner and bound.
#[derive(Clone, Debug, Default)]
pub(crate) struct Usage {
    events: VecDeque<String>,
}

impl Usage {
    pub(crate) fn record(&mut self, command: Command) {
        if self.events.len() == WINDOW {
            self.events.pop_front();
        }
        self.events.push_back(command.name().to_owned());
    }

    /// Rank admitted candidates within one text-match rank. Historical names
    /// cannot introduce candidates, and every identity appears only once.
    pub(crate) fn rank(&self, candidates: &mut [Command]) {
        let mut usage = BTreeMap::<&str, (usize, usize)>::new();
        for (position, name) in self.events.iter().enumerate() {
            let entry = usage.entry(name).or_default();
            entry.0 += 1;
            entry.1 = position;
        }
        let mut used: Vec<_> = candidates
            .iter()
            .filter_map(|command| {
                usage
                    .get(command.name())
                    .map(|&(count, last)| (command.name(), count, last))
            })
            .collect();
        used.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(b.0)));
        let mut promoted: Vec<_> = used.iter().take(RECENT).map(|entry| entry.0).collect();
        used.drain(..used.len().min(RECENT));
        used.sort_by(|a, b| {
            b.1.cmp(&a.1)
                .then_with(|| b.2.cmp(&a.2))
                .then_with(|| a.0.cmp(b.0))
        });
        promoted.extend(used.iter().take(FREQUENT).map(|entry| entry.0));
        // Own the names before mutating the candidate slice they came from.
        let promoted: Vec<String> = promoted.into_iter().map(str::to_owned).collect();
        candidates.sort_by(|a, b| {
            let position = |command: &Command| {
                promoted
                    .iter()
                    .position(|name| name == command.name())
                    .unwrap_or(promoted.len())
            };
            position(a)
                .cmp(&position(b))
                .then_with(|| a.name().cmp(b.name()))
        });
    }
}
