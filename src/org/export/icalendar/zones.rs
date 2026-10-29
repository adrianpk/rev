//! Source-selected named zones are observed only during owned preparation.
use super::*;
use crate::org::time::{Zone, ZoneEnvironment, ZoneSelection};
impl Snapshot {
    pub(in crate::org::export) fn with_calendar_zones(
        mut self,
        cancel: &Cancellation,
    ) -> Result<Self, String> {
        if self.request.backend == ExportBackend::Icalendar {
            self.calendar_zones = capture(&self, cancel)?;
        }
        Ok(self)
    }
}
fn capture(snapshot: &Snapshot, cancel: &Cancellation) -> Result<BTreeMap<String, Zone>, String> {
    let entries = observe::entries(snapshot, cancel)?;
    let environment = ZoneEnvironment::system();
    let mut zones = BTreeMap::new();
    for entry in entries.iter().filter(|e| !e.excluded && e.has_data()) {
        let Some(name) = entry.properties.get("TIMEZONE").cloned() else {
            continue;
        };
        if zones.contains_key(&name) {
            continue;
        }
        if zones.len() >= 64 {
            return Err("Calendar export exceeds 64 explicit zones".into());
        }
        let selection = selection(&name)?;
        zones.insert(name, Zone::observe(&selection, &environment)?);
        document::check(cancel)?;
    }
    Ok(zones)
}

fn selection(name: &str) -> Result<ZoneSelection, String> {
    match ZoneSelection::parse(name)? {
        ZoneSelection::Utc => Ok(ZoneSelection::Utc),
        ZoneSelection::Iana(name) => Ok(ZoneSelection::Iana(name)),
        _ => Err("Calendar TIMEZONE requires UTC or an IANA name".into()),
    }
}
