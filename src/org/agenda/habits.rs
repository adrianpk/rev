//! Disposable habit observations from canonical Org source. Absent or unresolved
//! logs never become inferred completions; LAST_REPEAT is separate metadata.
mod history;
use crate::{
    buffer::{Buffer, Span},
    coordinates::Scalar,
    org::{
        Cancellation, Structure,
        time::{Date, HabitSchedule, Timestamp},
    },
};
pub(crate) use history::{completion, inactive};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HabitObservation {
    Unrecorded,
    Completed(usize),
}
#[derive(Clone, Debug)]
pub struct HabitRecord {
    pub source: Span,
    pub schedule: HabitSchedule,
    pub last_repeat: Option<Timestamp>,
    pub completions: Vec<Timestamp>,
    pub unresolved: usize,
}
impl HabitRecord {
    /// Observe one selected heading, never children or literal examples. Both
    /// source revision and complete structure are required; no IO or edits occur.
    pub fn observe(
        buffer: &Buffer,
        structure: &Structure,
        point: Scalar,
        policy: &crate::config::Todo,
        cancel: &Cancellation,
    ) -> Result<Option<Self>, String> {
        crate::org::source::current(buffer, structure.stamp()).map_err(|e| e.to_string())?;
        let text = crate::org::Source::new(buffer)
            .snapshot()
            .map_err(|e| e.to_string())?;
        let model = crate::org::Outline::observe(structure, &text, cancel);
        let index = model.tag_target(buffer, point)?;
        let record = model.habit_record(buffer, index, policy)?;
        if cancel.is_cancelled() {
            return Err("Org habit observation cancelled".into());
        }
        Ok(record)
    }
    pub fn observation(&self, day: Date) -> HabitObservation {
        let count = self.completions.iter().filter(|s| s.date() == day).count();
        if count == 0 {
            HabitObservation::Unrecorded
        } else {
            HabitObservation::Completed(count)
        }
    }
}

mod graph;
pub use graph::{HabitCell, HabitGraph, HabitState};
mod view;
pub(crate) use view::HabitScan;
pub use view::{HabitAgenda, HabitRow};
