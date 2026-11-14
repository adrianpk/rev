use crate::command::Command;

use super::Palette;

impl Palette {
    // Only query changes invalidate matching and usage order. Navigation and
    // painting reuse this snapshot instead of sorting the catalog per row.
    pub(super) fn refresh_candidates(&mut self) {
        let query = self.query.as_deref().unwrap_or(self.prompt.text());
        let mut candidates: Vec<_> = Command::ALL
            .into_iter()
            .chain(self.ai_commands.iter().copied())
            .filter(|command| command.name().contains(query))
            .collect();
        if self.policy.r#match == crate::config::Match::Prefix {
            candidates.retain(|command| command.name().starts_with(query));
        }
        let rank = |command: &Command| {
            if command.name() == query {
                0
            } else if self.policy.r#match == crate::config::Match::PrefixThenSubstring
                && !command.name().starts_with(query)
            {
                2
            } else {
                1
            }
        };
        candidates.sort_by_key(rank);
        for group in candidates.chunk_by_mut(|a, b| rank(a) == rank(b)) {
            self.usage.rank(group);
        }
        self.candidates = candidates;
    }
}
