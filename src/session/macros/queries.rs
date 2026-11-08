use super::*;
use crate::input::MacroQuery;
use crate::macros::{PAUSE_LIMIT, Pause, QueryMode, Step};

impl Session {
    pub(in crate::session) fn macro_manual(&self) -> bool {
        self.macros.runner.is_none()
            && self.macros.recorder.is_none()
            && !self.macros.pauses.is_empty()
    }

    pub(super) fn macro_query_command(&mut self, argument: PrefixArgument) -> Action {
        if !self.macro_source() || (self.macros.recorder.is_none() && !self.macros.expanded) {
            return self.message("Macro query requires recording or playback");
        }
        let mode = if argument.supplied() {
            QueryMode::Edit
        } else {
            QueryMode::Confirm
        };
        if let Some(recorder) = &mut self.macros.recorder
            && let Err(reason) = recorder
                .stage(Step::Query(mode))
                .and_then(|()| recorder.accept())
        {
            return self.stop_macro(reason, "Keyboard macro recording limit exceeded");
        }
        if self.macros.recorder.is_some() && mode == QueryMode::Confirm {
            return self.message("Macro query recorded");
        }
        self.macro_query_mode(mode)
    }

    pub(super) fn macro_query_mode(&mut self, mode: QueryMode) -> Action {
        if mode == QueryMode::Edit {
            return self.macro_pause(false);
        }
        if let Err(error) = self.input.macro_query() {
            return self.error(&error.to_string());
        }
        let run = self.macros.runner.as_mut().expect("query playback owner");
        run.query = true;
        run.limits.pause();
        self.focus_message("")
    }

    pub(in crate::session) fn macro_query_answer(&mut self, answer: MacroQuery) -> Action {
        let expected = (self.windows.selected_id(), self.view().point);
        let Some(run) = &mut self.macros.runner else {
            return self.message("No macro query is active");
        };
        if !run.query || run.expected != expected {
            return self.stop_macro(StopReason::OwnerChanged, "Macro query source changed");
        }
        if answer == MacroQuery::Edit {
            return self.macro_pause(true);
        }
        run.query = false;
        run.limits.resume();
        match answer {
            MacroQuery::Skip => {
                run.step = run.definition.steps.len();
                run.trigger.clear();
                run.arguments.clear();
            }
            MacroQuery::Stop => {
                // An incomplete append pre-execution cannot open capture.
                run.append = false;
                return self.macro_run_finished();
            }
            _ => {}
        }
        self.message("Keyboard macro resumed")
    }

    fn macro_pause(&mut self, query: bool) -> Action {
        if self.macros.pauses.len() == PAUSE_LIMIT {
            return self.stop_macro(StopReason::PauseLimit, "Macro manual pause limit reached");
        }
        if let Some(run) = &self.macros.runner {
            run.limits.pause();
        }
        let pause = Pause {
            recorder: self.macros.recorder.take(),
            runner: self.macros.runner.take(),
            query,
            expected: (self.windows.selected_id(), self.view().point),
            entry: self.macro_entry(),
            waiting: false,
        };
        self.macros.pauses.push(pause);
        self.buffers.automatic_group(None);
        self.input.cancel();
        self.input.macro_editing = true;
        self.message("Macro edit: C-M-c resumes; C-] or C-g aborts")
    }

    pub(super) fn macro_resume(&mut self) -> Action {
        if !self.macro_manual() {
            return self.message("No macro editing pause is active");
        }
        if let Some(action) = self.macro_manual_guard() {
            return action;
        }
        if !self.macro_source() {
            return self.message("Finish the native prompt before resuming the macro");
        }
        let pause = self.macros.pauses.pop().expect("manual pause owner");
        self.input.macro_editing = false;
        self.macros.recorder = pause.recorder;
        self.macros.runner = pause.runner;
        let expected = (self.windows.selected_id(), self.view().point);
        if let Some(run) = &mut self.macros.runner {
            run.expected = expected;
            if pause.query {
                if let Err(error) = self.input.macro_query() {
                    return self.error(&error.to_string());
                }
                return self.focus_message("");
            }
            run.limits.resume();
        }
        self.message("Keyboard macro resumed")
    }

    pub(in crate::session) fn macro_manual_guard(&mut self) -> Option<Action> {
        let pause = self.macros.pauses.last()?;
        let current = (self.windows.selected_id(), self.view().point);
        if pause.expected != current || pause.entry.profile != self.input.interaction_profile() {
            return Some(self.stop_macro(
                StopReason::OwnerChanged,
                "Macro manual editing source changed",
            ));
        }
        None
    }

    pub(in crate::session) fn macro_manual_progress(&mut self) {
        if !self.macro_manual() {
            return;
        }
        let current = (self.windows.selected_id(), self.view().point);
        let waiting = self.layout_active();
        let pause = self.macros.pauses.last_mut().expect("manual pause owner");
        if pause.expected.0 != current.0 || pause.expected.1.stamp.buffer != current.1.stamp.buffer
        {
            self.stop_macro(
                StopReason::OwnerChanged,
                "Macro manual editing source changed",
            );
        } else {
            pause.expected = current;
            pause.waiting = waiting;
        }
    }

    pub(super) fn macro_child_finished(&mut self) {
        if self
            .macros
            .step_editor
            .as_ref()
            .is_some_and(|e| e.capturing)
        {
            self.macro_step_capture_progress();
            if let Some(run) = self.macros.pauses.last().and_then(|p| p.runner.as_ref()) {
                run.limits.pause();
            }
            return;
        }
        if !self.macro_manual() {
            return;
        }
        self.input.macro_editing = true;
        self.macro_manual_progress();
        if let Some(run) = self
            .macros
            .pauses
            .last()
            .and_then(|pause| pause.runner.as_ref())
        {
            run.limits.pause();
        }
    }
}
