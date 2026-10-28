use super::*;
use crate::{
    coordinates::Scalar,
    org::{Outline, time::TimerDuration},
};
impl Outline {
    pub(crate) fn timed_list_at(&self, buffer: &Buffer, point: Scalar) -> Result<bool, String> {
        let (byte, eof) = super::operation::location(buffer, point)?;
        let Some(index) = self.lists.at(byte, eof) else {
            return Ok(false);
        };
        timed(&self.lists.items[index], buffer)
    }
    pub(crate) fn timer_item_edit(
        &self,
        buffer: &Buffer,
        point: Scalar,
        data: (&str, &crate::config::Todo),
    ) -> Result<ListEdit, String> {
        let (byte, eof) = self.admit_timer_item(buffer, point)?;
        if let Some(index) = self.lists.at(byte, eof) {
            if !timed(&self.lists.items[index], buffer)? {
                return Err("This is not an Org timer list".into());
            }
            let mut plan = self.list_edit(
                buffer,
                crate::command::OrgCommand::MetaReturn,
                point,
                data.1,
            )?;
            fill(&mut plan, data.0)?;
            return Ok(plan);
        }
        new_item(buffer, point, data.0)
    }
    fn admit_timer_item(&self, buffer: &Buffer, point: Scalar) -> Result<(usize, bool), String> {
        let location = self.admit_list(buffer, point)?;
        if buffer.mark_active() {
            return Err("Timer item requires unselected source".into());
        }
        if !self.timestamp_context(buffer, point)? || self.timestamp_task_header(buffer, point) {
            return Err("Timer item requires ordinary Org body/list source".into());
        }
        Ok(location)
    }
}
fn timed(item: &Item, buffer: &Buffer) -> Result<bool, String> {
    if !item.description || item.checkbox.is_some() || item.header.len() > 4096 {
        return Ok(false);
    }
    let body = source_text(buffer, item.content..item.header.end)?;
    let Some((tag, _)) = body.split_once(" :: ") else {
        return Ok(false);
    };
    Ok(TimerDuration::stamp_seconds(tag).is_ok())
}
fn fill(plan: &mut ListEdit, value: &str) -> Result<(), String> {
    let edit = plan
        .edits
        .iter_mut()
        .find(|e| e.source.range.is_empty())
        .ok_or("Timer list insertion is unavailable")?;
    let at = edit
        .text
        .find(" :: ")
        .ok_or("Timer list needs a description separator")?;
    edit.text.insert_str(at, value);
    if let Some(cursor) = &mut plan.cursor {
        cursor.0 += value.chars().count() + 4;
    }
    Ok(())
}
fn new_item(buffer: &Buffer, point: Scalar, value: &str) -> Result<ListEdit, String> {
    let c = crate::coordinates::Coordinates::new(buffer).map_err(|e| e.to_string())?;
    let start = c.line_start(point).map_err(|e| e.to_string())?;
    let end = c.line_end(point).map_err(|e| e.to_string())?;
    let span = buffer.span(start.0..end.0).map_err(|e| e.to_string())?;
    let line = buffer.slice(&span).map_err(|e| e.to_string())?;
    let indent = line.chars().take_while(|c| matches!(c, ' ' | '\t')).count();
    let at = start.0 + indent;
    let text = format!("- {value} :: ");
    let cursor = Scalar(at + text.chars().count());
    Ok(ListEdit {
        edits: vec![Replacement {
            source: buffer.span(at..at).map_err(|e| e.to_string())?,
            text,
        }],
        order: Vec::new(),
        cursor: Some(cursor),
    })
}
