use super::*;
use crate::org::preview::Resource;
impl Snapshot {
    pub(crate) fn preview(
        self,
        all: bool,
        force: bool,
        cancel: &Cancellation,
    ) -> Result<Vec<Resource>, String> {
        let document = Document::observe(self.text, &self.policy, cancel)?;
        let range = document.preview_range(self.point, all, self.region);
        let mut resources = Vec::new();
        for link in document
            .links
            .iter()
            .filter(|l| range.contains(&l.bytes.start))
        {
            crate::org::preview::check(cancel)?;
            if !force && !link.description.is_empty() {
                continue;
            }
            let target = Target::parse(&link.target)?;
            if matches!(target, Target::Internal(_) | Target::Id { .. }) {
                continue;
            }
            if resources.len() == 32 {
                return Err("Preview selection exceeds 32 resources".into());
            }
            resources.push(Resource {
                row: document.text.byte_to_line(link.bytes.start),
                target: document.follow(target, link.bytes.start, &self.directory, &self.policy)?,
            });
        }
        Ok(resources)
    }
}
impl Document {
    fn preview_range(&self, point: usize, all: bool, region: Option<Range<usize>>) -> Range<usize> {
        if all {
            return region.unwrap_or(0..self.text.len_bytes());
        }
        if let Some(region) = region {
            return region;
        }
        if let Some(link) = self.at(point) {
            return link.bytes.clone();
        }
        let start = self
            .headings
            .iter()
            .rev()
            .find(|h| h.byte <= point)
            .map_or(0, |h| h.byte);
        let end = self
            .headings
            .iter()
            .find(|h| h.byte > point)
            .map_or(self.text.len_bytes(), |h| h.byte);
        start..end
    }
}
