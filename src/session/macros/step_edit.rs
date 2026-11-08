//! Interactive questions share the ordinary runner's source/history/budget owner.
use super::*;
use crate::input::MacroStep;
use crate::input::keymap::Sequence;
use crate::macros::{
    Pause, Step,
    editing::Selection,
    library::Record,
    step_edit::{Editor, Insertion},
};
use crate::session::Pending;
use crossterm::event::Event;

impl Session {
    pub(super) fn macro_step_start(&mut self) -> Action {
        if self.macros.active() || self.macro_draft.is_some() {
            return self
                .message("Step editing requires an idle source and no retained definition draft");
        }
        let Some(source) = self.macros.ring.front().cloned() else {
            return self.message("No keyboard macro is available");
        };
        let action = self.start_definition(source.clone(), 1, None);
        if self.macros.runner.is_some() {
            self.macros.step_editor = Some(Editor {
                target: Selection::Ring(source.clone()),
                candidate: source,
                allowance: 0,
                continuing: false,
                asking: false,
                after: None,
                input: None,
                capturing: false,
            });
        }
        action
    }
    pub(super) fn macro_step_boundary(&mut self) -> Option<Action> {
        let editor = self.macros.step_editor.as_ref()?;
        let run = self.macros.runner.as_ref()?;
        if editor.capturing
            || !self.macros.pauses.is_empty()
            || editor.input.is_some()
            || run.definition.id != editor.candidate.id
            || run.parent.is_some()
            || !run.trigger.is_empty()
            || !run.arguments.is_empty()
            || self.pending.is_some()
            || self.input.focus() != Focus::Editing
            || self.input.macro_input_pending()
        {
            return None;
        }
        if run.expected != (self.windows.selected_id(), self.view().point) {
            return Some(self.stop_macro(StopReason::OwnerChanged, "Step editor source changed"));
        }
        if let Some(request) = editor.after.filter(|r| run.step >= r.index) {
            self.macros.step_editor.as_mut().expect("step editor").after = None;
            return Some(self.macro_step_read(request));
        }
        if run.step >= editor.candidate.steps.len() || editor.continuing || editor.allowance > 0 {
            return None;
        }
        Some(self.macro_step_ask())
    }
    fn macro_step_ask(&mut self) -> Action {
        let Some(run) = self.macros.runner.as_mut() else {
            return self.message("No step editor runner");
        };
        let next = run
            .definition
            .steps
            .get(run.step)
            .map(Step::summary)
            .unwrap_or_else(|| "[finish]".into());
        let label = format!(
            "Step {}: {}  [SPC/y execute, n delete, f skip, ? help] ",
            run.step + 1,
            next
        );
        run.query = true;
        run.limits.pause();
        self.macros
            .step_editor
            .as_mut()
            .expect("step editor")
            .asking = true;
        if let Err(e) = self.input.macro_step(label) {
            return self.error(&e.to_string());
        }
        self.message("")
    }
    pub(in crate::session) fn macro_step_answer(&mut self, answer: MacroStep) -> Action {
        if self.macros.step_editor.as_ref().is_none_or(|e| !e.asking) {
            return self.message("No step question is active");
        }
        if answer == MacroStep::Cancel {
            return self.stop_macro(
                StopReason::Canceled,
                "Step editing canceled; real source edits retained",
            );
        }
        let text_entry = self.editing_state().is_none_or(|s| s.text_entry());
        let current = (self.windows.selected_id(), self.view().point);
        let Some(run) = self.macros.runner.as_mut() else {
            return self.message("Step editor runner was lost");
        };
        if run.expected != current {
            return self.stop_macro(StopReason::OwnerChanged, "Step editor source changed");
        }
        run.query = false;
        run.limits.resume();
        let index = run.step;
        let editor = self.macros.step_editor.as_mut().expect("step editor");
        editor.asking = false;
        match answer {
            MacroStep::Execute => editor.allowance = 1,
            MacroStep::Continue => editor.continuing = true,
            MacroStep::Skip => run.step += 1,
            MacroStep::Group => {
                let text = |s: &Step| {
                    matches!(s, Step::Text(_))
                        || matches!(s,Step::Key(key) if key.modifiers.is_empty() && matches!(key.code,crossterm::event::KeyCode::Char(_)))
                };
                editor.allowance = editor.candidate.steps[index..]
                    .iter()
                    .take_while(|s| text_entry && text(s))
                    .count()
                    .max(1);
            }
            MacroStep::Remove | MacroStep::RemoveRest => {
                let mut steps = editor.candidate.steps.to_vec();
                if answer == MacroStep::Remove {
                    if index < steps.len() {
                        steps.remove(index);
                    }
                } else {
                    steps.truncate(index);
                }
                return match self.macro_step_candidate(steps) {
                    Ok(()) => self.message("Pending definition updated; source effects retained"),
                    Err(e) => self.stop_macro(StopReason::CaptureLimit, &e),
                };
            }
            MacroStep::Insert
            | MacroStep::InsertKey
            | MacroStep::Replace
            | MacroStep::ReplaceKey => {
                let request = Insertion {
                    index,
                    replace: matches!(answer, MacroStep::Replace | MacroStep::ReplaceKey),
                    key_only: matches!(answer, MacroStep::InsertKey | MacroStep::ReplaceKey),
                };
                return self.macro_step_read(request);
            }
            MacroStep::After => {
                editor.allowance = 1;
                editor.after = Some(Insertion {
                    index: index + 1,
                    replace: false,
                    key_only: false,
                });
            }
            MacroStep::Append => {
                editor.continuing = true;
                editor.after = Some(Insertion {
                    index: editor.candidate.steps.len(),
                    replace: false,
                    key_only: false,
                });
            }
            MacroStep::Help => {
                self.macro_step_ask();
                return self.message("Step: SPC/y execute; n/d/DEL remove; f skip once; TAB text group; c continue; C-k remove rest; i/I insert; r/R replace; a after; A run remainder then append; lowercase input ends C-j; q/C-g cancels changes");
            }
            MacroStep::Cancel => {}
        }
        self.message("Step editor resumed")
    }
    fn macro_step_candidate(&mut self, steps: Vec<Step>) -> Result<(), String> {
        if steps.len() > crate::macros::STEP_LIMIT {
            return Err("Step edit exceeds 4096 steps".into());
        }
        let editor = self.macros.step_editor.as_ref().ok_or("No step editor")?;
        let mut definition = (*editor.candidate).clone();
        definition.retained_bytes = steps
            .iter()
            .map(|s| s.bytes() + std::mem::size_of::<Step>())
            .sum();
        definition.steps = steps.into();
        let candidate = Arc::new(definition);
        Record::draft(&Selection::Ring(candidate.clone()), None).map_err(|e| e.to_string())?;
        self.macros
            .admit_memory(Some(&candidate))
            .map_err(|_| "Step edit exceeds 48 MiB retained memory")?;
        self.macros
            .step_editor
            .as_mut()
            .expect("step editor")
            .candidate = candidate.clone();
        self.macros
            .runner
            .as_mut()
            .ok_or("Step editor runner was lost")?
            .definition = candidate;
        Ok(())
    }
    fn macro_step_read(&mut self, request: Insertion) -> Action {
        self.input.cancel();
        let editor = self.macros.step_editor.as_mut().expect("step editor");
        editor.input = Some(request);
        editor.asking = false;
        editor.continuing = false;
        editor.allowance = 0;
        if request.key_only {
            if let Err(e) = self.input.macro_literal_key_capture() {
                return self.error(&e.to_string());
            }
            let run = self.macros.runner.as_mut().expect("step runner");
            run.query = true;
            run.limits.pause();
            self.pending = Some(Pending::MacroStepKey);
            return self.message("Read one native key sequence; C-g cancels step editing");
        }
        if self.macros.pauses.len() == crate::macros::PAUSE_LIMIT {
            return self.stop_macro(StopReason::PauseLimit, "Step input pause limit reached");
        }
        let mut run = self.macros.runner.take().expect("step runner");
        run.query = false;
        run.limits.pause();
        let mut recorder = Recorder::new(self.macro_entry());
        recorder.counter = run.counter.clone();
        recorder.initial_format = run.counter.format.clone();
        self.macros.pauses.push(Pause {
            runner: Some(run),
            recorder: None,
            query: false,
            expected: (self.windows.selected_id(), self.view().point),
            entry: self.macro_entry(),
            waiting: false,
        });
        self.macros.recorder = Some(recorder);
        self.macros
            .step_editor
            .as_mut()
            .expect("step editor")
            .capturing = true;
        self.message("Step input: ordinary editing is recorded; C-j finishes at a complete command; C-g cancels definition changes")
    }
    pub(in crate::session) fn macro_step_key(&mut self, keys: Sequence) -> Action {
        let Some(run) = &mut self.macros.runner else {
            return self.message("Step key target was lost");
        };
        run.query = false;
        run.limits.resume();
        self.macro_step_insert(keys.chords().iter().map(|k| Step::Key(*k)).collect(), false)
    }
    fn macro_step_insert(&mut self, input: Vec<Step>, executed: bool) -> Action {
        let Some(request) = self
            .macros
            .step_editor
            .as_mut()
            .and_then(|e| e.input.take())
        else {
            return self.message("No step insertion is pending");
        };
        let editor = self.macros.step_editor.as_ref().expect("step editor");
        let mut steps = editor.candidate.steps.to_vec();
        if request.index > steps.len() {
            return self.stop_macro(StopReason::OwnerChanged, "Step insertion target changed");
        }
        let end = (request.index + usize::from(request.replace)).min(steps.len());
        let count = input.len();
        steps.splice(request.index..end, input);
        if let Err(e) = self.macro_step_candidate(steps) {
            self.macro_step_ask();
            return self.error(&e);
        }
        let current = (self.windows.selected_id(), self.view().point);
        let run = self.macros.runner.as_mut().expect("step runner");
        run.step = request.index + if executed { count } else { 0 };
        run.expected = current;
        let editor = self.macros.step_editor.as_mut().expect("step editor");
        editor.allowance = if executed { 0 } else { count };
        editor.capturing = false;
        self.message("Step input admitted; definition publication remains pending")
    }
    pub(in crate::session) fn macro_step_input_guard(&mut self, event: &Event) -> Option<Action> {
        if self.macros.expanded
            || self
                .macros
                .step_editor
                .as_ref()
                .is_none_or(|e| !e.capturing)
        {
            return None;
        }
        let expected = self.macros.pauses.last().map(|p| p.expected);
        if expected != Some((self.windows.selected_id(), self.view().point)) {
            return Some(self.stop_macro(StopReason::OwnerChanged, "Step input source changed"));
        }
        if Input::interrupt(event) {
            return Some(self.stop_macro(
                StopReason::Canceled,
                "Step editing canceled; real source edits retained",
            ));
        }
        if matches!(event, Event::Key(key) if key.kind != crossterm::event::KeyEventKind::Release && key.modifiers == crossterm::event::KeyModifiers::CONTROL && matches!(key.code, crossterm::event::KeyCode::Char('j' | 'J')))
            && self.pending.is_none()
            && self.input.focus() == Focus::Editing
            && !self.input.macro_input_pending()
        {
            return Some(self.macro_step_input_finish());
        }
        None
    }
    fn macro_step_input_finish(&mut self) -> Action {
        self.macro_finish_capture();
        let Some(recorder) = self.macros.recorder.take() else {
            return self.stop_macro(StopReason::NativeFailure, "Step input recording was lost");
        };
        let Some(mut pause) = self.macros.pauses.pop() else {
            return self.stop_macro(StopReason::OwnerChanged, "Step input source was lost");
        };
        let Some(mut run) = pause.runner.take() else {
            return self.stop_macro(StopReason::OwnerChanged, "Step input runner was lost");
        };
        run.counter = recorder.counter;
        run.expected = (self.windows.selected_id(), self.view().point);
        run.waiting = self.layout_active();
        run.limits.resume();
        self.macros.runner = Some(run);
        self.macros
            .step_editor
            .as_mut()
            .expect("step editor")
            .capturing = false;
        self.macro_step_insert(recorder.steps, true)
    }
    pub(in crate::session) fn macro_step_capture_progress(&mut self) {
        if self
            .macros
            .step_editor
            .as_ref()
            .is_some_and(|e| e.capturing)
        {
            let expected = (self.windows.selected_id(), self.view().point);
            let waiting = self.layout_active();
            if let Some(pause) = self.macros.pauses.last_mut() {
                pause.expected = expected;
                pause.waiting = waiting;
            }
        }
    }
    pub(super) fn macro_step_finished(&mut self) -> Option<Action> {
        if !self.macros.pauses.is_empty()
            || self.macros.step_editor.as_ref().is_none_or(|e| e.capturing)
            || self
                .macros
                .runner
                .as_ref()
                .is_none_or(|r| r.parent.is_some())
        {
            return None;
        }
        let editor = self.macros.step_editor.take()?;
        let mut definition = (*editor.candidate).clone();
        let run = self.macros.runner.take().expect("step runner");
        definition.counter = run.counter;
        self.buffers.automatic_group(None);
        self.input.cancel();
        let limit = self.buffers.snapshot().settings().macros.ring_limit;
        Some(
            match self
                .macros
                .replace_definition(&editor.target, definition, None, limit)
            {
                Ok(()) => {
                    self.sync_macro_catalog();
                    self.message("Step-edited definition applied once; saving remains explicit")
                }
                Err(e) => self.error(&e),
            },
        )
    }
}
