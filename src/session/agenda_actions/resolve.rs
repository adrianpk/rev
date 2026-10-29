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
            SourceVersion::Disk(_) => self
                .resolve_disk_action(target, run.deadline)
                .map(|()| None),
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
        deadline: Instant,
    ) -> Result<(), String> {
        let navigation = Navigation::prepare(
            &target.source,
            target.byte,
            self.agenda_sources
                .as_ref()
                .ok_or("Refresh agenda sources first")?,
            &self.buffers.snapshot().settings().org.limits,
        )?;
        self.org.start_sources(
            self.view().point.stamp,
            super::super::agenda::Work::ActionVisit {
                navigation,
                view: self.windows.selected_id(),
                deadline,
            },
        )
    }
}
