use super::*;
use crate::coordinates::{Byte, Scalar};
use crate::org::agenda::{HabitRecord, habit_completion, habit_inactive};

impl Outline {
    pub(crate) fn habit_record(
        &self,
        buffer: &Buffer,
        index: usize,
        policy: &crate::config::Todo,
    ) -> Result<Option<HabitRecord>, String> {
        let Some(schedule) = self.habit_schedule(buffer, index, policy)? else {
            return Ok(None);
        };
        let heading = &self.headings[index];
        let source = buffer
            .byte_span(heading.start..heading.section)
            .map_err(|e| e.to_string())?;
        if heading.section - heading.start > 262144 {
            return Err("Org habit observation exceeds a 262144-byte entry".into());
        }
        let mut record = HabitRecord {
            source,
            schedule,
            last_repeat: None,
            completions: Vec::new(),
            unresolved: 0,
        };
        let last = self.task_property(buffer, index, "LAST_REPEAT")?;
        if !last.is_empty() {
            match habit_inactive(&last) {
                Ok(value) => record.last_repeat = Some(value),
                Err(_) => record.unresolved += 1,
            }
        }
        self.habit_history(buffer, index, policy, &mut record)?;
        Ok(Some(record))
    }
    fn habit_history(
        &self,
        buffer: &Buffer,
        index: usize,
        policy: &crate::config::Todo,
        record: &mut HabitRecord,
    ) -> Result<(), String> {
        let workflow = self.task_workflow(policy)?;
        let text = buffer
            .slice(&record.source)
            .map_err(|e| e.to_string())?
            .to_string();
        let mut byte = self.headings[index].start;
        let mut count = 0;
        for line in text.split_inclusive('\n') {
            if self.habit_history_line(buffer, byte, line)? {
                match habit_completion(line, &workflow) {
                    Ok(Some(value)) => {
                        record.completions.push(value);
                        count += 1;
                    }
                    Err(_) => {
                        record.unresolved += 1;
                        count += 1;
                    }
                    Ok(None) => (),
                }
            }
            if count > 4096 {
                return Err("Org habit observation exceeds 4096 completion records".into());
            }
            byte += line.len();
        }
        Ok(())
    }
    fn habit_history_line(&self, buffer: &Buffer, byte: usize, line: &str) -> Result<bool, String> {
        if !line.trim_start().starts_with("- ") && !line.trim_start().starts_with("-\t") {
            return Ok(false);
        }
        if line.len() > 4096 {
            return Err("Org habit history line exceeds 4096 UTF-8 bytes".into());
        }
        let point: Scalar = Coordinates::new(buffer)
            .map_err(|e| e.to_string())?
            .from_byte(Byte(byte))
            .map_err(|e| e.to_string())?;
        self.timestamp_context(buffer, point)
    }
}
