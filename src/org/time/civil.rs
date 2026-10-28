use super::Date;

/// A checked wall-clock time; leap seconds are not admitted by this profile.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CivilTime {
    seconds: u32,
}
impl CivilTime {
    pub fn new(hour: u8, minute: u8, second: u8) -> Result<Self, String> {
        if hour > 23 || minute > 59 || second > 59 {
            return Err("Org civil time requires 00:00:00..23:59:59".into());
        }
        Ok(Self {
            seconds: u32::from(hour) * 3600 + u32::from(minute) * 60 + u32::from(second),
        })
    }
    pub fn hour(self) -> u8 {
        (self.seconds / 3600) as u8
    }
    pub fn minute(self) -> u8 {
        (self.seconds / 60 % 60) as u8
    }
    pub fn second(self) -> u8 {
        (self.seconds % 60) as u8
    }
}
/// A civil date and time, without an implicit host zone or UTC offset.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CivilDateTime {
    pub date: Date,
    pub time: CivilTime,
}
/// Half-open ordered civil bounds [start, end). This does not imply an elapsed
/// duration: resolving a zone can cross an offset transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CivilInterval {
    start: CivilDateTime,
    end: CivilDateTime,
}
impl CivilInterval {
    pub fn new(start: CivilDateTime, end: CivilDateTime) -> Result<Self, String> {
        if end < start {
            return Err("Org civil interval ends before it starts".into());
        }
        Ok(Self { start, end })
    }
    pub fn start(self) -> CivilDateTime {
        self.start
    }
    pub fn end(self) -> CivilDateTime {
        self.end
    }
    pub fn contains(self, value: CivilDateTime) -> bool {
        self.start <= value && value < self.end
    }
}
