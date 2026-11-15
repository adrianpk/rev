use ropey::RopeSlice;

use super::{Context, Kind, Limit, MAX_DEPTH, Outcome, Query, Reason, Record, Work};
use crate::pairs::reader::Pause;

pub(super) struct Progress {
    depth: usize,
}

impl Work {
    pub(super) fn capture_wrapping(&mut self) -> Result<(), Pause> {
        let Query::Wrap(input) = self.query else {
            unreachable!()
        };
        self.context = self.lexical.context();
        let admitted = if input.kind.quote() {
            self.lexical
                .admits_typed_quote(input.kind, self.reader.previous)
                && quote_count(self.lexical.profile, input)
        } else {
            self.lexical.admits_typed_structure()
        };
        if self.context != Context::Code || !admitted || input.count == 0 {
            self.finish(Outcome::Uncertain(Reason::LiteralContext));
            return Ok(());
        }
        if self
            .stack
            .len()
            .checked_add(input.count)
            .is_none_or(|d| d > MAX_DEPTH)
        {
            return Err(Pause::Limit(Limit::Depth));
        }
        self.stats.maximum_depth = self.stats.maximum_depth.max(self.stack.len() + input.count);
        self.point_seen = true;
        self.wrapping = Some(Progress {
            depth: self.stack.len(),
        });
        Ok(())
    }

    pub(super) fn wrapping_boundary(&mut self, text: RopeSlice<'_>) -> Result<(), Pause> {
        let Some(progress) = &self.wrapping else {
            return Ok(());
        };
        let Query::Wrap(input) = self.query else {
            unreachable!()
        };
        let balanced =
            self.stack.len() == progress.depth && self.lexical.context() == Context::Code;
        if self.reader.at > input.end.0 || self.stack.len() < progress.depth {
            self.stop(Outcome::Uncertain(Reason::LiteralContext));
        } else if self.reader.at == input.end.0 {
            self.finish_wrapping(text, balanced)?;
        }
        Ok(())
    }

    fn finish_wrapping(&mut self, text: RopeSlice<'_>, balanced: bool) -> Result<(), Pause> {
        let Query::Wrap(input) = self.query else {
            unreachable!()
        };
        let boundary = !input.kind.quote()
            || self
                .reader
                .peek(text, 0)?
                .is_none_or(|c| !c.is_alphanumeric() && c != '_' && c != input.kind.characters().0);
        let outcome = if balanced && boundary {
            Outcome::Wrap(input.kind)
        } else {
            Outcome::Uncertain(Reason::LiteralContext)
        };
        self.finish(outcome);
        Ok(())
    }

    pub(super) fn wrapping_literal(&mut self, text: RopeSlice<'_>) -> Result<bool, Pause> {
        let Query::Wrap(input) = self.query else {
            return Ok(false);
        };
        if self.wrapping.is_none() || !input.kind.quote() {
            return Ok(false);
        }
        let c = self
            .reader
            .peek(text, 0)?
            .expect("wrapping content precedes its end");
        let unsafe_content = unsafe_character(self.lexical.profile, input.kind, c);
        if unsafe_content {
            self.stop(Outcome::Uncertain(Reason::AmbiguousQuote));
        } else {
            self.reader.consume(1);
            // This is a virtual quoted payload, not lexical proof about original
            // code. Prefix checkpoints and facts end before the selection.
            self.terminal = Some(Outcome::Uncertain(Reason::LiteralContext));
        }
        Ok(true)
    }

    pub(super) fn observe_quote_literal(&mut self, text: RopeSlice<'_>) -> Result<(), Pause> {
        let Context::String(kind) = self.lexical.context() else {
            return Ok(());
        };
        let c = self.reader.peek(text, 0)?;
        let unsafe_literal = c.is_some_and(|c| {
            c == '\\' || matches!(c, '\'' | '"' | '`') && c != kind.characters().1
        });
        if unsafe_literal && let Some(opening) = self.stack.last_mut() {
            opening.literal_safe = false;
        }
        Ok(())
    }

    pub(super) fn finish_pair(&mut self, pair: Record) {
        if matches!(self.query, Query::Unwrap(_)) && !pair.literal_safe {
            self.finish(Outcome::Uncertain(Reason::AmbiguousQuote));
        } else {
            self.finish(Outcome::Confirmed(pair.qualified(self.ticket.stamp)));
        }
    }
}

fn unsafe_character(profile: crate::pairs::Profile, kind: Kind, c: char) -> bool {
    let raw = kind == Kind::Backtick
        || profile == crate::pairs::Profile::Toml && kind == Kind::SingleQuote;
    c == kind.characters().0
        || !raw && c == '\\'
        || profile.single_line(kind) && matches!(c, '\r' | '\n')
}

fn quote_count(profile: crate::pairs::Profile, input: crate::pairs::Wrapping) -> bool {
    let character = matches!(
        profile,
        crate::pairs::Profile::C | crate::pairs::Profile::Go
    ) && input.kind == Kind::SingleQuote;
    input.count == 1 && (!character || input.end.0 - input.start.0 == 1)
}
