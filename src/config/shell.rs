use serde::Deserialize;

/// Startup-only shell selection and finite one-shot resource limits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shell {
    pub program: String,
    pub args: Vec<String>,
    pub timeout_seconds: usize,
    pub max_input_bytes: usize,
    pub max_output_bytes: usize,
}

impl Default for Shell {
    fn default() -> Self {
        Self {
            program: String::new(),
            args: vec!["-c".into()],
            timeout_seconds: 600,
            max_input_bytes: 8_388_608,
            max_output_bytes: 8_388_608,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialShell {
    pub program: Option<String>,
    pub args: Option<Vec<String>>,
    pub timeout_seconds: Option<i64>,
    pub max_input_bytes: Option<i64>,
    pub max_output_bytes: Option<i64>,
}

impl PartialShell {
    pub(super) fn project(&self) -> Shell {
        let mut shell = Shell::default();
        if let Some(program) = &self.program {
            shell.program.clone_from(program);
        }
        if let Some(args) = &self.args {
            shell.args.clone_from(args);
        }
        for (target, source) in [
            (&mut shell.timeout_seconds, self.timeout_seconds),
            (&mut shell.max_input_bytes, self.max_input_bytes),
            (&mut shell.max_output_bytes, self.max_output_bytes),
        ] {
            if let Some(value) = source {
                *target = value as usize;
            }
        }
        shell
    }
}

impl super::merge::Merge {
    pub(super) fn shell(&mut self, base: &mut PartialShell, later: &PartialShell) {
        self.leaf(&["shell", "program"], &mut base.program, &later.program);
        self.leaf(&["shell", "args"], &mut base.args, &later.args);
        self.leaf(
            &["shell", "timeout_seconds"],
            &mut base.timeout_seconds,
            &later.timeout_seconds,
        );
        self.leaf(
            &["shell", "max_input_bytes"],
            &mut base.max_input_bytes,
            &later.max_input_bytes,
        );
        self.leaf(
            &["shell", "max_output_bytes"],
            &mut base.max_output_bytes,
            &later.max_output_bytes,
        );
    }
}

pub(super) fn validate(shell: &PartialShell) -> Result<(), super::validate::Failure> {
    use super::validate::{Failure, Target};
    if shell
        .program
        .as_ref()
        .is_some_and(|s| s.len() > 4096 || s.contains('\0'))
    {
        return Err(Failure::at(
            &["shell", "program"],
            "expected one executable string, at most 4096 UTF-8 bytes, without NUL",
            Target::Value,
        ));
    }
    if shell.args.as_ref().is_some_and(|args| {
        !(1..=32).contains(&args.len())
            || args.iter().any(|s| s.len() > 4096 || s.contains('\0'))
            || args.iter().map(String::len).sum::<usize>() > 32768
    }) {
        return Err(Failure::at(
            &["shell", "args"],
            "expected 1..32 strings, at most 4096 bytes each and 32768 total, without NUL",
            Target::Value,
        ));
    }
    for (key, value, maximum) in [
        ("timeout_seconds", shell.timeout_seconds, 600),
        ("max_input_bytes", shell.max_input_bytes, 16_777_216),
        ("max_output_bytes", shell.max_output_bytes, 16_777_216),
    ] {
        if value.is_some_and(|value| !(1..=maximum).contains(&value)) {
            return Err(Failure::at(
                &["shell", key],
                &format!("expected integer 1..={maximum}"),
                Target::Value,
            ));
        }
    }
    Ok(())
}
