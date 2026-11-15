use ropey::RopeSlice;

use super::{Context, Kind, Limit, MAX_DEPTH, Outcome, Query, Reason, Work};
use crate::pairs::{Typing, lexical::Lexical, reader::Pause};

pub(super) struct Progress {
    kind: Kind,
    opening: bool,
    reused: usize,
}

impl Work {
    pub(super) fn rewind_typing(&mut self) {
        if self.reader.at > self.query.point().0 && !self.restore_checkpoint(self.query.point().0) {
            self.reader.rewind((0, None, true));
            self.lexical = Lexical::new(self.lexical.profile);
            self.stack.clear();
            self.facts.clear();
            self.checkpoints.iter_mut().for_each(|c| c.valid = false);
            self.checkpoint_at = 0;
            self.terminal = None;
        }
        self.point_seen = false;
    }

    pub(super) fn capture_typing(&mut self, text: RopeSlice<'_>) -> Result<(), Pause> {
        let Query::Typing(input) = self.query else {
            unreachable!()
        };
        self.context = self.lexical.context();
        let Some((kind, opening)) = typed_kind(input.character, self.context) else {
            self.finish(Outcome::Uncertain(Reason::LiteralContext));
            return Ok(());
        };
        if self.context != Context::Code && !(kind.quote() && !opening) {
            self.finish(Outcome::Uncertain(Reason::LiteralContext));
            return Ok(());
        }
        if kind.quote() && opening {
            return self.capture_quote(text, kind);
        }
        if !kind.quote() && !self.lexical.admits_typed_structure() {
            self.finish(Outcome::Uncertain(Reason::LiteralContext));
            return Ok(());
        }
        self.point_seen = true;
        self.typing = Some(Progress {
            kind,
            opening,
            reused: 0,
        });
        let depth = self.stack.len() + self.typing_extra_depth();
        self.stats.maximum_depth = self.stats.maximum_depth.max(depth.min(MAX_DEPTH));
        if depth > MAX_DEPTH {
            return Err(Pause::Limit(Limit::Depth));
        }
        Ok(())
    }

    fn capture_quote(&mut self, text: RopeSlice<'_>, kind: Kind) -> Result<(), Pause> {
        let next = self.reader.peek(text, 0)?;
        let boundary = next.is_none_or(|c| !c.is_alphanumeric() && c != '_');
        let admitted = self.lexical.admits_typed_quote(kind, self.reader.previous) && boundary;
        if admitted && self.stack.len() == MAX_DEPTH {
            return Err(Pause::Limit(Limit::Depth));
        }
        if admitted && self.reusable_quote(text, kind, next)? {
            self.stats.maximum_depth = self.stats.maximum_depth.max(self.stack.len() + 1);
            self.finish(Outcome::Typing(Typing::Open {
                kind,
                reused: usize::from(next == Some(kind.characters().0)),
            }));
        } else {
            self.finish(Outcome::Uncertain(Reason::AmbiguousQuote));
        }
        self.point_seen = true;
        Ok(())
    }

    fn reusable_quote(
        &mut self,
        text: RopeSlice<'_>,
        kind: Kind,
        next: Option<char>,
    ) -> Result<bool, Pause> {
        let Query::Typing(input) = self.query else {
            unreachable!()
        };
        if next != Some(input.character) {
            return Ok(true);
        }
        if input.end != input.point {
            return Ok(false);
        }
        Ok(kind == Kind::Backtick || self.reader.peek(text, 1)? != Some(input.character))
    }

    pub(super) fn typing_extra_depth(&self) -> usize {
        if let Query::Wrap(input) = self.query {
            return self.wrapping.as_ref().map_or(0, |_| input.count);
        }
        let Query::Typing(input) = self.query else {
            return 0;
        };
        self.typing
            .as_ref()
            .filter(|p| p.opening)
            .map_or(0, |p| input.count - p.reused)
    }

    pub(super) fn typing_boundary(&mut self, text: RopeSlice<'_>) -> Result<(), Pause> {
        let Some(progress) = &self.typing else {
            return Ok(());
        };
        let Query::Typing(input) = self.query else {
            unreachable!()
        };
        if !progress.opening && (!input.skip || self.reader.peek(text, 0)? != Some(input.character))
        {
            self.finish_typing();
        }
        Ok(())
    }

    pub(super) fn typing_replacement(&mut self, text: RopeSlice<'_>) -> Result<bool, Pause> {
        let Query::Typing(input) = self.query else {
            return Ok(false);
        };
        if !self.point_seen
            || self.reader.at >= input.end.0
            || !self.typing.as_ref().is_some_and(|p| p.opening)
        {
            return Ok(false);
        }
        self.reader.peek(text, 0)?;
        self.reader.consume(1);
        // A Replace query observes a virtual suffix. Its facts cannot become
        // proof about the original source after the overwritten range.
        self.terminal = Some(Outcome::Uncertain(Reason::LiteralContext));
        Ok(true)
    }

    pub(super) fn typing_unmatched(&mut self, kind: Kind) -> bool {
        let Some(progress) = &mut self.typing else {
            return false;
        };
        if !progress.opening || progress.kind != kind {
            return false;
        }
        progress.reused += 1;
        self.terminal = Some(Outcome::Unmatched);
        let Query::Typing(input) = self.query else {
            unreachable!()
        };
        if progress.reused == input.count {
            self.finish_typing();
        }
        true
    }

    pub(super) fn typing_closed(&mut self, at: usize, policy: (Kind, bool)) -> bool {
        let Some(progress) = &mut self.typing else {
            return false;
        };
        let Query::Typing(input) = self.query else {
            unreachable!()
        };
        if progress.opening
            || !policy.1
            || progress.kind != policy.0
            || at != input.point.0 + progress.reused
        {
            return false;
        }
        progress.reused += 1;
        if progress.reused == input.count || progress.kind.quote() {
            self.finish_typing();
        }
        true
    }

    pub(super) fn finish_typing(&mut self) -> bool {
        let Some(progress) = self.typing.take() else {
            return false;
        };
        let outcome = if progress.opening {
            Outcome::Typing(Typing::Open {
                kind: progress.kind,
                reused: progress.reused,
            })
        } else {
            Outcome::Typing(Typing::Skip(progress.reused))
        };
        self.finish(outcome);
        true
    }
}

fn typed_kind(c: char, context: Context) -> Option<(Kind, bool)> {
    if let Some(pair) = Kind::structural(c) {
        return Some(pair);
    }
    let kind = match c {
        '"' => Kind::DoubleQuote,
        '\'' => Kind::SingleQuote,
        '`' => Kind::Backtick,
        _ => return None,
    };
    Some((kind, context != Context::String(kind)))
}
