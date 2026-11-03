//! One explicitly requested shell invocation. The worker owns only its process
//! and bounded output; native session state stays on the editing thread.
use crate::{
    config::Shell,
    process::{Cancellation, Outcome, Program},
};
use std::{
    path::PathBuf,
    thread::{self, JoinHandle},
};

pub(crate) struct Operation {
    cancel: Cancellation,
    worker: Option<JoinHandle<Result<Outcome, String>>>,
}

impl Operation {
    pub(crate) fn start(
        policy: &Shell,
        startup_shell: &str,
        directory: PathBuf,
        command: &str,
    ) -> Result<Self, String> {
        if command.trim().is_empty() || command.len() > 4096 || command.contains('\0') {
            return Err("Shell command requires 1..=4096 UTF-8 bytes without NUL".into());
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
            directory,
            seconds: policy.timeout_seconds,
            output_bytes: policy.max_output_bytes,
        };
        let cancel = Cancellation::default();
        let control = cancel.clone();
        let worker = thread::Builder::new()
            .name("shell-command".into())
            .spawn(move || program.run(&control))
            .map_err(|e| format!("Start shell worker: {e}"))?;
        Ok(Self {
            cancel,
            worker: Some(worker),
        })
    }
    pub(crate) fn cancel(&self) {
        self.cancel.cancel();
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
