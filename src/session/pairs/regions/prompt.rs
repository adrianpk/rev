use crate::command::PrefixArgument;
use crate::coordinates::Scalar;
use crate::pairs::Wrapping;
use crate::session::{Action, Pending, Session};

use super::{Intent, RegionPrompt, delimiter, ordinary_region};

impl Session {
    pub(in crate::session) fn start_pair_wrap_prompt(
        &mut self,
        argument: PrefixArgument,
    ) -> Action {
        if argument.value() <= 0 {
            return if argument.value() == 0 {
                self.message("")
            } else {
                self.error("Pair wrapping requires a nonnegative count")
            };
        }
        let origin = match self.region_origin() {
            Ok(origin) => origin,
            Err(e) => return self.error(&e.to_string()),
        };
        let range = match self.wrap_prompt_region() {
            Ok(range) => range,
            Err(e) => return self.error(&e.to_string()),
        };
        if let Err(e) = self
            .input
            .argument("Pair to wrap (opening or closing delimiter): ", "")
        {
            return self.error(&e.to_string());
        }
        self.pending = Some(Pending::PairWrap(Box::new(RegionPrompt {
            origin,
            range,
            count: usize::try_from(argument.value()).expect("positive native argument"),
        })));
        self.message("")
    }

    fn wrap_prompt_region(&self) -> Result<std::ops::Range<usize>, crate::buffer::edit::Error> {
        let buffer = self.buffer()?;
        if !ordinary_region(buffer) {
            return Err(crate::buffer::edit::Error::Mark);
        }
        let range = buffer
            .region(self.view().point)?
            .ok_or(crate::buffer::edit::Error::Mark)?
            .range;
        if range.is_empty() {
            return Err(crate::buffer::edit::Error::Mark);
        }
        Ok(range)
    }

    pub(in crate::session) fn pair_wrap_argument(
        &mut self,
        prompt: RegionPrompt,
        text: &str,
    ) -> Action {
        if !self.region_origin_current(&prompt.origin) {
            return self.error("Pair wrapping source or selection changed");
        }
        let mut chars = text.chars();
        let kind = chars
            .next()
            .and_then(delimiter)
            .filter(|_| chars.next().is_none());
        let Some((kind, _)) = kind else {
            return self.error("Choose one admitted opening or closing delimiter");
        };
        let intent = Intent::Wrap {
            input: Wrapping {
                start: Scalar(prompt.range.start),
                end: Scalar(prompt.range.end),
                kind,
                count: prompt.count,
            },
            closing: false,
            typed: None,
        };
        self.start_region_work(prompt.origin, intent)
    }

    pub(in crate::session) fn cancel_pair_prompt(&mut self) -> Option<Action> {
        if !matches!(self.pending, Some(Pending::PairWrap(_))) {
            return None;
        }
        self.pending = None;
        self.input.cancel();
        Some(self.message("Pair wrapping canceled; selection retained"))
    }
}
