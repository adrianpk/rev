use super::*;
use crate::org::time::OperationTime;

pub(in crate::session::outline) struct End {
    pub(super) edit: Replacement,
    pub(super) line: Span,
    pub(super) owner: Point,
}
impl Session {
    pub(super) fn clock_same_task(&self, model: &Outline, index: usize) -> Result<bool, String> {
        let Some(active) = &self.active_clock else {
            return Ok(false);
        };
        let owner = model.clock_heading(self.buffer().map_err(|e| e.to_string())?, index)?;
        if active.owner.stamp.buffer != owner.stamp.buffer {
            return Ok(false);
        }
        self.clock_current()?;
        Ok(active.owner == owner)
    }
    pub(super) fn clock_current(&self) -> Result<&Active, String> {
        let active = self.active_clock.as_ref().ok_or("No active Org clock")?;
        let buffer = self
            .buffers
            .get(active.source.stamp.buffer)
            .map_err(|e| e.to_string())?;
        buffer
            .slice(&active.source)
            .map_err(|e| format!("Org clock source changed: {e}; open interval retained"))?;
        Ok(active)
    }
    pub(in crate::session::outline) fn clock_ended(
        &self,
        instant: OperationTime,
    ) -> Result<Option<End>, String> {
        if self.active_clock.is_none() {
            return Ok(None);
        }
        let active = self.clock_current()?;
        let buffer = self
            .buffers
            .get(active.source.stamp.buffer)
            .map_err(|e| e.to_string())?;
        let record = active.interval.close(instant)?;
        let remove = self.buffers.snapshot().settings().org.clock.remove_zero
            && record.duration.minutes() == 0.0;
        let edit = if remove {
            Replacement {
                source: active.cleanup.as_ref().unwrap_or(&active.source).clone(),
                text: String::new(),
            }
        } else {
            Replacement {
                source: active.source.clone(),
                text: format!("{}{}", record.source, buffer.newline()),
            }
        };
        Ok(Some(End {
            edit,
            line: active.source.clone(),
            owner: active.owner,
        }))
    }
    pub(super) fn switch_clock(
        &mut self,
        mut insertion: ClockInsertion,
        interval: ClockInterval,
        instant: OperationTime,
    ) -> Result<(), String> {
        let Some(end) = self.clock_ended(instant)? else {
            return self.publish_clock_in(insertion, interval);
        };
        adjust_insertion(&mut insertion, &end.edit)?;
        let owner = Point {
            stamp: insertion.edit.source.stamp,
            position: insertion.owner,
        };
        let previous = end.owner;
        let mut edits = vec![end.edit, insertion.edit];
        edits.sort_by_key(|e| (e.source.stamp.buffer, e.source.range.start));
        let mapped = self.publish_clock_batch(&edits, &[owner, previous])?;
        self.remember_clock(mapped[1]);
        let mapped = mapped[0];
        let buffer = self
            .buffers
            .get(mapped.stamp.buffer)
            .map_err(|e| e.to_string())?;
        self.active_clock = Some(Active {
            source: buffer
                .span(insertion.record)
                .expect("prepared switched clock range"),
            cleanup: insertion
                .cleanup
                .map(|r| buffer.span(r).expect("prepared switched clock wrapper")),
            owner: mapped,
            acknowledged: interval.started(),
            interval,
        });
        self.retained_clock = None;
        self.remember_clock(mapped);
        self.checkpoint_clock()
    }
}
fn adjust_insertion(insertion: &mut ClockInsertion, closed: &Replacement) -> Result<(), String> {
    if insertion.edit.source.stamp.buffer != closed.source.stamp.buffer
        || closed.source.range.start > insertion.edit.source.range.start
    {
        return Ok(());
    }
    let removed = closed.source.range.len();
    let added = closed.text.chars().count();
    for range in std::iter::once(&mut insertion.record).chain(insertion.cleanup.iter_mut()) {
        range.start = range
            .start
            .checked_sub(removed)
            .and_then(|v| v.checked_add(added))
            .ok_or("Org clock source offset overflows")?;
        range.end = range
            .end
            .checked_sub(removed)
            .and_then(|v| v.checked_add(added))
            .ok_or("Org clock source offset overflows")?;
    }
    Ok(())
}
