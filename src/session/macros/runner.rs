use crate::macros::{Step, StopReason};
use crate::session::{Action, Session};
use crossterm::event::{Event, KeyEvent};
use std::time::{Duration, Instant};

impl Session {
    /// One finite host turn; native layout continuations finish before the next
    /// logical input. Only a live runner requests another immediate wake.
    pub(in crate::session) fn expire_macro(&mut self) -> Action {
        if self.macros.runner.is_none() {
            return Action::Wait;
        }
        if self.layout_active() {
            return Action::Wait;
        }
        let started = Instant::now();
        for _ in 0..256 {
            let entry = self.macro_entry();
            let point = (self.windows.selected_id(), self.view().point);
            let Some(run) = self.macros.runner.as_mut() else {
                break;
            };
            if run.waiting {
                run.expected = point;
                run.waiting = false;
            }
            if point != run.expected {
                return self.stop_macro(
                    StopReason::OwnerChanged,
                    "Keyboard macro source owner changed",
                );
            }
            if run.step == run.definition.steps.len() {
                run.iteration += 1;
                if run.iteration == run.iterations {
                    self.macros.runner = None;
                    self.buffers.automatic_group(None);
                    return self.message("Keyboard macro finished");
                }
                if entry != run.definition.entry {
                    return self.stop_macro(
                        StopReason::EntryState,
                        "Keyboard macro did not restore its entry state for another iteration",
                    );
                }
                run.step = 0;
                run.group = crate::buffer::edit::AutomaticGroup::new(run.budget.clone());
            }
            if run.started.elapsed() >= Duration::from_secs(60) {
                return self.stop_macro(
                    StopReason::TimeBudget,
                    "Keyboard macro active time limit reached",
                );
            }
            if run.dispatched >= crate::macros::RUN_STEPS {
                return self.stop_macro(
                    StopReason::StepBudget,
                    "Keyboard macro logical step limit reached",
                );
            }
            let event = match &run.definition.steps[run.step] {
                Step::Key(key) => Event::Key(KeyEvent::new(key.code, key.modifiers)),
                Step::Text(text) => Event::Paste(text.clone()),
            };
            run.step += 1;
            run.dispatched += 1;
            let group = run.group.clone();
            self.buffers.automatic_group(Some(group));
            self.macros.expanded = true;
            self.handle(event);
            self.macros.expanded = false;
            self.buffers.automatic_group(None);
            let waiting = self.layout_active();
            let expected = (self.windows.selected_id(), self.view().point);
            if let Some(run) = self.macros.runner.as_mut() {
                run.expected = expected;
                run.waiting = waiting;
            } else {
                return Action::Redraw;
            }
            if waiting || started.elapsed() >= crate::session::events::INPUT_BATCH_TIME {
                break;
            }
        }
        Action::Redraw
    }

    pub(in crate::session) fn macro_layout_group(&mut self, active: bool) {
        let group = active
            .then(|| self.macros.runner.as_ref().map(|r| r.group.clone()))
            .flatten();
        self.buffers.automatic_group(group);
        self.macros.expanded = active && self.macros.runner.is_some();
        self.macros.dispatching = active && self.macros.recorder.is_some() && self.layout_active();
    }
}
