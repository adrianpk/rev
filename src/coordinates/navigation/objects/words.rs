use super::super::runs::{Class, Run, Runs};
use crate::coordinates::{Coordinates, Error, Scalar};
use std::ops::Range;

impl Coordinates<'_> {
    pub(super) fn object_word(
        &self,
        point: Scalar,
        policy: (bool, bool),
        count: usize,
    ) -> Result<Option<Range<usize>>, Error> {
        let (big, around) = policy;
        let Some(first) = Runs::new(self.text, point, false, big)?.next()? else {
            return Ok(None);
        };
        let Some(full) = Runs::new(self.text, first.range.end, true, big)?.next()? else {
            return Ok(None);
        };
        let mut range = full.range.start.0..first.range.end.0;
        let mut runs = Runs::new(self.text, first.range.end, false, big)?;
        self.extend_words(&mut runs, &mut range, count.saturating_sub(1))?;
        if around {
            self.around_word(&mut range, big)?;
        }
        Ok(Some(range))
    }

    fn extend_words(
        &self,
        runs: &mut Runs<'_>,
        range: &mut Range<usize>,
        mut count: usize,
    ) -> Result<(), Error> {
        while count > 0 {
            let Some(run) = runs.next()? else { break };
            if run.class != Class::Space {
                range.end = run.range.end.0;
                count -= 1;
            }
        }
        Ok(())
    }

    fn around_word(&self, range: &mut Range<usize>, big: bool) -> Result<(), Error> {
        let following = Runs::new(self.text, Scalar(range.end), false, big)?.next()?;
        if let Some(Run {
            class: Class::Space,
            range: space,
        }) = following
        {
            range.end = space.end.0;
            return Ok(());
        }
        let preceding = Runs::new(self.text, Scalar(range.start), true, big)?.next()?;
        if let Some(Run {
            class: Class::Space,
            range: space,
        }) = preceding
        {
            range.start = space.start.0;
        }
        Ok(())
    }
}
