use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// One observed entry in visible preorder. Symlinks are always leaves.
#[derive(Debug)]
pub struct Entry {
    pub(super) path: PathBuf,
    pub(super) depth: usize,
    pub(super) directory: bool,
    pub(super) symlink: bool,
    pub(super) expanded: bool,
    pub(super) incomplete: bool,
}

pub(super) struct Children {
    pub(super) entries: Vec<Entry>,
    pub(super) incomplete: bool,
}

impl Entry {
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn depth(&self) -> usize {
        self.depth
    }
    pub fn is_directory(&self) -> bool {
        self.directory
    }
    pub fn is_symlink(&self) -> bool {
        self.symlink
    }
    pub fn expanded(&self) -> bool {
        self.expanded
    }

    pub(super) fn root(path: PathBuf) -> Self {
        Self {
            path,
            depth: 0,
            directory: true,
            symlink: false,
            expanded: false,
            incomplete: false,
        }
    }

    pub(super) fn observe(path: PathBuf, depth: usize) -> io::Result<Self> {
        let kind = fs::symlink_metadata(&path)
            .map_err(|e| failure("inspect entry", &path, e))?
            .file_type();
        Ok(Self {
            path,
            depth,
            directory: kind.is_dir(),
            symlink: kind.is_symlink(),
            expanded: false,
            incomplete: false,
        })
    }

    pub(super) fn children(&self, policy: &crate::config::Tree) -> io::Result<Children> {
        use crate::file::completion::{ENTRY_LIMIT, NAME_BYTE_LIMIT};
        // Revalidate the selected entry before reading: a replaced symlink must
        // not turn a stale directory observation into a traversable link.
        let current = Self::observe(self.path.clone(), self.depth)?;
        if !current.directory {
            return Err(failure(
                "read directory",
                &self.path,
                io::Error::other("entry is not a directory"),
            ));
        }
        let directory =
            fs::read_dir(&self.path).map_err(|e| failure("read directory", &self.path, e))?;
        let mut children = Vec::new();
        let mut name_bytes = 0;
        let mut incomplete = false;
        for (index, child) in directory.take(ENTRY_LIMIT).enumerate() {
            let child = child.map_err(|e| failure("read directory", &self.path, e))?;
            let bytes = child.file_name().as_encoded_bytes().len();
            if bytes > NAME_BYTE_LIMIT - name_bytes {
                incomplete = true;
                break;
            }
            name_bytes += bytes;
            if policy.show_hidden || !child.file_name().as_encoded_bytes().starts_with(b".") {
                children.push(Self::observe(child.path(), self.depth + 1)?);
            }
            if index + 1 == ENTRY_LIMIT || name_bytes == NAME_BYTE_LIMIT {
                incomplete = true;
                break;
            }
        }
        children.sort_by(|a, b| {
            a.order_key(policy.directories_first)
                .cmp(&b.order_key(policy.directories_first))
        });
        Ok(Children {
            entries: children,
            incomplete,
        })
    }

    fn order_key(&self, directories_first: bool) -> (bool, Option<&std::ffi::OsStr>) {
        (directories_first && !self.directory, self.path.file_name())
    }

    pub(super) fn label(&self, width: u16) -> String {
        let marker = match (self.directory, self.expanded, self.symlink) {
            (true, true, _) => "v",
            (true, false, _) => ">",
            (_, _, true) => "@",
            _ => " ",
        };
        let indent = self.depth.min(usize::from(width.saturating_sub(4)) / 2);
        let name = self
            .path
            .file_name()
            .unwrap_or(self.path.as_os_str())
            .to_string_lossy();
        let slash = if self.directory && self.path.file_name().is_some() {
            "/"
        } else {
            ""
        };
        let diagnostic = if self.incomplete { " [incomplete]" } else { "" };
        format!("{}{marker} {name}{slash}{diagnostic}", "  ".repeat(indent))
    }
}

pub(super) fn failure(operation: &str, path: &Path, error: io::Error) -> io::Error {
    io::Error::new(
        error.kind(),
        format!("{operation} {}: {error}", path.display()),
    )
}
