use crate::buffer::{Buffer, edit::Replacement};
use crate::coordinates::{Byte, Coordinates, LineByte, Scalar};

use super::Outline;

pub(crate) struct Insertion {
    pub(crate) edit: Replacement,
    pub(crate) cursor: Scalar,
}

impl Outline {
    pub(crate) fn insertion(&self, buffer: &Buffer, point: Scalar) -> Result<Insertion, String> {
        self.complete().map_err(str::to_owned)?;
        crate::org::source::current(buffer, self.stamp).map_err(|e| e.to_string())?;
        let coordinates = Coordinates::new(buffer).map_err(|e| e.to_string())?;
        let byte = coordinates.to_byte(point).map_err(|e| e.to_string())?.0;
        let blocked = self.blocked.partition_point(|range| range.end <= byte);
        if self
            .blocked
            .get(blocked)
            .is_some_and(|range| range.contains(&byte))
        {
            return Err("Org heading insertion is unavailable in this source context".into());
        }
        self.at_point(buffer, &coordinates, point)
    }

    fn at_point(
        &self,
        buffer: &Buffer,
        coordinates: &Coordinates<'_>,
        point: Scalar,
    ) -> Result<Insertion, String> {
        let byte = coordinates.to_byte(point).map_err(|e| e.to_string())?.0;
        let heading = self.heading(byte).map(|i| &self.headings[i]);
        let level = heading.map_or(1, |h| h.level);
        let line = coordinates.to_line(point).map_err(|e| e.to_string())?;
        let start = coordinates
            .from_line(LineByte {
                row: line.row,
                column: 0,
            })
            .map_err(|e| e.to_string())?;
        let mut position = point;
        if let Some(h) = heading.filter(|h| byte > h.start && byte < h.body) {
            position = coordinates
                .from_byte(Byte(byte.clamp(h.title.start, h.title.end)))
                .map_err(|e| e.to_string())?;
        }
        let above = heading.is_some_and(|h| h.start == byte);
        make(buffer, position, start, above, level)
    }
}

fn make(
    buffer: &Buffer,
    point: Scalar,
    start: Scalar,
    above: bool,
    level: usize,
) -> Result<Insertion, String> {
    let newline = buffer.newline();
    let mut text = String::new();
    if point != start {
        text.push_str(newline);
    }
    text.push_str(&"*".repeat(level));
    text.push(' ');
    let cursor = Scalar(point.0 + text.chars().count());
    if above || terminated_eof(buffer, point).map_err(|e| e.to_string())? {
        text.push_str(newline);
    }
    let source = buffer.span(point.0..point.0).map_err(|e| e.to_string())?;
    Ok(Insertion {
        edit: Replacement { source, text },
        cursor,
    })
}

fn terminated_eof(buffer: &Buffer, point: Scalar) -> Result<bool, crate::buffer::Error> {
    if point.0 != buffer.len_chars() || point.0 == 0 {
        return Ok(false);
    }
    let tail = buffer.span(point.0 - 1..point.0)?;
    Ok(buffer.slice(&tail)?.chars().next() == Some('\n'))
}
