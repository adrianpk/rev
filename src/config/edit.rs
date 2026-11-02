//! Restricted, format-preserving user-root transactions. No resolved defaults
//! are serialized, and an invalid startup graph never grants write permission.
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use toml_edit::{DocumentMut, Item, Table, Value, value};

use super::{Diagnostic, Preference, Server, Snapshot, Sources, load, resolve};
use crate::file::temporary::Temporary;

/// A captured complete source graph and editable user root. Mutations are limited
/// to server profiles, language preferences and file associations.
pub struct SettingsEdit {
    root: PathBuf,
    sources: Sources,
    document: DocumentMut,
}

/// Publication succeeded even when directory synchronization could not confirm
/// durability. Callers must apply the snapshot and report any warning truthfully.
pub struct Publication {
    pub snapshot: Snapshot,
    pub warning: Option<String>,
}

impl SettingsEdit {
    pub fn capture(root: &Path) -> Result<Self, Diagnostic> {
        let sources = load(root)?;
        resolve(&sources)?;
        let text = sources.sources.first().map_or("", |s| {
            std::str::from_utf8(&s.bytes).expect("validated UTF-8 configuration")
        });
        let document = text
            .parse()
            .map_err(|e| Diagnostic::new(root, format!("edit configuration: {e}")))?;
        Ok(Self {
            root: root.to_owned(),
            sources,
            document,
        })
    }

    pub fn snapshot(&self) -> Result<Snapshot, Diagnostic> {
        resolve(&self.sources).map(|s| s.with_root(&self.root))
    }

    pub fn register(&mut self, name: &str, server: &Server) {
        let table = &mut self.document["lsp"]["servers"][name];
        if table.is_none() {
            *table = Item::Table(Table::new());
        }
        table["languages"] = strings(&server.languages);
        table["command"] = value(&server.command);
        table["args"] = strings(&server.args);
        table["root_markers"] = strings(&server.root_markers);
        table["initialization_options"] = Item::Value(data(&server.initialization_options));
        table["settings"] = Item::Value(data(&server.settings));
        let mut env = toml_edit::InlineTable::new();
        for (key, entry) in &server.env {
            env.insert(key, Value::from(entry.as_str()));
        }
        table["env"] = Item::Value(Value::InlineTable(env));
    }

    pub fn prefer(&mut self, language: &str, preference: &Preference) {
        self.document["lsp"]["preferred"][language] = match preference {
            Preference::Server(name) => value(name),
            Preference::Native(native) => value(*native),
        };
    }

    pub fn associate(&mut self, pattern: &str, language: &str) {
        self.document["files"]["associations"][pattern] = value(language);
    }

    /// Serialize manager mutations with an OS-owned lock in the resolved state
    /// root. A busy instance returns immediately. The root symlink stays intact.
    pub fn publish(self, state: &Path) -> Result<Publication, Diagnostic> {
        self.publish_inner(state)
            .map_err(|error| Diagnostic::new(&self.root, error))
    }

    fn publish_inner(&self, state: &Path) -> Result<Publication, String> {
        let bytes = self.document.to_string().into_bytes();
        let candidate =
            super::load::candidate(&self.root, bytes.clone()).map_err(|e| e.to_string())?;
        let snapshot = resolve(&candidate)
            .map_err(|e| e.to_string())?
            .with_root(&self.root);
        let lock_dir = state.join("lsp");
        fs::create_dir_all(&lock_dir).map_err(|e| format!("create LSP lock directory: {e}"))?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .open(lock_dir.join("mutation.lock"))
            .map_err(|e| format!("open LSP mutation lock: {e}"))?;
        lock.try_lock()
            .map_err(|_| "another LSP management operation owns the mutation lock".to_owned())?;
        self.unchanged()?;
        let target = self
            .sources
            .sources
            .first()
            .map_or(self.root.as_path(), |s| s.canonical_path.as_path());
        let parent = target.parent().ok_or("configuration has no parent")?;
        fs::create_dir_all(parent).map_err(|e| format!("create configuration directory: {e}"))?;
        let mut temporary = Temporary::new(parent).map_err(|e| e.to_string())?;
        if let Ok(metadata) = fs::metadata(target) {
            temporary
                .file
                .set_permissions(metadata.permissions())
                .map_err(|e| e.to_string())?;
        }
        temporary
            .file
            .write_all(&bytes)
            .and_then(|()| temporary.file.flush())
            .and_then(|()| temporary.file.sync_all())
            .map_err(|e| format!("prepare configuration: {e}"))?;
        self.unchanged()?;
        if self.sources.sources.is_empty() {
            fs::hard_link(&temporary.path, target)
                .map_err(|e| format!("publish absent configuration: {e}"))?;
        } else {
            fs::rename(&temporary.path, target)
                .map_err(|e| format!("publish configuration: {e}"))?;
            temporary.disarm();
        }
        let warning = File::open(parent)
            .and_then(|directory| directory.sync_all())
            .err()
            .map(|e| format!("configuration published; directory durability unconfirmed: {e}"));
        Ok(Publication { snapshot, warning })
    }

    fn unchanged(&self) -> Result<(), String> {
        let current =
            load(&self.root).map_err(|e| format!("configuration source conflict: {e}"))?;
        if current != self.sources {
            return Err("configuration source conflict; reload settings and retry".into());
        }
        Ok(())
    }
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
