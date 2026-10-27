use crate::command::GutterCommand;
use crate::gutter::{Change, CurrentLine, Mode, Policy, Side};
use crate::input::Focus;

use super::{Action, Session};

impl Session {
    pub(super) fn gutter_command(&mut self, command: GutterCommand) -> Action {
        if self.input.focus() != Focus::Editing || self.tree.focused() {
            return self.message("Gutter commands require an editing view");
        }
        if command == GutterCommand::Inspect {
            return self.inspect_gutter();
        }
        apply(&mut self.windows.selected_mut().gutter, command);
        self.message("")
    }

    fn inspect_gutter(&mut self) -> Action {
        let policy = self.view().gutter;
        let effective = policy.effective();
        let overrides = policy.overrides();
        let format = effective.format;
        self.message(&format!(
            "Gutter: visible={}({}) mode={:?}({}) current_line={:?}({}) side={:?}({}) digits={} padding={}/{}(startup)",
            effective.visible, source(overrides.visible),
            effective.mode, source(overrides.mode),
            effective.current_line, source(overrides.current_line),
            effective.side, source(overrides.side),
            format.minimum_digits(), format.left_padding(), format.right_padding(),
        ))
    }
}

fn source<T>(value: Option<T>) -> &'static str {
    if value.is_some() { "view" } else { "startup" }
}

fn apply(policy: &mut Policy, command: GutterCommand) {
    let settings = policy.effective();
    match command {
        GutterCommand::Toggle => {
            policy.apply(Change::Visible(!settings.visible));
        }
        GutterCommand::CycleMode => {
            policy.apply(Change::Mode(next_mode(settings.mode)));
        }
        GutterCommand::Absolute => show(policy, Mode::Absolute),
        GutterCommand::Relative => show(policy, Mode::Relative),
        GutterCommand::CycleCurrentLine => {
            policy.apply(Change::CurrentLine(next_current(settings.current_line)));
        }
        GutterCommand::ToggleSide => {
            policy.apply(Change::Side(next_side(settings.side)));
        }
        GutterCommand::Reset => {
            policy.apply(Change::Reset);
        }
        GutterCommand::Inspect => {}
    }
}

fn show(policy: &mut Policy, mode: Mode) {
    policy.apply(Change::Mode(mode));
    policy.apply(Change::Visible(true));
}

fn next_mode(mode: Mode) -> Mode {
    match mode {
        Mode::Absolute => Mode::Relative,
        Mode::Relative => Mode::Absolute,
    }
}

fn next_current(current: CurrentLine) -> CurrentLine {
    match current {
        CurrentLine::Absolute => CurrentLine::Zero,
        CurrentLine::Zero => CurrentLine::Hidden,
        CurrentLine::Hidden => CurrentLine::Absolute,
    }
}

fn next_side(side: Side) -> Side {
    match side {
        Side::Left => Side::Right,
        Side::Right => Side::Left,
    }
}
