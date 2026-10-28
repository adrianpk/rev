//! Small revision-qualified timestamp observations. A draft has no publication
//! authority: the canonical buffer validates this span again on acceptance.
mod cursor;
mod observe;
use super::time::{CivilDateTime, Timestamp, TimestampField, TimestampRange};
use crate::buffer::{Buffer, Span, edit::Replacement};
use crate::coordinates::Scalar;

pub(crate) struct Target {
    pub(crate) source: Span,
    value: Option<TimestampRange>,
    selected_end: bool,
    append_end: bool,
    prefix: String,
    suffix: String,
}
impl Target {
    pub(crate) fn at(buffer: &Buffer, point: Scalar) -> Result<Self, String> {
        observe::at(buffer, point)
    }
    pub(crate) fn insertion(buffer: &Buffer, point: Scalar) -> Result<Self, String> {
        let byte = crate::coordinates::Coordinates::new(buffer)
            .map_err(|e| e.to_string())?
            .to_byte(point)
            .map_err(|e| e.to_string())?
            .0;
        Self::new(buffer, byte..byte)
    }
    pub(crate) fn append_end(buffer: &Buffer, point: Scalar) -> Result<Self, String> {
        let mut target = Self::at(buffer, point)?;
        let value = target
            .value
            .as_ref()
            .ok_or("No Org timestamp for range entry")?;
        if value.end.is_some() {
            return Err("Org date range already has an end".into());
        }
        target.append_end = true;
        Ok(target)
    }
    pub(crate) fn appending(&self) -> bool {
        self.append_end
    }
    pub(crate) fn repeats(&self) -> bool {
        self.value.is_none() && self.prefix.is_empty()
    }
    pub(crate) fn field(&self, buffer: &Buffer, point: Scalar) -> Result<TimestampField, String> {
        let source = buffer
            .slice(&self.source)
            .map_err(|e| e.to_string())?
            .to_string();
        let at = self.point_offset(buffer, point)?;
        let offset = if self.selected_end {
            source.find(['>', ']']).ok_or("Unclosed Org timestamp")? + 3
        } else {
            0
        };
        let value = self.value.as_ref().ok_or("No Org timestamp at point")?;
        let stamp = if self.selected_end {
            value.end.as_ref().expect("observed end")
        } else {
            &value.start
        };
        stamp.field_at(
            at.checked_sub(offset)
                .ok_or("Org point precedes timestamp field")?,
        )
    }
    fn point_offset(&self, buffer: &Buffer, point: Scalar) -> Result<usize, String> {
        if point.0 > self.source.range.end {
            return Err("Org point follows timestamp source".into());
        }
        let span = buffer
            .span(self.source.range.start..point.0)
            .map_err(|e| e.to_string())?;
        Ok(buffer.slice(&span).map_err(|e| e.to_string())?.len_bytes())
    }
    pub(crate) fn new(buffer: &Buffer, range: std::ops::Range<usize>) -> Result<Self, String> {
        let source = buffer.byte_span(range).map_err(|e| e.to_string())?;
        Ok(Self {
            source,
            value: None,
            selected_end: false,
            append_end: false,
            prefix: String::new(),
            suffix: String::new(),
        })
    }
    pub(crate) fn decorated(mut self, prefix: String, suffix: String) -> Self {
        self.prefix = prefix;
        self.suffix = suffix;
        self
    }
    pub(crate) fn exists(&self) -> bool {
        self.value.is_some()
    }
    pub(crate) fn template(
        &self,
        now: CivilDateTime,
        active: bool,
    ) -> Result<TimestampRange, String> {
        let Some(value) = &self.value else {
            return Ok(TimestampRange {
                start: Timestamp::new(now.date, None, active)?,
                end: None,
            });
        };
        if self.append_end {
            return Ok(TimestampRange {
                start: value.start.clone(),
                end: None,
            });
        }
        if self.selected_end {
            return Ok(TimestampRange {
                start: value.end.clone().expect("selected range end"),
                end: None,
            });
        }
        Ok(value.clone())
    }
    pub(crate) fn for_resolution(&self, value: &TimestampRange) -> Result<TimestampRange, String> {
        self.merge(value)
    }
    pub(crate) fn check(&self, value: &TimestampRange) -> Result<(), String> {
        self.merge(value)?.check()
    }
    pub(crate) fn replacement(
        &self,
        buffer: &Buffer,
        value: &TimestampRange,
    ) -> Result<Option<Replacement>, String> {
        let original = buffer.slice(&self.source).map_err(|e| e.to_string())?;
        let value = self.merge(value)?;
        value.check()?;
        let text = format!("{}{}{}", self.prefix, value.format(), self.suffix);
        if original == text {
            return Ok(None);
        }
        Ok(Some(Replacement {
            source: self.source.clone(),
            text,
        }))
    }
    fn merge(&self, value: &TimestampRange) -> Result<TimestampRange, String> {
        if !self.selected_end && !self.append_end {
            return Ok(value.clone());
        }
        if value.end.is_some() {
            return Err("Cannot nest an Org date range in its end timestamp".into());
        }
        let mut original = self.value.clone().expect("observed range");
        original.start.set_active(value.start.active());
        original.end = Some(value.start.clone());
        Ok(original)
    }
}
