use super::Client;
use crate::buffer::Buffers;
use crate::config::Lsp;

impl Client {
    /// Retire changed sources before publishing new settings. The runtime mode
    /// and excluded buffer IDs remain owned by this client throughout reload.
    pub(crate) fn apply_settings(&mut self, mut policy: Lsp, buffers: &Buffers) {
        policy.enabled = self.policy.enabled;
        let mut old_limits = self.policy.clone();
        old_limits.servers = policy.servers.clone();
        old_limits.preferred = policy.preferred.clone();
        let limits_changed = old_limits != policy;
        let changed: Vec<_> = self
            .attachments
            .iter()
            .filter_map(|(id, attachment)| {
                let new_language = buffers.get(*id).ok()?.syntax_language();
                let selected = policy.selected(new_language).map(|(name, _)| name);
                (limits_changed
                    || new_language != attachment.language
                    || selected != Some(&attachment.key.0)
                    || policy.servers.get(&attachment.key.0)
                        != self.policy.servers.get(&attachment.key.0))
                .then_some(*id)
            })
            .collect();
        for id in changed {
            self.detach(id);
        }
        for ((name, _), connection) in &mut self.connections {
            let selected = policy.servers.get(name).is_some_and(|server| {
                server
                    .languages
                    .iter()
                    .any(|language| policy.selected(language).is_some_and(|(id, _)| id == name))
            });
            if limits_changed
                || !selected
                || policy.servers.get(name) != self.policy.servers.get(name)
            {
                connection.retiring = true;
                connection.inbox.stop();
            }
        }
        self.policy = policy;
        self.selected = None;
        self.fallback = super::Status::Unavailable;
        self.collect();
    }
}
