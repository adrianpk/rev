mod plan;
mod prompt;

use std::ops::Range;
use std::time::Instant;

use crate::buffer::edit::{Error, Point};
use crate::command::{Command, PrefixArgument};
use crate::config::Pairs;
use crate::coordinates::Scalar;
use crate::input::{EditingState, TextInvocation};
use crate::pairs::{Kind, Outcome, Pair, Query, Status, Ticket, Wrapping};
use crate::session::{Action, Session};
use crate::window::ViewId;

struct Origin {
    point: Point,
    view: ViewId,
    mark: (Option<Scalar>, bool),
    state: Option<EditingState>,
    policy: Pairs,
}

enum Intent {
    Wrap {
        input: Wrapping,
        closing: bool,
        typed: Option<(char, PrefixArgument)>,
    },
    Unwrap,
}

enum Plan {
    Refused,
    Wrap(Wrapping, bool),
    Unwrap(Pair),
    Literal(char, PrefixArgument),
}

pub(in crate::session) struct Regions {
    origin: Origin,
    intent: Intent,
    ticket: Option<Ticket>,
    plan: Option<Plan>,
}

pub(in crate::session) struct RegionPrompt {
    origin: Origin,
    range: Range<usize>,
    count: usize,
}

impl Session {
    pub(in crate::session) fn pair_command(&mut self, command: Command) -> Action {
        match command {
            Command::PairWrap => self.start_pair_wrap_prompt(PrefixArgument::default()),
            Command::PairUnwrap => self.start_pair_unwrap(),
            Command::MatchingDelimiter => self.matching_delimiter(),
            Command::ForwardPair
            | Command::BackwardPair
            | Command::UpPair
            | Command::DownPair
            | Command::ForwardSexp
            | Command::BackwardSexp => self.group_motion(command, 1),
            _ => self.dispatch(command),
        }
    }

    pub(in crate::session) fn start_region_typing(
        &mut self,
        c: char,
        argument: PrefixArgument,
    ) -> Option<Action> {
        let (kind, closing) = self.automatic_region_kind(c)?;
        let buffer = self.buffer().ok()?;
        let range = buffer.region(self.view().point).ok()??.range;
        if range.is_empty() {
            return None;
        }
        let intent = Intent::Wrap {
            input: Wrapping {
                start: Scalar(range.start),
                end: Scalar(range.end),
                kind,
                count: usize::try_from(argument.value()).ok()?,
            },
            closing,
            typed: Some((c, argument)),
        };
        Some(self.begin_pair_regions(intent))
    }

    fn automatic_region_kind(&self, c: char) -> Option<(Kind, bool)> {
        let buffer = self.buffer().ok()?;
        let policy = buffer.pair_policy();
        if !policy.enabled || !policy.wrap_selection || !ordinary_region(buffer) {
            return None;
        }
        let (kind, closing) = delimiter(c)?;
        if kind.quote() && !policy.quotes {
            return None;
        }
        Some((kind, closing))
    }

    fn pair_region_focus(&self) -> bool {
        self.source_layout_focus()
            && !self.terminals.focused
            && !self.git_visible()
            && !self.capture_blocks_input()
            && !self.context_storage.locked()
            && self.pending.is_none()
    }

    fn region_origin(&self) -> Result<Origin, Error> {
        if !self.pair_region_focus() {
            return Err(Error::ChangedPositions);
        }
        let buffer = self.buffer()?;
        if buffer.is_readonly() {
            return Err(Error::ReadOnly);
        }
        Ok(Origin {
            point: self.view().point,
            view: self.windows.selected_id(),
            mark: (buffer.mark(), buffer.mark_active()),
            state: self.editing_state(),
            policy: buffer.pair_policy(),
        })
    }

    fn region_origin_current(&self, origin: &Origin) -> bool {
        self.pair_region_focus()
            && origin.point == self.view().point
            && origin.view == self.windows.selected_id()
            && origin.state == self.editing_state()
            && self.buffer().is_ok_and(|b| {
                !b.is_readonly()
                    && origin.mark == (b.mark(), b.mark_active())
                    && origin.policy == b.pair_policy()
            })
    }

    fn begin_pair_regions(&mut self, intent: Intent) -> Action {
        let origin = match self.region_origin() {
            Ok(origin) => origin,
            Err(e) => return self.error(&e.to_string()),
        };
        self.start_region_work(origin, intent)
    }

    fn start_region_work(&mut self, origin: Origin, intent: Intent) -> Action {
        self.layout.regions = Some(Regions {
            origin,
            intent,
            ticket: None,
            plan: None,
        });
        self.layout_pair_regions()
            .unwrap_or_else(|| self.publish_pair_regions())
    }

