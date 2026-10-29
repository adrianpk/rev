use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Value {
    Date(Date),
    Utc(i64),
}
impl Calendar<'_> {
    pub(super) fn value(&self, stamp: &crate::org::time::Timestamp) -> Result<Value, String> {
        stamp.civil().map_or_else(
            || Ok(Value::Date(stamp.date())),
            |civil| self.resolve(civil).map(Value::Utc),
        )
    }
    fn resolve(&self, civil: CivilDateTime) -> Result<i64, String> {
        match self.zone.resolve(civil)? {
            Resolution::Unique(value) => Ok(value.unix_seconds),
            Resolution::Ambiguous(_) => Err(
                "Ambiguous calendar local time; choose an unambiguous source time or UTC zone"
                    .into(),
            ),
            Resolution::Nonexistent => {
                Err("Nonexistent calendar local time; correct source time or select UTC".into())
            }
        }
    }
    pub(super) fn date_property(&mut self, name: &str, value: Value) -> Result<(), String> {
        match value {
            Value::Date(date) => self.raw(&format!("{name};VALUE=DATE"), &date_text(date)),
            Value::Utc(seconds) => self.raw(name, &utc_text(seconds)?),
        }
    }
    pub(super) fn event_dates(&mut self, value: &TimestampRange) -> Result<(), String> {
        let start = self.value(&value.start)?;
        let end = self.event_end(value)?;
        compatible(start, end)?;
        self.date_property("DTSTART", start)?;
        self.date_property("DTEND", end)
    }
    fn range_end(&self, last: &crate::org::time::Timestamp) -> Result<Value, String> {
        if let Some(end) = last.end_civil()? {
            return self.resolve(end).map(Value::Utc);
        }
        match self.value(last)? {
            Value::Date(date) => Ok(Value::Date(date.add_days(1)?)),
            value => Ok(value),
        }
    }
    fn event_end(&self, value: &TimestampRange) -> Result<Value, String> {
        if let Some(last) = &value.end {
            return self.range_end(last);
        }
        if let Some(end) = value.start.end_civil()? {
            return self.resolve(end).map(Value::Utc);
        }
        match self.value(&value.start)? {
            Value::Date(date) => Ok(Value::Date(date.add_days(1)?)),
            Value::Utc(seconds) => seconds
                .checked_add(7200)
                .map(Value::Utc)
                .ok_or_else(|| "Calendar default duration overflows".into()),
        }
    }
}
pub(super) fn compatible(start: Value, end: Value) -> Result<(), String> {
    let matching = matches!(
        (start, end),
        (Value::Date(_), Value::Date(_)) | (Value::Utc(_), Value::Utc(_))
    );
    if !matching || end <= start {
        return Err(
            "Calendar bounds require matching value types and an end after the start".into(),
        );
    }
    Ok(())
}
fn date_text(date: Date) -> String {
    format!("{:04}{:02}{:02}", date.year(), date.month(), date.day())
}
pub(super) fn utc_text(seconds: i64) -> Result<String, String> {
    let civil = crate::org::time::OperationTime::from_unix_seconds(seconds).civil_at_offset(0)?;
    Ok(format!(
        "{}T{:02}{:02}{:02}Z",
        date_text(civil.date),
        civil.time.hour(),
        civil.time.minute(),
        civil.time.second()
    ))
}
