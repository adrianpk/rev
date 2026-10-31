use super::{PartialLsp, PartialServer};

impl super::merge::Merge {
    pub(super) fn lsp(&mut self, base: &mut PartialLsp, later: &PartialLsp) {
        self.leaf(&["lsp", "enabled"], &mut base.enabled, &later.enabled);
        self.leaf(
            &["lsp", "shutdown_on_last_buffer"],
            &mut base.shutdown_on_last_buffer,
            &later.shutdown_on_last_buffer,
        );
        self.leaf(
            &["lsp", "request_timeout_ms"],
            &mut base.request_timeout_ms,
            &later.request_timeout_ms,
        );
        self.leaf(
            &["lsp", "initialize_timeout_ms"],
            &mut base.initialize_timeout_ms,
            &later.initialize_timeout_ms,
        );
        self.leaf(
            &["lsp", "shutdown_timeout_ms"],
            &mut base.shutdown_timeout_ms,
            &later.shutdown_timeout_ms,
        );
        self.leaf(
            &["lsp", "max_message_bytes"],
            &mut base.max_message_bytes,
            &later.max_message_bytes,
        );
        self.leaf(
            &["lsp", "max_servers"],
            &mut base.max_servers,
            &later.max_servers,
        );
        self.leaf(
            &["lsp", "max_pending_requests"],
            &mut base.max_pending_requests,
            &later.max_pending_requests,
        );
        for (name, server) in &later.servers {
            self.server(name, base.servers.entry(name.clone()).or_default(), server);
        }
    }

    fn server(&mut self, name: &str, base: &mut PartialServer, later: &PartialServer) {
        self.leaf(
            &["lsp", "servers", name, "languages"],
            &mut base.languages,
            &later.languages,
        );
        self.leaf(
            &["lsp", "servers", name, "command"],
            &mut base.command,
            &later.command,
        );
        self.leaf(
            &["lsp", "servers", name, "args"],
            &mut base.args,
            &later.args,
        );
        self.leaf(
            &["lsp", "servers", name, "root_markers"],
            &mut base.root_markers,
            &later.root_markers,
        );
        self.leaf(
            &["lsp", "servers", name, "initialization_options"],
            &mut base.initialization_options,
            &later.initialization_options,
        );
        self.leaf(
            &["lsp", "servers", name, "settings"],
            &mut base.settings,
            &later.settings,
        );
    }
}
