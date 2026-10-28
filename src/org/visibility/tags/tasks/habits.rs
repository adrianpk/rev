mod history;
use super::*;
use crate::org::time::{HabitSchedule, TimestampRange};

impl Outline {
    pub(crate) fn habit_schedule(
        &self,
        buffer: &Buffer,
        index: usize,
        policy: &crate::config::Todo,
    ) -> Result<Option<HabitSchedule>, String> {
        let workflow = self.task_workflow(policy)?;
        if !workflow
            .state(self.task_word(index))
            .is_some_and(|s| !s.done)
            || self.task_property(buffer, index, "STYLE")? != "habit"
        {
            return Ok(None);
        }
        self.habit_timestamp(buffer, index).map(Some)
    }
    fn habit_timestamp(&self, buffer: &Buffer, index: usize) -> Result<HabitSchedule, String> {
        let planning = self.tags.metadata.entries[index]
            .planning
            .clone()
            .ok_or("Org habit requires repeating SCHEDULED")?;
        let target = super::super::timestamps::in_planning(buffer, planning, "SCHEDULED")?;
        if !target.exists() {
            return Err("Org habit requires repeating SCHEDULED".into());
        }
        let text = buffer
            .slice(&target.source)
            .map_err(|e| e.to_string())?
            .to_string();
        let range = TimestampRange::parse(&text)?;
        if range.end.is_some() {
            return Err("Org habit SCHEDULED requires a single timestamp".into());
        }
        HabitSchedule::new(range.start)
    }
}

impl TaskChange {
    pub(super) fn habit_reset(&self, habit: bool, state: &str) -> Result<(), String> {
        if habit && !self.workflow.state(state).is_some_and(|s| !s.done) {
            return Err("Org habit reset must select an unfinished task state".into());
        }
        Ok(())
    }
}

impl Outline {
    pub(super) fn habit_sources(
        &self,
        buffer: &Buffer,
        index: usize,
        habit: bool,
        values: &mut [super::repeat::RepeatSource],
    ) -> Result<(), String> {
        if !habit {
            return Ok(());
        }
        let planning = self.tags.metadata.entries[index]
            .planning
            .clone()
            .ok_or("Missing habit planning")?;
        let target = super::super::timestamps::in_planning(buffer, planning, "SCHEDULED")?;
        for value in values {
            value.habit_schedule = value.source == target.source;
        }
        Ok(())
    }
}
