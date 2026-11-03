//! Private binding metadata only. Pi owns JSONL contents and reconstruction.
use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rustix::fs::{Mode, OFlags};
use serde::Deserialize;

pub(super) const SESSION_BYTES: u64 = 32 * 1024 * 1024;
const INDEX_BYTES: u64 = 64 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Index {
    version: u8,
    bindings: Vec<Binding>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    id: String,
    root: PathBuf,
    fingerprint: String,
    model: String,
    session: String,
    created: u64,
    last_used: u64,
}

pub(super) struct Store {
    root: PathBuf,
    directory: File,
    sessions: File,
}

/// Keeps this binding exclusive across editor instances, including idle reuse.
pub(super) struct Lease {
    store: Store,
    id: String,
    pub path: PathBuf,
    _lock: File,
    observation: Option<(u64, u64)>,
}

impl Store {
    pub fn open(state: &Path) -> Result<Self, String> {
        let root = state.join("ai/pi");
        let directory = private_directory(&root)?;
        let root = root
            .canonicalize()
            .map_err(|_| "Pi private root unavailable")?;
        same(&root, &directory)?;
        let sessions = private_directory(&root.join("sessions"))?;
        Ok(Self {
            root,
            directory,
            sessions,
        })
    }
    fn same(&self) -> Result<(), String> {
        same(&self.root, &self.directory)?;
        same(&self.root.join("sessions"), &self.sessions)
    }
    fn at(&self, name: &str) -> PathBuf {
        descriptor(&self.directory).join(name)
    }
    fn lock(&self) -> Result<File, String> {
        self.same()?;
        let file = private_file(&self.at("index.lock"), true)?;
        file.try_lock()
            .map_err(|_| "Pi binding index is busy; retry explicitly".to_string())?;
        same_file(&self.at("index.lock"), &file)?;
        Ok(file)
    }
    pub fn candidate(&self) -> Result<PathBuf, String> {
        self.same()?;
        let id = format!(
            "session-{:x}-{:x}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| "Pi clock unavailable")?
                .as_nanos()
        );
        let path = self.root.join("sessions").join(format!("{id}.jsonl"));
        if fs::symlink_metadata(&path).is_ok() {
            return Err("Pi session identity collision".into());
        }
        Ok(path)
    }
    fn read(&self) -> Result<(Index, Option<Vec<u8>>), String> {
        let file = match private_file(&self.at("sessions.toml"), false) {
            Ok(file) => file,
            Err(_)
                if fs::symlink_metadata(self.at("sessions.toml"))
                    .is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound) =>
            {
                return Ok((
                    Index {
                        version: 1,
                        bindings: Vec::new(),
                    },
                    None,
                ));
            }
            Err(e) => return Err(e),
        };
        if file
            .metadata()
            .map_err(|_| "Pi index metadata unavailable")?
            .len()
            > INDEX_BYTES
        {
            return Err(
                "Pi binding index exceeds 64 KiB; reset/remove history or disable persistence"
                    .into(),
            );
        }
        let mut bytes = Vec::new();
        file.take(INDEX_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "Pi binding index read failed")?;
        if bytes.len() as u64 > INDEX_BYTES {
            return Err("Pi binding index exceeds 64 KiB".into());
        }
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| "Pi binding index is not UTF-8; preserve it and use ephemeral context")?;
        let index: Index = toml::from_str(text)
            .map_err(|_| "Pi binding index is invalid; preserve it and disable persistence")?;
        validate(&index)?;
        Ok((index, Some(bytes)))
    }
    fn publish(&self, index: &Index, previous: Option<&[u8]>, lock: &File) -> Result<(), String> {
        validate(index)?;
        same_file(&self.at("index.lock"), lock)?;
        let mut document = toml_edit::DocumentMut::new();
        document["version"] = toml_edit::value(1);
        let mut tables = toml_edit::ArrayOfTables::new();
        for b in &index.bindings {
            let mut table = toml_edit::Table::new();
            for (name, value) in [
                ("id", b.id.as_str()),
                ("root", b.root.to_str().ok_or("Pi root is not UTF-8")?),
                ("fingerprint", b.fingerprint.as_str()),
                ("model", b.model.as_str()),
                ("session", b.session.as_str()),
            ] {
                table[name] = toml_edit::value(value);
            }
            table["created"] = toml_edit::value(
                i64::try_from(b.created).map_err(|_| "Pi creation timestamp exceeds range")?,
            );
            table["last_used"] = toml_edit::value(
                i64::try_from(b.last_used).map_err(|_| "Pi last-used timestamp exceeds range")?,
            );
            tables.push(table);
        }
        document["bindings"] = toml_edit::Item::ArrayOfTables(tables);
        let text = document.to_string();
        if text.len() as u64 > INDEX_BYTES {
            return Err(
                "Pi binding index exceeds 64 KiB; reset/remove history or disable persistence"
                    .into(),
            );
        }
        self.same()?;
        let (_, observed) = self.read()?;
        if observed.as_deref() != previous {
            return Err("Pi binding index changed; preserve it and retry".into());
        }
        let mut temporary = crate::file::temporary::Temporary::new(&descriptor(&self.directory))
            .map_err(|_| "Pi index temporary creation failed")?;
        temporary
            .file
            .write_all(text.as_bytes())
            .and_then(|_| temporary.file.sync_all())
            .map_err(|_| "Pi index write/sync failed")?;
        self.same()?;
        let (_, observed) = self.read()?;
        if observed.as_deref() != previous {
            return Err("Pi binding index changed before publication".into());
        }
        same_file(&self.at("index.lock"), lock)?;
        fs::rename(&temporary.path, self.at("sessions.toml"))
            .map_err(|_| "Pi index publication failed")?;
        temporary.disarm();
        self.directory
            .sync_all()
            .map_err(|_| "Pi index directory sync failed".to_string())
    }
    pub fn bind(
        self,
        root: &Path,
        fingerprint: &str,
        model: &str,
        candidate: &Path,
        reset: bool,
    ) -> Result<Lease, String> {
        let index_lock = self.lock()?;
        let (mut index, previous) = self.read()?;
        let candidate_name = candidate
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("Invalid Pi candidate name")?
            .to_string();
        let found = index
            .bindings
            .iter()
            .position(|b| b.root == root && b.fingerprint == fingerprint && b.model == model);
        let id = found
            .map(|i| index.bindings[i].id.clone())
            .unwrap_or_else(|| candidate_name.trim_end_matches(".jsonl").to_string());
        let lock = private_file(&self.at(&format!("{id}.lock")), true)?;
        lock.try_lock().map_err(|_| {
            "Pi history is active in another editor; disconnect there or disable persistence"
                .to_string()
        })?;
        let mut lease = Lease {
            store: self,
            id,
            path: candidate.to_owned(),
            _lock: lock,
            observation: None,
        };
        let mut retired = None;
        if let Some(i) = found {
            let old = &index.bindings[i];
            lease.path = lease.store.root.join("sessions").join(&old.session);
            lease.check(!reset)?;
            if reset {
                // Publish the replacement binding before retiring the old owned
                // file. A failed index write must preserve the previous history.
                retired = lease.observation.map(|id| (old.session.clone(), id));
                lease.path = candidate.to_owned();
                lease.observation = None;
                index.bindings[i].session = candidate_name;
                index.bindings[i].created = now()?;
                index.bindings[i].last_used = index.bindings[i].created;
            }
        } else {
            if index.bindings.len() == 16 {
                return Err(
                    "Pi retained binding limit is 16; reset/remove history or disable persistence"
                        .into(),
                );
            }
            index.bindings.push(Binding {
                id: lease.id.clone(),
                root: root.to_owned(),
                fingerprint: fingerprint.into(),
                model: model.into(),
                session: candidate_name,
                created: now()?,
                last_used: now()?,
            });
        }
        lease
            .store
            .publish(&index, previous.as_deref(), &index_lock)?;
        if let Some((name, identity)) = retired {
            let path = descriptor(&lease.store.sessions).join(name);
            let file = private_file(&path, false)?;
            let meta = file
                .metadata()
                .map_err(|_| "Pi reset history metadata failed")?;
            if (meta.dev(), meta.ino()) != identity {
                return Err("Pi reset history changed; preserve the retired file".into());
            }
            fs::remove_file(path).map_err(
                |_| "Pi replacement binding published, but retired history cleanup failed",
            )?;
            lease
                .store
                .sessions
                .sync_all()
                .map_err(|_| "Pi history reset directory sync failed")?;
        }
        drop(index_lock);
        Ok(lease)
    }
}

