use super::super::{CivilDateTime, OperationTime};
use super::{Zone, ZoneEnvironment, ZoneSelection};
use std::time::SystemTime;

/// One operation owns its captured instant, effective date and full zone rules.
/// Later environment/configuration/file changes affect only subsequent captures.
#[derive(Clone, Debug)]
pub struct ZonedOperation {
    instant: OperationTime,
    zone: Zone,
    now: CivilDateTime,
}
impl ZonedOperation {
    pub fn capture(
        selection: &ZoneSelection,
        env: &ZoneEnvironment,
        clock: impl FnOnce() -> Result<SystemTime, String>,
    ) -> Result<Self, String> {
        let instant = OperationTime::capture(clock)?;
        let zone = Zone::observe(selection, env)?;
        let now = zone.civil_at(instant.unix_seconds())?;
        Ok(Self { instant, zone, now })
    }
    pub fn instant(&self) -> OperationTime {
        self.instant
    }
    pub fn zone(&self) -> &Zone {
        &self.zone
    }
    pub fn now(&self) -> CivilDateTime {
        self.now
    }
}
