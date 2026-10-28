use super::*;
impl Properties {
    pub(super) fn resolve(
        &self,
        buffer: &Buffer,
        entry: usize,
        key: &str,
        _inherit: &[String],
    ) -> Result<PropertyValue, String> {
        let records: Vec<_> = self.entries[entry]
            .records
            .iter()
            .copied()
            .filter(|i| self.records[*i].key == key.trim_end_matches('+'))
            .collect();
        let mut value = PropertyValue {
            text: String::new(),
            records: Vec::new(),
        };
        for record in records {
            let part = text(buffer, self.records[record].value.clone())?;
            if !self.records[record].additive {
                value.text.clear();
                value.records.clear();
            }
            if !value.text.is_empty() && !part.is_empty() {
                value.text.push(' ');
            }
            value.text.push_str(&part);
            value.records.push(record);
        }
        Ok(value)
    }
}
