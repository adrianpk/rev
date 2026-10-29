//! Typed capture selection and destination discovery, without source mutation.
use crate::config::{CaptureTemplate, Snapshot};
use std::path::{Path, PathBuf};

/// The exact selected template and an explicitly configured/entered file.
/// A missing file selection is distinct from a selected file that does not exist.
#[derive(Clone, Debug)]
pub struct Choice {
    pub template: CaptureTemplate,
    pub file: Option<PathBuf>,
}
impl Choice {
    /// Select an exact key. Empty configured templates use the native Task
    /// fallback. Relative paths retain the assigning leaf's configuration root.
    pub fn configured(snapshot: &Snapshot, key: &str) -> Result<Self, String> {
        let capture = &snapshot.settings().org.capture;
        let template = if capture.templates.is_empty() {
            CaptureTemplate::task()
        } else {
            capture
                .templates
                .iter()
                .find(|t| t.key == key)
                .cloned()
                .ok_or("Unknown capture template key")?
        };
        if template.key != key {
            return Err("Unknown capture template key".into());
        }
        template.validate()?;
        let (file, leaf) = if template.target.file().is_empty() {
            (&capture.default_file[..], "default_file")
        } else {
            (template.target.file(), "templates")
        };
        let root = snapshot
            .origin(&["org", "capture", leaf])
            .and_then(Path::parent);
        Ok(Self {
            file: configured_file(file, root),
            template,
        })
    }
    /// Enter a literal file explicitly, relative to the supplied origin directory.
    /// This neither creates that file nor probes a parent, expands an environment
    /// variable, visits a buffer or installs a fabricated sample destination.
    pub fn enter_file(&mut self, value: &str, directory: &Path) -> Result<(), String> {
        crate::config::capture::path(value)?;
        if value.is_empty() {
            return Err("Enter a capture destination file".into());
        }
        self.file = configured_file(value, Some(directory));
        Ok(())
    }
}
fn configured_file(value: &str, root: Option<&Path>) -> Option<PathBuf> {
    if value.is_empty() {
        return None;
    }
    let path = PathBuf::from(value);
    Some(if path.is_relative() {
        root.map_or(path.clone(), |r| r.join(&path))
    } else {
        path
    })
}
