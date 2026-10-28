use super::*;
use crate::org::{MatchData, MatchQuery};

impl Outline {
    pub(crate) fn match_tags(
        &self,
        buffer: &Buffer,
        index: usize,
        query: &MatchQuery,
        policy: &crate::config::Org,
    ) -> Result<bool, String> {
        let mut data = MatchData {
            tags: self.tag_membership(index, policy.tags.inherit),
            ..MatchData::default()
        };
        let workflow = self.tags.workflow.effective(&policy.todo)?;
        let header = &self.tags.headers[index];
        data.finished = workflow.state(&header.word).is_some_and(|s| s.done);
        for key in query.properties() {
            if let Some(value) = self.match_property(buffer, index, key, policy)? {
                data.properties.insert(key.into(), value);
            }
        }
        query.matches(&data)
    }
    fn match_property(
        &self,
        buffer: &Buffer,
        index: usize,
        key: &str,
        policy: &crate::config::Org,
    ) -> Result<Option<String>, String> {
        let header = &self.tags.headers[index];
        match key {
            "TODO" => Ok(self
                .tags
                .workflow
                .effective(&policy.todo)?
                .state(&header.word)
                .map(|s| s.name.clone())),
            "LEVEL" => Ok(Some(self.headings[index].level.to_string())),
            "PRIORITY" => self.match_priority(index, policy).map(Some),
            "TAGS" => Ok(tag_text(
                header.local.iter().map(|t| t.name.clone()).collect(),
            )),
            "ALLTAGS" => Ok(tag_text(self.tag_membership(index, policy.tags.inherit))),
            "ITEM" => self.match_title(buffer, index, policy).map(Some),
            _ => self.match_stored_property(buffer, index, key, policy),
        }
    }
    fn match_stored_property(
        &self,
        buffer: &Buffer,
        index: usize,
        key: &str,
        policy: &crate::config::Org,
    ) -> Result<Option<String>, String> {
        let c = Coordinates::new(buffer).map_err(|e| e.to_string())?;
        let point = c
            .from_byte(crate::coordinates::Byte(self.headings[index].start))
            .map_err(|e| e.to_string())?;
        let entry = self.property_target(buffer, point)?;
        let value = self.property_value(buffer, entry, key, &policy.properties.inherit)?;
        Ok((!value.undefined && !value.records.is_empty()).then_some(value.text))
    }
    fn match_title(
        &self,
        buffer: &Buffer,
        index: usize,
        policy: &crate::config::Org,
    ) -> Result<String, String> {
        let header = &self.tags.headers[index];
        if header.title.len() > 1048576 {
            return Err("Org match title exceeds 1048576 UTF-8 bytes".into());
        }
        let span = buffer
            .byte_span(header.title.clone())
            .map_err(|e| e.to_string())?;
        let title = buffer.slice(&span).map_err(|e| e.to_string())?.to_string();
        let task = self
            .tags
            .workflow
            .effective(&policy.todo)?
            .state(&header.word)
            .is_some();
        let title = if task {
            title
                .strip_prefix(&header.word)
                .unwrap_or(&title)
                .trim_start()
        } else {
            title.trim()
        };
        let title = if self.priority_value(index, policy)?.is_some() {
            title.get(4..).unwrap_or("").trim()
        } else {
            title.trim()
        };
        Ok(title.into())
    }
}

impl Outline {
    fn match_priority(&self, index: usize, policy: &crate::config::Org) -> Result<String, String> {
        let value = self
            .priority_value(index, policy)?
            .unwrap_or(self.priority_policy(policy)?.default);
        Ok(char::from(value).to_string())
    }
}

fn tag_text(names: Vec<String>) -> Option<String> {
    (!names.is_empty()).then(|| format!(":{}:", names.join(":")))
}
