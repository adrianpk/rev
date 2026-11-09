use crate::command::{PluginCommand, PluginName};

use super::{Action, Pending, Session};

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
        if self.buffers.snapshot().plugins().package(id).is_none() {
            return self.error("Plugin package is not declared");
        }
        if command == PluginCommand::Deactivate {
            return self.message(&format!("Plugin {id} is inactive; no process is running"));
        }
        self.error(&format!(
            "Plugin {id} is installed; activation is unavailable"
        ))
    }

    pub(super) fn plugin_defined(&mut self, name: PluginName) -> Action {
        let state = if self
            .buffers
            .snapshot()
            .plugins()
            .command(name.name())
            .is_some()
        {
            "installed; activation unavailable"
        } else {
            "not declared"
        };
        self.error(&format!(
            "Plugin command unavailable ({state}): {}",
            name.name()
        ))
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
            text.push_str(&format!(
                "{} {}: installed, inactive (activation unavailable)\n",
                manifest.plugin.id,
                manifest.plugin.version.as_str()
            ));
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
