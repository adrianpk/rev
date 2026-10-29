use super::*;
use std::collections::BTreeMap;
impl Outline {
    pub(in crate::org) fn archive_properties(
        &self,
        buffer: &Buffer,
        entry: usize,
        values: &BTreeMap<String, String>,
    ) -> Result<Vec<Replacement>, String> {
        let mut edits = Vec::new();
        let mut missing = String::new();
        for (key, value) in values {
            let found = self.properties.entries[entry]
                .records
                .iter()
                .any(|i| self.properties.local(*i, key));
            if found {
                edits.extend(self.properties.edit(buffer, entry, key, Some(value))?);
            } else {
                missing.push_str(&format!(":{key}: {value}{}", buffer.newline()));
            }
        }
        if !missing.is_empty() {
            edits.push(self.properties.insert_lines(buffer, entry, &missing)?);
        }
        edits.sort_by_key(|e| e.source.range.start);
        Ok(edits)
    }
}
