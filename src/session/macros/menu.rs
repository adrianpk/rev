//! Read-only ring projection. Rows address stable definitions, never source text.
use super::*;
use crate::buffer::edit::Point;
use crate::command::{Motion, RegionCommand, Scroll};
use crate::macros::{Definition, editing::Selection};
use crate::session::Pending;
use crate::window::ViewId;
use std::collections::BTreeSet;

pub(in crate::session) struct Menu {
    view: ViewId,
    origin: Point,
    rows: Vec<(u64, u64)>,
    cursor: usize,
    column: usize,
    anchor: Option<u64>,
    marked: BTreeSet<u64>,
    flagged: BTreeSet<u64>,
}
pub(in crate::session) struct Prompt {
    target: (u64, u64),
    property: MacroCommand,
}
pub(in crate::session) struct Consent {
    targets: Vec<(u64, u64)>,
    copy: bool,
}
impl Consent {
    pub(in crate::session) fn question(&self) -> &'static str {
        if self.copy {
            "Duplicate selected ring macros? [y/n]"
        } else {
            "Delete selected ring macros? [y/n]"
        }
    }
}

impl Session {
    pub(in crate::session) fn macro_menu_active(&self) -> bool {
        self.macro_menu.as_ref().is_some_and(|m| {
            m.view == self.windows.selected_id()
                && m.origin.stamp.buffer == self.view().point.stamp.buffer
        }) && !self.macro_definition_selected()
    }
    pub(super) fn macro_menu_open(&mut self) -> Action {
        if self.macros.active() {
            return self.stop_macro(
                StopReason::Unsupported,
                "The macro menu requires an idle session",
            );
        }
        if self.macro_menu_active() {
            self.macro_menu_refresh();
            return self.message("Macro menu refreshed");
        }
        if !self.macro_source() {
            return self.message("The macro menu requires ordinary source editing");
        }
        self.macro_menu = Some(Menu {
            view: self.windows.selected_id(),
            origin: self.view().point,
            rows: Vec::new(),
            cursor: 0,
            column: 0,
            anchor: None,
            marked: BTreeSet::new(),
            flagged: BTreeSet::new(),
        });
        self.macro_menu_refresh();
        self.tree.edit();
        self.sync_input_context();
        self.message("Macro menu: ? help, e definition, m mark, d flag, x execute flags, q return")
    }
    fn macro_menu_refresh(&mut self) {
        let Some(menu) = &mut self.macro_menu else {
            return;
        };
        let selected = menu.rows.get(menu.cursor).map(|r| r.0);
        menu.rows = self
            .macros
            .ring
            .iter()
            .map(|d| (d.id, d.revision))
            .collect();
        menu.cursor = selected
            .and_then(|id| menu.rows.iter().position(|r| r.0 == id))
            .unwrap_or(menu.cursor.min(menu.rows.len().saturating_sub(1)));
        let ids: BTreeSet<_> = menu.rows.iter().map(|r| r.0).collect();
        menu.marked.retain(|id| ids.contains(id));
        menu.flagged.retain(|id| ids.contains(id));
        menu.anchor = menu.anchor.filter(|id| ids.contains(id));
    }
    pub(in crate::session) fn macro_menu_command(&mut self, command: Command) -> Option<Action> {
        if !self.macro_menu_active() {
            return None;
        }
        if command == Command::Macro(MacroCommand::MenuQuit) {
            return Some(self.macro_menu_close());
        }
        if command == Command::ExecuteExtended {
            return None;
        }
        if command == Command::KeyboardQuit {
            if matches!(self.pending, Some(Pending::Command)) {
                self.pending = None;
                self.input.cancel();
                return Some(self.message("Macro menu command canceled"));
            }
            return Some(self.macro_menu_close());
        }
        if command == Command::Quit {
            self.macro_menu_close();
            return None;
        }
        self.macro_menu_checked_command(command)
    }

    fn macro_menu_checked_command(&mut self, command: Command) -> Option<Action> {
        if matches!(command, Command::Window(_) | Command::Split(_)) {
            return None;
        }
        if matches!(
            command,
            Command::Macro(MacroCommand::MenuRefresh | MacroCommand::List)
        ) {
            self.macro_menu_refresh();
            return Some(self.message("Macro menu refreshed"));
        }
        let current: Vec<_> = self
            .macros
            .ring
            .iter()
            .map(|d| (d.id, d.revision))
            .collect();
        if current != self.macro_menu.as_ref().expect("active menu").rows {
            self.macro_menu_refresh();
            return Some(self.message("Macro ring changed; menu refreshed; retry the action"));
        }
        Some(self.macro_menu_dispatch(command))
    }

