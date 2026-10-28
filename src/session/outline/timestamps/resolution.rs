use crate::org::time::{CivilDateTime, Resolution, ResolvedTime, TimestampRange, Zone};

pub(super) struct Validation {
    times: Vec<CivilDateTime>,
    instants: Vec<i64>,
    candidates: Vec<ResolvedTime>,
}
impl Validation {
    pub(super) fn new(value: &TimestampRange) -> Result<Self, String> {
        value.check()?;
        let mut times = Vec::new();
        for stamp in std::iter::once(&value.start).chain(value.end.iter()) {
            times.extend(stamp.civil());
            times.extend(stamp.end_civil()?);
        }
        Ok(Self {
            times,
            instants: Vec::new(),
            candidates: Vec::new(),
        })
    }
    pub(super) fn advance(&mut self, zone: &Zone) -> Result<bool, String> {
        self.candidates.clear();
        while let Some(value) = self.times.get(self.instants.len()) {
            match zone.resolve(*value)? {
                Resolution::Unique(time) => self.instants.push(time.unix_seconds),
                Resolution::Ambiguous(choices) => {
                    self.candidates = choices;
                    return Ok(false);
                }
                Resolution::Nonexistent => {
                    return Err(
                        "Nonexistent Org local time; choose another date or time; C-g cancels"
                            .into(),
                    );
                }
            }
        }
        if self.instants.windows(2).any(|pair| pair[1] < pair[0]) {
            return Err("Org time range ends before it starts in the selected zone".into());
        }
        Ok(true)
    }
    pub(super) fn select(&mut self, text: &str) -> Result<(), String> {
        let choice = text
            .trim()
            .parse::<usize>()
            .ok()
            .and_then(|i| i.checked_sub(1))
            .and_then(|i| self.candidates.get(i))
            .ok_or("Choose an observed Org UTC offset number")?;
        self.instants.push(choice.unix_seconds);
        Ok(())
    }
    pub(super) fn choices(&self) -> String {
        let choices = self
            .candidates
            .iter()
            .enumerate()
            .map(|(i, time)| {
                let offset = time.offset_seconds;
                let sign = if offset < 0 { '-' } else { '+' };
                let seconds = offset.unsigned_abs();
                format!(
                    "{} UTC{sign}{:02}:{:02}:{:02}",
                    i + 1,
                    seconds / 3600,
                    seconds / 60 % 60,
                    seconds % 60
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "Ambiguous Org local time; choose 1..{}: {choices}; C-g cancels",
            self.candidates.len()
        )
    }
}
