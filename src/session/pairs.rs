mod deletion;
mod navigation;
mod plan;
mod presentation;
mod regions;
pub(super) use regions::{RegionPrompt, Regions};

pub(super) use deletion::Deletion;
pub(super) use navigation::Navigation;

use std::time::Instant;

use crate::buffer::edit::{Error, Point};
use crate::command::{Command, PrefixArgument};
use crate::config::Pairs;
use crate::input::EditingState;
use crate::pairs::{Query, Status, Ticket, Typed};
use crate::window::ViewId;

use super::{Action, Session};

pub(super) struct Typing {
    point: Point,
    view: ViewId,
    mark: (Option<crate::coordinates::Scalar>, bool),
    state: Option<EditingState>,
    policy: Pairs,
    input: Typed,
    counted: bool,
    ticket: Option<Ticket>,
    plan: Option<plan::Plan>,
}

impl Session {
    pub(super) fn pair_control_command(&mut self, command: Command) -> Action {
        match command {
            Command::AutoPairMode => self.toggle_pairs(),
            Command::PairHighlightMode | Command::PairHighlightStyle => {
                self.pair_highlight_command(command)
            }
            _ => self.pair_command(command),
        }
    }

    pub(super) fn toggle_pairs(&mut self) -> Action {
        if !self.source_layout_focus() || self.terminals.focused || self.git_visible() {
            return self.message("Automatic pairing requires source input focus");
        }
        let result = self.buffers.get_mut(self.view().point.stamp.buffer);
        match result {
            Ok(buffer) => {
                let enabled = buffer.toggle_auto_pairs();
                self.message(if enabled {
                    "Auto pair mode enabled"
                } else {
                    "Auto pair mode disabled"
                })
            }
            Err(error) => self.error(&error.to_string()),
        }
    }

    pub(super) fn start_pair_typing(
        &mut self,
        character: char,
        argument: PrefixArgument,
    ) -> Option<Action> {
        if !matches!(
            character,
            '(' | ')' | '[' | ']' | '{' | '}' | '"' | '\'' | '`'
        ) || !self.pair_entry_focus()
        {
            return None;
        }
        if let Some(action) = self.start_region_typing(character, argument) {
            return Some(action);
        }
        let policy = self.pair_typing_policy(character)?;
        let buffer = self.buffer().ok()?;
        if buffer.is_readonly() {
            return Some(self.error("Source text is read-only"));
        }
        let count = usize::try_from(argument.value()).ok()?;
        let point = self.view().point;
        self.layout.pair = Some(Typing {
            point,
            view: self.windows.selected_id(),
            mark: (buffer.mark(), buffer.mark_active()),
            state: self.editing_state(),
            policy,
            input: Typed {
                point: point.position,
                end: point.position,
                character,
                count,
                skip: policy.skip_closing,
            },
            counted: argument.supplied(),
            ticket: None,
            plan: None,
        });
        Some(
            self.layout_pair_typing()
                .unwrap_or_else(|| self.publish_pair_typing()),
        )
    }

    fn pair_typing_policy(&self, character: char) -> Option<Pairs> {
        let buffer = self.buffer().ok()?;
        let policy = buffer.pair_policy();
        if !policy.enabled
            || buffer.mark_active()
            || !policy.quotes && matches!(character, '"' | '\'' | '`')
        {
            return None;
        }
        Some(policy)
    }

    fn pair_entry_focus(&self) -> bool {
        self.source_layout_focus()
            && !self.terminals.focused
            && !self.git_visible()
            && !self.capture_blocks_input()
            && !self.context_storage.locked()
            && self.pending.is_none()
            && self.editing_state().is_none_or(EditingState::text_entry)
    }

    fn pair_typing_current(&self, work: &Typing) -> bool {
        self.pair_entry_focus()
            && work.view == self.windows.selected_id()
            && work.point == self.view().point
            && work.state == self.editing_state()
            && self.buffer().is_ok_and(|b| {
                !b.is_readonly()
                    && work.policy == b.pair_policy()
                    && work.mark == (b.mark(), b.mark_active())
            })
    }