impl Lease {
    /// Only metadata is inspected; no native JSONL parser/transcript is created.
    pub fn check(&mut self, required: bool) -> Result<(), String> {
        self.store.same()?;
        same_file(&self.store.at(&format!("{}.lock", self.id)), &self._lock)?;
        let name = self.path.file_name().ok_or("Pi history name unavailable")?;
        let path = descriptor(&self.store.sessions).join(name);
        let file = match private_file(&path, false) {
            Ok(file) => file,
            Err(error) if !required && self.observation.is_none() => {
                match fs::symlink_metadata(&self.path) {
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
                    // Pi can create its first history between open and stat.
                    // Reopen through the same strict admission; an observed
                    // history never receives this initial-creation exception.
                    Ok(_) => private_file(&path, false)?,
                    Err(_) => return Err(error),
                }
            }
            Err(error) => return Err(error),
        };
        let meta = file.metadata().map_err(|_| "Pi session metadata failed")?;
        if meta.len() >= SESSION_BYTES {
            return Err(
                "Pi session exceeds 32 MiB; reset/remove history or disable persistence".into(),
            );
        }
        let identity = (meta.dev(), meta.ino());
        if self.observation.is_some_and(|old| old != identity) {
            return Err(
                "Pi owned history identity changed; preserve it and disable persistence".into(),
            );
        }
        self.observation = Some(identity);
        if required {
            file.sync_all().map_err(|_| "Pi history sync failed")?;
        }
        Ok(())
    }
    pub fn touch(&mut self) -> Result<(), String> {
        self.check(true)?;
        self.store
            .sessions
            .sync_all()
            .map_err(|_| "Pi history directory sync failed")?;
        let _lock = self.store.lock()?;
        let (mut index, previous) = self.store.read()?;
        let binding = index
            .bindings
            .iter_mut()
            .find(|b| b.id == self.id)
            .ok_or("Pi retained binding disappeared")?;
        if self.store.root.join("sessions").join(&binding.session) != self.path {
            return Err("Pi retained binding changed".into());
        }
        binding.last_used = now()?;
        self.store.publish(&index, previous.as_deref(), &_lock)
    }
}

