use super::{Intent, Observation, Payload, Pending, Record, TEXT_BYTES, Trim};
use crate::command::{Command, Editing, ModalCommand, editing::Action};
use crate::input::{EditingState, Focus};
use crate::session::Session;

impl Session {
    pub(in crate::session) fn qualify_repeat(&mut self) {
        if self
            .repeat
            .pending
            .as_ref()
            .is_some_and(|p| p.view != self.windows.selected_id() || p.point != self.view().point)
        {
            // An external revision or changed view cannot complete a partial
            // insertion record. Completed semantic changes remain portable.
            self.repeat.pending = None;
        }
    }

    pub(in crate::session) fn finish_repeat_entry(&mut self) {
        if let Some(pending) = self.repeat.pending.take()
            && pending.changed
            && pending.valid
        {
            self.repeat.last = Some(pending.record);
        }
    }

    pub(in crate::session) fn before_repeat_command(&mut self, command: Command) -> Observation {
        self.qualify_repeat();
        let text = self.editing_state().is_some_and(EditingState::text_entry)
            && matches!(
                command,
                Command::Newline
                    | Command::InsertTab
                    | Command::DeleteChar
                    | Command::BackwardDeleteChar
            );
        if text {
            self.begin_repeat_text();
        } else {
            self.finish_repeat_entry();
        }
        self.repeat.text = text;
        self.qualify_repeat_control(command);
        let record = self.repeat_candidate(command);
        let entry = matches!(record.as_ref().map(|r| &r.intent), Some(Intent::Entry(_)))
            || matches!(command, Command::Edit(r) if r.action.changes() && (!r.pending() || self.editing_state().and_then(EditingState::visual).is_some()));
        let preserve = matches!(
            command,
            Command::History(_)
                | Command::Edit(Editing {
                    action: Action::Repeat,
                    ..
                })
        );
        Observation {
            preserve,
            point: self.view().point,
            record,
            entry,
        }
    }

    fn qualify_repeat_control(&mut self, command: Command) {
        let unsupported = command == Command::DeleteChar
            || command == Command::BackwardDeleteChar
                && self.editing_state() == Some(EditingState::Replace);
        if unsupported && let Some(pending) = &mut self.repeat.pending {
            pending.valid = false;
        }
    }

    fn repeat_candidate(&self, command: Command) -> Option<Record> {
        if self.repeat.replaying
            || self.interaction_profile() != crate::config::InteractionProfile::Modal
            || self.input.focus() != Focus::Editing
            || self.tree.focused()
        {
            return None;
        }
        let intent = match command {
            Command::Modal(entry)
                if matches!(
                    entry,
                    ModalCommand::Insert
                        | ModalCommand::InsertAfter
                        | ModalCommand::InsertLineFirst
                        | ModalCommand::InsertLineEnd
                        | ModalCommand::OpenBelow
                        | ModalCommand::OpenAbove
                        | ModalCommand::Replace
                ) =>
            {
                Intent::Entry(entry)
            }
            Command::Edit(request) => self.repeat_edit_intent(request)?,
            _ => return None,
        };
        Some(Record {
            intent,
            text: String::new(),
        })
    }

    fn repeat_edit_intent(&self, request: Editing) -> Option<Intent> {
        if !repeatable(request) {
            return None;
        }
        if request.pending()
            && self
                .editing_state()
                .and_then(EditingState::visual)
                .is_none()
        {
            return None;
        }
        if matches!(request.action, Action::PasteAfter | Action::PasteBefore) {
            return Some(Intent::Paste {
                request,
                payload: self.repeat_payload()?,
            });
        }
        let geometry = self
            .editing_state()
            .and_then(EditingState::visual)
            .map(|shape| self.repeat_geometry(shape))
            .transpose()
            .ok()?;
        Some(Intent::Edit(request, geometry))
    }

    fn repeat_payload(&self) -> Option<Payload> {
        if let Some(block) = self.kills.block() {
            let size = block
                .rows()
                .iter()
                .try_fold(0usize, |n, row| n.checked_add(row.len()))?;
            return (size <= TEXT_BYTES && block.rows().len() <= TEXT_BYTES)
                .then(|| Payload::Block((*block).clone()));
        }
        let text = self.kills.text()?;
        (text.len_bytes() <= TEXT_BYTES).then(|| {
            Payload::Linear(
                self.kills.shape().unwrap_or(crate::kill::Shape::Linear),
                text.to_string(),
            )
        })
    }

    pub(in crate::session) fn after_repeat_command(&mut self, before: Observation) {
        let changed = self.view().point.stamp != before.point.stamp;
        let text = std::mem::take(&mut self.repeat.text);
        if self.repeat.replaying || text || before.preserve {
            return;
        }
        if let Some(record) = before.record {
            if before.entry && self.editing_state().is_some_and(EditingState::text_entry) {
                self.start_repeat_entry(record, changed);
            } else if changed {
                self.repeat.last = Some(record);
            }
        } else if changed && self.view().point.stamp.buffer == before.point.stamp.buffer {
            self.repeat.last = None;
        }
    }

    fn start_repeat_entry(&mut self, record: Record, changed: bool) {
        let first = changed
            .then(|| self.buffer().ok()?.history().back().map(|r| r.before))
            .flatten();
        if changed {
            self.repeat.last = None;
        }
        let trim = self.repeat_trim(&record, changed);
        self.repeat.pending = Some(Pending {
            trim,
            view: self.windows.selected_id(),
            point: self.view().point,
            first,
            record,
            valid: true,
            changed,
        });
    }

    fn repeat_trim(&self, record: &Record, changed: bool) -> Trim {
        match record.intent {
            Intent::Entry(ModalCommand::OpenBelow) => Trim::Prefix,
            Intent::Entry(ModalCommand::OpenAbove) => Trim::Suffix,
            Intent::Edit(_, _) if changed => {
                let newline = self.buffer().expect("entry owner").newline();
                let inserted = self
                    .buffer()
                    .ok()
                    .and_then(|b| b.history().back())
                    .and_then(|r| r.changes().first())
                    .map(|c| c.inserted().to_string());
                if inserted.as_deref() == Some(newline) {
                    Trim::Suffix
                } else {
                    Trim::None
                }
            }
            _ => Trim::None,
        }
    }

    pub(in crate::session) fn begin_repeat_text(&mut self) {
        self.qualify_repeat();
        if self
            .repeat
            .pending
            .as_ref()
            .is_some_and(|p| matches!(p.record.intent, Intent::Entry(ModalCommand::Replace)))
            && self.editing_state() != Some(EditingState::Replace)
        {
            self.repeat.pending = None;
        }
        if self.repeat.pending.is_none()
            && self.editing_state().is_some_and(EditingState::text_entry)
        {
            let entry = if self.editing_state() == Some(EditingState::Replace) {
                ModalCommand::Replace
            } else {
                ModalCommand::Insert
            };
            self.start_repeat_entry(
                Record {
                    intent: Intent::Entry(entry),
                    text: String::new(),
                },
                false,
            );
        }
    }
}

fn repeatable(request: Editing) -> bool {
    match request.action {
        Action::Copy
        | Action::CopyEnd
        | Action::SelectInner
        | Action::SelectAround
        | Action::Repeat => false,
        Action::Replace => request.character.is_some(),
        _ => !matches!(
            request.operand,
            Some(crate::command::editing::Operand::Resolved { .. })
        ),
    }
}
