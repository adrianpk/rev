use std::ops::Range;

use crate::buffer::edit::Error;
use crate::coordinates::Scalar;
use crate::pairs::{Outcome, Typing as Decision};
use crate::session::Session;

use super::Typing;

pub(super) struct Plan {
    pub range: Range<usize>,
    pub text: String,
    pub cursor: Scalar,
    pub literal: bool,
}

impl Session {
    pub(super) fn pair_plan(&self, work: &Typing, outcome: &Outcome) -> Result<Plan, Error> {
        match *outcome {
            Outcome::Typing(Decision::Open { kind, reused }) => {
                let count = work.input.count;
                let text = opener_payload(work.input, kind, reused);
                self.admit_pair_payload(text.len())?;
                Ok(Plan {
                    range: work.input.point.0..work.input.end.0,
                    text,
                    cursor: Scalar(work.input.point.0 + count),
                    literal: false,
                })
            }
            Outcome::Typing(Decision::Skip(skipped)) => self.skipped_pair_plan(work, skipped),
            Outcome::Limited(_) if work.counted => Err(Error::Capacity),
            _ => self.skipped_pair_plan(work, 0),
        }
    }

    fn skipped_pair_plan(&self, work: &Typing, skipped: usize) -> Result<Plan, Error> {
        let text = work
            .input
            .character
            .to_string()
            .repeat(work.input.count - skipped);
        self.admit_pair_payload(text.len())?;
        let at = self.pair_skip_boundary(Scalar(work.input.point.0 + skipped))?;
        let range = if skipped == 0 {
            work.input.point.0..work.input.end.0
        } else if text.is_empty() {
            at.0..at.0
        } else {
            self.overwrite_range_for_pair(at, &text)?
        };
        Ok(Plan {
            range,
            text,
            cursor: Scalar(at.0 + work.input.count - skipped),
            literal: skipped == 0,
        })
    }

    fn pair_skip_boundary(&self, at: Scalar) -> Result<Scalar, Error> {
        let coordinates = crate::coordinates::Coordinates::new(self.buffer()?)?;
        if coordinates.is_boundary(at)? {
            Ok(at)
        } else {
            Ok(coordinates.next(at)?)
        }
    }

    fn overwrite_range_for_pair(&self, at: Scalar, text: &str) -> Result<Range<usize>, Error> {
        if self.editing_state() == Some(crate::input::EditingState::Replace) {
            self.overwrite_range(at, text)
        } else {
            Ok(at.0..at.0)
        }
    }
}

// Symmetric entry first creates/reuses a mate, then consumes it when skipping
// is enabled. Subsequent adjacent quotes remain literal under the quote policy.
fn opener_payload(input: crate::pairs::Typed, kind: crate::pairs::Kind, reused: usize) -> String {
    let count = input.count;
    let entered = count - usize::from(kind.quote() && reused == 1 && count > 1 && input.skip);
    let mut text = input.character.to_string().repeat(entered);
    let mates = if kind.quote() {
        usize::from(reused == 0 && (count == 1 || !input.skip))
    } else {
        count - reused
    };
    text.extend(std::iter::repeat_n(kind.characters().1, mates));
    text
}
