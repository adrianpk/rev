use super::*;
impl Properties {
    pub(super) fn edit(
        &self,
        buffer: &Buffer,
        entry: usize,
        key: &str,
        value: Option<&str>,
    ) -> Result<Vec<Replacement>, String> {
        let local: Vec<_> = self.entries[entry]
            .records
            .iter()
            .copied()
            .filter(|i| self.local(*i, key))
            .collect();
        let Some(value) = value else {
            return local
                .iter()
                .map(|i| replacement(buffer, self.records[*i].source.clone(), String::new()))
                .collect();
        };
        validate_value(value)?;
        if !key.ends_with('+') && local.len() > 1 {
            return Err("Duplicate local Org property; inspect source before editing".into());
        }
        if !key.ends_with('+')
            && let Some(record) = local.first()
        {
            return self.replace(buffer, *record, value).map(|r| vec![r]);
        }
        self.insert(buffer, entry, key, value).map(|r| vec![r])
    }
    pub(super) fn local(&self, record: usize, key: &str) -> bool {
        let record = &self.records[record];
        !record.directive
            && record.key == key.trim_end_matches('+')
            && record.additive == key.ends_with('+')
    }
    pub(super) fn replace(
        &self,
        buffer: &Buffer,
        record: usize,
        value: &str,
    ) -> Result<Replacement, String> {
        validate_value(value)?;
        let record = &self.records[record];
        let separator = if record.value.is_empty()
            && !matches!(previous(buffer, record.value.start)?, Some(' ' | '\t'))
        {
            " "
        } else {
            ""
        };
        replacement(buffer, record.value.clone(), format!("{separator}{value}"))
    }
    fn insert(
        &self,
        buffer: &Buffer,
        entry: usize,
        key: &str,
        value: &str,
    ) -> Result<Replacement, String> {
        let newline = buffer.newline();
        let lines = format!(":{key}: {value}{newline}");
        self.insert_lines(buffer, entry, &lines)
    }
    pub(super) fn insert_lines(
        &self,
        buffer: &Buffer,
        entry: usize,
        lines: &str,
    ) -> Result<Replacement, String> {
        let entry = &self.entries[entry];
        let newline = buffer.newline();
        if let Some(drawer) = &entry.drawer {
            let line = lines
                .split_inclusive('\n')
                .map(|s| format!("{}{s}", drawer.indent))
                .collect::<String>();
            return replacement(buffer, drawer.closing..drawer.closing, line);
        }
        let anchor = entry.anchor;
        let terminated = previous(buffer, anchor)? == Some('\n');
        let lead = if anchor > 0 && !terminated {
            newline
        } else {
            ""
        };
        let tail = if anchor < buffer.len_bytes() || terminated {
            newline
        } else {
            ""
        };
        let content = format!("{lead}:PROPERTIES:{newline}{lines}:END:{tail}");
        replacement(buffer, anchor..anchor, content)
    }
}
fn validate_value(value: &str) -> Result<(), String> {
    if value.len() > 262144 || value.chars().any(char::is_control) {
        return Err("Org property value must be one line of at most 262144 UTF-8 bytes".into());
    }
    Ok(())
}

fn previous(buffer: &Buffer, byte: usize) -> Result<Option<char>, String> {
    let at = buffer
        .byte_span(byte..byte)
        .map_err(|e| e.to_string())?
        .range
        .start;
    if at == 0 {
        return Ok(None);
    }
    let span = buffer.span(at - 1..at).map_err(|e| e.to_string())?;
    Ok(buffer
        .slice(&span)
        .map_err(|e| e.to_string())?
        .chars()
        .next())
}
