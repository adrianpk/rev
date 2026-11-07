use std::io::Read;

use super::{Error, FileReference, NativePath, SourcePosition, Viewport};

macro_rules! identity {
    ($name:ident) => {
        /// Stable opaque identity; display names and paths cannot select it.
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize)]
        #[serde(try_from = "String")]
        pub struct $name(String);
        impl $name {
            pub fn new() -> Result<Self, Error> {
                let mut bytes = [0u8; 16];
                std::fs::File::open("/dev/urandom")
                    .and_then(|mut file| file.read_exact(&mut bytes))
                    .map_err(|_| Error::Unavailable("cannot allocate an identity"))?;
                Ok(Self(
                    bytes.iter().map(|byte| format!("{byte:02x}")).collect(),
                ))
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
        impl TryFrom<String> for $name {
            type Error = Error;
            fn try_from(value: String) -> Result<Self, Error> {
                if value.len() != 32
                    || !value
                        .bytes()
                        .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
                {
                    return Err(Error::Invalid("identity"));
                }
                Ok(Self(value))
            }
        }
    };
}
identity!(ProjectId);
identity!(WorkspaceId);
identity!(RootId);

/// Positive values representable in the TOML integer format. Never wraps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize)]
#[serde(try_from = "u64")]
pub struct Revision(u64);
impl Revision {
    pub const INITIAL: Self = Self(1);
    pub fn get(self) -> u64 {
        self.0
    }
    pub fn next(self) -> Result<Self, Error> {
        self.0
            .checked_add(1)
            .filter(|value| *value <= i64::MAX as u64)
            .map(Self)
            .ok_or(Error::RevisionExhausted)
    }
}
impl TryFrom<u64> for Revision {
    type Error = Error;
    fn try_from(value: u64) -> Result<Self, Error> {
        if value == 0 || value > i64::MAX as u64 {
            return Err(Error::Invalid("revision"));
        }
        Ok(Self(value))
    }
}

/// Unicode-preserving, case-sensitive name, trimmed only at its edges.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize)]
#[serde(try_from = "String")]
pub struct Name(String);
impl Name {
    pub fn new(value: &str) -> Result<Self, Error> {
        let value = value.trim();
        if value.is_empty()
            || value.len() > super::MAX_NAME_BYTES
            || value.chars().any(char::is_control)
        {
            return Err(Error::Invalid("name"));
        }
        Ok(Self(value.into()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl TryFrom<String> for Name {
    type Error = Error;
    fn try_from(value: String) -> Result<Self, Error> {
        Self::new(&value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize)]
#[serde(
    tag = "kind",
    content = "id",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Owner {
    Project(ProjectId),
    Workspace(WorkspaceId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionRef {
    pub owner: Owner,
    pub revision: Revision,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Root {
    pub id: RootId,
    pub path: NativePath,
    pub label: Name,
    /// Historical provenance only; never a foreign key or authority grant.
    pub origin_project_id: Option<ProjectId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Project {
    pub id: ProjectId,
    pub revision: Revision,
    pub name: Name,
    pub roots_revision: Revision,
    pub root: Root,
    pub session_ref: SessionRef,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Workspace {
    pub id: WorkspaceId,
    pub revision: Revision,
    pub name: Name,
    pub roots_revision: Revision,
    pub roots: Vec<Root>,
    pub session_ref: SessionRef,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileEntry {
    pub file: FileReference,
    pub cursor: SourcePosition,
    pub viewport: Option<Viewport>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Snapshot {
    pub files: Vec<FileEntry>,
    pub active_file: Option<FileReference>,
}

/// An invalid session remains associated with its registration and is preserved
/// verbatim through unrelated mutations. Only a valid explicit replacement may
/// replace its payload, with a checked revision.
#[derive(Clone, Debug)]
pub struct SavedSession {
    pub reference: SessionRef,
    pub roots_revision: Revision,
    pub snapshot: Result<Snapshot, Error>,
    pub(crate) preserved: Option<toml_edit::Table>,
}
impl PartialEq for SavedSession {
    fn eq(&self, other: &Self) -> bool {
        self.reference == other.reference
            && self.roots_revision == other.roots_revision
            && self.snapshot == other.snapshot
            && self.preserved.as_ref().map(ToString::to_string)
                == other.preserved.as_ref().map(ToString::to_string)
    }
}
impl SavedSession {
    pub fn new(reference: SessionRef, roots_revision: Revision, snapshot: Snapshot) -> Self {
        Self {
            reference,
            roots_revision,
            snapshot: Ok(snapshot),
            preserved: None,
        }
    }
}

/// Live context descriptors have no durable buffer, pane, process or text IDs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Context {
    Standalone,
    Directory(NativePath),
    Project(ProjectId),
    Workspace(WorkspaceId),
}

/// A checked live generation prevents stale preparation from activating a new
/// context. Dropping a prepared descriptor has no storage or live-state effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Generation(u64);
impl Generation {
    pub const INITIAL: Self = Self(1);
    pub fn next(self) -> Result<Self, Error> {
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or(Error::RevisionExhausted)
    }
}
#[derive(Debug)]
pub struct PreparedContext {
    pub generation: Generation,
    pub destination: Context,
}
impl PreparedContext {
    pub fn activate(self, current: Generation) -> Result<Context, Error> {
        if current != self.generation {
            return Err(Error::Conflict("superseded context preparation"));
        }
        Ok(self.destination)
    }
}