    fn macro_menu_scroll(&mut self, command: Command) -> Option<Action> {
        let action = match command {
            Command::Scroll(Scroll::NextLine) => self.macro_menu_move(1, false),
            Command::Scroll(Scroll::PreviousLine) => self.macro_menu_move(1, true),
            Command::Scroll(Scroll::PageForward) => {
                self.macro_menu_move(self.pane_size().height as usize, false)
            }
            Command::Scroll(Scroll::PageBackward) => {
                self.macro_menu_move(self.pane_size().height as usize, true)
            }
            Command::Motion(Motion::BufferStart) => self.macro_menu_move(usize::MAX, true),
            Command::Motion(Motion::BufferEnd) => self.macro_menu_move(usize::MAX, false),
            _ => return None,
        };
        Some(action)
    }

    fn macro_menu_dispatch(&mut self, command: Command) -> Action {
        if let Some(action) = self.macro_menu_scroll(command) {
            return action;
        }
        match command {
            Command::Macro(c) => self.macro_menu_action(c),
            Command::Motion(Motion::ForwardChar | Motion::BackwardChar) => {
                let menu = self.macro_menu.as_mut().expect("active menu");
                menu.column = (menu.column
                    + if command == Command::Motion(Motion::ForwardChar) {
                        1
                    } else {
                        5
                    })
                    % 6;
                self.message("Macro menu column changed; RET edits that property")
            }
            Command::Region(RegionCommand::Set) => {
                let menu = self.macro_menu.as_mut().expect("active menu");
                menu.anchor = menu.rows.get(menu.cursor).map(|r| r.0);
                self.message("Macro menu region anchor set")
            }
            _ => self.message("Read-only macro menu; ? shows local commands; q returns to source"),
        }
    }
    fn macro_menu_move(&mut self, count: usize, backward: bool) -> Action {
        let menu = self.macro_menu.as_mut().expect("active menu");
        menu.cursor = if backward {
            menu.cursor.saturating_sub(count)
        } else {
            menu.cursor
                .saturating_add(count)
                .min(menu.rows.len().saturating_sub(1))
        };
        Action::Redraw
    }
    fn macro_menu_targets(&self) -> Vec<(u64, u64)> {
        let menu = self.macro_menu.as_ref().expect("active menu");
        if let Some(anchor) = menu
            .anchor
            .and_then(|id| menu.rows.iter().position(|r| r.0 == id))
        {
            let lo = anchor.min(menu.cursor);
            let hi = anchor.max(menu.cursor);
            return menu.rows.get(lo..=hi).unwrap_or_default().to_vec();
        }
        let marked: Vec<_> = menu
            .rows
            .iter()
            .copied()
            .filter(|r| menu.marked.contains(&r.0))
            .collect();
        if !marked.is_empty() {
            marked
        } else {
            menu.rows.get(menu.cursor).copied().into_iter().collect()
        }
    }
    fn macro_menu_action(&mut self, command: MacroCommand) -> Action {
        match command {
            MacroCommand::MenuQuit=>return self.macro_menu_close(),
            MacroCommand::MenuHelp=>return self.message("Macro menu: # position; C-x C-t transpose; c counter; f format; e definition; TAB/RET column; d flag; x execute; m mark; C copy; D delete; u/DEL/U unmark; g refresh; C-SPC region; q return"),
            MacroCommand::MenuUnmarkAll=>{let menu=self.macro_menu.as_mut().expect("active menu");menu.marked.clear();menu.flagged.clear();menu.anchor=None;return self.message("All macro menu marks and flags cleared");},
            _=>{},
        }
        let Some(target) = self
            .macro_menu
            .as_ref()
            .and_then(|m| m.rows.get(m.cursor))
            .copied()
        else {
            return self.message("Macro ring is empty");
        };
        self.macro_menu_row_action(command, target)
    }

    fn macro_menu_row_action(&mut self, command: MacroCommand, target: (u64, u64)) -> Action {
        if let Some(action) = self.macro_menu_selection_action(command) {
            return action;
        }
        match command {
            MacroCommand::MenuTranspose => self.macro_menu_transpose(target),
            MacroCommand::MenuKeys => self.macro_menu_edit(target),
            MacroCommand::MenuColumn => self.macro_menu_column(target),
            MacroCommand::MenuPosition | MacroCommand::MenuCounter | MacroCommand::MenuFormat => {
                self.macro_menu_prompt(command, target)
            }
            _ => self.message("This macro command is unavailable in the read-only menu"),
        }
    }

