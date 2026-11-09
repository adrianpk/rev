use crate::command::{PluginCommand, PluginName};

use super::{Action, Pending, Session};
use crate::plugins::lifecycle::{Host, Inspection, Service};
use rev_plugin_sdk::identity::{Revision, ScalarRange, Stamp};
mod activation;
mod buffers;
mod files;
mod invocation;
mod transactions;

#[derive(Default)]
pub(super) struct Work {
    pub(super) host: Host,
    activation: activation::Activation,
    notice: Option<String>,
}

impl Session {
    pub(super) fn plugin_command(&mut self, command: PluginCommand) -> Action {
        if command == PluginCommand::List {
            return self.plugin_list();
        }
        if !self.buffers.snapshot().settings().plugins.enabled {
            return self.error("Experimental plugins are disabled");
        }
        let names: Vec<_> = self
            .buffers
            .snapshot()
            .plugins()
            .packages()
            .map(|p| p.manifest.plugin.id.to_string())
            .collect();
        if names.is_empty() {
            return self.error("No experimental plugin packages are configured");
        }
        if let Err(error) = self.input.names_argument("Plugin package: ", "", names) {
            return self.error(&error.to_string());
        }
        self.pending = Some(Pending::Plugin(command));
        self.message("Choose a qualified package ID; C-g cancels")
    }

    pub(super) fn plugin_argument(&mut self, command: PluginCommand, id: &str) -> Action {
        let Some(package) = self.buffers.snapshot().plugins().package(id) else {
            return self.error("Plugin package is not declared");
        };
        let result = match command {
            PluginCommand::Activate => self.plugins.host.activate(package),
            PluginCommand::Restart => self.plugins.host.restart(package),
            PluginCommand::Deactivate => self
                .plugins
                .host
                .deactivate(&package.manifest.plugin.id, false),
            PluginCommand::List => return self.plugin_list(),
        };
        match result {
            Ok(()) => self.message(&format!("Plugin {id}: {} requested", command.name())),
            Err(error) => self.error(&error),
        }
    }

    pub(super) fn plugin_defined(&mut self, name: PluginName) -> Action {
        let Some((package, command)) = self.buffers.snapshot().plugins().command(name.name())
        else {
            return self.error(&format!(
                "Plugin command unavailable (not declared): {}",
                name.name()
            ));
        };
        if let Err(error) = command.declaration().validate_arguments(
            &serde_json::json!({}),
            None,
            &rev_plugin_sdk::limits::Limits::default(),
        ) {
            return self.error(&error.to_string());
        }
        let mut context = self.plugins.host.context();
        if let Ok(buffer) = self.buffer()
            && !buffer.is_transient()
            && buffer.terminal().is_none()
        {
            context.stamp = Some(Stamp {
                buffer: self.plugins.host.handle(format!("buffer-{}", buffer.id())),
                revision: Revision::new(buffer.stamp().revision.to_string())
                    .expect("native revision"),
            });
            context.pane = Some(
                self.plugins
                    .host
                    .handle(format!("pane-{}", self.windows.selected_id())),
            );
            context.point = Some(self.view().point.position.0 as u64);
            let selection = match buffer.region(self.view().point) {
                Ok(selection) => selection,
                Err(error) => return self.error(&error.to_string()),
            };
            context.selection = selection.map(|s| {
                ScalarRange::new(s.range.start as u64, s.range.end as u64).expect("native region")
            });
        }
        let id = package.manifest.plugin.id.command(&command.name);
        match self.plugins.host.invoke(package, id, context) {
            Ok(()) => self.message(&format!(
                "Plugin command pending: {}; C-g cancels",
                name.name()
            )),
            Err(error) => self.error(&error),
        }
    }

    /// Ephemeral package status. This performs no process or filesystem probe.
    pub fn plugin_status(&self, id: &str) -> Option<Inspection> {
        let package = self.buffers.snapshot().plugins().package(id)?;
        Some(self.plugins.host.inspect(&package.manifest.plugin.id))
    }

