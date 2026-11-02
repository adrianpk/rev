use serde::Deserialize;

/// Optional AI settings. Construction never starts a process or resolves secrets.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Ai {
    pub pi: Pi,
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
                *target = value as usize;
            }
        }
        Ai { pi }
    }
}

impl super::merge::Merge {
    pub(super) fn ai(&mut self, base: &mut PartialAi, later: &PartialAi) {
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