    pub(super) fn cancel_pair_typing(&mut self) {
        if let Some(work) = self.layout.pair.take()
            && let Some(ticket) = work.ticket
            && let Ok(buffer) = self.buffers.get_mut(ticket.stamp.buffer)
        {
            buffer.cancel_pair_request(ticket);
        }
    }

    pub(super) fn prepare_pair_typing(&mut self) -> Result<(), crate::view::Error> {
        let mut work = self.layout.pair.take().expect("owned pair entry");
        let result = self.prepare_pair_plan(&mut work);
        self.layout.pair = Some(work);
        result.map_err(Into::into)
    }

    fn prepare_pair_plan(&mut self, work: &mut Typing) -> Result<(), Error> {
        if !self.pair_typing_current(work) {
            return Err(Error::ChangedPositions);
        }
        self.admit_pair_request(work)?;
        let buffer = self.buffers.get_mut(work.point.stamp.buffer)?;
        let ticket = work.ticket.expect("admitted pair ticket");
        match buffer.poll_pair_request(ticket, Instant::now() + crate::pairs::SLICE_TIME) {
            Status::Pending => return Err(crate::coordinates::Error::Pending.into()),
            Status::Retired => return Err(Error::ChangedPositions),
            Status::Ready => (),
        }
        let outcome = buffer
            .pair_answer(ticket)
            .expect("ready pair answer")
            .outcome
            .clone();
        work.plan = Some(self.pair_plan(work, &outcome)?);
        Ok(())
    }

    fn admit_pair_request(&mut self, work: &mut Typing) -> Result<(), Error> {
        if work.ticket.is_none() {
            self.admit_pair_payload(work.input.count.checked_mul(2).ok_or(Error::Capacity)?)?;
            let literal = work.input.character.to_string().repeat(work.input.count);
            work.input.end = crate::coordinates::Scalar(self.entry_text_range(&literal)?.end);
            work.ticket = Some(
                self.buffers
                    .get_mut(work.point.stamp.buffer)?
                    .start_pair_request(Query::Typing(work.input))?,
            );
        }
        Ok(())
    }

    pub(super) fn publish_pair_typing(&mut self) -> Action {
        let Some(work) = self.layout.pair.take() else {
            return self.message("Pair entry retired");
        };
        if !self.pair_typing_current(&work) {
            return self.message("Pair entry source changed");
        }
        let plan = work.plan.expect("complete pair plan");
        self.dismiss_completion();
        if plan.text.is_empty() && plan.range.is_empty() {
            return self.publish_pair_skip(plan.cursor);
        }
        let result = self.apply_pair_plan(&plan, work.counted);
        self.sync_input_context();
        let changed = matches!(result, Ok(true));
        let action = self.finish(result);
        if changed {
            self.schedule_completion(Instant::now());
        }
        action
    }

    fn publish_pair_skip(&mut self, position: crate::coordinates::Scalar) -> Action {
        self.qualify_repeat();
        let result = (|| {
            self.stop_typing()?;
            let mut point = self.view().point;
            point.position = position;
            self.place_point(point)
        })();
        if result.is_ok() {
            self.retain_pair_skip();
        }
        self.sync_input_context();
        match result {
            Ok(()) => self.message(""),
            Err(error) => self.error(&error.to_string()),
        }
    }

    fn apply_pair_plan(&mut self, plan: &plan::Plan, counted: bool) -> Result<bool, Error> {
        if counted {
            self.stop_typing()?;
            self.finish_repeat_entry();
            return self.edit(plan.range.clone(), &plan.text, false, Some(plan.cursor));
        }
        if plan.literal {
            self.type_text(&plan.text)
        } else {
            self.entry_pair_text(plan.range.clone(), &plan.text, plan.cursor)
        }
    }

    fn admit_pair_payload(&self, bytes: usize) -> Result<(), Error> {
        if bytes > 8 * 1024 * 1024 {
            return Err(Error::Capacity);
        }
        self.buffer()?.check_edit_payload(bytes)
    }
}
