use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyModifiers};

use super::{Input, Key, Outcome, Prefix};

/// Deadline for a separately reported Escape to modify its next plain chord.
pub const META_TIMEOUT: Duration = Duration::from_millis(100);

impl Input {
    /// Finite poll duration while Escape is pending; no timer is needed otherwise.
    pub fn remaining(&self, now: Instant) -> Option<Duration> {
        self.meta_deadline
            .map(|deadline| deadline.saturating_duration_since(now))
    }

    /// Expire once without leaving the focused prompt/decision. C-g is the
    /// explicit interaction cancellation command; standalone Escape clears chords.
    pub fn expire(&mut self, now: Instant) -> Outcome {
        if self.meta_deadline.is_some_and(|deadline| now >= deadline) {
            self.clear_prefix();
            self.suppress_repeat = true;
            return Outcome::Message("Escape");
        }
        Outcome::Ignored
    }

    pub(super) fn meta_key(&mut self, mut key: Key, now: Instant) -> Outcome {
        if key.plain() == Some(KeyCode::Esc) {
            return self.escape(now);
        }
        if self.meta_deadline.take().is_some() && key.modifiers.is_empty() {
            self.suppress_repeat = true;
            key.modifiers = KeyModifiers::ALT;
        }
        self.route(key)
    }

    fn escape(&mut self, now: Instant) -> Outcome {
        if self.meta_deadline.take().is_some() {
            self.clear_prefix();
            self.suppress_repeat = true;
            return Outcome::Message("Escape");
        }
        self.meta_deadline = now.checked_add(META_TIMEOUT);
        Outcome::Prefix(Prefix::Meta)
    }
}
