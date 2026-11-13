use super::{Buffer, Stamp};
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

    pub(crate) fn markdown_admitted(&self) -> bool {
        self.is_markdown()
            && self.len_bytes() <= MAX_BYTES
            && (self.syntax_policy.max_bytes == 0
                || self.len_bytes() <= self.syntax_policy.max_bytes)
    }

    pub(crate) fn request_markdown(&mut self) {
        if self.markdown.observation.is_none() {
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
            self.markdown.observation = Some(observation);
        }
    }

    pub(crate) fn has_markdown_colors(&self) -> bool {
        self.markdown_admitted()
            && self.syntax_policy.enabled
            && self
                .markdown
                .observation
                .as_ref()
                .is_some_and(|value| value.stamp == self.stamp())
    }

    pub(crate) fn markdown_captures(
        &self,
        bytes: std::ops::Range<usize>,
    ) -> Vec<crate::syntax::Capture> {
        if !self.has_markdown_colors() {
            return Vec::new();
        }
        self.markdown
            .observation
            .as_ref()
            .map_or_else(Vec::new, |value| value.captures(bytes))
    }
}

impl Buffer {
    pub(crate) fn markdown_heading_prefix(
        &self,
        row: usize,
    ) -> Option<(u8, std::ops::Range<usize>)> {
        if !self.markdown_admitted() || row >= self.len_lines() {
            return None;
        }
        let observation = self.markdown.observation.as_ref()?;
        if observation.stamp != self.stamp() {
            return None;
        }
        let start = self.text.line_to_byte(row);
        let end = start + self.text.line(row).len_bytes();
        observation.heading_prefix(start..end)
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

    pub(crate) fn markdown_items(
        &self,
        row: usize,
    ) -> impl Iterator<Item = &crate::markdown::Item> {
        let line = (row < self.len_lines()).then(|| {
            let start = self.text.line_to_byte(row);
            start..start + self.text.line(row).len_bytes()
        });
        self.markdown_observation()
            .into_iter()
            .flat_map(|value| value.items())
            .filter(move |item| {
                line.as_ref()
                    .is_some_and(|line| line.contains(&item.marker.start))
            })
    }
}
