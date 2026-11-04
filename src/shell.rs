//! One explicitly requested shell invocation. The worker owns only its process
//! and bounded output; native session state stays on the editing thread.
use crate::process::input;
mod target;
pub(crate) use target::{Kind, Target};

use crate::{
    config::Shell,
    process::{Cancellation, Program},
};
use std::{
    path::PathBuf,
    thread::{self, JoinHandle},
};

pub(crate) struct Invocation {
    pub directory: PathBuf,
    pub input: Option<Vec<u8>>,
    pub file: Option<crate::file::Location>,
}

pub(crate) struct Outcome {
    pub status: std::process::ExitStatus,
    pub stdout: String,
    pub stderr: String,
    pub observed: Result<Option<crate::file::Observation>, String>,
}

pub(crate) struct Operation {
    cancel: Cancellation,
    worker: Option<JoinHandle<Result<Outcome, String>>>,
}

impl Operation {
    pub(crate) fn start(
        policy: &Shell,
        startup_shell: &str,
        command: &str,
        invocation: Invocation,
    ) -> Result<Self, String> {
        if command.trim().is_empty() || command.len() > 4096 || command.contains('\0') {
            return Err("Shell command requires 1..=4096 UTF-8 bytes without NUL".into());
        }
        if invocation
            .input
            .as_ref()
            .is_some_and(|bytes| bytes.len() > policy.max_input_bytes)
        {
            return Err("Shell region exceeds max_input_bytes".into());
        }
        let program = if policy.program.is_empty() {
            startup_shell
        } else {
            &policy.program
        };
        let mut argv = Vec::with_capacity(policy.args.len() + 2);
        argv.push(program.to_owned());
        argv.extend(policy.args.iter().cloned());
        argv.push(command.to_owned());
        let program = Program {
            argv,
            directory: invocation.directory,
            seconds: policy.timeout_seconds,
            output_bytes: policy.max_output_bytes,
        };
        let cancel = Cancellation::default();
        let control = cancel.clone();
        let worker = thread::Builder::new()
            .name("shell-command".into())
            .spawn(move || run(program, invocation.input, invocation.file, control))
            .map_err(|e| format!("Start shell worker: {e}"))?;
        Ok(Self {
            cancel,
            worker: Some(worker),
        })
    }
    pub(crate) fn cancel(&self) {
        self.cancel.cancel();
    }
    pub(crate) fn stopping(&self) -> bool {
        self.cancel.is_cancelled()
    }
    pub(crate) fn poll(&mut self) -> Option<Result<Outcome, String>> {
        if !self.worker.as_ref()?.is_finished() {
            return None;
        }
        let worker = self.worker.take().expect("finished shell worker");
        let result = worker
            .join()
            .unwrap_or_else(|_| Err("Shell worker panicked".into()));
        Some(if self.cancel.is_cancelled() {
            Err("Shell command cancelled".into())
        } else {
            result
        })
    }
}

fn run(
    program: Program,
    input: Option<Vec<u8>>,
    file: Option<crate::file::Location>,
    control: Cancellation,
) -> Result<Outcome, String> {
    let staged = input::stage(input.as_deref())?;
    drop(input);
    let collected = match staged {
        Some(file) => program.run_input(Some(file), None, &control),
        None => program.run(&control),
    }?;
    let observed = file.as_ref().map_or(Ok(None), |file| {
        crate::file::observe(file.path()).map_err(|e| format!("Observe shell source: {e}"))
    });
    Ok(Outcome {
        status: collected.status,
        stdout: text(collected.stdout)?,
        stderr: text(collected.stderr)?,
        observed,
    })
}

impl Drop for Operation {
    fn drop(&mut self) {
        self.cancel();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// Native text ingress is strict UTF-8 without binary NUL. No whitespace, BOM,
/// newline or ANSI transformation may change the captured text.
pub(crate) fn text(bytes: Vec<u8>) -> Result<String, String> {
    let text = String::from_utf8(bytes).map_err(|_| "Shell output is not valid UTF-8")?;
    if text.contains('\0') {
        return Err("Shell output contains unsupported NUL text".into());
    }
    Ok(text)
}
