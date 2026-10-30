//! Explicit, source-qualified resource observations. No visit or redraw reads IO.
mod png;
use crate::{
    buffer::Stamp,
    file,
    org::{
        Cancellation,
        links::{Follow, Snapshot},
    },
};
pub use png::Png;
use std::sync::Arc;

/// A disposable image observation; bytes are never installed as editable text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    pub row: usize,
    pub png: Png,
    pub(crate) source: file::Location,
}
/// One view's explicit preview preference and revision-qualified image set.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct State {
    pub enabled: bool,
    pub(crate) stamp: Option<Stamp>,
    pub(crate) images: Vec<Arc<Image>>,
}
impl State {
    pub fn images(&self, stamp: Stamp) -> &[Arc<Image>] {
        if self.enabled && self.stamp == Some(stamp) {
            &self.images
        } else {
            &[]
        }
    }
    pub(crate) fn clear(&mut self) {
        self.images.clear();
        self.stamp = None;
    }
}
pub(crate) struct Resource {
    pub row: usize,
    pub target: Follow,
}
pub(crate) struct Output {
    pub images: Vec<Arc<Image>>,
    pub fallback: usize,
    pub details: Vec<String>,
}
pub(crate) struct Work {
    snapshot: Snapshot,
    all: bool,
    force: bool,
    limit: usize,
}
impl Work {
    pub fn new(snapshot: Snapshot, all: bool, force: bool, limit: usize) -> Self {
        Self {
            snapshot,
            all,
            force,
            limit,
        }
    }
    pub fn stamp(&self) -> Stamp {
        self.snapshot.stamp
    }
    pub fn run(self, cancel: &Cancellation) -> Result<Output, String> {
        let resources = self.snapshot.preview(self.all, self.force, cancel)?;
        let mut output = Output {
            images: Vec::new(),
            fallback: 0,
            details: Vec::new(),
        };
        let mut total = 0;
        for resource in resources {
            check(cancel)?;
            let limit = self.limit.min(16777216usize.saturating_sub(total));
            match resource.read(limit, cancel) {
                Ok(image) => {
                    total += image.png.bytes.len();
                    output.images.push(Arc::new(image));
                }
                Err(detail) => {
                    output.fallback += 1;
                    output.details.push(detail);
                }
            }
        }
        check(cancel)?;
        Ok(output)
    }
}
impl Resource {
    fn read(self, limit: usize, cancel: &Cancellation) -> Result<Image, String> {
        let Follow::Local {
            path, search, root, ..
        } = self.target
        else {
            return Err(
                "Resource stays readable; org-open-at-point uses an explicit configured viewer"
                    .into(),
            );
        };
        if !search.is_empty() || !png_path(&path) {
            return Err(
                "Native previews support local PNG; org-open-at-point selects the viewer path"
                    .into(),
            );
        }
        let source = file::resolve(&path).map_err(|e| e.to_string())?;
        super::links::preview_contained(source.path(), root.as_deref())?;
        let data = super::io::bytes::read(&source, limit, cancel)?;
        let png = Png::parse(data, cancel)?;
        Ok(Image {
            row: self.row,
            png,
            source,
        })
    }
}
fn png_path(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.eq_ignore_ascii_case("png"))
}
pub(crate) fn check(cancel: &Cancellation) -> Result<(), String> {
    if cancel.is_cancelled() {
        Err("Preview cancelled or time limit reached".into())
    } else {
        Ok(())
    }
}
impl Output {
    pub fn current(&self) -> Result<(), String> {
        for image in &self.images {
            if file::resolve(image.source.path())
                .map_err(|e| e.to_string())?
                .observation()
                != image.source.observation()
            {
                return Err("Preview resource changed; refresh required".into());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
