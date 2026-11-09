use super::*;

/// Exact argv and staged stdin move together into one owned native child.
/// This envelope is consumed on launch, including when no process can start.
pub(super) struct Invocation {
    pub argv: Vec<OsString>,
    pub input: Option<std::fs::File>,
}

impl Runner {
    pub(super) fn command(
        &mut self,
        args: &[&str],
        phase: Phase,
        allow_absent: bool,
    ) -> Result<Vec<u8>, String> {
        self.command_os(
            &args.iter().map(OsString::from).collect::<Vec<_>>(),
            phase,
            allow_absent,
            None,
        )
        .map(|(bytes, _)| bytes)
    }
    pub(super) fn command_os(
        &mut self,
        args: &[OsString],
        phase: Phase,
        allow_absent: bool,
        detail_cap: Option<usize>,
    ) -> Result<(Vec<u8>, bool), String> {
        self.command_argv(
            self.git_invocation(args, None),
            phase,
            allow_absent,
            detail_cap,
        )
    }
    pub(super) fn git_invocation(
        &self,
        args: &[OsString],
        input: Option<std::fs::File>,
    ) -> Invocation {
        Invocation {
            argv: self.git_argv(args),
            input,
        }
    }
    pub(super) fn git_argv(&self, args: &[OsString]) -> Vec<OsString> {
        std::iter::once(OsString::from(&self.policy.program))
            .chain(std::iter::once(OsString::from("--no-optional-locks")))
            .chain(args.iter().cloned())
            .collect()
    }
    pub(super) fn command_argv(
        &mut self,
        invocation: Invocation,
        phase: Phase,
        allow_absent: bool,
        detail_cap: Option<usize>,
    ) -> Result<(Vec<u8>, bool), String> {
        let remaining = self.capture_remaining()?;
        let program = self.capture_program(
            invocation,
            detail_cap.map_or(remaining, |cap| cap.min(remaining)),
        );
        let progress = std::sync::Arc::clone(&self.progress);
        let wake = std::sync::Arc::clone(&self.wake);
        let result = program.run(&self.cancel.bounded(self.deadline), move |pid, cleanup| {
            let mut progress = progress.lock().expect("Git progress owner");
            progress.phase = if cleanup { Phase::Cleanup } else { phase };
            progress.child = Some(pid);
            drop(progress);
            wake();
        });
        self.record_capture(&result, phase);
        self.finish_capture(result, allow_absent, detail_cap)
    }
    fn capture_remaining(&mut self) -> Result<usize, String> {
        if Instant::now() >= self.deadline {
            self.end = capture::End::TimedOut;
            return Err("Git observation timed out".into());
        }
        let capture_limit = self
            .policy
            .max_output_bytes
            .min(self.policy.max_process_record_bytes - 16384);
        let remaining = capture_limit.saturating_sub(self.captured);
        if remaining == 0 {
            self.end = capture::End::OutputLimit;
            return Err("Git aggregate job output limit reached".into());
        }
        Ok(remaining)
    }
    fn capture_program(&mut self, invocation: Invocation, output_bytes: usize) -> capture::Program {
        self.argv = invocation.argv;
        capture::Program {
            argv: self.argv.clone(),
            directory: self.directory.clone(),
            seconds: self.seconds,
            output_bytes,
            input: invocation.input,
            git_index: self.private_index.clone(),
            workspace: self.workspace.clone(),
            editors: if let Some(file) = self.todo_editor.take() {
                capture::Editors::Todo(file)
            } else if self.accept_message {
                capture::Editors::AcceptMessage
            } else {
                capture::Editors::Disabled
            },
        }
    }
    fn record_capture(&mut self, result: &capture::Collected, phase: Phase) {
        {
            let mut progress = self.progress.lock().expect("Git progress owner");
            if result.retained.is_none() {
                progress.phase = phase;
                progress.child = None;
            }
        }
        self.captured += result.stdout.len() + result.stderr.len();
        // Record retention is one aggregate diagnostic prefix; complete raw
        // observation bytes stay only in this worker until parsed/rejected.
        let cap = self
            .policy
            .max_diagnostic_bytes
            .min(self.policy.max_process_record_bytes.saturating_sub(16384));
        let stderr_end = result.stderr.len().min(cap);
        let stdout_end = result.stdout.len().min(cap.saturating_sub(stderr_end));
        self.stdout = result.stdout[..stdout_end].to_vec();
        self.stderr = result.stderr[..stderr_end].to_vec();
        self.truncated = result.stdout.len() > stdout_end
            || result.stderr.len() > stderr_end
            || result.end == capture::End::OutputLimit;
        self.exit = result.status;
        self.end = if result.end == capture::End::Cancelled && !self.cancel.is_cancelled() {
            capture::End::TimedOut
        } else {
            result.end
        };
        if self.private_output {
            self.redact_transport_capture(result);
        }
    }
    fn redact_transport_capture(&mut self, result: &capture::Collected) {
        if result.status.is_some_and(|status| !status.success()) {
            for (needle, diagnosis) in [
                (
                    b"terminal prompts disabled".as_slice(),
                    "Noninteractive Git credential input is unavailable",
                ),
                (
                    b"Authentication failed".as_slice(),
                    "Git authentication failed",
                ),
                (
                    b"Permission denied (publickey".as_slice(),
                    "Git SSH authentication failed",
                ),
            ] {
                if result
                    .stderr
                    .windows(needle.len())
                    .any(|part| part == needle)
                {
                    self.transport_diagnostic = Some(diagnosis);
                    break;
                }
            }
        }
        self.stdout.clear();
        self.stderr.clear();
    }
    fn finish_capture(
        &mut self,
        result: capture::Collected,
        allow_absent: bool,
        detail_cap: Option<usize>,
    ) -> Result<(Vec<u8>, bool), String> {
        self.retained = result.retained;
        if self.retained.is_some() {
            return Err(result
                .diagnostic
                .unwrap_or("Git cleanup remains owned".into()));
        }
        if self.end == capture::End::OutputLimit && detail_cap.is_some() {
            return Ok((result.stdout, true));
        }
        if self.end != capture::End::Exited {
            return Err(result
                .diagnostic
                .unwrap_or("Git observation did not complete".into()));
        }
        if result
            .status
            .is_some_and(|status| status.success() || (allow_absent && status.code() == Some(1)))
        {
            Ok((result.stdout, false))
        } else {
            Err("Git rejected the requested repository observation; inspect its retained diagnostic".into())
        }
    }
}
