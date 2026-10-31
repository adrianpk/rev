use std::ops::Range;

use crate::buffer::Buffer;
use crate::coordinates::{Cell, Coordinates, Scalar};

use super::{Error, Fragment, Size, View, wrap::Line};

impl View {
    pub(crate) fn text_width(&self, buffer: &Buffer, size: Size) -> Result<u16, Error> {
        if !size.usable() {
            return Ok(0);
        }
        let content = crate::window::Area::new(0, 0, size.width, size.text_rows())
            .ok_or(crate::coordinates::Error::Overflow)?;
        Ok(self.gutter_geometry(buffer, content)?.text().width())
    }

    pub(super) fn visual_fragment(&self, buffer: &Buffer, width: u16) -> Result<Fragment, Error> {
        let row = self.cell(buffer)?.row;
        let mut line = Line::new(buffer, row, width)?;
        while let Some(fragment) = line.next()? {
            if fragment.contains(self.point.position, width) {
                return Ok(fragment);
            }
        }
        Err(crate::coordinates::Error::Scalar(self.point.position).into())
    }

    // Emacs leaves C-e on the wrap character. On an actual source EOL it
    // reaches the insertion boundary instead; display continuations are not LF.
    fn visual_end(buffer: &Buffer, fragment: &Fragment) -> Result<Scalar, Error> {
        if fragment.end_of_line {
            Ok(fragment.source.end)
        } else {
            Ok(Coordinates::new(buffer)?.previous(fragment.source.end)?)
        }
    }

    pub(crate) fn visual_boundary(
        &self,
        buffer: &Buffer,
        width: u16,
        end: bool,
    ) -> Result<Scalar, Error> {
        let fragment = self.visual_fragment(buffer, width)?;
        if end {
            Self::visual_end(buffer, &fragment)
        } else {
            Ok(fragment.source.start)
        }
    }

    pub(crate) fn visual_kill(&self, buffer: &Buffer, width: u16) -> Result<Range<usize>, Error> {
        let fragment = self.visual_fragment(buffer, width)?;
        let coordinates = Coordinates::new(buffer)?;
        let edge = Self::visual_end(buffer, &fragment)?;
        let point = self.point.position;
        let end = if point == edge {
            if fragment.end_of_line {
                coordinates.next(point)?
            } else {
                fragment.source.end
            }
        } else if !fragment.end_of_line
            && buffer
                .slice(&buffer.span(edge.0..fragment.source.end.0)?)?
                .chars()
                .all(|ch| ch == ' ' || ch == '\t')
        {
            fragment.source.end
        } else {
            edge
        };
        Ok(point.0..end.0)
    }

    pub(super) fn adjacent_fragment(
        &self,
        buffer: &Buffer,
        width: u16,
        current: &Fragment,
        backward: bool,
    ) -> Result<Option<Fragment>, Error> {
        let mut line = Line::new(buffer, current.line, width)?;
        let mut previous = None;
        let mut found = false;
        while let Some(fragment) = line.next()? {
            if found {
                return Ok(Some(fragment));
            }
            if fragment.source == current.source && fragment.columns == current.columns {
                if backward && previous.is_some() {
                    return Ok(previous);
                }
                if !backward {
                    found = true;
                }
            }
            previous = Some(fragment);
        }
        let projected = self
            .project_row(buffer, current.line)
            .ok_or(crate::coordinates::Error::Overflow)?;
        let neighbor = if backward {
            projected.checked_sub(1)
        } else {
            projected.checked_add(1)
        };
        let Some(source) = neighbor.and_then(|row| self.source_at(buffer, row)) else {
            return Ok(None);
        };
        let mut line = Line::new(buffer, source, width)?;
        let mut result = line.next()?;
        if backward {
            while let Some(fragment) = line.next()? {
                result = Some(fragment);
            }
        }
        Ok(result)
    }

    pub(super) fn fragment_point(
        buffer: &Buffer,
        fragment: &Fragment,
        column: usize,
    ) -> Result<Scalar, Error> {
        let coordinates = Coordinates::new(buffer)?;
        let edge = Self::visual_end(buffer, fragment)?;
        let last = coordinates.to_cell(edge)?.column;
        Ok(coordinates.from_cell(Cell {
            row: fragment.line,
            column: fragment.columns.start.saturating_add(column).min(last),
        })?)
    }

    pub(crate) fn visual_scroll(
        &mut self,
        buffer: &Buffer,
        width: u16,
        backward: bool,
        distance: usize,
    ) -> Result<(), Error> {
        if distance == 0 {
            return Ok(());
        }
        let mut fragment = self.visual_fragment(buffer, width)?;
        let point = self.cell(buffer)?;
        let column = *self
            .column
            .get_or_insert(point.column - fragment.columns.start);
        for _ in 0..distance {
            let Some(next) = self.adjacent_fragment(buffer, width, &fragment, backward)? else {
                break;
            };
            fragment = next;
        }
        self.point.position = Self::fragment_point(buffer, &fragment, column)?;
        Ok(())
    }
}
