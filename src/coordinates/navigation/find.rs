use crate::command::navigation::Find;

use super::super::{Coordinates, Error, Scalar};
use super::runs::Runs;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Found {
    pub point: Scalar,
    pub matched: Scalar,
}

impl Coordinates<'_> {
    /// Search a logical line once. Failure publishes neither an intermediate
    /// match nor a partial count. A scalar inside a cluster selects its boundary.
    pub(crate) fn find_target(
        &self,
        origin: Scalar,
        find: Find,
        character: char,
        mut count: usize,
    ) -> Result<Option<Found>, Error> {
        let backward = find.backward();
        let start = self.line_start(origin)?;
        let end = self.line_end(origin)?;
        let mut runs = Runs::new(self.text, origin, backward, false)?;
        while let Some(cluster) = runs.cluster()? {
            if outside(&cluster.range, start..end) {
                break;
            }
            if cluster.range.start == origin {
                continue;
            }
            if self.contains_character(&cluster.range, character) {
                count -= 1;
                if count == 0 {
                    return self.found(cluster.range, find).map(Some);
                }
            }
        }
        Ok(None)
    }

    fn contains_character(&self, range: &std::ops::Range<Scalar>, character: char) -> bool {
        self.text
            .slice(range.start.0..range.end.0)
            .chars()
            .any(|c| c == character)
    }

    fn found(&self, range: std::ops::Range<Scalar>, find: Find) -> Result<Found, Error> {
        let matched = range.start;
        let point = match find {
            Find::Forward | Find::Backward => matched,
            Find::TillForward => self.previous_observed(matched)?,
            Find::TillBackward => range.end,
        };
        Ok(Found { point, matched })
    }
}

fn outside(cluster: &std::ops::Range<Scalar>, line: std::ops::Range<Scalar>) -> bool {
    cluster.start < line.start || cluster.end > line.end
}
