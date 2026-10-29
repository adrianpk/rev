use super::*;
use crate::org::agenda::{Navigation, NavigationCandidate, SourceVersion};
impl Session {
    pub(in crate::session) fn resolving_agenda_action(&self) -> bool {
        self.agenda_run
            .as_ref()
            .is_some_and(|r| r.phase == Phase::Waiting)
    }
    pub(in crate::session) fn resolved_agenda_action(&mut self, candidate: NavigationCandidate) {
        let Some(mut run) = self.agenda_run.take() else {
            return;
        };
        let result = self
            .agenda_candidate(candidate)
            .and_then(|point| self.pin_agenda_target(point));
        match result {
            Ok(pin) => run.pins[run.index] = Some(pin),
            Err(e) => run.outcome(run.index, ActionStatus::Rejected, e, None),
        }
        run.index += 1;
        run.phase = Phase::Resolve;
        self.agenda_run = Some(run);
    }
    pub(in crate::session) fn failed_agenda_action(&mut self, error: &str) {
        let Some(mut run) = self.agenda_run.take() else {
            return;
        };
        run.outcome(run.index, ActionStatus::Rejected, error.into(), None);
        run.index += 1;
        run.phase = Phase::Resolve;
        self.agenda_run = Some(run);
    }
    pub(super) fn resolve_agenda_target(&mut self, run: &mut Run) {
        if run.index == run.targets.len() {
            run.index = 0;
            run.phase = Phase::Edit;
            return;
        }
        if self.org.busy() {
            return;
        }
        if Instant::now() >= run.deadline {
            run.outcome(
                run.index,
                ActionStatus::Rejected,
                "Agenda source preparation time limit reached".into(),
                None,
            );
            run.index += 1;
            return;
        }
        let target = &run.targets[run.index];
        let result = match target.source.version {
            SourceVersion::Buffer(_) => target
                .point(&self.buffers)
                .and_then(|point| self.pin_agenda_target(point))
                .map(Some),
            SourceVersion::Disk(_) => self.resolve_disk_action(target, run),
        };
        match result {
            Ok(Some(pin)) => {
                run.pins[run.index] = Some(pin);
                run.index += 1;
            }
            Ok(None) => run.phase = Phase::Waiting,
            Err(e) => {
                run.outcome(run.index, ActionStatus::Rejected, e, None);
                run.index += 1;
            }
        }
    }
    fn pin_agenda_target(&mut self, point: Point) -> Result<usize, String> {
        let buffer = self
            .buffers
            .get(point.stamp.buffer)
            .map_err(|e| e.to_string())?;
        if buffer.stamp() != point.stamp {
            return Err("Agenda source changed during preparation".into());
        }
        if buffer.is_readonly() {
            return Err("buffer is read-only".into());
        }
        self.windows.pin_source_action(point)
    }
    fn resolve_disk_action(
        &mut self,
        target: &ActionTarget,
        run: &Run,
    ) -> Result<Option<usize>, String> {
        if let Some(point) = self.reuse_prepared_source(target, run)? {
            return self.pin_agenda_target(point).map(Some);
        }
        let mut limits = self.buffers.snapshot().settings().org.limits.clone();
        if let SourceVersion::Disk(observed) = &target.source.version {
            limits.agenda_file_bytes = limits.agenda_file_bytes.min(observed.length as usize);
        }
        let navigation = Navigation::prepare(
            &target.source,
            target.byte,
            self.agenda_sources
                .as_ref()
                .ok_or("Refresh agenda sources first")?,
            &limits,
        )?;
        self.org.start_sources(
            self.view().point.stamp,
            super::super::agenda::Work::ActionVisit {
                navigation,
                view: self.windows.selected_id(),
                deadline: run.deadline,
            },
        )?;
        Ok(None)
    }
    fn reuse_prepared_source(
        &self,
        target: &ActionTarget,
        run: &Run,
    ) -> Result<Option<Point>, String> {
        let previous = run.targets[..run.index]
            .iter()
            .enumerate()
            .find_map(|(index, old)| {
                (old.source.path == target.source.path
                    && old.source.version == target.source.version)
                    .then_some(run.pins[index])
                    .flatten()
            });
        let Some(pin) = previous else {
            return Ok(None);
        };
        let anchor = self
            .windows
            .source_anchor(pin)
            .ok_or("Prepared source anchor disappeared")?;
        let buffer = self
            .buffers
            .get(anchor.stamp.buffer)
            .map_err(|e| e.to_string())?;
        if buffer.stamp() != anchor.stamp {
            return Err("Prepared canonical source changed; refresh and select again".into());
        }
        self.check_prepared_text(buffer, target)?;
        let position = buffer
            .byte_span(target.byte..target.byte)
            .map_err(|e| e.to_string())?
            .range
            .start;
        Ok(Some(Point {
            stamp: buffer.stamp(),
            position: crate::coordinates::Scalar(position),
        }))
    }
    fn check_prepared_text(
        &self,
        buffer: &crate::buffer::Buffer,
        target: &ActionTarget,
    ) -> Result<(), String> {
        let original = self
            .agenda_sources
            .as_ref()
            .and_then(|set| {
                set.documents
                    .iter()
                    .find(|s| s.path == target.source.path && s.version == target.source.version)
            })
            .ok_or("Original source snapshot unavailable")?;
        let whole = buffer
            .span(0..buffer.len_chars())
            .map_err(|e| e.to_string())?;
        let actual = buffer.slice(&whole).map_err(|e| e.to_string())?;
        if actual != original.text() {
            return Err(
                "Prepared canonical source text differs from the selected observation".into(),
            );
        }
        Ok(())
    }
}