    /// Retire every generation and collect bounded receipts outside input/render.
    /// `run_terminal` calls this after restoring the host terminal. Library hosts
    /// call it before discarding their fixture resources or ending their session.
    pub fn shutdown_plugins(&mut self) -> std::io::Result<()> {
        self.plugins.host.shutdown()
    }

    pub(super) fn plugins_remaining(&self) -> Option<std::time::Duration> {
        self.plugins
            .host
            .remaining()
            .or_else(|| self.plugins.activation.remaining())
            .or_else(|| {
                (self.plugins.notice.is_some()
                    && self.input.focus() == crate::input::Focus::Editing
                    && self.pending.is_none())
                .then_some(std::time::Duration::ZERO)
            })
    }

    pub(super) fn expire_plugins(&mut self) -> Action {
        let mut service = Service::new();
        let mut notices = self.plugins.host.service(&mut service);
        self.serve_plugin_buffers(&mut service);
        self.advance_plugin_activation(&mut service, &mut notices);
        if let Some(notice) = notices.pop() {
            self.plugins.notice = Some(notice);
        }
        if self.input.focus() == crate::input::Focus::Editing
            && self.pending.is_none()
            && let Some(notice) = self.plugins.notice.take()
        {
            return self.message(&notice);
        }
        Action::Wait
    }

    fn plugin_list(&mut self) -> Action {
        let snapshot = self.buffers.snapshot();
        let settings = &snapshot.settings().plugins;
        let mut text = format!(
            "Experimental plugins: {}\nAPI: {}\n\n",
            if settings.enabled {
                "enabled"
            } else {
                "disabled"
            },
            rev_plugin_sdk::API
        );
        for package in snapshot.plugins().packages() {
            let manifest = &package.manifest;
            let status = self.plugins.host.inspect(&manifest.plugin.id);
            let state = if status.state == crate::plugins::lifecycle::State::Installed {
                "inactive (not started)"
            } else {
                status.state.name()
            };
            text.push_str(&format!(
                "{} {}: installed, {}\n",
                manifest.plugin.id,
                manifest.plugin.version.as_str(),
                state,
            ));
            if let Some(generation) = status.generation {
                text.push_str(&format!(
                    "  Generation {generation}; {} pending request(s)\n",
                    status.pending
                ));
            }
            for (label, value) in [
                ("Diagnostic", status.diagnostic),
                ("Cleanup", status.cleanup),
                ("Stderr", status.stderr),
            ] {
                if !value.is_empty() {
                    text.push_str(&format!("  {label}: {value}\n"));
                }
            }
            for command in manifest.commands.iter() {
                text.push_str(&format!(
                    "  {}: {}\n",
                    manifest.plugin.id.command(&command.name),
                    command.description.as_str()
                ));
            }
            for binding in manifest.bindings.iter() {
                text.push_str(&format!(
                    "  Default binding {} -> {} ({:?}, {:?}; declared, ineffective)\n",
                    binding.keys.as_str(),
                    binding.command,
                    binding.context,
                    binding.scope
                ));
            }
        }
        for binding in &settings.keybindings {
            text.push_str(&format!(
                "User binding {} -> {} ({:?}, {:?}; declared, ineffective)\n",
                binding.keys.as_str(),
                binding.command,
                binding.context,
                binding.scope
            ));
        }
        if settings.enabled && snapshot.plugins().packages().next().is_none() {
            text.push_str("No packages configured.\n");
        }
        // The catalog is finite; keep its display snapshot smaller than the
        // maximum admitted declarations without truncating a UTF-8 scalar.
        if text.len() > 65536 {
            let mut end = 65000;
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            text.truncate(end);
            text.push_str("\nAdditional output omitted (64 KiB display limit).\n");
        }
        match self.buffers.result("*Plugins*", &text) {
            Ok(id) => {
                self.activate(id);
                self.message("Read-only plugin inventory; switch-to-buffer returns to your source")
            }
            Err(error) => self.error(&error.to_string()),
        }
    }
}
