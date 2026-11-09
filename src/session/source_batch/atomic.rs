//! Native multi-source coordinator. Private candidates and every mapped view,
//! history reservation and observation exist before final admission. Publication
//! consumes only reserved native state and cannot return a partial failure.
use crate::{
    buffer::{
        BufferId, Buffers, Stamp,
        edit::{Error, Published, Replacement, ReservedEdit},
    },
    window::{Points, Viewport, Windows},
};

struct Participant {
    id: BufferId,
    edit: ReservedEdit,
    points: Points,
    tops: Vec<Viewport>,
}
pub(in crate::session) struct Atomic {
    participants: Vec<Participant>,
    stamps: Vec<Stamp>,
    observations: Vec<(BufferId, Option<Published>)>,
}
#[derive(Debug)]
pub(in crate::session) enum Failure {
    Edit(Error),
    Cancelled,
}
impl Atomic {
    pub fn prepare(
        buffers: &mut Buffers,
        windows: &Windows,
        targets: &[(BufferId, Vec<Replacement>)],
    ) -> Result<Self, Error> {
        let mut participants = Vec::new();
        let mut stamps = Vec::new();
        let mut observations = Vec::new();
        participants
            .try_reserve_exact(targets.len())
            .map_err(|_| Error::Capacity)?;
        stamps
            .try_reserve_exact(targets.len())
            .map_err(|_| Error::Capacity)?;
        observations
            .try_reserve_exact(targets.len())
            .map_err(|_| Error::Capacity)?;
        for (id, edits) in targets {
            if participants.iter().any(|p: &Participant| p.id == *id) {
                return Err(Error::Order);
            }
            let buffer = buffers.get_mut(*id)?;
            let points = windows.capture_points(buffer)?;
            let prepared = buffer.prepare(edits, &points.values, None)?;
            let tops = points.map_tops(&prepared)?;
            let edit = buffer.reserve_edit(prepared, &points.values)?;
            stamps.push(edit.after());
            participants.push(Participant {
                id: *id,
                edit,
                points,
                tops,
            });
        }
        Ok(Self {
            participants,
            stamps,
            observations,
        })
    }
    pub fn check(&self, buffers: &Buffers, windows: &Windows) -> Result<(), Error> {
        for participant in &self.participants {
            let buffer = buffers.get(participant.id)?;
            windows.check_points(buffer, &participant.points)?;
            buffer.check_reserved_edit(&participant.edit, &participant.points.values)?;
        }
        Ok(())
    }
    pub fn stamps(&self) -> &[Stamp] {
        &self.stamps
    }
    /// Final stamp/view checks and cancellation complete before entering the
    /// infallible publication phase. `authorized` is the request's live token.
    pub fn publish(
        self,
        buffers: &mut Buffers,
        windows: &mut Windows,
        authorized: impl FnOnce() -> bool,
    ) -> Result<Vec<Stamp>, Failure> {
        self.check(buffers, windows).map_err(Failure::Edit)?;
        if !authorized() {
            return Err(Failure::Cancelled);
        }
        Ok(self.commit(buffers, windows))
    }
    /// Serialized native ownership excludes structural/source changes between
    /// `check` and this non-fallible commit. Notify only after all buffers AND
    /// their shared views have reached the new revisions.
    fn commit(mut self, buffers: &mut Buffers, windows: &mut Windows) -> Vec<Stamp> {
        for mut participant in self.participants {
            let buffer = buffers
                .get_mut(participant.id)
                .expect("rechecked atomic participant");
            let observation =
                buffer.commit_reserved_edit(participant.edit, &mut participant.points.values);
            windows.publish_points(participant.points, participant.tops);
            self.observations.push((participant.id, observation));
        }
        for (id, observation) in self.observations {
            buffers
                .get(id)
                .expect("published atomic participant")
                .notify_publication(observation);
        }
        self.stamps
    }
}

#[cfg(test)]
mod tests;
