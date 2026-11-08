use super::*;
use crate::command::ArgumentForm;
use crate::macros::Repeat;

impl Session {
    pub(super) fn macro_start_recording(&mut self, argument: PrefixArgument) -> Action {
        self.macros.repeat = None;
        if !self.macros.pauses.is_empty() {
            return self.message("Resume or abort manual macro editing before recording");
        }
        if self.macros.recorder.is_some() {
            return self.stop_macro(
                StopReason::Unsupported,
                "A keyboard macro is already being recorded",
            );
        }
        if !self.macro_source() {
            return self.message("Keyboard macro requires ordinary source editing");
        }
        self.macros.stop = None;
        if !argument.supplied() {
            let mut recorder = Recorder::new(self.macro_entry());
            recorder.initial_format = self.macros.default_format.clone();
            recorder.counter.format = recorder.initial_format.clone();
            self.macros.recorder = Some(recorder);
        } else {
            let Some(definition) = self.macros.ring.front().cloned() else {
                return self.message("No keyboard macro is available to append");
            };
            if definition.entry != self.macro_entry() {
                return self.message("Keyboard macro append entry state differs");
            }
            let execute = self
                .buffers
                .snapshot()
                .settings()
                .macros
                .execute_before_append
                && !matches!(argument.form(), ArgumentForm::PlainPrefix { depth: 2 });
            if execute {
                let action = self.start_macro(1);
                if let Some(run) = &mut self.macros.runner {
                    run.append = true;
                }
                return action;
            }
            self.macros.recorder = Some(Recorder::append(&definition));
        }
        self.message("Def: recording keyboard macro")
    }

    pub(super) fn macro_run_finished(&mut self) -> Action {
        let run = self.macros.runner.as_ref().expect("completed runner");
        if run.append && (!self.macro_source() || self.macro_entry() != run.definition.entry) {
            return self.stop_macro(
                StopReason::EntryState,
                "Macro append did not restore its entry state",
            );
        }
        let mut run = self.macros.finish_run().expect("completed runner");
        self.buffers.automatic_group(None);
        if let Some(mut parent) = run.parent.take() {
            parent.expected = (self.windows.selected_id(), self.view().point);
            parent.waiting = self.layout_active();
            self.macros.runner = Some(*parent);
            return self.message("Named macro returned");
        }
        if run.append {
            let mut recorder = Recorder::append(&run.definition);
            recorder.counter = run.counter;
            self.macros.recorder = Some(recorder);
            return self.message("Def: appending keyboard macro");
        }
        self.macro_child_finished();
        self.message("Keyboard macro finished")
    }

    pub(super) fn macro_ring_command(
        &mut self,
        command: MacroCommand,
        argument: PrefixArgument,
    ) -> Action {
        if self.macros.recorder.is_some() {
            return self.stop_macro(
                StopReason::Unsupported,
                "Ring selection stops macro recording",
            );
        }
        let len = self.macros.ring.len();
        self.macros.repeat = Some(Repeat::Ring);
        if len == 0 {
            return self.message("Keyboard macro ring is empty");
        }
        match command {
            MacroCommand::Next | MacroCommand::Previous => {
                let count = i64::from(argument.value());
                let count = if command == MacroCommand::Previous {
                    -count
                } else {
                    count
                };
                self.macros
                    .ring
                    .rotate_left(count.rem_euclid(len as i64) as usize);
            }
            MacroCommand::Delete => {
                self.macros.ring.pop_front();
            }
            MacroCommand::Swap if len >= 2 => self.macros.ring.swap(0, 1),
            MacroCommand::CallSecond if len >= 2 => {
                // Pin the second definition without changing the selected head.
                self.macros.ring.swap(0, 1);
                let count = argument.value();
                let action = self.start_macro(count);
                self.macros.ring.swap(0, 1);
                return action;
            }
            _ => return self.message("Keyboard macro ring needs two entries"),
        }
        self.message(&format!(
            "Keyboard macro ring: {} entries",
            self.macros.ring.len()
        ))
    }
}
