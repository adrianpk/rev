use super::switching::End;
use super::*;
use crate::org::TaskChange;

impl Session {
    pub(in crate::session::outline) fn clock_completes(
        &self,
        model: &Outline,
        change: &TaskChange,
    ) -> Result<bool, String> {
        if !self.buffers.snapshot().settings().org.clock.out_when_done
            || !change.workflow.state(&change.next).is_some_and(|s| s.done)
        {
            return Ok(false);
        }
        let Some(active) = &self.active_clock else {
            return Ok(false);
        };
        let owner = model.clock_heading(self.buffer().map_err(|e| e.to_string())?, change.index)?;
        if owner.stamp.buffer != active.owner.stamp.buffer {
            return Ok(false);
        }
        self.clock_current()?;
        Ok(owner == active.owner)
    }
    pub(in crate::session::outline) fn publish_clock_completion(
        &mut self,
        mut edits: Vec<Replacement>,
        end: Option<&End>,
    ) -> Result<(), String> {
        let Some(end) = end else {
            return self.publish_org_edits(&edits).map_err(|e| e.to_string());
        };
        self.clock_current()?;
        if self
            .active_clock
            .as_ref()
            .is_none_or(|a| a.source != end.line)
        {
            return Err("Org clock changed during task entry; transition cancelled".into());
        }
        let mut clock = end.edit.clone();
        if clock.text.is_empty()
            && edits.iter().any(|e| {
                e.source.range.start > clock.source.range.start
                    && e.source.range.start < clock.source.range.end
            })
        {
            clock.source = end.line.clone();
        }
        edits.push(clock);
        edits.sort_by_key(|e| e.source.range.start);
        let edits = coalesce(edits)?;
        self.publish_clock_batch(&edits, end.owner)?;
        self.active_clock = None;
        Ok(())
    }
}
fn coalesce(edits: Vec<Replacement>) -> Result<Vec<Replacement>, String> {
    let mut result: Vec<Replacement> = Vec::new();
    for edit in edits {
        let Some(previous) = result
            .last_mut()
            .filter(|e| e.source.range.start == edit.source.range.start)
        else {
            result.push(edit);
            continue;
        };
        if !previous.source.range.is_empty() && !edit.source.range.is_empty() {
            return Err("Overlapping Org clock completion metadata".into());
        }
        if previous.source.range.is_empty() {
            previous.source = edit.source;
            previous.text.push_str(&edit.text);
        } else {
            previous.text.insert_str(0, &edit.text);
        }
    }
    Ok(result)
}
