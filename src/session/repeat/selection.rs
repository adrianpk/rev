use crate::buffer::Visual;
use crate::command::{
    Editing,
    editing::{Action, Operand},
    navigation::Target,
};
use crate::coordinates::{Coordinates, Scalar};
use crate::kill::Shape;
use crate::session::{
    Session,
    editing::{Error, Selection},
};

#[derive(Clone, Copy)]
pub(super) enum Geometry {
    Linear { shape: Shape, count: u32 },
    Block { rows: usize, width: usize },
}

impl Session {
    pub(super) fn repeat_geometry(&self, shape: Visual) -> Result<Geometry, Error> {
        let buffer = self.buffer()?;
        if shape == Visual::Block {
            let block = crate::rectangle::Selection::new(buffer, self.view().point)?
                .ok_or(Error::Target)?;
            let (rows, width) = block.dimensions();
            return Ok(Geometry::Block { rows, width });
        }
        self.linear_repeat_geometry(shape)
    }

    fn linear_repeat_geometry(&self, shape: Visual) -> Result<Geometry, Error> {
        let buffer = self.buffer()?;
        let source = buffer.region(self.view().point)?.ok_or(Error::Target)?;
        let coordinates = Coordinates::new(buffer)?;
        let rows = region_rows(&coordinates, &source)?;
        let (shape, count) = if shape == Visual::Line {
            (Shape::Line, rows.end() - rows.start() + 1)
        } else {
            (
                Shape::Linear,
                region_graphemes(&coordinates, source.range, rows)?,
            )
        };
        Ok(Geometry::Linear {
            shape,
            count: u32::try_from(count).map_err(|_| Error::Target)?,
        })
    }

    pub(super) fn repeat_selection(
        &self,
        geometry: Geometry,
        count: Option<u32>,
    ) -> Result<Selection, Error> {
        let Geometry::Linear {
            shape,
            count: amount,
        } = geometry
        else {
            return Err(Error::Target);
        };
        let amount = count.unwrap_or(amount).max(1);
        if shape == Shape::Line {
            let mut request = Editing::new(Action::Delete);
            request.count = Some(amount);
            request.operand = Some(Operand::Lines);
            return self.editing_range(request)?.ok_or(Error::Target);
        }
        let start = self.view().point.position;
        let end = Coordinates::new(self.buffer()?)?.navigation(
            start,
            Target::Character(crate::search::Direction::Forward),
            amount as usize,
        )?;
        Ok(Selection {
            source: self.buffer()?.span(start.0..end.0)?,
            shape,
            backward: false,
        })
    }

    pub(super) fn repeat_block(
        &mut self,
        request: Editing,
        geometry: Geometry,
        text: &str,
    ) -> Result<bool, Error> {
        let selection = self.repeat_block_selection(geometry, request.count)?;
        let change = self.prepare_repeat_block(&selection, request, text)?;
        let changed = self.replace_many(&change.edits, Some(change.cursor))?;
        if changed {
            self.kills.publish_transfer(change.entry);
        }
        Ok(changed)
    }

    pub(super) fn repeat_block_selection(
        &self,
        geometry: Geometry,
        count: Option<u32>,
    ) -> Result<crate::rectangle::Selection, Error> {
        let Geometry::Block { rows, width } = geometry else {
            return Err(Error::Target);
        };
        let width = count.map_or(width, |n| n.max(1) as usize);
        Ok(crate::rectangle::Selection::at(
            self.buffer()?,
            self.view().point,
            rows,
            width,
        )?)
    }

    fn prepare_repeat_block(
        &self,
        selection: &crate::rectangle::Selection,
        request: Editing,
        text: &str,
    ) -> Result<BlockChange, Error> {
        if request.action.shifts() {
            return Err(Error::Target);
        }
        let entry = self.repeat_block_entry(selection)?;
        let cursor = selection.cursor(self.buffer()?)?;
        let mut edits = selection.deletions(self.buffer()?)?;
        if request.action.changes() {
            block_text(&mut edits, self.buffer()?, cursor, text)?;
        }
        Ok(BlockChange {
            edits,
            cursor: Scalar(cursor.0 + text.chars().count()),
            entry,
        })
    }

    fn repeat_block_entry(
        &self,
        selection: &crate::rectangle::Selection,
    ) -> Result<crate::kill::Entry, Error> {
        let rectangle = std::sync::Arc::new(crate::rectangle::Rectangle::capture(
            self.buffer()?,
            selection,
            crate::rectangle::Capture::Kill,
        )?);
        Ok(self.kills.prepare_block(rectangle))
    }
}

fn region_graphemes(
    coordinates: &Coordinates<'_>,
    range: std::ops::Range<usize>,
    rows: std::ops::RangeInclusive<usize>,
) -> Result<usize, Error> {
    let mut count = rows.end() - rows.start();
    for row in rows {
        for glyph in coordinates.glyphs(row)? {
            let glyph = glyph?;
            count +=
                usize::from(glyph.source.start.0 >= range.start && glyph.source.end.0 <= range.end);
        }
    }
    Ok(count)
}

fn block_text(
    edits: &mut Vec<crate::buffer::edit::Replacement>,
    buffer: &crate::buffer::Buffer,
    cursor: Scalar,
    text: &str,
) -> Result<(), Error> {
    if text.is_empty() {
        return Ok(());
    }
    if let Some(first) = edits
        .first_mut()
        .filter(|e| e.source.range.start == cursor.0)
    {
        first.text.insert_str(0, text);
    } else {
        edits.insert(
            0,
            crate::buffer::edit::Replacement {
                source: buffer.span(cursor.0..cursor.0)?,
                text: text.to_owned(),
            },
        );
    }
    Ok(())
}

struct BlockChange {
    edits: Vec<crate::buffer::edit::Replacement>,
    cursor: Scalar,
    entry: crate::kill::Entry,
}

fn region_rows(
    coordinates: &Coordinates<'_>,
    source: &crate::buffer::Span,
) -> Result<std::ops::RangeInclusive<usize>, Error> {
    let first = coordinates.to_line(Scalar(source.range.start))?.row;
    let end = if source.range.is_empty() {
        Scalar(source.range.end)
    } else {
        coordinates.previous(Scalar(source.range.end))?
    };
    let last = coordinates.to_line(end)?.row;
    Ok(first..=last)
}
