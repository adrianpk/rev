use serde::Deserialize;

/// Installed Git selection and startup-only limits. Loading these preferences
/// performs no repository discovery and never launches a process.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Git {
    pub program: String,
    pub observation_timeout_seconds: usize,
    pub operation_timeout_seconds: usize,
    pub transport_timeout_seconds: usize,
    pub max_status_paths: usize,
    pub max_path_bytes: usize,
    pub history_page_size: usize,
    pub max_history_pages: usize,
    pub max_refs: usize,
    pub max_remotes: usize,
    pub max_output_bytes: usize,
    pub max_detail_bytes: usize,
    pub max_diagnostic_bytes: usize,
    pub max_process_records: usize,
    pub max_process_record_bytes: usize,
}

impl Default for Git {
    fn default() -> Self {
        Self {
            program: "git".into(),
            observation_timeout_seconds: 10,
            operation_timeout_seconds: 60,
            transport_timeout_seconds: 120,
            max_status_paths: 1024,
            max_path_bytes: 262144,
            history_page_size: 128,
            max_history_pages: 4,
            max_refs: 256,
            max_remotes: 32,
            max_output_bytes: 1048576,
            max_detail_bytes: 524288,
            max_diagnostic_bytes: 65536,
            max_process_records: 8,
            max_process_record_bytes: 2097152,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialGit {
    pub program: Option<String>,
    pub observation_timeout_seconds: Option<i64>,
    pub operation_timeout_seconds: Option<i64>,
    pub transport_timeout_seconds: Option<i64>,
    pub max_status_paths: Option<i64>,
    pub max_path_bytes: Option<i64>,
    pub history_page_size: Option<i64>,
    pub max_history_pages: Option<i64>,
    pub max_refs: Option<i64>,
    pub max_remotes: Option<i64>,
    pub max_output_bytes: Option<i64>,
    pub max_detail_bytes: Option<i64>,
    pub max_diagnostic_bytes: Option<i64>,
    pub max_process_records: Option<i64>,
    pub max_process_record_bytes: Option<i64>,
}

impl PartialGit {
    pub(super) fn project(&self) -> Git {
        let mut git = Git::default();
        if let Some(program) = &self.program {
            git.program.clone_from(program);
        }
        macro_rules! project {
            ($($field:ident),+ $(,)?) => {$(
                if let Some(value) = self.$field {
                    git.$field = value as usize;
                }
            )+};
        }
        project!(
            observation_timeout_seconds,
            operation_timeout_seconds,
            transport_timeout_seconds,
            max_status_paths,
            max_path_bytes,
            history_page_size,
            max_history_pages,
            max_refs,
            max_remotes,
            max_output_bytes,
            max_detail_bytes,
            max_diagnostic_bytes,
            max_process_records,
            max_process_record_bytes,
        );
        git
    }
}

impl super::merge::Merge {
    pub(super) fn git(&mut self, base: &mut PartialGit, later: &PartialGit) {
        macro_rules! leaves {
            ($($field:ident),+ $(,)?) => {$(
                self.leaf(&["git", stringify!($field)], &mut base.$field, &later.$field);
            )+};
        }
        leaves!(
            program,
            observation_timeout_seconds,
            operation_timeout_seconds,
            transport_timeout_seconds,
            max_status_paths,
            max_path_bytes,
            history_page_size,
            max_history_pages,
            max_refs,
            max_remotes,
            max_output_bytes,
            max_detail_bytes,
            max_diagnostic_bytes,
            max_process_records,
            max_process_record_bytes,
        );
    }
}

pub(super) fn validate(git: &PartialGit, complete: bool) -> Result<(), super::validate::Failure> {
    use super::validate::{Failure, Target};
    let fail = |key: &str, message: &str| Failure::at(&["git", key], message, Target::Value);
    if git
        .program
        .as_ref()
        .is_some_and(|s| s.is_empty() || s.len() > 4096 || s.contains('\0'))
    {
        return Err(fail(
            "program",
            "expected a nonempty literal executable, at most 4096 bytes without NUL",
        ));
    }
    for (key, value, minimum, maximum) in [
        (
            "observation_timeout_seconds",
            git.observation_timeout_seconds,
            1,
            60,
        ),
        (
            "operation_timeout_seconds",
            git.operation_timeout_seconds,
            1,
            600,
        ),
        (
            "transport_timeout_seconds",
            git.transport_timeout_seconds,
            1,
            600,
        ),
        ("max_status_paths", git.max_status_paths, 1, 65536),
        ("max_path_bytes", git.max_path_bytes, 4096, 4194304),
        ("history_page_size", git.history_page_size, 1, 1024),
        ("max_history_pages", git.max_history_pages, 1, 16),
        ("max_refs", git.max_refs, 1, 16384),
        ("max_remotes", git.max_remotes, 1, 256),
        ("max_output_bytes", git.max_output_bytes, 4096, 16777216),
        ("max_detail_bytes", git.max_detail_bytes, 4096, 8388608),
        (
            "max_diagnostic_bytes",
            git.max_diagnostic_bytes,
            1024,
            1048576,
        ),
        ("max_process_records", git.max_process_records, 1, 32),
        (
            "max_process_record_bytes",
            git.max_process_record_bytes,
            65536,
            16777216,
        ),
    ] {
        if value.is_some_and(|value| !(minimum..=maximum).contains(&value)) {
            return Err(fail(
                key,
                &format!("expected integer {minimum}..={maximum}"),
            ));
        }
    }
    let effective = git.project();
    if (complete || (git.max_detail_bytes.is_some() && git.max_output_bytes.is_some()))
        && effective.max_detail_bytes > effective.max_output_bytes
    {
        return Err(fail(
            "max_detail_bytes",
            "must not exceed git.max_output_bytes",
        ));
    }
    if (complete || (git.max_diagnostic_bytes.is_some() && git.max_output_bytes.is_some()))
        && effective.max_diagnostic_bytes > effective.max_output_bytes
    {
        return Err(fail(
            "max_diagnostic_bytes",
            "must not exceed git.max_output_bytes",
        ));
    }
    if (complete || (git.max_process_record_bytes.is_some() && git.max_output_bytes.is_some()))
        && effective.max_process_record_bytes < effective.max_output_bytes
    {
        return Err(fail(
            "max_process_record_bytes",
            "must be at least git.max_output_bytes",
        ));
    }
    Ok(())
}
