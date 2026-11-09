use serde::Deserialize;
use std::collections::BTreeMap;

/// Optional AI settings. Construction never starts a process or resolves secrets.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Ai {
    pub pi: Pi,
    pub commands: BTreeMap<String, AiCommand>,
}

/// Trusted Pi executable and finite operation limits, admitted at startup.
#[derive(Clone, Debug, PartialEq)]
pub struct Pi {
    pub command: String,
    pub model: Option<String>,
    pub persist_sessions: bool,
    pub idle_timeout_ms: usize,
    pub request_timeout_ms: usize,
    pub max_input_bytes: usize,
    pub max_result_bytes: usize,
}

impl Default for Pi {
    fn default() -> Self {
        Self {
            command: "pi".into(),
            model: None,
            persist_sessions: true,
            idle_timeout_ms: 900_000,
            request_timeout_ms: 300_000,
            max_input_bytes: 1_048_576,
            max_result_bytes: 262_144,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialAi {
    pub pi: PartialPi,
    pub commands: BTreeMap<String, AiCommand>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialPi {
    pub command: Option<String>,
    pub model: Option<String>,
    pub persist_sessions: Option<bool>,
    pub idle_timeout_ms: Option<i64>,
    pub request_timeout_ms: Option<i64>,
    pub max_input_bytes: Option<i64>,
    pub max_result_bytes: Option<i64>,
}

impl PartialAi {
    pub(super) fn project(&self) -> Ai {
        let mut pi = Pi::default();
        if let Some(command) = &self.pi.command {
            pi.command.clone_from(command);
        }
        pi.model.clone_from(&self.pi.model);
        if let Some(persist) = self.pi.persist_sessions {
            pi.persist_sessions = persist;
        }
        for (target, source) in [
            (&mut pi.idle_timeout_ms, self.pi.idle_timeout_ms),
            (&mut pi.request_timeout_ms, self.pi.request_timeout_ms),
            (&mut pi.max_input_bytes, self.pi.max_input_bytes),
            (&mut pi.max_result_bytes, self.pi.max_result_bytes),
        ] {
            if let Some(value) = source {
                *target = usize::try_from(value).expect("validated configuration count");
            }
        }
        Ai {
            pi,
            commands: self.commands.clone(),
        }
    }
}

impl super::merge::Merge {
    pub(super) fn ai(&mut self, base: &mut PartialAi, later: &PartialAi) {
        for (name, definition) in &later.commands {
            base.commands.insert(name.clone(), definition.clone());
            self.paths
                .push(vec!["ai".into(), "commands".into(), name.clone()]);
        }
        let (base, later) = (&mut base.pi, &later.pi);
        self.leaf(&["ai", "pi", "command"], &mut base.command, &later.command);
        self.leaf(&["ai", "pi", "model"], &mut base.model, &later.model);
        self.leaf(
            &["ai", "pi", "persist_sessions"],
            &mut base.persist_sessions,
            &later.persist_sessions,
        );
        self.leaf(
            &["ai", "pi", "idle_timeout_ms"],
            &mut base.idle_timeout_ms,
            &later.idle_timeout_ms,
        );
        self.leaf(
            &["ai", "pi", "request_timeout_ms"],
            &mut base.request_timeout_ms,
            &later.request_timeout_ms,
        );
        self.leaf(
            &["ai", "pi", "max_input_bytes"],
            &mut base.max_input_bytes,
            &later.max_input_bytes,
        );
        self.leaf(
            &["ai", "pi", "max_result_bytes"],
            &mut base.max_result_bytes,
            &later.max_result_bytes,
        );
    }
}

pub(super) fn validate(ai: &PartialAi) -> Result<(), super::validate::Failure> {
    use super::validate::{Failure, Target, integer};
    if ai.commands.len() > 64 {
        return Err(Failure::at(
            &["ai", "commands"],
            "AI permits at most 64 definitions",
            Target::Value,
        ));
    }
    for (name, definition) in &ai.commands {
        crate::command::AiName::new(name)
            .and_then(|_| definition.validate())
            .map_err(|message| Failure::at(&["ai", "commands", name], &message, Target::Value))?;
    }
    let pi = &ai.pi;
    for (name, value, limit) in [("command", &pi.command, 4096), ("model", &pi.model, 256)] {
        if value
            .as_ref()
            .is_some_and(|s| s.trim().is_empty() || s.len() > limit || s.contains('\0'))
        {
            return Err(Failure::at(
                &["ai", "pi", name],
                "expected a bounded nonempty string without NUL",
                Target::Value,
            ));
        }
    }
    for (name, value, range) in [
        ("idle_timeout_ms", pi.idle_timeout_ms, 0..=3_600_000),
        (
            "request_timeout_ms",
            pi.request_timeout_ms,
            1000..=1_800_000,
        ),
        ("max_input_bytes", pi.max_input_bytes, 1..=4_194_304),
        ("max_result_bytes", pi.max_result_bytes, 1..=1_048_576),
    ] {
        integer(&format!("ai.pi.{name}"), value, range)?;
    }
    Ok(())
}

/// Literal data only; every invocation captures a new native target.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiCommand {
    pub prompt: String,
    pub input: AiInput,
    pub context: AiContext,
    pub output: AiOutput,
    pub model: Option<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiInput {
    Selection,
    Point,
    Buffer,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiContext {
    Input,
    Directory,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiOutput {
    Explanation,
    Insert,
    Replace,
}
impl AiInput {
    pub fn name(self) -> &'static str {
        match self {
            Self::Selection => "selection",
            Self::Point => "point",
            Self::Buffer => "buffer",
        }
    }
}
impl AiContext {
    pub fn name(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Directory => "directory",
        }
    }
}
impl AiOutput {
    pub fn name(self) -> &'static str {
        match self {
            Self::Explanation => "explanation",
            Self::Insert => "insert",
            Self::Replace => "replace",
        }
    }
    pub fn edits(self) -> bool {
        self != Self::Explanation
    }
}
impl AiCommand {
    pub fn validate(&self) -> Result<(), String> {
        if self.prompt.trim().is_empty() || self.prompt.len() > 16384 || self.prompt.contains('\0')
        {
            return Err("AI prompt requires 1..=16384 UTF-8 bytes without NUL".into());
        }
        if self
            .model
            .as_ref()
            .is_some_and(|m| m.trim().is_empty() || m.len() > 256 || m.contains('\0'))
        {
            return Err("AI model requires 1..=256 bytes without NUL".into());
        }
        if matches!(
            (self.input, self.output),
            (AiInput::Point, AiOutput::Replace)
                | (AiInput::Selection | AiInput::Buffer, AiOutput::Insert)
        ) {
            return Err("AI insert requires point; replace requires selection or buffer".into());
        }
        Ok(())
    }
}
