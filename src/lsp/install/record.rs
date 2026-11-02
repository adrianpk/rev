use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;
use toml_edit::{ArrayOfTables, DocumentMut, Item, Table, Value, value};

use crate::config::Server;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    Preparing,
    Ready,
    Incomplete,
}

/// Installation facts only. User choices remain in the admitted settings graph.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Installation {
    pub id: String,
    pub recipe: String,
    pub version: String,
    pub target: String,
    pub payload: PathBuf,
    pub source: String,
    pub evidence: String,
    pub runtime: String,
    pub launches: Vec<Launch>,
    pub status: State,
    pub created_at: u64,
    pub verified_at: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Launch {
    pub id: String,
    pub languages: Vec<String>,
    pub executable: PathBuf,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub root_markers: Vec<String>,
    pub initialization_options: toml::Value,
    pub settings: toml::Value,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Registry {
    pub schema: u32,
    pub installations: Vec<Installation>,
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            schema: 1,
            installations: Vec::new(),
        }
    }
}

impl Registry {
    /// Invalid facts block management, without acquiring or changing anything.
    pub fn load(data: &Path) -> Result<Self, String> {
        super::storage::read(data).map(|(registry, _)| registry)
    }

    pub(super) fn validate(&self, data: &Path) -> Result<(), String> {
        if self.schema != 1 || self.installations.len() > super::RECORDS {
            return Err("installation registry requires schema 1 and at most 32 records".into());
        }
        let mut ids = BTreeSet::new();
        for installation in &self.installations {
            if !ids.insert(&installation.id) {
                return Err("installation registry contains a duplicate ID".into());
            }
            installation.validate(data)?;
        }
        Ok(())
    }

    pub(super) fn encode(&self) -> String {
        let mut document = DocumentMut::new();
        document["schema"] = value(1);
        let mut records = ArrayOfTables::new();
        for installation in &self.installations {
            records.push(installation.table());
        }
        document["installations"] = if self.installations.is_empty() {
            value(toml_edit::Array::new())
        } else {
            Item::ArrayOfTables(records)
        };
        document.to_string()
    }
}

impl Installation {
    fn validate(&self, data: &Path) -> Result<(), String> {
        self.location()?;
        self.facts()?;
        self.verification()?;
        self.profiles(data)?;
        super::storage::contained(data, &data.join(&self.payload), self.status == State::Ready)
    }

    fn location(&self) -> Result<(), String> {
        if !identity(&self.id) || !identity(&self.recipe) {
            return Err("installation ID and recipe must be bounded path components".into());
        }
        let expected = Path::new("lsp/installations")
            .join(&self.recipe)
            .join(&self.id);
        if self.payload != expected || !relative(&self.payload) {
            return Err("installation payload must be its owned relative versioned path".into());
        }
        Ok(())
    }

    fn facts(&self) -> Result<(), String> {
        for text in [&self.version, &self.target, &self.source, &self.runtime] {
            if text.is_empty() || text.len() > 4096 || text.contains('\0') {
                return Err("installation facts contain invalid bounded strings".into());
            }
        }
        if self.created_at > i64::MAX as u64
            || self.verified_at.is_some_and(|t| t > i64::MAX as u64)
        {
            return Err("installation timestamp exceeds its signed persistence range".into());
        }
        Ok(())
    }

    fn profiles(&self, data: &Path) -> Result<(), String> {
        if self.launches.len() > 32 || (self.status == State::Ready && self.launches.is_empty()) {
            return Err("installation requires 1..=32 verified launch profiles when ready".into());
        }
        let mut ids = BTreeSet::new();
        for launch in &self.launches {
            if !ids.insert(&launch.id) {
                return Err("installation contains duplicate launch IDs".into());
            }
            launch.validate(&data.join(&self.payload), self.status == State::Ready)?;
        }
        Ok(())
    }