    fn start_pair_unwrap(&mut self) -> Action {
        self.begin_pair_regions(Intent::Unwrap)
    }

    pub(in crate::session) fn cancel_pair_regions(&mut self) {
        if let Some(work) = self.layout.regions.take()
            && let Some(ticket) = work.ticket
            && let Ok(buffer) = self.buffers.get_mut(ticket.stamp.buffer)
        {
            buffer.cancel_pair_request(ticket);
        }
    }

    pub(in crate::session) fn prepare_pair_regions(&mut self) -> Result<(), crate::view::Error> {
        let mut work = self.layout.regions.take().expect("owned pair region edit");
        let result = self.prepare_region_plan(&mut work);
        self.layout.regions = Some(work);
        result.map_err(Into::into)
    }

    fn prepare_region_plan(&mut self, work: &mut Regions) -> Result<(), Error> {
        if !self.region_origin_current(&work.origin) {
            return Err(Error::ChangedPositions);
        }
        self.admit_region_request(work)?;
        let b = self.buffers.get_mut(work.origin.point.stamp.buffer)?;
        let ticket = work.ticket.expect("admitted region request");
        match b.poll_pair_request(ticket, Instant::now() + crate::pairs::SLICE_TIME) {
            Status::Pending => return Err(crate::coordinates::Error::Pending.into()),
            Status::Retired => return Err(Error::ChangedPositions),
            Status::Ready => (),
        }
        let outcome = &b.pair_answer(ticket).expect("ready region proof").outcome;
        let plan = region_plan(&work.intent, outcome)?;
        self.admit_region_boundaries(&plan)?;
        work.plan = Some(plan);
        Ok(())
    }

    fn admit_region_request(&mut self, work: &mut Regions) -> Result<(), Error> {
        if work.ticket.is_some() {
            return Ok(());
        }
        let query = match work.intent {
            Intent::Wrap { input, .. } => {
                self.admit_pair_payload(input.count.checked_mul(2).ok_or(Error::Capacity)?)?;
                Query::Wrap(input)
            }
            Intent::Unwrap => Query::Unwrap(work.origin.point.position),
        };
        work.ticket = Some(
            self.buffers
                .get_mut(work.origin.point.stamp.buffer)?
                .start_pair_request(query)?,
        );
        Ok(())
    }

    pub(in crate::session) fn publish_pair_regions(&mut self) -> Action {
        let Some(work) = self.layout.regions.take() else {
            return self.message("Pair region edit retired");
        };
        if !self.region_origin_current(&work.origin) {
            return self.message("Pair region source changed");
        }
        match work.plan.expect("complete region plan") {
            Plan::Literal(c, argument) => self.region_literal(c, argument),
            Plan::Refused => {
                self.error("Pair edit is unconfirmed or cannot preserve literal contents")
            }
            plan => {
                self.dismiss_completion();
                let result = self.publish_region_plan(plan);
                self.sync_input_context();
                self.finish(result)
            }
        }
    }

    fn region_literal(&mut self, c: char, argument: PrefixArgument) -> Action {
        if argument.supplied() {
            return self.execute_argument_text(TextInvocation {
                text: c.to_string(),
                typed: false,
                argument,
                origin: self.input.invocation_origin(),
            });
        }
        let result = self.type_text(&c.to_string());
        self.finish(result)
    }
}

fn region_plan(intent: &Intent, outcome: &Outcome) -> Result<Plan, Error> {
    match (intent, outcome) {
        (Intent::Wrap { input, closing, .. }, Outcome::Wrap(_)) => Ok(Plan::Wrap(*input, *closing)),
        (Intent::Unwrap, Outcome::Confirmed(pair)) => Ok(Plan::Unwrap(pair.clone())),
        (
            Intent::Wrap {
                input,
                typed: Some((c, argument)),
                ..
            },
            _,
        ) if !input.kind.quote() => {
            if argument.supplied() && matches!(outcome, Outcome::Limited(_)) {
                return Err(Error::Capacity);
            }
            Ok(Plan::Literal(*c, *argument))
        }
        (_, Outcome::Limited(_)) => Err(Error::Capacity),
        _ => Ok(Plan::Refused),
    }
}

fn ordinary_region(buffer: &crate::buffer::Buffer) -> bool {
    buffer.mark_active() && !buffer.rectangle_active() && buffer.visual().is_none()
}

fn delimiter(c: char) -> Option<(Kind, bool)> {
    if let Some((kind, opening)) = Kind::structural(c) {
        return Some((kind, !opening));
    }
    let kind = match c {
        '"' => Kind::DoubleQuote,
        '\'' => Kind::SingleQuote,
        '`' => Kind::Backtick,
        _ => return None,
    };
    Some((kind, false))
}
