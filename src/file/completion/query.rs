use std::io;
use std::path::{Path, PathBuf};

/// Literal editable path split into one directory observation and a component
/// prefix. Construction has no IO or environment access; callers supply roots.
#[derive(Debug)]
pub struct Query {
    directory: PathBuf,
    prefix: String,
}

impl Query {
    /// Resolve relative text against an absolute base, and leading `~/` against
    /// an explicit absolute home. Other tilde, variable, glob and shell-looking
    /// text is literal. Keep `.` and `..` for OS resolution through symlinks.
    pub fn parse(base: &Path, home: Option<&Path>, draft: &str) -> io::Result<Self> {
        if !base.is_absolute() || draft.contains('\0') {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "completion requires an absolute base and a path without NUL",
            ));
        }
        let (parent, prefix) = match draft.rfind('/') {
            Some(index) => (&draft[..=index], &draft[index + 1..]),
            None => ("", draft),
        };
        Ok(Self {
            directory: directory(base, home, parent)?,
            prefix: prefix.to_owned(),
        })
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }
}

fn directory(base: &Path, home: Option<&Path>, parent: &str) -> io::Result<PathBuf> {
    let Some(suffix) = parent.strip_prefix("~/") else {
        return Ok(base.join(parent));
    };
    let home = home.filter(|home| home.is_absolute()).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "home directory is unavailable for ~/ completion",
        )
    })?;
    Ok(home.join(suffix.trim_start_matches('/')))
}
