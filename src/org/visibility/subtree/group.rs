use crate::buffer::Buffer;
use crate::coordinates::{Coordinates, Scalar};

use super::{Group, Outline};

impl Outline {
    pub(super) fn subtree_group(&self, buffer: &Buffer, point: Scalar) -> Result<Group, String> {
        if buffer.rectangle_active() {
            return Err("Org subtree edits require a linear region".into());
        }
        let coordinates = Coordinates::new(buffer).map_err(|e| e.to_string())?;
        let bytes = match buffer
            .region(crate::buffer::edit::Point {
                stamp: buffer.stamp(),
                position: point,
            })
            .map_err(|e| e.to_string())?
        {
            Some(region) => {
                coordinates
                    .to_byte(Scalar(region.range.start))
                    .map_err(|e| e.to_string())?
                    .0
                    ..coordinates
                        .to_byte(Scalar(region.range.end))
                        .map_err(|e| e.to_string())?
                        .0
            }
            None => self.point_subtree(buffer, &coordinates, point)?,
        };
        self.group(bytes).map_err(str::to_owned)
    }

    fn point_subtree(
        &self,
        buffer: &Buffer,
        coordinates: &Coordinates<'_>,
        point: Scalar,
    ) -> Result<std::ops::Range<usize>, String> {
        let byte = coordinates.to_byte(point).map_err(|e| e.to_string())?.0;
        let in_line = coordinates
            .to_line(point)
            .map_err(|e| e.to_string())?
            .column
            != 0;
        if self.unavailable(byte, byte == buffer.len_bytes(), in_line) {
            return Err("Org subtree edit is unavailable in this source context".into());
        }
        let index = self.heading(byte).ok_or("No Org subtree here")?;
        let heading = &self.headings[index];
        Ok(heading.start..heading.end)
    }

    pub(super) fn group(&self, bytes: std::ops::Range<usize>) -> Result<Group, &'static str> {
        let first = self
            .headings
            .binary_search_by_key(&bytes.start, |h| h.start)
            .map_err(|_| "Region must start at an Org heading")?;
        let root = &self.headings[first];
        let mut cursor = bytes.start;
        let mut roots = Vec::new();
        while cursor < bytes.end {
            let i = self
                .headings
                .binary_search_by_key(&cursor, |h| h.start)
                .map_err(|_| "Region must contain complete Org subtrees")?;
            let h = &self.headings[i];
            if h.level != root.level || h.parent != root.parent || h.end > bytes.end {
                return Err("Region must contain complete sibling Org subtrees");
            }
            roots.push(i);
            cursor = h.end;
        }
        if roots.is_empty() {
            return Err("Org subtree region is empty");
        }
        let end = self.headings.partition_point(|h| h.start < bytes.end);
        Ok(Group {
            bytes,
            roots,
            headings: first..end,
        })
    }

    pub(super) fn admit_group(&self, group: &Group) -> Result<(), &'static str> {
        for i in group.headings.clone() {
            self.admit_heading(i)?;
        }
        let mut parent = self.headings[group.roots[0]].parent;
        while let Some(i) = parent {
            self.admit_heading(i)?;
            parent = self.headings[i].parent;
        }
        if self.malformed_between(&group.bytes) {
            return Err("Org subtree contains an unclosed structure");
        }
        Ok(())
    }

    pub(super) fn admit_heading(&self, index: usize) -> Result<(), &'static str> {
        let h = &self.headings[index];
        let parent_level = h.parent.map_or(0, |p| self.headings[p].level);
        if h.level != parent_level + 1 {
            return Err("Org subtree hierarchy has a missing parent level");
        }
        if self.malformed_between(&(h.start..h.section)) {
            return Err("Org parent section contains an unclosed structure");
        }
        Ok(())
    }

    fn malformed_between(&self, range: &std::ops::Range<usize>) -> bool {
        let index = self.malformed.partition_point(|r| r.end <= range.start);
        self.malformed
            .get(index)
            .is_some_and(|r| r.start < range.end)
    }
}
