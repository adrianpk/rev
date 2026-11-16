use std::time::{Duration, Instant};

use crate::buffer::edit::Point;
use crate::config::{PairHighlight, Pairs};
use crate::pairs::PresentationTicket;

use super::PairPresentation;

/// One input generation; qualification can finish only within its original life.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Pulse {
    pub point: Point,
    pub expires: Instant,
    pub ticket: PresentationTicket,
}

impl PairPresentation {
    pub(crate) fn arm_pulse(
        &mut self,
        point: Point,
        now: Instant,
        duration: usize,
        ticket: PresentationTicket,
    ) {
        self.generation = self
            .generation
            .checked_add(1)
            .expect("pair pulse generation exhausted");
        self.pulse = Some(Pulse {
            point,
            ticket,
            expires: now
                + Duration::from_millis(u64::try_from(duration).expect("admitted pulse duration")),
        });
    }

    pub(crate) fn cancel_pulse(&mut self) -> bool {
        if self.pulse.take().is_none() {
            return false;
        }
        self.retire();
        true
    }

    pub(crate) fn visible(self, defaults: Pairs, point: Point) -> bool {
        match self.style(defaults) {
            PairHighlight::Depth | PairHighlight::Match => true,
            PairHighlight::Pulse => self.pulse.is_some_and(|pulse| pulse.point == point),
            PairHighlight::Off => false,
        }
    }
}
