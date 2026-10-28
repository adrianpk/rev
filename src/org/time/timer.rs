//! Session-only monotonic timers. No wall clock, file or periodic writer is owned.
mod duration;
pub use duration::TimerDuration;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerMode {
    Relative,
    Countdown,
}
#[derive(Clone, Debug)]
struct Active {
    mode: TimerMode,
    value: Duration,
    anchor: Option<Instant>,
    deadline: Option<Instant>,
}
#[derive(Clone, Debug, Default)]
pub struct Timer {
    generation: u64,
    active: Option<Active>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimerReading {
    pub mode: TimerMode,
    pub paused: bool,
    pub seconds: u64,
}
impl TimerReading {
    pub fn format(self) -> String {
        format!(
            "{}:{:02}:{:02}",
            self.seconds / 3600,
            self.seconds / 60 % 60,
            self.seconds % 60
        )
    }
}
impl Timer {
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn mode(&self) -> Option<TimerMode> {
        self.active.as_ref().map(|v| v.mode)
    }
    pub fn start(&mut self, now: Instant) -> Result<(), String> {
        if self.mode() == Some(TimerMode::Countdown) {
            return Err("Countdown timer is running; stop it first".into());
        }
        self.replace(Some(Active {
            mode: TimerMode::Relative,
            value: Duration::ZERO,
            anchor: Some(now),
            deadline: None,
        }))
    }
    pub fn countdown(&mut self, now: Instant, value: TimerDuration) -> Result<(), String> {
        if self.mode() == Some(TimerMode::Relative) {
            return Err("Relative timer is running; stop it first".into());
        }
        if value.seconds() == 0 {
            return Err("Countdown requires a positive duration".into());
        }
        let value = Duration::from_secs(value.seconds());
        let deadline = now
            .checked_add(value)
            .ok_or("Countdown deadline overflows")?;
        self.replace(Some(Active {
            mode: TimerMode::Countdown,
            value,
            anchor: Some(now),
            deadline: Some(deadline),
        }))
    }
    pub fn reading(&self, now: Instant) -> Result<Option<TimerReading>, String> {
        self.active.as_ref().map(|a| a.reading(now)).transpose()
    }
    pub fn pause_or_continue(&mut self, now: Instant) -> Result<TimerReading, String> {
        let active = self.active.as_ref().ok_or("No Org timer is running")?;
        let next = active.toggle(now)?;
        let reading = next.reading(now)?;
        self.replace(Some(next))?;
        Ok(reading)
    }
    pub fn stop(&mut self) -> Result<(), String> {
        if self.active.is_none() {
            return Err("No Org timer is running".into());
        }
        self.replace(None)
    }
    pub fn deadline(&self) -> Option<Instant> {
        self.active.as_ref().and_then(|v| v.deadline)
    }
    /// A delayed wake delivers once; no repeating deadline or catch-up remains.
    pub fn expire(&mut self, now: Instant) -> Result<bool, String> {
        if !self.deadline().is_some_and(|due| now >= due) {
            return Ok(false);
        }
        self.replace(None)?;
        Ok(true)
    }
    fn replace(&mut self, next: Option<Active>) -> Result<(), String> {
        let generation = self
            .generation
            .checked_add(1)
            .ok_or("Timer generation exhausted")?;
        self.active = next;
        self.generation = generation;
        Ok(())
    }
}
impl Active {
    fn value(&self, now: Instant) -> Result<Duration, String> {
        let Some(anchor) = self.anchor else {
            return Ok(self.value);
        };
        let elapsed = now
            .checked_duration_since(anchor)
            .ok_or("Monotonic timer observation moved backwards")?;
        match self.mode {
            TimerMode::Relative => self
                .value
                .checked_add(elapsed)
                .ok_or("Relative timer overflows".into()),
            TimerMode::Countdown => Ok(self.value.saturating_sub(elapsed)),
        }
    }
    fn reading(&self, now: Instant) -> Result<TimerReading, String> {
        let value = self.value(now)?;
        let seconds = value.as_secs()
            + u64::from(self.mode == TimerMode::Countdown && value.subsec_nanos() != 0);
        Ok(TimerReading {
            mode: self.mode,
            paused: self.anchor.is_none(),
            seconds,
        })
    }
    fn toggle(&self, now: Instant) -> Result<Self, String> {
        let value = self.value(now)?;
        if self.anchor.is_some() {
            return Ok(Self {
                mode: self.mode,
                value,
                anchor: None,
                deadline: None,
            });
        }
        let deadline = if self.mode == TimerMode::Countdown {
            Some(
                now.checked_add(value)
                    .ok_or("Countdown deadline overflows")?,
            )
        } else {
            None
        };
        Ok(Self {
            mode: self.mode,
            value,
            anchor: Some(now),
            deadline,
        })
    }
}
