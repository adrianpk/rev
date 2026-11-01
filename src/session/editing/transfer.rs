use super::Error;
use crate::coordinates::{Cell, Coordinates, Scalar};
use crate::kill::Shape;
use crate::session::Session;

// Count expansion has one explicit native materialization bound; an ordinary
// single transfer remains valid independently of history-retention targets.
use crate::kill::EXPANSION_BYTES;

pub(super) fn repeat_text(text: &str, count: usize) -> Result<String, crate::buffer::edit::Error> {
    let length = text
        .len()
        .checked_mul(count)
        .ok_or(crate::buffer::edit::Error::Capacity)?;
    if count > 1 && length > EXPANSION_BYTES {
        return Err(crate::buffer::edit::Error::Capacity);
    }
    let mut output = String::new();
    output
        .try_reserve_exact(length)
        .map_err(|_| crate::buffer::edit::Error::Capacity)?;
    if text.is_empty() {
        return Ok(output);
    }
    output.push_str(text);
    // Doubling composes bounded text, never count-sized primitive commands.
    while output.len() < length {
        let remaining = length - output.len();
        let take = remaining.min(output.len());
        output.extend_from_within(..take);
    }
    Ok(output)
}

impl Session {
    pub(super) fn paste_transfer(&mut self, count: usize, before: bool) -> Result<bool, Error> {
        if let Some(block) = self.kills.block() {
            return self.paste_block(&block, count, before);
        }
        let text = self.kills.text().ok_or(Error::Target)?.to_string();
        if self.kills.shape() == Some(Shape::Line) {
            return self.paste_line(&text, count, before);
        }
        self.paste_linear(&text, count, before)
    }

    fn paste_linear(&mut self, text: &str, count: usize, before: bool) -> Result<bool, Error> {
        let coordinates = Coordinates::new(self.buffer()?)?;
        let point = self.view().point.position;
        let at = if before {
            point
        } else {
            coordinates.next(point)?.min(coordinates.line_end(point)?)
        };
        let text = repeat_text(text, count)?;
        Ok(self.publish_yank_at(at.0..at.0, &text, 0, None)?)
    }

    fn paste_block(
        &mut self,
        block: &crate::rectangle::Rectangle,
        count: usize,
        before: bool,
    ) -> Result<bool, Error> {
        let coordinates = Coordinates::new(self.buffer()?)?;
        let mut point = self.view().point;
        if !before {
            point.position = coordinates
                .next_observed(point.position)?
                .min(coordinates.line_end(point.position)?);
        }
        let block = block.repeated(count)?;
        let insertion = block.insertions(self.buffer()?, point)?;
        let changed = self.replace_many(&insertion.edits, Some(insertion.cursor))?;
        self.yank = None;
        self.kills.break_chain();
        Ok(changed)
    }

    fn paste_line(&mut self, text: &str, count: usize, before: bool) -> Result<bool, Error> {
        let at = self.line_paste_point(before)?;
        let prefix = self.line_paste_prefix(at, before)?;
        let mut insertion = self.line_payload(text, count, at)?;
        let newline = self.buffer()?.newline();
        let offset = if prefix { newline.chars().count() } else { 0 };
        if prefix {
            insertion.insert_str(0, newline);
        }
        let cursor = Scalar(
            at.0.checked_add(offset)
                .ok_or(crate::buffer::edit::Error::Capacity)?,
        );
        Ok(self.publish_yank_at(at.0..at.0, &insertion, 0, Some(cursor))?)
    }

    fn line_paste_point(&self, before: bool) -> Result<Scalar, Error> {
        let buffer = self.buffer()?;
        let coordinates = Coordinates::new(buffer)?;
        let point = self.view().point.position;
        if before {
            return Ok(coordinates.line_start(point)?);
        }
        let row = coordinates.to_line(point)?.row;
        if row + 1 < buffer.len_lines() {
            return Ok(coordinates.from_cell(Cell {
                row: row + 1,
                column: 0,
            })?);
        }
        Ok(coordinates.end())
    }

    fn line_paste_prefix(&self, at: Scalar, before: bool) -> Result<bool, Error> {
        if before || at.0 == 0 {
            return Ok(false);
        }
        let coordinates = Coordinates::new(self.buffer()?)?;
        let point = self.view().point.position;
        Ok(at == coordinates.end()
            && at == coordinates.line_end(point)?
            && coordinates.line_start(point)? < at)
    }

    fn line_payload(&self, text: &str, count: usize, at: Scalar) -> Result<String, Error> {
        let buffer = self.buffer()?;
        let newline = buffer.newline();
        let mut unit = text.replace("\r\n", "\n").replace('\n', newline);
        let terminated = unit.ends_with('\n');
        if !terminated {
            unit.push_str(newline);
        }
        let mut insertion = repeat_text(&unit, count)?;
        if !terminated && at.0 == buffer.len_chars() {
            insertion.truncate(insertion.len() - newline.len());
        }
        Ok(insertion)
    }
}
