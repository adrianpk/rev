use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::{Publication, Roots, SettingsEdit, mutation};
use crate::file::temporary::Temporary;

use super::{Installation, Registry, State};

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Image {
    bytes: Vec<u8>,
    device: u64,
    inode: u64,
}

/// A held cross-process mutation transaction. Dropping it releases both locks;
/// it never removes or repairs existing user data implicitly.
pub struct Storage {
    pub(super) progress: super::progress::Progress,
    roots: Roots,
    registry: Registry,
    image: Option<Image>,
    lock: mutation::Lock,
    _data_lock: File,
}

/// A unique final path, created before acquisition and never moved afterward.
/// Its preparing record survives a crash without granting selection authority.
pub struct Candidate {
    pub(super) progress: super::progress::Progress,
    pub installation: Installation,
    pub directory: PathBuf,
}

impl Storage {
    pub fn acquire(mut roots: Roots) -> Result<Self, String> {
        fs::create_dir_all(&roots.data)
            .map_err(|e| format!("create installation data root: {e}"))?;
        roots.data = roots.data.canonicalize().map_err(|e| e.to_string())?;
        let directory = roots.data.join("lsp");
        fs::create_dir_all(&directory)
            .map_err(|e| format!("create installation directory: {e}"))?;
        contained(&roots.data, &directory, true)?;
        // Lock the shared data-directory inode as well as the settings lock:
        // two processes can share data while using different state roots.
        let data_lock = File::open(&directory).map_err(|e| e.to_string())?;
        data_lock
            .try_lock()
            .map_err(|_| "another installation job owns this data root".to_owned())?;
        let lock = mutation::Lock::acquire(&roots.state)?;
        let (registry, image) = read(&roots.data)?;
        Ok(Self {
            progress: Default::default(),
            roots,
            registry,
            image,
            lock,
            _data_lock: data_lock,
        })
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }
    pub fn roots(&self) -> &Roots {
        &self.roots
    }

    pub fn candidate(
        &mut self,
        recipe: &str,
        version: &str,
        target: &str,
        source: &str,
    ) -> Result<Candidate, String> {
        if !super::record::identity(recipe) || self.registry.installations.len() >= super::RECORDS {
            return Err("invalid recipe ID or installation record limit reached".into());
        }
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?;
        let id = format!("{:x}-{:x}", time.as_nanos(), std::process::id());
        let payload = Path::new("lsp/installations").join(recipe).join(&id);
        let directory = self.roots.data.join(&payload);
        self.create(&directory)?;
        let installation = Installation {
            id,
            recipe: recipe.into(),
            version: version.into(),
            target: target.into(),
            payload,
            source: source.into(),
            evidence: String::new(),
            runtime: "unverified".into(),
            launches: Vec::new(),
            status: State::Preparing,
            created_at: time.as_secs(),
            verified_at: None,
        };
        let warning = self.save(installation.clone())?;
        if let Some(warning) = warning {
            return Err(warning);
        }
        Ok(Candidate {
            progress: self.progress.clone(),
            installation,
            directory,
        })
    }

    fn create(&self, directory: &Path) -> Result<(), String> {
        let parent = directory.parent().ok_or("candidate has no parent")?;
        contained(&self.roots.data, parent, false)?;
        fs::create_dir_all(parent).map_err(|e| format!("create recipe directory: {e}"))?;
        contained(&self.roots.data, parent, true)?;
        fs::create_dir(directory).map_err(|e| format!("create unique final candidate: {e}"))
    }

    /// Atomically publish complete facts. A returned warning means publication
    /// committed, but parent-directory durability could not be confirmed.
    pub fn save(&mut self, installation: Installation) -> Result<Option<String>, String> {
        self.progress.set(super::Phase::Publishing);
        let next = self.propose(installation)?;
        let bytes = next.encode().into_bytes();
        if bytes.len() > super::REGISTRY_BYTES {
            return Err("installation registry exceeds 2 MiB".into());
        }
        self.unchanged()?;
        let parent = self.roots.data.join("lsp");
        let mut temporary = Temporary::new(&parent).map_err(|e| e.to_string())?;
        temporary
            .file
            .write_all(&bytes)
            .and_then(|()| temporary.file.sync_all())
            .map_err(|e| format!("prepare installation registry: {e}"))?;
        self.unchanged()?;
        self.commit(temporary, next, bytes)
    }

