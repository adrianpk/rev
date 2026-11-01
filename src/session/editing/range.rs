use super::{Error, Selection};
use crate::command::editing::{Action, Object, Operand};
use crate::command::navigation::Target;
use crate::command::{Editing, Navigation, Scroll};
use crate::coordinates::{Cell, Coordinates, Scalar};
use crate::kill::Shape;
use crate::session::Session;
use std::ops::Range;

impl Session {
    pub(super) fn editing_range(&self, request: Editing) -> Result<Option<Selection>, Error> {
        let operand = request.operand.or_else(|| end_operand(request));
        let Some(operand) = operand else {
            return Ok(None);
        };
        let buffer = self.buffer()?;
        self.view().check_point(buffer)?;
        let (range, shape) = match operand {
            Operand::Lines => (self.operator_lines(request.amount())?, Shape::Line),
            Operand::Object(object) => return self.object_selection(object, request.amount()),
            Operand::Motion(navigation) => return self.operator_motion(navigation),
            Operand::Resolved { origin, end } => {
                if origin != self.view().point {
                    return Err(Error::Target);
                }
                (
                    origin.position.0.min(end.0)..origin.position.0.max(end.0),
                    Shape::Linear,
                )
            }
        };
        Ok(Some(Selection {
            source: buffer.span(range)?,
            shape,
            backward: false,
        }))
    }

    fn operator_motion(&self, navigation: Navigation) -> Result<Option<Selection>, Error> {
        let Some(target) = self.operator_destination(navigation)? else {
            return Ok(None);
        };
        let point = self.view().point.position;
        let linewise = matches!(
            navigation.target,
            Target::Line(_)
                | Target::Buffer(_)
                | Target::Page { .. }
                | Target::Screen(_)
                | Target::Paragraph(_)
        );
        let (range, shape) = if linewise {
            (self.logical_range(point, target)?, Shape::Line)
        } else {
            (point.0.min(target.0)..point.0.max(target.0), Shape::Linear)
        };
        Ok(Some(Selection {
            source: self.buffer()?.span(range)?,
            shape,
            backward: target < point,
        }))
    }

    fn object_selection(&self, object: Object, count: usize) -> Result<Option<Selection>, Error> {
        let buffer = self.buffer()?;
        let range =
            Coordinates::new(buffer)?.text_object(self.view().point.position, object, count)?;
        let Some(range) = range else { return Ok(None) };
        let shape = if matches!(object, Object::Paragraph { .. }) {
            Shape::Line
        } else {
            Shape::Linear
        };
        Ok(Some(Selection {
            source: buffer.span(range)?,
            shape,
            backward: false,
        }))
    }

    fn operator_destination(&self, navigation: Navigation) -> Result<Option<Scalar>, Error> {
        if let Target::Search(search) = navigation.target {
            return Ok(self.search_target(search, navigation.amount())?);
        }
        if matches!(navigation.target, Target::Find(_) | Target::RepeatFind(_)) {
            return Ok(self.navigation_find(navigation)?.map(|found| found.point));
        }
        if matches!(navigation.target, Target::Align(_)) {
            return Err(Error::Target);
        }
        Ok(Some(self.navigation_view(navigation)?.point.position))
    }

    fn operator_lines(&self, count: usize) -> Result<Range<usize>, Error> {
        let mut candidate = self.view().clone();
        candidate.scroll(self.buffer()?, Scroll::NextLine, count.saturating_sub(1))?;
        self.logical_range(self.view().point.position, candidate.point.position)
    }

    fn logical_range(&self, first: Scalar, last: Scalar) -> Result<Range<usize>, Error> {
        let coordinates = Coordinates::new(self.buffer()?)?;
        let first = coordinates.to_line(first)?.row;
        let last = coordinates.to_line(last)?.row;
        let start = coordinates.from_cell(Cell {
            row: first.min(last),
            column: 0,
        })?;
        let last = first.max(last);
        let end = if last + 1 < self.buffer()?.len_lines() {
            coordinates.from_cell(Cell {
                row: last + 1,
                column: 0,
            })?
        } else {
            coordinates.end()
        };
        Ok(start.0..end.0)
    }
}

fn end_operand(request: Editing) -> Option<Operand> {
    matches!(
        request.action,
        Action::DeleteEnd | Action::ChangeEnd | Action::CopyEnd
    )
    .then(|| {
        let mut navigation = Navigation::new(Target::LineEnd);
        navigation.count = request.count;
        Operand::Motion(navigation)
    })
}
