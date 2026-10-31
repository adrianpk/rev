//! Bounded, explicit filesystem observations for path completion. Filtering and
//! label access use retained memory; this owner neither visits nor creates files.

mod candidate;
mod navigation;
mod query;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub use candidate::Candidate;
pub use navigation::Navigation;
pub use query::Query;

pub const ENTRY_LIMIT: usize = 4096;
pub const NAME_BYTE_LIMIT: usize = 512 * 1024;
pub const INCOMPLETE_MESSAGE: &str =
    "Incomplete candidate list: directory observation limit reached";

/// One cached directory component, including hidden names. No recursive reads
/// or child symlink resolution occur. A limit is conservative: reaching it marks
/// the observation incomplete without reading another entry to prove exhaustion.
#[derive(Debug)]
pub struct Provider {
    directory: PathBuf,
    entries: Vec<Candidate>,
    name_bytes: usize,
    incomplete: bool,
}

impl Provider {
    /// Observe an explicit absolute directory. Errors retain their IO kind and
    /// identify the requested directory; no partially read provider is returned.
    pub fn observe(directory: &Path) -> io::Result<Self> {
        if !directory.is_absolute() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "completion directory must be absolute",
            ));
        }
        Self::read(directory).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("observe completion directory {directory:?}: {error}"),
            )
        })
    }

    fn read(directory: &Path) -> io::Result<Self> {
        let mut provider = Self {
            directory: directory.to_owned(),
            entries: Vec::new(),
            name_bytes: 0,
            incomplete: false,
        };
        for entry in fs::read_dir(directory)?.take(ENTRY_LIMIT) {
            if !provider.include(entry?)? {
                break;
            }
        }
        provider
            .entries
            .sort_by(|left, right| left.order_key().cmp(&right.order_key()));
        Ok(provider)
    }

    fn include(&mut self, entry: fs::DirEntry) -> io::Result<bool> {
        let name = entry.file_name();
        let bytes = name.as_encoded_bytes().len();
        if bytes > NAME_BYTE_LIMIT - self.name_bytes {
            self.incomplete = true;
            return Ok(false);
        }
        self.entries.push(Candidate::observe(entry, name)?);
        self.name_bytes += bytes;
        self.incomplete = self.entries.len() == ENTRY_LIMIT || self.name_bytes == NAME_BYTE_LIMIT;
        Ok(!self.incomplete)
    }

    /// Replace this observation only after a complete successful read. Failure
    /// preserves the previous snapshot, including its explicit directory identity.
    pub fn refresh(&mut self) -> io::Result<()> {
        let observed = Self::observe(&self.directory)?;
        *self = observed;
        Ok(())
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }

    pub fn entries(&self) -> &[Candidate] {
        &self.entries
    }

    /// Case-sensitive literal component-prefix matching in retained byte order.
    /// The returned references cannot outlive or mutate the cached observation.
    pub fn matching(&self, prefix: &str) -> Vec<&Candidate> {
        self.entries
            .iter()
            .filter(|entry| entry.matches(prefix))
            .collect()
    }

    pub fn name_bytes(&self) -> usize {
        self.name_bytes
    }

    pub fn diagnostic(&self) -> Option<&'static str> {
        self.incomplete.then_some(INCOMPLETE_MESSAGE)
    }
}
