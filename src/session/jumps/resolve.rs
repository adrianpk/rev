use super::*;
use crate::location::Source;

impl Session {
    pub(super) fn resolve_jump(&self, anchor: &Anchor) -> Result<(Visit, Point), String> {
        let source = anchor.source();
        let position = source
            .position
            .ok_or("Jump target was removed; undo can restore it")?;
        if !source.closed {
            return self.live_jump(&source, position);
        }
        let visit = self.reopen_jump(&source)?;
        let buffer = visit.buffer(&self.buffers).map_err(|e| e.to_string())?;
        if !source.context.matches(buffer, position.0)? {
            return Err("Jump source context changed".into());
        }
        let point = Point {
            stamp: buffer.stamp(),
            position,
        };
        self.check_jump_point(buffer, point)?;
        Ok((visit, point))
    }

    fn live_jump(
        &self,
        source: &Source,
        position: crate::coordinates::Scalar,
    ) -> Result<(Visit, Point), String> {
        let buffer = self
            .buffers
            .get(source.point.stamp.buffer)
            .map_err(|e| e.to_string())?;
        if source.point.stamp != buffer.stamp() {
            return Err("Jump source changed".into());
        }
        let point = Point {
            position,
            ..source.point
        };
        self.check_jump_point(buffer, point)?;
        Ok((Visit::Existing(buffer.id()), point))
    }

    fn reopen_jump(&self, source: &Source) -> Result<Visit, String> {
        let file = source
            .file
            .as_ref()
            .ok_or("Closed jump source has no file")?;
        let expected = file.observation().ok_or("Jump file had no disk identity")?;
        let actual = crate::file::resolve(file.path()).map_err(|e| e.to_string())?;
        if !actual
            .observation()
            .is_some_and(|observed| expected.same_content(observed))
        {
            return Err("Jump file is missing, replaced or changed".into());
        }
        let visit = self
            .buffers
            .prepare_visit(file.path())
            .map_err(|e| e.to_string())?;
        let buffer = visit.buffer(&self.buffers).map_err(|e| e.to_string())?;
        self.admit_reopened_jump(source, buffer)?;
        Ok(visit)
    }

    pub(super) fn admit_reopened_jump(
        &self,
        source: &Source,
        buffer: &Buffer,
    ) -> Result<(), String> {
        if buffer.is_modified() {
            return Err("Reopened jump source has different unsaved text".into());
        }
        let file = source.file.as_ref().ok_or("Jump file unavailable")?;
        let expected = file.observation().ok_or("Jump identity unavailable")?;
        let current = crate::file::observe(file.path()).map_err(|e| e.to_string())?;
        let captured = buffer.file().and_then(|file| file.observation());
        if !captured.is_some_and(|actual| expected.same_content(actual))
            || !current
                .as_ref()
                .is_some_and(|actual| expected.same_content(actual))
        {
            return Err("Jump file changed during preparation".into());
        }
        Ok(())
    }

    fn check_jump_point(&self, buffer: &Buffer, point: Point) -> Result<(), String> {
        buffer
            .slice(&crate::buffer::Span {
                stamp: point.stamp,
                range: point.position.0..point.position.0,
            })
            .map_err(|e| e.to_string())?;
        if !Coordinates::new(buffer)
            .map_err(|e| e.to_string())?
            .is_boundary(point.position)
            .map_err(|e| e.to_string())?
        {
            return Err("Jump target is no longer a grapheme boundary".into());
        }
        Ok(())
    }
}