    fn macro_menu_selection_action(&mut self, command: MacroCommand) -> Option<Action> {
        let action = match command {
            MacroCommand::MenuFlag | MacroCommand::MenuMark | MacroCommand::MenuUnmark => {
                self.macro_menu_mark(command)
            }
            MacroCommand::MenuUnmarkBackward => self.macro_menu_unmark_backward(),
            MacroCommand::MenuFlaggedDelete => self.macro_menu_delete_flagged(),
            MacroCommand::MenuCopy | MacroCommand::MenuDelete => self.macro_menu_consent(command),
            _ => return None,
        };
        Some(action)
    }

    fn macro_menu_mark(&mut self, command: MacroCommand) -> Action {
        let targets = self.macro_menu_targets();
        let menu = self.macro_menu.as_mut().expect("active menu");
        for (id, _) in targets {
            match command {
                MacroCommand::MenuFlag => {
                    menu.flagged.insert(id);
                }
                MacroCommand::MenuMark => {
                    menu.marked.insert(id);
                }
                _ => {
                    menu.flagged.remove(&id);
                    menu.marked.remove(&id);
                }
            }
        }
        menu.anchor = None;
        self.macro_menu_move(1, false)
    }

    fn macro_menu_unmark_backward(&mut self) -> Action {
        self.macro_menu_move(1, true);
        let menu = self.macro_menu.as_mut().expect("active menu");
        if let Some((id, _)) = menu.rows.get(menu.cursor) {
            menu.marked.remove(id);
            menu.flagged.remove(id);
        }
        menu.anchor = None;
        Action::Redraw
    }

    fn macro_menu_delete_flagged(&mut self) -> Action {
        let menu = self.macro_menu.as_ref().expect("active menu");
        let targets = menu
            .rows
            .iter()
            .copied()
            .filter(|r| menu.flagged.contains(&r.0))
            .collect();
        self.macro_menu_apply(&Consent {
            targets,
            copy: false,
        })
    }

    fn macro_menu_consent(&mut self, command: MacroCommand) -> Action {
        let targets = self.macro_menu_targets();
        let copy = command == MacroCommand::MenuCopy;
        let consent = Consent { targets, copy };
        let limit = self.buffers.snapshot().settings().macros.ring_limit;
        if !copy
            || consent.targets.len() > 1
            || self.macros.ring.len() + consent.targets.len() > limit
        {
            if let Err(e) = self.input.decision(consent.question()) {
                return self.error(&e.to_string());
            }
            self.pending = Some(Pending::MacroMenuConsent(Box::new(consent)));
            self.focus_message("Named and saved snapshots remain independent")
        } else {
            self.macro_menu_apply(&consent)
        }
    }

    fn macro_menu_transpose(&mut self, target: (u64, u64)) -> Action {
        let index = self
            .macros
            .ring
            .iter()
            .position(|d| d.id == target.0)
            .expect("current row");
        if index > 0 {
            self.macros.ring.swap(index, index - 1);
        }
        self.macro_menu_refresh();
        self.message("Macro row transposed")
    }

    fn macro_menu_edit(&mut self, target: (u64, u64)) -> Action {
        let source = Arc::clone(
            self.macros
                .ring
                .iter()
                .find(|d| d.id == target.0)
                .expect("current row"),
        );
        self.macro_open_definition(Selection::Ring(source), false)
    }

    fn macro_menu_column(&mut self, target: (u64, u64)) -> Action {
        match self.macro_menu.as_ref().expect("active menu").column {
            0 => self.macro_menu_action(MacroCommand::MenuPosition),
            1 => {
                let source = Arc::clone(
                    self.macros
                        .ring
                        .iter()
                        .find(|d| d.id == target.0)
                        .expect("current row"),
                );
                self.macro_name_source(source)
            }
            2 | 4 => self.macro_menu_action(MacroCommand::MenuCounter),
            3 => self.macro_menu_action(MacroCommand::MenuFormat),
            _ => self.macro_menu_action(MacroCommand::MenuKeys),
        }
    }

