use std::collections::BTreeMap;

use serde::Deserialize;

/// Optional owned stdio connections. Limits are admitted before process creation.
#[derive(Clone, Debug, PartialEq)]
pub struct Lsp {
    pub enabled: bool,
    pub shutdown_on_last_buffer: bool,
    pub request_timeout_ms: usize,
    pub initialize_timeout_ms: usize,
    pub shutdown_timeout_ms: usize,
    pub max_message_bytes: usize,
    pub max_servers: usize,
    pub max_pending_requests: usize,
    pub servers: BTreeMap<String, Server>,
    pub preferred: BTreeMap<String, Preference>,
}

impl Default for Lsp {
    fn default() -> Self {
        let rust = Server {
            languages: vec!["rust".into()],
            command: "rust-analyzer".into(),
            args: Vec::new(),
            root_markers: vec!["Cargo.toml".into()],
            initialization_options: table(),
            settings: table(),
            env: BTreeMap::new(),
        };
        Self {
            enabled: false,
            shutdown_on_last_buffer: false,
            request_timeout_ms: 2000,
            initialize_timeout_ms: 10000,
            shutdown_timeout_ms: 2000,
            max_message_bytes: 8_388_608,
            max_servers: 8,
            max_pending_requests: 16,
            servers: BTreeMap::from([("rust".into(), rust)]),
            preferred: BTreeMap::new(),
        }
    }
}

fn table() -> toml::Value {
    toml::Value::Table(toml::Table::new())
}

/// An admitted server is selected independently of the grammar catalog.
#[derive(Clone, Debug, PartialEq)]
pub struct Server {
    pub languages: Vec<String>,
    pub command: String,
    pub args: Vec<String>,
    pub root_markers: Vec<String>,
    pub initialization_options: toml::Value,
    pub settings: toml::Value,
    pub env: BTreeMap<String, String>,
}

/// A server name or an explicit native-only choice. `true` is never admitted.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum Preference {
    Server(String),
    Native(bool),
}

impl Lsp {
    /// Missing preferences preserve lexical server selection, without a grammar lookup.
    pub fn selected(&self, language: &str) -> Option<(&String, &Server)> {
        match self.preferred.get(language) {
            Some(Preference::Native(_)) => None,
            Some(Preference::Server(name)) => self.servers.get_key_value(name),
            None => self
                .servers
                .iter()
                .find(|(_, s)| s.languages.iter().any(|id| id == language)),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialServer {
    pub languages: Option<Vec<String>>,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub root_markers: Option<Vec<String>>,
    pub initialization_options: Option<toml::Value>,
    pub settings: Option<toml::Value>,
    pub env: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialLsp {
    pub enabled: Option<bool>,
    pub shutdown_on_last_buffer: Option<bool>,
    pub request_timeout_ms: Option<i64>,
    pub initialize_timeout_ms: Option<i64>,
    pub shutdown_timeout_ms: Option<i64>,
    pub max_message_bytes: Option<i64>,
    pub max_servers: Option<i64>,
    pub max_pending_requests: Option<i64>,
    pub servers: BTreeMap<String, PartialServer>,
    pub preferred: BTreeMap<String, Preference>,
}

impl PartialLsp {
    pub(super) fn project(&self) -> Lsp {
        let mut result = Lsp::default();
        result.preferred.clone_from(&self.preferred);
        if let Some(value) = self.enabled {
            result.enabled = value;
        }
        if let Some(value) = self.shutdown_on_last_buffer {
            result.shutdown_on_last_buffer = value;
        }
        for (target, source) in [
            (&mut result.request_timeout_ms, self.request_timeout_ms),
            (
                &mut result.initialize_timeout_ms,
                self.initialize_timeout_ms,
            ),
            (&mut result.shutdown_timeout_ms, self.shutdown_timeout_ms),
            (&mut result.max_message_bytes, self.max_message_bytes),
            (&mut result.max_servers, self.max_servers),
            (&mut result.max_pending_requests, self.max_pending_requests),
        ] {
            if let Some(value) = source {
                *target = usize::try_from(value).expect("validated configuration count");
            }
        }
        for (name, partial) in &self.servers {
            let base = result
                .servers
                .entry(name.clone())
                .or_insert_with(|| Server {
                    languages: Vec::new(),
                    command: String::new(),
                    args: Vec::new(),
                    root_markers: Vec::new(),
                    initialization_options: table(),
                    settings: table(),
                    env: BTreeMap::new(),
                });
            partial.project(base);
        }
        result
    }
}

impl PartialServer {
    fn project(&self, result: &mut Server) {
        result.env.extend(self.env.clone());
        if let Some(value) = &self.languages {
            result.languages.clone_from(value);
        }
        if let Some(value) = &self.command {
            result.command.clone_from(value);
        }
        if let Some(value) = &self.args {
            result.args.clone_from(value);
        }
        if let Some(value) = &self.root_markers {
            result.root_markers.clone_from(value);
        }
        if let Some(value) = &self.initialization_options {
            result.initialization_options.clone_from(value);
        }
        if let Some(value) = &self.settings {
            result.settings.clone_from(value);
        }
    }
}
