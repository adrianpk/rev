//! Explicit heading attachment locations and checked real-file operations.
pub(in crate::org) mod directory;
mod location;
mod operation;
mod plan;
mod selection;
use crate::{buffer::Buffer, org::Outline};
pub(crate) use directory::Listing;
pub(crate) use location::location;
pub use operation::{Effect, FileOutcome};
pub(crate) use plan::Plan;
pub(crate) use selection::Selected;
use std::path::PathBuf;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Method {
    Copy,
    Move,
    Link,
}
#[derive(Clone)]
pub(crate) struct Context {
    pub base: PathBuf,
    pub dir: Option<String>,
    pub id: Option<String>,
    pub policy: crate::config::Org,
}
impl Context {
    pub(crate) fn new(
        model: &Outline,
        buffer: &Buffer,
        point: crate::coordinates::Scalar,
        policy: &crate::config::Org,
    ) -> Result<Self, String> {
        let heading = model.tag_target(buffer, point)?;
        let entry = model.heading_property_entry(buffer, heading)?;
        let value = |key| {
            model
                .property_value(buffer, entry, key, &policy.properties.inherit)
                .map(|v| (!v.undefined && !v.records.is_empty()).then_some(v.text))
        };
        let path = buffer
            .file()
            .ok_or("Associate the Org source with a file before attaching")?
            .path();
        Ok(Self {
            base: path
                .parent()
                .ok_or("Attachment source has no parent directory")?
                .into(),
            dir: value("DIR")?,
            id: value("ID")?,
            policy: policy.clone(),
        })
    }
    pub(crate) fn root(&self, id: Option<&str>) -> Result<PathBuf, String> {
        location(
            self.dir.as_deref(),
            id.or(self.id.as_deref()),
            &self.base,
            &self.policy,
        )
    }
    pub(crate) fn selected(
        &self,
        name: &str,
        native: bool,
    ) -> Result<crate::org::links::Follow, String> {
        location::name(name)?;
        let root = self.root(None)?;
        Ok(crate::org::links::Follow::Local {
            path: root.join(name),
            search: String::new(),
            mode: if native {
                crate::org::links::FileMode::Native
            } else {
                crate::org::links::FileMode::Automatic
            },
            root: Some(root),
        })
    }
}

#[cfg(test)]
mod tests;