    fn macro_menu_prompt(&mut self, command: MacroCommand, target: (u64, u64)) -> Action {
        let source = self
            .macros
            .ring
            .iter()
            .find(|d| d.id == target.0)
            .expect("current row");
        let (label, initial) = match command {
            MacroCommand::MenuPosition => (
                "Ring position (zero based): ",
                self.macro_menu
                    .as_ref()
                    .expect("active menu")
                    .cursor
                    .to_string(),
            ),
            MacroCommand::MenuCounter => ("Macro counter: ", source.counter.current.to_string()),
            _ => ("Macro counter format: ", source.initial_format.clone()),
        };
        if let Err(e) = self.input.argument(label, &initial) {
            return self.error(&e.to_string());
        }
        self.pending = Some(Pending::MacroMenuPrompt(Box::new(Prompt {
            target,
            property: command,
        })));
        self.message("")
    }

    pub(in crate::session) fn macro_menu_argument(
        &mut self,
        prompt: &Prompt,
        value: &str,
    ) -> Action {
        if !self.macro_menu_active() {
            return self.message("Macro menu owner changed");
        }
        let result = self.macro_update_menu_property(prompt, value);
        match result {
            Ok(()) => {
                self.macro_menu_refresh();
                self.message("Macro ring property updated; named and saved snapshots are unchanged")
            }
            Err(e) => self.error(&e),
        }
    }
    fn macro_update_menu_property(&mut self, prompt: &Prompt, value: &str) -> Result<(), String> {
        let source = self
            .macros
            .ring
            .iter()
            .find(|d| (d.id, d.revision) == prompt.target)
            .cloned()
            .ok_or("Macro menu target changed")?;
        if prompt.property == MacroCommand::MenuPosition {
            return self.macro_menu_position(source, value);
        }
        let mut candidate = (*source).clone();
        if prompt.property == MacroCommand::MenuCounter {
            candidate.counter.current = value
                .parse::<i64>()
                .map_err(|_| "Macro counter requires a signed 64-bit integer")?;
        } else {
            let format = if value.is_empty() { "%d" } else { value };
            crate::macros::counter::render(format, i64::MIN).map_err(str::to_owned)?;
            candidate.counter.format = format.into();
            candidate.initial_format = format.into();
        }
        let limit = self.buffers.snapshot().settings().macros.ring_limit;
        self.macros
            .replace_definition(&Selection::Ring(source), candidate, None, limit)?;
        Ok(())
    }

    fn macro_menu_position(&mut self, source: Arc<Definition>, value: &str) -> Result<(), String> {
        let position = value
            .parse::<usize>()
            .map_err(|_| "Ring position requires a nonnegative integer")?;
        if position >= self.macros.ring.len() {
            return Err("Ring position is outside the current ring".into());
        }
        let current = self
            .macros
            .ring
            .iter()
            .position(|d| d.id == source.id)
            .ok_or("Macro row was removed")?;
        self.macros.ring.remove(current);
        self.macros.ring.insert(position, source);
        Ok(())
    }

