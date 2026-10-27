use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

/// Independently resolved user directories, each including the rev namespace.
/// Resolving locations never reads settings or creates filesystem entries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Roots {
    pub config: PathBuf,
    pub data: PathBuf,
    pub state: PathBuf,
    pub cache: PathBuf,
}

impl Roots {
    /// Resolve from the current process environment without mutating it.
    pub fn from_env() -> io::Result<Self> {
        Self::resolve(|name| std::env::var_os(name))
    }

    /// Resolve from one supplied environment snapshot. Empty or relative XDG
    /// values use an absolute HOME fallback; each missing required root names
    /// its variable in an InvalidInput error. Paths need not exist yet.
    pub fn resolve(mut lookup: impl FnMut(&str) -> Option<OsString>) -> io::Result<Self> {
        let home = lookup("HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute());
        Ok(Self {
            config: directory(&mut lookup, home.as_deref(), "XDG_CONFIG_HOME", ".config")?,
            data: directory(
                &mut lookup,
                home.as_deref(),
                "XDG_DATA_HOME",
                ".local/share",
            )?,
            state: directory(
                &mut lookup,
                home.as_deref(),
                "XDG_STATE_HOME",
                ".local/state",
            )?,
            cache: directory(&mut lookup, home.as_deref(), "XDG_CACHE_HOME", ".cache")?,
        })
    }

    /// The sole implicit configuration source. Includes are always explicit.
    pub fn config_file(&self) -> PathBuf {
        self.config.join("config.toml")
    }
}

fn directory(
    lookup: &mut impl FnMut(&str) -> Option<OsString>,
    home: Option<&Path>,
    variable: &str,
    fallback: &str,
) -> io::Result<PathBuf> {
    if let Some(path) = lookup(variable)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
    {
        return Ok(path.join("rev"));
    }
    let home = home.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("cannot resolve {variable}: provide an absolute {variable} or HOME"),
        )
    })?;
    Ok(home.join(fallback).join("rev"))
}
