mod display;

use super::{Buffer, Buffers, Stamp};
use crate::markdown::{MAX_BYTES, Observation, Work};

pub(super) struct State {
    pub(super) active: bool,
    pub(super) enabled: bool,
    generation: u64,
    pub(super) observation: Option<Observation>,
    attempt: Option<Stamp>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            active: false,
            enabled: true,
            generation: 0,
            observation: None,
            attempt: None,
        }
    }
}

impl Buffer {
    pub fn is_markdown(&self) -> bool {
        self.markdown.active
    }

    pub(super) fn activate_markdown(&mut self, active: bool) {
        if self.markdown.active != active {
            self.markdown.active = active;
            self.reset_markdown();
        }
    }

    pub(super) fn reset_markdown(&mut self) {
        self.presentation = None;
        self.markdown.observation = None;
        self.markdown.attempt = None;
        self.markdown.generation = self
            .markdown
            .generation
            .checked_add(1)
            .expect("Markdown observation generation exhausted");
    }

    pub(crate) fn markdown_generation(&self) -> u64 {
        self.markdown.generation
    }

    pub(crate) fn markdown_byte_limit(&self) -> usize {
        if self.syntax_policy.max_bytes == 0 {
            MAX_BYTES
        } else {
            self.syntax_policy.max_bytes.min(MAX_BYTES)
        }
    }

    pub(crate) fn markdown_admitted(&self) -> bool {
        self.is_markdown() && self.len_bytes() <= self.markdown_byte_limit()
    }

    pub(crate) fn request_markdown(&mut self) {
        if self.markdown_observation().is_none() {
            self.markdown.attempt = None;
        }
    }

    pub(crate) fn markdown_work(&mut self) -> Option<Work> {
        if !self.markdown_admitted() || self.markdown.attempt == Some(self.stamp()) {
            return None;
        }
        self.markdown.attempt = Some(self.stamp());
        Some(Work::new(self.stamp(), self.text.clone()))
    }

    pub(crate) fn install_markdown(&mut self, observation: Observation) {
        if self.markdown_admitted() && observation.stamp == self.stamp() {
            self.presentation = None;
            self.markdown.observation = Some(observation);
            self.markdown.attempt = Some(self.stamp());
        }
    }

    pub(crate) fn has_markdown_colors(&self) -> bool {
        self.markdown_admitted() && self.syntax_policy.enabled && self.has_markdown_display()
    }

    pub(crate) fn markdown_captures(
        &self,
        bytes: std::ops::Range<usize>,
    ) -> Vec<crate::syntax::Capture> {
        if !self.has_markdown_colors() {
            return Vec::new();
        }
        if let Some(observation) = self.markdown_observation() {
            observation.captures(bytes)
        } else {
            // No inline context is re-proven by structural carry. Suppress all
            // old lexical captures, including cross-row parts, until exact.
            self.markdown_structural_captures(bytes)
        }
    }
}

impl Buffer {
    pub(crate) fn markdown_observation(&self) -> Option<&Observation> {
        if !self.markdown_admitted() {
            return None;
        }
        self.markdown
            .observation
            .as_ref()
            .filter(|value| value.stamp == self.stamp())
    }

    pub(crate) fn retire_markdown_presentation(&mut self) {
        self.markdown.observation = None;
        self.presentation = None;
        // Preserve a newer explicit retry request while an old canceled worker
        // is being reaped. The work owner already recorded its own attempt.
    }
}

impl Buffers {
    pub(crate) fn retire_markdown_presentations(&mut self) {
        for buffer in self
            .entries
            .values_mut()
            .filter(|buffer| buffer.is_markdown())
        {
            if buffer.presentation.is_some() {
                buffer.retire_markdown_presentation();
                buffer.markdown.attempt = Some(buffer.stamp());
            }
        }
    }
}