    fn verification(&self) -> Result<(), String> {
        if self.evidence.len() > 65536 {
            return Err("installation verification evidence exceeds 64 KiB".into());
        }
        if self.status == State::Ready {
            if !super::evidence::valid(&self.recipe, &self.evidence) {
                return Err(
                    "ready installation lacks admitted integrity or package evidence".into(),
                );
            }
            if self.verified_at.is_none_or(|time| time < self.created_at) {
                return Err("ready installation lacks a valid verification timestamp".into());
            }
        }
        Ok(())
    }

    fn table(&self) -> Table {
        let mut table = Table::new();
        table["id"] = value(&self.id);
        table["recipe"] = value(&self.recipe);
        table["version"] = value(&self.version);
        table["target"] = value(&self.target);
        table["payload"] = value(self.payload.to_string_lossy().as_ref());
        table["source"] = value(&self.source);
        table["evidence"] = value(&self.evidence);
        table["runtime"] = value(&self.runtime);
        table["launches"] = value(
            self.launches
                .iter()
                .map(|launch| Value::InlineTable(launch.table().into_inline_table()))
                .collect::<toml_edit::Array>(),
        );
        table["status"] = value(match self.status {
            State::Preparing => "preparing",
            State::Ready => "ready",
            State::Incomplete => "incomplete",
        });
        table["created_at"] = value(self.created_at as i64);
        if let Some(time) = self.verified_at {
            table["verified_at"] = value(time as i64);
        }
        table
    }
}

impl Launch {
    pub fn server(&self, payload: &Path) -> Server {
        Server {
            languages: self.languages.clone(),
            command: payload
                .join(&self.executable)
                .to_string_lossy()
                .into_owned(),
            args: self.args.clone(),
            root_markers: self.root_markers.clone(),
            initialization_options: self.initialization_options.clone(),
            settings: self.settings.clone(),
            env: self.env.clone(),
        }
    }

    fn validate(&self, payload: &Path, ready: bool) -> Result<(), String> {
        if !identity(&self.id) || !relative(&self.executable) {
            return Err("installation launch requires a bounded ID and relative executable".into());
        }
        crate::config::lsp_validation::profile(&self.id, &self.server(payload))?;
        if ready {
            super::storage::contained(payload, &payload.join(&self.executable), true)?;
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(payload.join(&self.executable))
                .map_err(|e| format!("ready executable: {e}"))?;
            if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
                return Err("ready installation executable is unavailable".into());
            }
        }
        Ok(())
    }

    fn table(&self) -> Table {
        let mut table = Table::new();
        table["id"] = value(&self.id);
        table["languages"] = strings(&self.languages);
        table["executable"] = value(self.executable.to_string_lossy().as_ref());
        table["args"] = strings(&self.args);
        table["env"] = Item::Value(Value::InlineTable(
            self.env
                .iter()
                .map(|(k, v)| (k.clone(), Value::from(v.as_str())))
                .collect(),
        ));
        table["root_markers"] = strings(&self.root_markers);
        table["initialization_options"] = Item::Value(data(&self.initialization_options));
        table["settings"] = Item::Value(data(&self.settings));
        table
    }
}

pub(super) fn identity(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= 128
        && text
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
}

fn relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}

fn strings(values: &[String]) -> Item {
    Item::Value(Value::Array(
        values.iter().map(|s| Value::from(s.as_str())).collect(),
    ))
}

fn data(input: &toml::Value) -> Value {
    match input {
        toml::Value::String(v) => Value::from(v.as_str()),
        toml::Value::Integer(v) => Value::from(*v),
        toml::Value::Float(v) => Value::from(*v),
        toml::Value::Boolean(v) => Value::from(*v),
        toml::Value::Datetime(v) => Value::from(v.to_string()),
        toml::Value::Array(v) => Value::Array(v.iter().map(data).collect()),
        toml::Value::Table(v) => {
            Value::InlineTable(v.iter().map(|(k, v)| (k.clone(), data(v))).collect())
        }
    }
}
