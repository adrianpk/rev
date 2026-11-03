use alacritty_terminal::grid::Dimensions;
use std::{
    io,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

/// Maximum owned cells, including both screens, retained rows and row caches.
pub const SESSION_CELLS: usize = 1_048_576;
pub const EDITOR_CELLS: usize = 2_097_152;
pub const SERVICE_BYTES: usize = 65_536;
pub const CONTROL_BYTES: usize = 4096;
pub const COMBINING_SCALARS: usize = 64;
/// Extra allocations have a separate finite budget from base grid cells.
pub const EXTRA_BYTES: usize = 8 * 1024 * 1024;

/// Admitted cell geometry. Construction does not allocate or resize a PTY.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Geometry {
    pub columns: u16,
    pub rows: u16,
}
impl Geometry {
    pub(super) fn cells(self, history: usize) -> io::Result<usize> {
        if self.columns < 2 || self.rows == 0 {
            return Err(invalid(
                "Terminal requires at least two columns and one row",
            ));
        }
        // Upstream caches up to 999 rows after growth/shrink in each grid.
        let rows = 2 * usize::from(self.rows) + history + 2 * 999;
        let cells = rows * usize::from(self.columns);
        if cells > SESSION_CELLS {
            return Err(invalid("Terminal geometry exceeds its cell limit"));
        }
        Ok(cells)
    }
}
impl Dimensions for Geometry {
    fn total_lines(&self) -> usize {
        usize::from(self.rows)
    }
    fn screen_lines(&self) -> usize {
        usize::from(self.rows)
    }
    fn columns(&self) -> usize {
        usize::from(self.columns)
    }
}

/// Per-instance preferences; invalid candidates never replace an owned session.
#[derive(Clone, Debug)]
pub struct Limits {
    pub max_sessions: usize,
    pub scrollback_lines: usize,
    pub max_pending_input_bytes: usize,
    pub max_pending_output_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_sessions: 8,
            scrollback_lines: 2000,
            max_pending_input_bytes: 262144,
            max_pending_output_bytes: 262144,
        }
    }
}
impl Limits {
    pub(super) fn validate(&self) -> io::Result<()> {
        if !(1..=32).contains(&self.max_sessions) || self.scrollback_lines > 20000 {
            return Err(invalid(
                "Terminal session or history preference exceeds its range",
            ));
        }
        for bytes in [self.max_pending_input_bytes, self.max_pending_output_bytes] {
            if !(4096..=1048576).contains(&bytes) {
                return Err(invalid("Terminal queue preference exceeds its range"));
            }
        }
        Ok(())
    }
}

/// Structured executable, arguments and initial directory. No implicit shell evaluation.
#[derive(Clone, Debug)]
pub struct Launch {
    pub program: String,
    pub args: Vec<String>,
    pub directory: PathBuf,
}
impl Launch {
    pub(super) fn validate(&self) -> io::Result<()> {
        let bytes: usize = self.args.iter().map(String::len).sum();
        if self.program.is_empty()
            || self.program.len() > CONTROL_BYTES
            || self.program.contains('\0')
        {
            return Err(invalid("Invalid terminal executable"));
        }
        if self.args.len() > 32 || bytes > 16384 || self.args.iter().any(|arg| arg.contains('\0')) {
            return Err(invalid(
                "Terminal arguments exceed the 32-token/16 KiB profile",
            ));
        }
        if !self.directory.is_absolute() || !self.directory.is_dir() {
            return Err(invalid(
                "Terminal startup directory must be an existing absolute directory",
            ));
        }
        self.executable()?;
        Ok(())
    }
    pub(super) fn executable(&self) -> io::Result<String> {
        let program = Path::new(&self.program);
        let paths: Vec<PathBuf> = if program.components().count() > 1 {
            vec![self.directory.join(program)]
        } else {
            std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
                .take(128)
                .map(|path| path.join(program))
                .collect()
        };
        for path in paths {
            if std::fs::metadata(&path)
                .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
            {
                return path
                    .canonicalize()?
                    .into_os_string()
                    .into_string()
                    .map_err(|_| invalid("Terminal executable is not UTF-8"));
            }
        }
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Terminal executable is unavailable",
        ))
    }
}
pub(super) fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
