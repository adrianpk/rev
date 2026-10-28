use super::*;
use crate::org::agenda::HabitScan;
use crate::org::time::Date;

impl Outline {
    pub(crate) fn habit_scan(
        &self,
        buffer: &Buffer,
        today: Date,
        policy: &crate::config::Org,
    ) -> Result<HabitScan, String> {
        self.complete().map_err(str::to_owned)?;
        crate::org::source::current(buffer, self.stamp).map_err(|e| e.to_string())?;
        let mut rows = Vec::new();
        for (index, heading) in self.headings.iter().enumerate() {
            if let Some(record) = self.habit_record(buffer, index, &policy.todo)? {
                if rows.len() == 128 {
                    return Err("Habit agenda exceeds 128 graph rows".into());
                }
                let title = super::super::edit::source(buffer, heading.title.clone())?;
                rows.push((title, record));
            }
        }
        Ok(HabitScan {
            source: buffer
                .span(0..buffer.len_chars())
                .map_err(|e| e.to_string())?,
            today,
            policy: policy.habit.clone(),
            rows,
        })
    }
}
