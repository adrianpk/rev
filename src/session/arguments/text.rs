use std::borrow::Cow;
use std::ops::Range;

use crate::buffer::edit::Error;
use crate::command::{Command, PrefixArgument};
use crate::config::IndentStyle;
use crate::coordinates::Coordinates;
use crate::input::{Invocation, InvocationOrigin, TextInvocation};
use crate::session::{Action, Session};

const INSERTION_BYTES: usize = 8 * 1024 * 1024;

pub(super) enum Entry {
    Literal(String),
    Newline,
    Tab,
}

/// The source stays untouched while coordinates and the complete payload are
/// admitted. Deferred input owns cancellation and source/point invalidation.
pub(in crate::session) struct Text {
    origin: InvocationOrigin,
    argument: PrefixArgument,
    entry: Entry,
    generated: Option<String>,
    range: Option<Range<usize>>,
    mark: (Option<crate::coordinates::Scalar>, bool),
}

impl Session {
    pub(in crate::session) fn execute_argument_text(
        &mut self,
        invocation: TextInvocation,
    ) -> Action {
        self.start_argument_text(
            invocation.origin,
            invocation.argument,
            Entry::Literal(invocation.text),
        )
    }

    pub(super) fn execute_argument_insertion(&mut self, invocation: Invocation) -> Action {
        let entry = if invocation.command == Command::Newline {
            Entry::Newline
        } else {
            Entry::Tab
        };
        self.start_argument_text(invocation.origin, invocation.argument, entry)
    }

    fn start_argument_text(
        &mut self,
        origin: InvocationOrigin,
        argument: PrefixArgument,
        entry: Entry,
    ) -> Action {
        if let Some(action) = self.argument_text_guard(origin) {
            return action;
        }
        if argument.value() < 0 {
            return self.error("Text insertion requires a nonnegative argument");
        }
        if argument.value() == 0 {
            return self.message("");
        }
        if matches!(&entry, Entry::Literal(text) if text.contains('\0')) {
            return self.error("Paste contains NUL; binary input rejected");
        }
        let buffer = match self.buffer() {
            Ok(buffer) => buffer,
            Err(error) => return self.error(&error.to_string()),
        };
        let mark = (buffer.mark(), buffer.mark_active());
        self.layout.text = Some(Text {
            origin,
            argument,
            entry,
            generated: None,
            range: None,
            mark,
        });
        if let Some(action) = self.layout_argument_text() {
            return action;
        }
        self.publish_argument_text()
    }

    fn argument_text_guard(&mut self, origin: InvocationOrigin) -> Option<Action> {
        if !self.source_layout_focus() || !self.input.invocation_current(origin) {
            return Some(self.message("Source text argument origin changed"));
        }
        if self.context_storage.locked() || self.terminals.focused || self.terminal_copy_active() {
            return Some(self.error("Source text is read-only"));
        }
        if self.capture_blocks_input()
            || self.tangling.is_some()
            || self.execution.is_some()
            || self.feeds.is_some()
        {
            return Some(self.message("Source output owns input; C-g cancels remaining work"));
        }
        None
    }

    pub(in crate::session) fn prepare_argument_text(&mut self) -> Result<(), crate::view::Error> {
        let mut work = self
            .layout
            .text
            .take()
            .expect("text request owns its preparation");
        let result = self.prepare_argument_payload(&mut work);
        self.layout.text = Some(work);
        result.map_err(Into::into)
    }

    fn prepare_argument_payload(&self, work: &mut Text) -> Result<(), Error> {
        let buffer = self.buffer()?;
        if !self.input.invocation_current(work.origin)
            || (buffer.mark(), buffer.mark_active()) != work.mark
        {
            return Err(Error::ChangedPositions);
        }
        let count = usize::try_from(work.argument.value()).expect("positive text argument");
        if work.generated.is_none() {
            work.generated = Some(self.generate_argument_payload(work, buffer, count)?);
        }
        let generated = work.generated.as_deref().expect("payload admitted");
        work.range = Some(self.argument_payload_range(work, buffer, generated)?);
        Ok(())
    }

    fn argument_text_unit<'a>(
        &self,
        entry: &'a Entry,
        buffer: &'a crate::buffer::Buffer,
    ) -> Result<(Cow<'a, str>, Option<usize>), Error> {
        Ok(match entry {
            Entry::Literal(text) => (Cow::Borrowed(text.as_str()), None),
            Entry::Newline => (Cow::Borrowed(buffer.newline()), None),
            Entry::Tab => match buffer.policy().indent_style {
                IndentStyle::Tabs => (Cow::Borrowed("\t"), None),
                IndentStyle::Spaces => {
                    let width = buffer.policy().indent_width;
                    let column = Coordinates::new(buffer)?
                        .to_cell(self.view().point.position)?
                        .column;
                    let first = width - column % width;
                    (Cow::Owned(" ".repeat(first)), Some(width))
                }
            },
        })
    }

    fn generate_argument_payload(
        &self,
        work: &Text,
        buffer: &crate::buffer::Buffer,
        count: usize,
    ) -> Result<String, Error> {
        let (unit, suffix) = self.argument_text_unit(&work.entry, buffer)?;
        let bytes = if let Some(width) = suffix {
            (count - 1)
                .checked_mul(width)
                .and_then(|bytes| bytes.checked_add(unit.len()))
        } else {
            unit.len().checked_mul(count)
        }
        .ok_or(Error::Capacity)?;
        if bytes > INSERTION_BYTES {
            return Err(Error::Capacity);
        }
        buffer.check_edit_payload(bytes)?;
        Ok(if suffix.is_some() {
            " ".repeat(bytes)
        } else {
            unit.repeat(count)
        })
    }

    fn argument_payload_range(
        &self,
        work: &Text,
        buffer: &crate::buffer::Buffer,
        generated: &str,
    ) -> Result<Range<usize>, Error> {
        // Literal replacement uses the admitted region once. Newline and tab
        // retain their existing insertion policy, independent of region state.
        let region = if matches!(work.entry, Entry::Literal(_)) {
            buffer.region(self.view().point)?
        } else {
            None
        };
        Ok(
            if let Some(region) = region.filter(|span| !span.range.is_empty()) {
                region.range
            } else if matches!(work.entry, Entry::Literal(_)) {
                self.entry_text_range(generated)?
            } else {
                let at = self.view().point.position.0;
                at..at
            },
        )
    }

    pub(in crate::session) fn publish_argument_text(&mut self) -> Action {
        let Some(work) = self.layout.text.take() else {
            return self.message("Text argument retired");
        };
        if !self.input.invocation_current(work.origin)
            || !self
                .buffer()
                .is_ok_and(|buffer| (buffer.mark(), buffer.mark_active()) == work.mark)
        {
            return self.message("Source text argument origin changed");
        }
        self.dismiss_completion();
        self.command_boundary(Command::Newline);
        let result = (|| {
            self.stop_typing()?;
            self.finish_repeat_entry();
            let changed = self.replace(
                work.range.expect("prepared range"),
                work.generated.as_deref().expect("prepared text"),
            )?;
            self.windows.selected_mut().reset_navigation();
            Ok(changed)
        })();
        self.sync_input_context();
        let changed = matches!(result, Ok(true));
        let action = self.finish(result);
        if changed {
            self.schedule_completion(std::time::Instant::now());
        }
        action
    }
}
