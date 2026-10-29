use super::*;
impl Session {
    pub(super) fn publish_capture(
        &mut self,
        insertion: Insertion,
        owner: ViewId,
    ) -> Result<Applied, String> {
        self.publish_capture_with(insertion, owner, &mut crate::file::Files)
    }
    pub(super) fn publish_capture_with(
        &mut self,
        insertion: Insertion,
        owner: ViewId,
        output: &mut impl crate::file::Output,
    ) -> Result<Applied, String> {
        let mut visit = insertion.source.prepare_visit(&self.buffers)?;
        let buffer = visit.buffer(&self.buffers).map_err(|e| e.to_string())?;
        let mut points = self
            .windows
            .capture_points(buffer)
            .map_err(|e| e.to_string())?;
        let edits = insertion.replacements(buffer)?;
        let mut prepared = buffer
            .prepare(&edits, &points.values, None)
            .map_err(|e| e.to_string())?;
        prepared.set_origin(owner);
        let tops = points.map_tops(&prepared).map_err(|e| e.to_string())?;
        let buffer = visit
            .buffer_mut(&mut self.buffers)
            .map_err(|e| e.to_string())?;
        let before = buffer.stamp();
        let outcome = buffer
            .publish_saved_edit(prepared, &mut points.values, output)
            .map_err(|e| e.to_string())?;
        if outcome.publication() == Publication::NotPublished {
            return Ok(Applied {
                source: before,
                outcome,
            });
        }
        let stamp = buffer.stamp();
        self.buffers.publish_visit(visit);
        self.windows.publish_points(points, tops);
        Ok(Applied {
            source: stamp,
            outcome,
        })
    }
    pub(super) fn retry_capture_save(&mut self, origin: Origin, stored: Stored) -> Action {
        let result = self.retained_capture_save(&stored);
        match result {
            Ok(outcome) if outcome.is_complete() => self.return_capture(origin, stored.draft),
            Ok(outcome) => {
                self.capture = Some(State {
                    origin,
                    stage: Stage::Published(Box::new(stored)),
                });
                self.error(&format!(
                    "Capture remains inserted once; draft retained: {}",
                    outcome.error().expect("incomplete save error")
                ))
            }
            Err(e) => {
                self.capture = Some(State {
                    origin,
                    stage: Stage::Published(Box::new(stored)),
                });
                self.error(&e)
            }
        }
    }
    fn retained_capture_save(&mut self, stored: &Stored) -> Result<Outcome, String> {
        self.check_capture_draft(stored.draft_stamp)?;
        let buffer = self
            .buffers
            .get(stored.source.buffer)
            .map_err(|e| e.to_string())?;
        if buffer.stamp() != stored.source
            || buffer.file().map(|f| f.path()) != Some(stored.path.as_path())
        {
            return Err("Published capture source changed; draft retained; inspect the source before discarding the draft".into());
        }
        self.buffers
            .save(stored.source.buffer)
            .map_err(|e| e.to_string())
    }
}
