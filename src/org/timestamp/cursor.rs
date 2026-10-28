use super::*;

impl Target {
    /// Keep repeated adjustments in the same semantic field, even when a date
    /// change replaces a localized weekday with a different number of scalars.
    pub(crate) fn cursor(
        &self,
        value: &TimestampRange,
        point: Scalar,
    ) -> Result<Option<Scalar>, String> {
        let Some(before) = &self.value else {
            return Ok(None);
        };
        let base = self.source.range.start + prefix(before, self.selected_end);
        let Some(at) = point.0.checked_sub(base) else {
            return Ok(None);
        };
        let old = selected(before, self.selected_end).format();
        if at > old.chars().count() {
            return Ok(None);
        }
        let next = self.merge(value)?;
        let new = selected(&next, self.selected_end).format();
        let byte = old.char_indices().nth(at).map_or(old.len(), |(i, _)| i);
        let field = selected(before, self.selected_end).field_at(byte).ok();
        let at = position(&old, &new, at, field);
        Ok(Some(Scalar(
            self.source.range.start + prefix(&next, self.selected_end) + at,
        )))
    }
}
fn selected(value: &TimestampRange, end: bool) -> &Timestamp {
    if end {
        value.end.as_ref().expect("observed range end")
    } else {
        &value.start
    }
}
fn prefix(value: &TimestampRange, end: bool) -> usize {
    if end {
        value.start.format().chars().count() + 2
    } else {
        0
    }
}
fn position(old: &str, new: &str, at: usize, field: Option<TimestampField>) -> usize {
    let old_len = old.chars().count();
    let new_len = new.chars().count();
    if at == old_len {
        return new_len;
    }
    if field == Some(TimestampField::Active) && at > 0 {
        return new_len - 1;
    }
    if field.is_some_and(clock) {
        return clock_position(old, new, at, field.expect("clock field"));
    }
    if field == Some(TimestampField::Day) && at > 10 {
        return at.min(
            anchor(new, false, false)
                .unwrap_or(new_len - 1)
                .saturating_sub(1),
        );
    }
    if field.is_none() {
        return new_len.saturating_sub(old_len - at);
    }
    at.min(new_len - 1)
}
fn clock(field: TimestampField) -> bool {
    matches!(
        field,
        TimestampField::Hour
            | TimestampField::Minute
            | TimestampField::EndHour
            | TimestampField::EndMinute
    )
}
fn clock_position(old: &str, new: &str, at: usize, field: TimestampField) -> usize {
    let end = matches!(field, TimestampField::EndHour | TimestampField::EndMinute);
    let minute = matches!(field, TimestampField::Minute | TimestampField::EndMinute);
    let Some(start) = anchor(old, end, minute) else {
        return 9;
    };
    let Some(next) = anchor(new, end, minute) else {
        return new.chars().count() - 1;
    };
    (next + at.saturating_sub(start)).min(new.chars().count() - 1)
}
fn anchor(text: &str, end: bool, minute: bool) -> Option<usize> {
    let colon = if end { text.rfind(':') } else { text.find(':') }?;
    if minute {
        return Some(text[..colon + 1].chars().count());
    }
    let digits = text[..colon]
        .bytes()
        .rev()
        .take_while(u8::is_ascii_digit)
        .count();
    Some(text[..colon - digits].chars().count())
}
