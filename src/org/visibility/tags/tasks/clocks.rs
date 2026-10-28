use super::*;
use crate::buffer::edit::Replacement;
use crate::coordinates::{Byte, Scalar};

pub(crate) struct ClockInsertion {
    pub edit: Replacement,
    pub record: Range<usize>,
    pub owner: Scalar,
}
impl Outline {
    pub(crate) fn clock_insertion(
        &self,
        buffer: &Buffer,
        index: usize,
        record: &str,
    ) -> Result<ClockInsertion, String> {
        crate::org::source::current(buffer, self.stamp).map_err(|e| e.to_string())?;
        self.clock_source_clear(buffer, index)?;
        let drawer = self.clock_drawer(buffer, index)?;
        let entry = &self.tags.metadata.entries[index];
        let at = clock_anchor(buffer, entry, &drawer)?;
        let found = at != entry.anchor || entry.drawers.iter().any(|d| d.body.start == at);
        let (prefix, suffix) = clock_wrapper(buffer, at, &drawer, found)?;
        let line = format!("{record}{}", buffer.newline());
        let edit = Replacement {
            source: buffer.byte_span(at..at).map_err(|e| e.to_string())?,
            text: format!("{prefix}{line}{suffix}"),
        };
        let start = edit.source.range.start + prefix.chars().count();
        let c = Coordinates::new(buffer).map_err(|e| e.to_string())?;
        Ok(ClockInsertion {
            record: start..start + line.chars().count(),
            owner: c
                .from_byte(Byte(self.headings[index].start))
                .map_err(|e| e.to_string())?,
            edit,
        })
    }
    fn clock_drawer(&self, buffer: &Buffer, index: usize) -> Result<String, String> {
        let clock = self.task_logging_property(buffer, index, "CLOCK_INTO_DRAWER")?;
        let log = self.task_logging_property(buffer, index, "LOG_INTO_DRAWER")?;
        let value = if clock.is_empty() { log } else { clock };
        let drawer = match value.as_str() {
            "" | "t" => "LOGBOOK".into(),
            "nil" => String::new(),
            _ => value,
        };
        if !drawer.is_empty() && !crate::org::task::drawer_name(&drawer) {
            return Err("Invalid Org clock drawer".into());
        }
        Ok(drawer)
    }
    fn clock_source_clear(&self, buffer: &Buffer, index: usize) -> Result<(), String> {
        let heading = &self.headings[index];
        if heading.section - heading.start > 262144 {
            return Err("Org clock entry exceeds 262144 UTF-8 bytes".into());
        }
        let text = edit::source(buffer, heading.body..heading.section)?;
        let mut byte = heading.body;
        for line in text.split_inclusive('\n') {
            self.clock_line_clear(buffer, byte, line)?;
            byte += line.len();
        }
        Ok(())
    }
    fn clock_line_clear(&self, buffer: &Buffer, byte: usize, line: &str) -> Result<(), String> {
        let Some(value) = line.trim().strip_prefix("CLOCK:") else {
            return Ok(());
        };
        let c = Coordinates::new(buffer).map_err(|e| e.to_string())?;
        let point = c.from_byte(Byte(byte)).map_err(|e| e.to_string())?;
        if !self.timestamp_context(buffer, point)? {
            return Ok(());
        }
        let value = value.trim();
        let close = value
            .find(']')
            .ok_or("Malformed Org CLOCK record; correct source first")?;
        let start = crate::org::time::Timestamp::parse(&value[..=close])?;
        if start.active() || start.time().is_none() {
            return Err("Org CLOCK requires an inactive timestamp with a time".into());
        }
        if value[close + 1..].trim().is_empty() {
            return Err("An incomplete Org CLOCK already exists; resolve its source first".into());
        }
        Ok(())
    }
}
fn clock_anchor(buffer: &Buffer, entry: &model::Entry, drawer: &str) -> Result<usize, String> {
    let mut found = None;
    for item in &entry.drawers {
        if !edit::source(buffer, item.name.clone())?.eq_ignore_ascii_case(drawer) {
            continue;
        }
        if found.replace(item.body.start).is_some() {
            return Err("Ambiguous Org clock drawer".into());
        }
    }
    Ok(found.unwrap_or(entry.anchor))
}
fn clock_wrapper(
    buffer: &Buffer,
    at: usize,
    drawer: &str,
    found: bool,
) -> Result<(String, String), String> {
    if found {
        return Ok((String::new(), String::new()));
    }
    let prefix = edit::prefix(buffer, at)?;
    if drawer.is_empty() {
        return Ok((prefix.into(), String::new()));
    }
    let nl = buffer.newline();
    Ok((format!("{prefix}:{drawer}:{nl}"), format!(":END:{nl}")))
}
