use super::{Action, Session, SessionId};
use crate::{
    org::links::Viewer,
    process::{Cancellation, Outcome, Program},
    term::Waker,
};
use alacritty_terminal::index::Point;
use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    time::Duration,
};

/// One explicit viewer activation per editor. Only the worker owns process IO
/// and waiting; the UI observes a bounded completion and never joins it.
pub(super) struct Activation {
    cancel: Cancellation,
    result: Receiver<Result<Outcome, String>>,
    ready: Arc<AtomicBool>,
}
struct Completion {
    ready: Arc<AtomicBool>,
    wake: Waker,
}
impl Drop for Completion {
    fn drop(&mut self) {
        self.ready.store(true, Ordering::Release);
        (self.wake)();
    }
}
impl Activation {
    fn start(program: Program, wake: Waker) -> io::Result<Self> {
        let cancel = Cancellation::default();
        let worker_cancel = cancel.clone();
        let ready = Arc::new(AtomicBool::new(false));
        let complete = Completion {
            ready: Arc::clone(&ready),
            wake,
        };
        let (send, result) = mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("terminal-link".into())
            .spawn(move || {
                let _complete = complete;
                let outcome = program.run(&worker_cancel);
                let _ = send.send(outcome);
            })?;
        Ok(Self {
            cancel,
            result,
            ready,
        })
    }
    fn ready(&self) -> bool {
        self.ready.load(Ordering::Acquire)
    }
    fn finish(&self) -> Result<Outcome, String> {
        self.result
            .try_recv()
            .expect("completed terminal link worker")
    }
    fn stop(&self) -> io::Result<()> {
        self.cancel.cancel();
        self.result
            .recv_timeout(Duration::from_secs(2))
            .map(|_| ())
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!("Terminal viewer cleanup pending: {e}"),
                )
            })
    }
}
impl Drop for Activation {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

impl Session {
    pub(super) fn terminal_link_at(&mut self, id: SessionId, point: Point) -> Action {
        let uri = self
            .terminals
            .sessions
            .get(id)
            .and_then(|s| s.screen().grid()[point].hyperlink())
            .map(|link| link.uri().to_owned());
        let Some(uri) = uri else {
            return self.message("No terminal hyperlink at this cell");
        };
        match self.activate_terminal_link(id, &uri) {
            Ok(()) => self.message("Terminal link viewer running"),
            Err(error) => self.error(&error),
        }
    }
    pub(super) fn activate_terminal_link(
        &mut self,
        id: SessionId,
        uri: &str,
    ) -> Result<(), String> {
        admit_uri(uri)?;
        if self.terminals.link.is_some() {
            return Err("A terminal link viewer is still owned".into());
        }
        let viewer = Viewer::configured(self.buffers.snapshot())?;
        let directory = self
            .terminals
            .contexts
            .get(&id)
            .ok_or("Terminal creation context unavailable")?
            .clone();
        let wake = Arc::clone(&self.observation.wake);
        let program = viewer.program(uri.into(), directory);
        self.terminals.link = Some(
            Activation::start(program, Arc::new(move || wake.signal()))
                .map_err(|e| format!("Start terminal viewer: {e}"))?,
        );
        Ok(())
    }
    pub(super) fn terminal_link_ready(&self) -> bool {
        self.terminals.link.as_ref().is_some_and(Activation::ready)
    }
    pub(super) fn expire_terminal_link(&mut self) -> Action {
        if !self.terminal_link_ready() {
            return Action::Wait;
        }
        let link = self.terminals.link.take().expect("ready terminal link");
        match link.finish() {
            Ok(outcome) if outcome.status.success() => {
                self.message("Terminal link viewer completed")
            }
            Ok(outcome) => self.error(&format!("Terminal link viewer failed: {}", outcome.status)),
            Err(error) => self.error(&error),
        }
    }
    pub(super) fn stop_terminal_link(&mut self) -> io::Result<()> {
        if let Some(link) = &self.terminals.link {
            link.stop()?;
        }
        self.terminals.link = None;
        Ok(())
    }
}

fn admit_uri(uri: &str) -> Result<(), String> {
    if !valid_uri_bytes(uri) {
        return Err(
            "Terminal link requires 1-4096 URI bytes without controls, whitespace or backslashes"
                .into(),
        );
    }
    let (scheme, body) = uri
        .split_once(':')
        .ok_or("Terminal link requires an admitted URI scheme")?;
    let admitted = match scheme.to_ascii_lowercase().as_str() {
        "http" | "https" => body
            .strip_prefix("//")
            .is_some_and(|s| !s.split(['/', '?', '#']).next().unwrap_or("").is_empty()),
        "mailto" => !body.is_empty(),
        "file" => body.starts_with("///") || body.starts_with("//localhost/"),
        _ => false,
    };
    if admitted {
        Ok(())
    } else {
        Err("Terminal link scheme or authority is not admitted".into())
    }
}

fn valid_uri_bytes(uri: &str) -> bool {
    !uri.is_empty()
        && uri.len() <= crate::term::CONTROL_BYTES
        && !uri
            .chars()
            .any(|c| c.is_control() || c.is_whitespace() || c == '\\')
}