    fn propose(&self, installation: Installation) -> Result<Registry, String> {
        let payload = (installation.status == State::Ready)
            .then(|| self.roots.data.join(&installation.payload));
        let mut next = self.registry.clone();
        match next
            .installations
            .iter_mut()
            .find(|entry| entry.id == installation.id)
        {
            Some(existing) => {
                if existing.payload != installation.payload
                    || existing.recipe != installation.recipe
                {
                    return Err("installation identity cannot be retargeted".into());
                }
                *existing = installation;
            }
            None => next.installations.push(installation),
        }
        next.validate(&self.roots.data)?;
        if let Some(payload) = payload {
            super::tree::bytes(&payload, super::PAYLOAD_BYTES)?;
        }
        Ok(next)
    }

    fn commit(
        &mut self,
        mut temporary: Temporary,
        next: Registry,
        bytes: Vec<u8>,
    ) -> Result<Option<String>, String> {
        let parent = self.roots.data.join("lsp");
        let target = parent.join("installations.toml");
        // Capture identity before commit. No fallible reopen can turn an actual
        // publication into a claimed rollback or adopt a manual replacement.
        let metadata = temporary.file.metadata().map_err(|e| e.to_string())?;
        if self.image.is_none() {
            fs::hard_link(&temporary.path, &target)
                .map_err(|e| format!("publish absent installation registry: {e}"))?;
        } else {
            fs::rename(&temporary.path, &target)
                .map_err(|e| format!("publish installation registry: {e}"))?;
            temporary.disarm();
        }
        self.registry = next;
        self.image = Some(Image {
            bytes,
            device: metadata.dev(),
            inode: metadata.ino(),
        });
        Ok(File::open(&parent)
            .and_then(|directory| directory.sync_all())
            .err()
            .map(|e| format!("registry published; directory durability unconfirmed: {e}")))
    }

    /// Ready facts are already durable before this independently fallible
    /// settings transaction. Configuration conflicts leave them unselected.
    pub fn select(&self, edit: &SettingsEdit) -> Result<Publication, String> {
        self.progress.set(super::Phase::Publishing);
        edit.publish_locked(&self.lock).map_err(|e| e.to_string())
    }

    fn unchanged(&self) -> Result<(), String> {
        let (_, image) = read(&self.roots.data)?;
        if self.image != image {
            return Err(
                "installation registry changed during management; retry after inspection".into(),
            );
        }
        Ok(())
    }
}

pub(super) fn read(data: &Path) -> Result<(Registry, Option<Image>), String> {
    let Some(image) = image(&data.join("lsp/installations.toml"))? else {
        return Ok((Registry::default(), None));
    };
    let text =
        std::str::from_utf8(&image.bytes).map_err(|_| "installation registry is not UTF-8")?;
    let registry: Registry = toml::from_str(text).map_err(
        |_| "invalid installation registry; preserve it for diagnosis before management",
    )?;
    registry.validate(data)?;
    Ok((registry, Some(image)))
}

fn image(path: &Path) -> Result<Option<Image>, String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("read installation registry: {error}")),
    };
    let file = open(path, &metadata)?;
    let mut bytes = Vec::new();
    file.take(super::REGISTRY_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("read installation registry: {e}"))?;
    if bytes.len() > super::REGISTRY_BYTES {
        return Err("installation registry exceeds 2 MiB".into());
    }
    Ok(Some(Image {
        bytes,
        device: metadata.dev(),
        inode: metadata.ino(),
    }))
}

fn open(path: &Path, metadata: &fs::Metadata) -> Result<File, String> {
    if !metadata.is_file() || metadata.len() > super::REGISTRY_BYTES as u64 {
        return Err("installation registry must be a regular file of at most 2 MiB".into());
    }
    let file = File::open(path).map_err(|e| format!("open installation registry: {e}"))?;
    let opened = file.metadata().map_err(|e| e.to_string())?;
    if (opened.dev(), opened.ino()) != (metadata.dev(), metadata.ino()) {
        return Err("installation registry changed while opening".into());
    }
    Ok(file)
}

/// Check every existing ancestor, including an absent final payload's parent.
/// Canonical containment admits internal links but never escaping links.
pub(super) fn contained(root: &Path, path: &Path, required: bool) -> Result<(), String> {
    let root = root
        .canonicalize()
        .map_err(|e| format!("installation root: {e}"))?;
    let path = ancestor(path, required)?;
    if !path.starts_with(&root) {
        return Err("installation path escapes its owned root".into());
    }
    Ok(())
}

fn ancestor(mut path: &Path, required: bool) -> Result<PathBuf, String> {
    loop {
        match path.canonicalize() {
            Ok(canonical) => return Ok(canonical),
            Err(e) if required || e.kind() != std::io::ErrorKind::NotFound => {
                return Err(format!("installation path: {e}"));
            }
            Err(_) => (),
        }
        // A dangling link is corruption, not an absent directory.
        if fs::symlink_metadata(path).is_ok() {
            return Err("installation contains a dangling link".into());
        }
        path = path
            .parent()
            .ok_or("installation path has no containing root")?;
    }
}