    pub(in crate::session) fn macro_menu_answer(
        &mut self,
        consent: &Consent,
        accepted: bool,
    ) -> Action {
        if !accepted {
            return self.message("Macro ring change canceled");
        }
        if !self.macro_menu_active() {
            return self.message("Macro menu owner changed");
        }
        self.macro_menu_apply(consent)
    }
    fn macro_menu_apply(&mut self, consent: &Consent) -> Action {
        if consent.targets.is_empty() {
            return self.message("No macro rows selected or flagged");
        }
        let result =
            (|| -> Result<(), String> {
                let sources: Vec<Arc<Definition>> = consent
                    .targets
                    .iter()
                    .map(|target| {
                        self.macros
                            .ring
                            .iter()
                            .find(|d| (d.id, d.revision) == *target)
                            .cloned()
                            .ok_or("Macro menu target changed")
                    })
                    .collect::<Result<_, _>>()?;
                if consent.copy {
                    let mut copies = Vec::new();
                    for source in sources {
                        copies.push(self.macros.snapshot(&source).map_err(
                            |_| "Macro copy exceeds retained memory or identity bounds",
                        )?);
                    }
                    self.macros
                        .admit_definitions(&copies)
                        .map_err(|_| "Macro copies exceed retained memory")?;
                    for copy in copies.into_iter().rev() {
                        self.macros.ring.push_front(copy);
                    }
                    let limit = self.buffers.snapshot().settings().macros.ring_limit;
                    self.macros.ring.truncate(limit);
                } else {
                    let ids: BTreeSet<_> = sources.iter().map(|d| d.id).collect();
                    self.macros.ring.retain(|d| !ids.contains(&d.id));
                }
                Ok(())
            })();
        match result {
            Ok(()) => {
                self.macro_menu_refresh();
                self.message(if consent.copy {
                    "Selected ring macros duplicated"
                } else {
                    "Selected ring macros removed; named and saved copies remain"
                })
            }
            Err(e) => self.error(&e),
        }
    }
    fn macro_menu_close(&mut self) -> Action {
        let Some(menu) = self.macro_menu.take() else {
            return self.message("No macro menu");
        };
        self.input.cancel();
        self.pending = None;
        if self.view().point.stamp == menu.origin.stamp {
            self.windows.selected_mut().point = menu.origin;
        }
        self.sync_input_context();
        self.message("Returned from macro menu; unexecuted flags discarded")
    }
    pub(in crate::session) fn macro_menu_frame(
        &self,
        frame: &mut crate::display::Frame,
        composition: Option<&crate::panel::Composition>,
    ) -> Result<(), crate::view::Error> {
        let Some(menu) = &self.macro_menu else {
            return Ok(());
        };
        if self.macro_definition_selected() {
            return Ok(());
        };
        let Some((_, panel)) =
            composition.and_then(|c| c.panels().iter().find(|(id, _)| *id == menu.view))
        else {
            return Ok(());
        };
        let extent = panel.content().height().saturating_sub(1).max(1) as usize;
        let top = menu.cursor.saturating_sub(extent - 1);
        let range = menu
            .anchor
            .and_then(|id| menu.rows.iter().position(|r| r.0 == id))
            .map(|a| a.min(menu.cursor)..=a.max(menu.cursor));
        let mut lines = vec![format!(
            "*Keyboard Macros* [read-only] {} rows; column {}",
            menu.rows.len(),
            [
                "position",
                "name",
                "counter",
                "format",
                "value",
                "definition"
            ][menu.column]
        )];
        for (index, (id, revision)) in menu.rows.iter().enumerate().skip(top).take(extent) {
            let marker = format!(
                "{}{}{}{}",
                if index == menu.cursor { ">" } else { " " },
                if menu.marked.contains(id) { "M" } else { " " },
                if menu.flagged.contains(id) { "D" } else { " " },
                if range.as_ref().is_some_and(|r| r.contains(&index)) {
                    "R"
                } else {
                    " "
                }
            );
            let text = self.macros.ring.iter().find(|d| d.id == *id).map_or_else(
                || "[removed; refresh]".into(),
                |d| {
                    let name = self
                        .macros
                        .names
                        .iter()
                        .find_map(|(n, v)| (v.source == (*id, *revision)).then_some(n.name()))
                        .unwrap_or("");
                    let rendered =
                        crate::macros::counter::render(&d.initial_format, d.counter.current)
                            .unwrap_or_else(str::to_owned);
                    let body = d
                        .steps
                        .iter()
                        .take(3)
                        .map(crate::macros::Step::summary)
                        .collect::<Vec<_>>()
                        .join(" ");
                    format!(
                        "{index} {name} {} {:?} {:?} {}",
                        d.counter.current,
                        d.initial_format.chars().take(24).collect::<String>(),
                        rendered.chars().take(24).collect::<String>(),
                        body.chars().take(90).collect::<String>()
                    )
                },
            );
            lines.push(format!("{marker} {text}"));
        }
        frame.macro_menu(&lines, *panel)
    }
}

pub(super) fn local_command(command: MacroCommand) -> bool {
    property_command(command) || selection_command(command) || view_command(command)
}

fn property_command(command: MacroCommand) -> bool {
    matches!(
        command,
        MacroCommand::MenuPosition
            | MacroCommand::MenuTranspose
            | MacroCommand::MenuCounter
            | MacroCommand::MenuFormat
            | MacroCommand::MenuKeys
            | MacroCommand::MenuColumn
    )
}

fn selection_command(command: MacroCommand) -> bool {
    matches!(
        command,
        MacroCommand::MenuFlag
            | MacroCommand::MenuFlaggedDelete
            | MacroCommand::MenuMark
            | MacroCommand::MenuCopy
            | MacroCommand::MenuDelete
            | MacroCommand::MenuUnmark
            | MacroCommand::MenuUnmarkBackward
            | MacroCommand::MenuUnmarkAll
    )
}

fn view_command(command: MacroCommand) -> bool {
    matches!(
        command,
        MacroCommand::MenuRefresh | MacroCommand::MenuQuit | MacroCommand::MenuHelp
    )
}
