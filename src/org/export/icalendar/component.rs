use super::*;
impl Calendar<'_> {
    pub(super) fn entry(&mut self, entry: &Entry) -> Result<(), String> {
        if entry.excluded || !entry.has_data() {
            return Ok(());
        }
        self.select_zone(entry)?;
        let id = self.identity(entry)?;
        if entry.task.is_some() {
            self.todo(entry, &id)?;
        } else {
            self.planning_events(entry, &id)?;
        }
        for (i, stamp) in entry.appointments.iter().enumerate() {
            self.event(entry, &format!("{id}-stamp-{i}"), stamp)?;
        }
        Ok(())
    }
    fn select_zone(&mut self, entry: &Entry) -> Result<(), String> {
        self.zone = match entry.properties.get("TIMEZONE") {
            Some(name) => self
                .snapshot
                .calendar_zones
                .get(name)
                .ok_or("Calendar zone was not captured during preparation")?,
            None => self.snapshot.request.time.zone(),
        };
        Ok(())
    }
    fn begin(&mut self, entry: &Entry, id: &str, kind: &str) -> Result<(), String> {
        if self.count >= 4096 {
            return Err("Calendar export exceeds 4096 components".into());
        }
        self.count += 1;
        self.raw("BEGIN", kind)?;
        self.property("UID", &format!("rev-org-{id}@org.rev"))?;
        self.raw(
            "DTSTAMP",
            &dates::utc_text(self.snapshot.request.time.instant().unix_seconds())?,
        )?;
        self.metadata(entry)
    }
    fn metadata(&mut self, entry: &Entry) -> Result<(), String> {
        self.property(
            "SUMMARY",
            entry.properties.get("SUMMARY").unwrap_or(&entry.summary),
        )?;
        if let Some(value) = entry.properties.get("DESCRIPTION") {
            self.property("DESCRIPTION", value)?;
        } else {
            self.property("DESCRIPTION", &self.body(entry))?;
        }
        for key in ["LOCATION", "ID"] {
            if let Some(value) = entry.properties.get(key) {
                self.property(if key == "ID" { "X-ORG-ID" } else { key }, value)?;
            }
        }
        self.class(entry)?;
        self.categories(entry)
    }
    fn categories(&mut self, entry: &Entry) -> Result<(), String> {
        if !self.snapshot.options.tags {
            return Ok(());
        }
        if let Some(tags) = entry
            .raw
            .split_whitespace()
            .last()
            .filter(|s| s.starts_with(':') && s.ends_with(':'))
        {
            for tag in tags.split(':').filter(|s| !s.is_empty()) {
                self.property("CATEGORIES", tag)?;
            }
        }
        Ok(())
    }
    fn class(&mut self, entry: &Entry) -> Result<(), String> {
        if let Some(class) = entry.properties.get("CLASS") {
            let class = class.to_ascii_uppercase();
            if !["PUBLIC", "PRIVATE", "CONFIDENTIAL"].contains(&class.as_str()) {
                return Err("Calendar CLASS requires PUBLIC, PRIVATE or CONFIDENTIAL".into());
            }
            self.raw("CLASS", &class)?;
        }
        Ok(())
    }
    fn body(&self, entry: &Entry) -> String {
        let mut value = String::new();
        let mut at = entry.body.start;
        for range in &entry.drawers {
            value.push_str(&self.snapshot.text.byte_slice(at..range.start).to_string());
            at = range.end;
        }
        value.push_str(
            &self
                .snapshot
                .text
                .byte_slice(at..entry.body.end)
                .to_string(),
        );
        value.trim().into()
    }
    fn todo(&mut self, entry: &Entry, id: &str) -> Result<(), String> {
        self.begin(entry, &format!("{id}-todo"), "VTODO")?;
        self.raw(
            "STATUS",
            if entry.task == Some(true) {
                "COMPLETED"
            } else {
                "NEEDS-ACTION"
            },
        )?;
        self.todo_dates(entry)?;
        self.todo_rule(entry)?;
        self.raw("END", "VTODO")
    }
    fn todo_dates(&mut self, entry: &Entry) -> Result<(), String> {
        if let (Some(start), Some(end)) = (&entry.scheduled, &entry.deadline) {
            dates::compatible(self.value(&start.start)?, self.value(&end.start)?)?;
        }
        for (key, stamp) in [("DTSTART", &entry.scheduled), ("DUE", &entry.deadline)] {
            if let Some(value) = stamp {
                self.date_property(key, self.value(&value.start)?)?;
                self.property(&format!("X-ORG-{key}"), &value.format())?;
            }
        }
        Ok(())
    }
    fn planning_events(&mut self, entry: &Entry, id: &str) -> Result<(), String> {
        for (name, stamp) in [
            ("scheduled", &entry.scheduled),
            ("deadline", &entry.deadline),
        ] {
            if let Some(stamp) = stamp {
                self.event(entry, &format!("{id}-{name}"), stamp)?;
            }
        }
        Ok(())
    }
    fn event(&mut self, entry: &Entry, id: &str, value: &TimestampRange) -> Result<(), String> {
        self.begin(entry, id, "VEVENT")?;
        self.event_dates(value)?;
        self.recurrence(value, None)?;
        self.property("X-ORG-TIMESTAMP", &value.format())?;
        self.raw("END", "VEVENT")
    }
}
