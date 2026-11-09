use serde::Deserialize;

/// Startup-only interactive shell and bounded session preferences.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Terminal {
    pub program: String,
    pub args: Vec<String>,
    pub panel_height: usize,
    pub max_sessions: usize,
    pub scrollback_lines: usize,
    pub max_pending_input_bytes: usize,
    pub max_pending_output_bytes: usize,
}
impl Default for Terminal {
    fn default() -> Self {
        Self {
            program: String::new(),
            args: Vec::new(),
            panel_height: 10,
            max_sessions: 8,
            scrollback_lines: 2000,
            max_pending_input_bytes: 262144,
            max_pending_output_bytes: 262144,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialTerminal {
    pub program: Option<String>,
    pub args: Option<Vec<String>>,
    pub panel_height: Option<i64>,
    pub max_sessions: Option<i64>,
    pub scrollback_lines: Option<i64>,
    pub max_pending_input_bytes: Option<i64>,
    pub max_pending_output_bytes: Option<i64>,
}
impl PartialTerminal {
    pub(super) fn project(&self) -> Terminal {
        let mut terminal = Terminal::default();
        if let Some(program) = &self.program {
            terminal.program.clone_from(program);
        }
        if let Some(args) = &self.args {
            terminal.args.clone_from(args);
        }
        for (target, value) in [
            (&mut terminal.panel_height, self.panel_height),
            (&mut terminal.max_sessions, self.max_sessions),
            (&mut terminal.scrollback_lines, self.scrollback_lines),
            (
                &mut terminal.max_pending_input_bytes,
                self.max_pending_input_bytes,
            ),
            (
                &mut terminal.max_pending_output_bytes,
                self.max_pending_output_bytes,
            ),
        ] {
            if let Some(value) = value {
                *target = usize::try_from(value).expect("validated configuration count");
            }
        }
        terminal
    }
}
impl super::merge::Merge {
    pub(super) fn terminal(&mut self, base: &mut PartialTerminal, later: &PartialTerminal) {
        self.leaf(&["terminal", "program"], &mut base.program, &later.program);
        self.leaf(&["terminal", "args"], &mut base.args, &later.args);
        self.leaf(
            &["terminal", "panel_height"],
            &mut base.panel_height,
            &later.panel_height,
        );
        self.leaf(
            &["terminal", "max_sessions"],
            &mut base.max_sessions,
            &later.max_sessions,
        );
        self.leaf(
            &["terminal", "scrollback_lines"],
            &mut base.scrollback_lines,
            &later.scrollback_lines,
        );
        self.leaf(
            &["terminal", "max_pending_input_bytes"],
            &mut base.max_pending_input_bytes,
            &later.max_pending_input_bytes,
        );
        self.leaf(
            &["terminal", "max_pending_output_bytes"],
            &mut base.max_pending_output_bytes,
            &later.max_pending_output_bytes,
        );
    }
}
pub(super) fn validate(terminal: &PartialTerminal) -> Result<(), super::validate::Failure> {
    use super::validate::{Failure, Target};
    if terminal
        .program
        .as_ref()
        .is_some_and(|p| p.len() > 4096 || p.contains('\0'))
    {
        return Err(Failure::at(
            &["terminal", "program"],
            "expected one executable string, at most 4096 bytes without NUL",
            Target::Value,
        ));
    }
    if terminal.args.as_ref().is_some_and(|args| {
        args.len() > 32
            || args.iter().any(|s| s.contains('\0'))
            || args.iter().map(String::len).sum::<usize>() > 16384
    }) {
        return Err(Failure::at(
            &["terminal", "args"],
            "expected at most 32 literal strings and 16384 bytes total, without NUL",
            Target::Value,
        ));
    }
    for (key, value, min, max) in [
        ("panel_height", terminal.panel_height, 3, 200),
        ("max_sessions", terminal.max_sessions, 1, 32),
        ("scrollback_lines", terminal.scrollback_lines, 0, 20000),
        (
            "max_pending_input_bytes",
            terminal.max_pending_input_bytes,
            4096,
            1048576,
        ),
        (
            "max_pending_output_bytes",
            terminal.max_pending_output_bytes,
            4096,
            1048576,
        ),
    ] {
        if value.is_some_and(|value| !(min..=max).contains(&value)) {
            return Err(Failure::at(
                &["terminal", key],
                &format!("expected integer {min}..={max}"),
                Target::Value,
            ));
        }
    }
    Ok(())
}