fn now() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|t| t.as_secs())
        .map_err(|_| "Pi clock is before the epoch".into())
}
fn validate(index: &Index) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let mut keys = BTreeSet::new();
    if index.version != 1 || index.bindings.len() > 16 {
        return Err(
            "Pi binding schema/count is unsupported; preserve index and disable persistence".into(),
        );
    }
    for b in &index.bindings {
        if b.id.is_empty()
            || b.id.len() > 96
            || !b.id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
            || !b.root.is_absolute()
            || b.root.as_os_str().len() > 4096
            || b.fingerprint.is_empty()
            || b.fingerprint.len() > 16384
            || b.model.is_empty()
            || b.model.len() > 256
            || !b.session.ends_with(".jsonl")
            || Path::new(&b.session).components().count() != 1
            || !matches!(
                Path::new(&b.session).components().next(),
                Some(Component::Normal(_))
            )
            || b.last_used < b.created
            || !ids.insert(&b.id)
            || !paths.insert(&b.session)
            || !keys.insert((&b.root, &b.fingerprint, &b.model))
        {
            return Err(
                "Pi binding record is invalid; preserve index and disable persistence".into(),
            );
        }
    }
    Ok(())
}
fn descriptor(file: &File) -> PathBuf {
    use std::os::fd::AsRawFd;
    PathBuf::from(format!("/proc/self/fd/{}", file.as_raw_fd()))
}
fn same(path: &Path, held: &File) -> Result<(), String> {
    let named = fs::symlink_metadata(path).map_err(|_| "Pi private directory disappeared")?;
    let meta = held.metadata().map_err(|_| "Pi directory handle failed")?;
    if !named.is_dir()
        || named.uid() != rustix::process::geteuid().as_raw()
        || named.mode() & 0o077 != 0
        || named.dev() != meta.dev()
        || named.ino() != meta.ino()
    {
        return Err("Pi private directory identity/ownership changed".into());
    }
    Ok(())
}
fn private_file(path: &Path, create: bool) -> Result<File, String> {
    let file = OpenOptions::new()
        .read(true)
        .write(create)
        .create(create)
        .truncate(false)
        .mode(0o600)
        .custom_flags((OFlags::NOFOLLOW | OFlags::NONBLOCK).bits() as i32)
        .open(path)
        .map_err(
            |_| "Pi private file is unavailable or linked; preserve it and disable persistence",
        )?;
    let meta = file
        .metadata()
        .map_err(|_| "Pi private file metadata unavailable")?;
    if !meta.is_file()
        || meta.nlink() != 1
        || meta.uid() != rustix::process::geteuid().as_raw()
        || meta.mode() & 0o077 != 0
    {
        return Err(
            "Pi private file ownership/mode is unsafe; preserve it and disable persistence".into(),
        );
    }
    Ok(file)
}
fn private_directory(path: &Path) -> Result<File, String> {
    let mut directory = File::from(
        rustix::fs::open(
            "/",
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|_| "Pi directory root unavailable")?,
    );
    if !path.is_absolute() {
        return Err("Pi state root must be absolute".into());
    }
    for component in path.components().skip(1) {
        let Component::Normal(name) = component else {
            return Err("Pi state root contains noncanonical components".into());
        };
        match rustix::fs::mkdirat(&directory, name, Mode::from_raw_mode(0o700)) {
            Ok(()) => (),
            Err(rustix::io::Errno::EXIST) => (),
            Err(_) => return Err("Pi private directory creation failed".into()),
        }
        directory = File::from(
            rustix::fs::openat(
                &directory,
                name,
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(|_| "Pi private directory contains an unavailable/link component")?,
        );
        let meta = directory
            .metadata()
            .map_err(|_| "Pi directory metadata unavailable")?;
        if meta.mode() & 0o022 != 0 && meta.mode() & 0o1000 == 0 {
            return Err("Pi state ancestor is writable by other users".into());
        }
    }
    same(path, &directory)?;
    Ok(directory)
}

fn same_file(path: &Path, held: &File) -> Result<(), String> {
    let file = private_file(path, false)?;
    let named = file
        .metadata()
        .map_err(|_| "Pi lock observation unavailable")?;
    let observed = held.metadata().map_err(|_| "Pi lock handle unavailable")?;
    if (named.dev(), named.ino()) != (observed.dev(), observed.ino()) {
        return Err("Pi lock identity changed; preserve state and disable persistence".into());
    }
    Ok(())
}
