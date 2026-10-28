use super::*;
use crate::org::time::{RepeatInterval, RepeatMode, RepeatUnit};
mod past;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HabitState {
    Clear,
    Ready,
    Alert,
    Overdue,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HabitCell {
    pub date: Date,
    pub state: HabitState,
    pub completed: bool,
    pub today: bool,
    pub future: bool,
}
impl HabitCell {
    /// ASCII status remains readable with NO_COLOR; actual completions and today
    /// have Org's familiar markers. Completion takes precedence on today.
    /// Org dims past unmarked non-overdue days as well as future days, so an
    /// available day cannot visually claim that a completion was recorded.
    pub fn subdued(&self) -> bool {
        self.future || !self.today && !self.completed && self.state != HabitState::Overdue
    }
    pub fn glyph(&self) -> char {
        if self.completed {
            return '*';
        }
        if self.today {
            return '!';
        }
        match self.state {
            HabitState::Clear => '.',
            HabitState::Ready => '+',
            HabitState::Alert => '?',
            HabitState::Overdue => '-',
        }
    }
}
#[derive(Clone, Debug)]
pub struct HabitGraph {
    pub cells: Vec<HabitCell>,
    pub column: usize,
    pub unresolved: usize,
}
impl HabitRecord {
    /// A bounded disposable graph. Due states derive from the selected schedule;
    /// only actual source logs supply completion marks. No source write or IO.
    pub fn graph(&self, today: Date, policy: &crate::config::Habit) -> Result<HabitGraph, String> {
        if policy.preceding_days > 366 || policy.following_days > 366 || policy.graph_column > 4096
        {
            return Err("Org habit graph exceeds 366-day sides or column 4096".into());
        }
        let past = past::Past::new(self)?;
        let mut cells = Vec::with_capacity(policy.preceding_days + policy.following_days + 1);
        for offset in -(policy.preceding_days as i64)..=policy.following_days as i64 {
            let date = today.add_days(offset)?;
            cells.push(past.cell(date, today, policy.done_always_green)?);
        }
        Ok(HabitGraph {
            cells,
            column: policy.graph_column,
            unresolved: self.unresolved,
        })
    }
}
/// Org's graph uses floored daily periods (month 30.4, year 365.25). This is
/// display reconstruction only; source repetition retains calendar arithmetic.
fn days(interval: RepeatInterval) -> Result<i64, String> {
    let n = i64::from(interval.count());
    Ok(match interval.unit() {
        RepeatUnit::Day => n,
        RepeatUnit::Week => n * 7,
        RepeatUnit::Month => n * 304 / 10,
        RepeatUnit::Year => n * 1461 / 4,
        RepeatUnit::Hour => return Err("Hourly period cannot define a habit graph".into()),
    })
}
