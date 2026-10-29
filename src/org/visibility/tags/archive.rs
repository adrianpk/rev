use super::*;
use crate::{
    buffer::{Buffer, edit::Replacement},
    coordinates::Scalar,
    org::{
        archive::{Context, Sibling},
        refile::Subtree,
    },
};
use std::collections::BTreeMap;
impl Outline {
    pub(crate) fn archive_tag(
        &self,
        buffer: &Buffer,
        point: Scalar,
    ) -> Result<Vec<Replacement>, String> {
        let index = self.tag_target(buffer, point)?;
        if self
            .tag_membership(index, false)
            .iter()
            .any(|t| t == "ARCHIVE")
        {
            self.tag_edit(buffer, index, "ARCHIVE", true)
        } else {
            self.tag_edit(buffer, index, &self.tag_appended(index, "ARCHIVE")?, false)
        }
    }
    pub(crate) fn archive_context(
        &self,
        buffer: &Buffer,
        point: Scalar,
        policy: &crate::config::Org,
    ) -> Result<Context, String> {
        let index = self.tag_target(buffer, point)?;
        let entry = self.property_target(buffer, point)?;
        let span = self.refile_span(buffer, point)?;
        if buffer
            .region(crate::buffer::edit::Point {
                stamp: buffer.stamp(),
                position: point,
            })
            .map_err(|e| e.to_string())?
            .is_some()
        {
            return Err("Archive requires the current subtree without an active region".into());
        }
        let values = self.archive_values(buffer, index, entry, policy)?;
        let location = self.archive_destination(buffer, entry, policy)?;
        Ok(Context {
            subtree: Subtree::checked(buffer, span)?,
            values,
            location,
            entry,
        })
    }
    fn archive_destination(
        &self,
        buffer: &Buffer,
        entry: usize,
        policy: &crate::config::Org,
    ) -> Result<String, String> {
        let location = self.property_value(buffer, entry, "ARCHIVE", &["ARCHIVE".into()])?;
        if !location.records.is_empty() && !location.undefined {
            return Ok(location.text);
        }
        Ok(self
            .archive_location
            .clone()?
            .unwrap_or_else(|| policy.archive.location.clone()))
    }
    fn archive_values(
        &self,
        buffer: &Buffer,
        index: usize,
        entry: usize,
        policy: &crate::config::Org,
    ) -> Result<BTreeMap<String, String>, String> {
        let mut values = self.archive_identity(buffer, index)?;
        let task = &self.tags.headers[index].word;
        if self
            .tags
            .workflow
            .effective(&policy.todo)?
            .state(task)
            .is_some()
        {
            values.insert("ARCHIVE_TODO".into(), task.clone());
        }
        let category = self.property_value(buffer, entry, "CATEGORY", &["CATEGORY".into()])?;
        let category = if category.records.is_empty() {
            buffer
                .file()
                .and_then(|f| f.path().file_stem())
                .and_then(|p| p.to_str())
                .unwrap_or(buffer.name())
                .to_owned()
        } else {
            category.text
        };
        values.insert("ARCHIVE_CATEGORY".into(), category);
        Ok(values)
    }
    fn archive_identity(
        &self,
        buffer: &Buffer,
        index: usize,
    ) -> Result<BTreeMap<String, String>, String> {
        let mut values = BTreeMap::new();
        if let Some(file) = buffer.file() {
            values.insert(
                "ARCHIVE_FILE".into(),
                file.path()
                    .to_str()
                    .ok_or("Archive origin path must be UTF-8")?
                    .into(),
            );
        }
        values.insert("ARCHIVE_OLPATH".into(), self.archive_path(buffer, index)?);
        let local = self.tag_membership(index, false);
        let inherited = self
            .tag_membership(index, true)
            .into_iter()
            .filter(|t| !local.contains(t))
            .collect::<Vec<_>>();
        values.insert(
            "ARCHIVE_ITAGS".into(),
            if inherited.is_empty() {
                String::new()
            } else {
                format!(":{}:", inherited.join(":"))
            },
        );
        Ok(values)
    }
    fn archive_path(&self, buffer: &Buffer, index: usize) -> Result<String, String> {
        let mut path = Vec::new();
        let mut parent = self.headings[index].parent;
        while let Some(i) = parent {
            let header = &self.tags.headers[i];
            let span = buffer
                .byte_span(header.title.clone())
                .map_err(|e| e.to_string())?;
            path.push(
                buffer
                    .slice(&span)
                    .map_err(|e| e.to_string())?
                    .to_string()
                    .trim()
                    .to_owned(),
            );
            parent = self.headings[i].parent;
        }
        path.reverse();
        Ok(path.join("/"))
    }
    pub(crate) fn archive_sibling(
        &self,
        buffer: &Buffer,
        point: Scalar,
    ) -> Result<Sibling, String> {
        let index = self.tag_target(buffer, point)?;
        let current = &self.headings[index];
        let mut found = Vec::new();
        for (i, _) in self
            .headings
            .iter()
            .enumerate()
            .filter(|(_, h)| h.parent == current.parent && h.level == current.level)
        {
            let title = buffer
                .byte_span(self.tags.headers[i].title.clone())
                .map_err(|e| e.to_string())?;
            if buffer
                .slice(&title)
                .map_err(|e| e.to_string())?
                .to_string()
                .trim()
                == "Archive"
                && self.tag_membership(i, false).iter().any(|t| t == "ARCHIVE")
            {
                found.push(i);
            }
        }
        if found.len() > 1 {
            return Err("Archive sibling is ambiguous".into());
        }
        if found.first() == Some(&index) {
            return Err("Cannot archive the Archive sibling into itself".into());
        }
        Ok(Sibling {
            at: found.first().map_or(current.end, |i| self.headings[*i].end),
            level: current.level,
            create: found.is_empty(),
        })
    }
}

impl Outline {
    pub(crate) fn archived(&self, index: usize) -> bool {
        self.tags.headers[index]
            .local
            .iter()
            .any(|t| t.name == "ARCHIVE")
    }
    pub(crate) fn archive_tagged(&self, buffer: &Buffer, point: Scalar) -> Result<bool, String> {
        Ok(self.archived(self.tag_target(buffer, point)?))
    }
}
