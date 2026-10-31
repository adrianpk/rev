use crate::buffer::BufferId;
use crate::completion::source::{MAX_BUFFERS, MAX_SNAPSHOTS, Observation, Source};

use super::Session;

impl Session {
    pub(super) fn completion_sources(
        &mut self,
        id: BufferId,
    ) -> Result<(Vec<Source>, bool), String> {
        self.touch_completion_source(id);
        let ids = self.completion_source_ids(id)?;
        let mut sources = Vec::new();
        let mut bytes = 0;
        let mut limited = false;
        for id in ids {
            let buffer = self.buffers.get(id).map_err(|e| e.to_string())?;
            if sources.len() == MAX_BUFFERS
                || bytes + buffer.len_bytes() > MAX_SNAPSHOTS
                || buffer.len_bytes() > crate::completion::MAX_BYTES
            {
                limited = true;
                continue;
            }
            let observation = Observation::capture(buffer);
            let buffer = self.buffers.get_mut(id).map_err(|e| e.to_string())?;
            if let Ok((text, syntax)) = buffer.completion_snapshot() {
                bytes += text.len_bytes();
                sources.push(Source {
                    observation,
                    text,
                    syntax,
                });
            }
        }
        if sources
            .first()
            .is_none_or(|source| source.observation.stamp.buffer != id)
        {
            self.restore_completion_sources(sources);
            return Err("Native completion syntax is unavailable".into());
        }
        Ok((sources, limited))
    }

    fn completion_source_ids(&self, current: BufferId) -> Result<Vec<BufferId>, String> {
        let buffer = self.buffers.get(current).map_err(|e| e.to_string())?;
        let language = buffer.syntax_language();
        let grammar = buffer.completion_grammar();
        let mut ids: Vec<_> = self
            .buffers
            .iter()
            .filter(|buffer| {
                buffer.completion_supported()
                    && buffer.syntax_language() == language
                    && buffer.completion_grammar() == grammar
            })
            .map(|buffer| buffer.id())
            .collect();
        // New sources precede older observations. Within the retained recent
        // inventory, actual source use determines eviction order.
        ids.sort_by_key(|id| {
            (
                *id != current,
                self.completion_recent
                    .iter()
                    .position(|recent| recent == id)
                    .unwrap_or(MAX_BUFFERS),
                std::cmp::Reverse(*id),
            )
        });
        Ok(ids)
    }

    pub(in crate::session) fn touch_completion_source(&mut self, id: BufferId) {
        self.completion_recent
            .retain(|prior| *prior != id && self.buffers.get(*prior).is_ok());
        self.completion_recent.insert(0, id);
        self.completion_recent.truncate(MAX_BUFFERS);
    }

    pub(super) fn restore_completion_sources(&mut self, sources: Vec<Source>) {
        for source in sources.into_iter().rev() {
            self.touch_completion_source(source.observation.stamp.buffer);
            if let Ok(buffer) = self.buffers.get_mut(source.observation.stamp.buffer) {
                buffer.restore_completion_syntax(source.syntax);
            }
        }
    }

    pub(super) fn abandon_completion_sources(&mut self, ids: Vec<BufferId>) {
        for id in ids {
            if let Ok(buffer) = self.buffers.get_mut(id) {
                buffer.abandon_completion_syntax();
            }
        }
    }
}
