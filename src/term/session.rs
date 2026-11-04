use super::{
    Geometry, Launch, Limits, Waker, cleanup::Cleanup, engine::Engine, process::Process,
    readiness::Readiness,
};
use alacritty_terminal::{Term, event::EventListener};
use std::{
    collections::VecDeque,
    fs::File,
    io::{self, Read, Write},
    process::ExitStatus,
    sync::{Arc, atomic::AtomicBool},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    Running,
    Draining,
    Closing,
    Exited(ExitStatus),
    Failed(String),
    Closed,
}
/// Conservative allocated cell reservation and separately tracked queues/extras.
#[derive(Clone, Copy, Debug)]
pub struct Storage {
    pub reserved_cells: usize,
    pub base_cell_bytes: usize,
    pub extra_bytes: usize,
    pub pending_input_bytes: usize,
    pub pending_output_bytes: usize,
}
pub struct Session {
    process: Option<Process>,
    pid: Option<u32>,
    file: Option<File>,
    cleanup: Option<Cleanup>,
    completed: Option<ExitStatus>,
    ready: Arc<AtomicBool>,
    wake: Waker,
    readiness: Option<Readiness>,
    pub(super) engine: Engine,
    size: Geometry,
    peak: Geometry,
    pub(super) limits: Limits,
    cells: usize,
    pub(super) input: VecDeque<u8>,
    pub(super) paste: Option<super::paste::Pending>,
    pub(super) input_serial: u64,
    pub(super) state: State,
    eof: bool,
    diagnostic: Option<String>,
    extra_bytes: usize,
}
impl Session {
    pub(super) fn open(
        launch: Launch,
        size: Geometry,
        limits: Limits,
        ready: Arc<AtomicBool>,
        wake: Waker,
    ) -> io::Result<Self> {
        let cells = size.cells(limits.scrollback_lines)?;
        let engine = Engine::new(size, limits.scrollback_lines);
        let mut session = Self {
            process: Some(Process::launch(&launch, size)?),
            pid: None,
            file: None,
            cleanup: None,
            completed: None,
            ready: ready.clone(),
            wake: wake.clone(),
            readiness: None,
            engine,
            size,
            peak: size,
            limits,
            cells,
            input: VecDeque::new(),
            paste: None,
            input_serial: 0,
            state: State::Running,
            eof: false,
            diagnostic: None,
            extra_bytes: 0,
        };
        let process = session.process.as_ref().expect("prepared terminal process");
        session.pid = Some(process.pid.as_raw_nonzero().get() as u32);
        session.file = Some(process.pty.file().try_clone()?);
        session.readiness = Some(Readiness::new(
            process.pty.file().try_clone()?,
            process.exit.clone(),
            ready,
            wake,
        )?);
        Ok(session)
    }
    pub fn state(&self) -> &State {
        &self.state
    }
    pub fn geometry(&self) -> Geometry {
        self.size
    }
    pub fn process_id(&self) -> Option<u32> {
        self.pid
    }
    pub fn screen(&self) -> &Term<impl EventListener> {
        &self.engine.term
    }
    pub fn title(&self) -> Option<String> {
        self.engine.effects().title.clone()
    }
    pub fn diagnostic(&self) -> Option<&str> {
        self.diagnostic.as_deref()
    }
    pub fn storage(&self) -> Storage {
        Storage {
            reserved_cells: self.cells,
            base_cell_bytes: self.cells * size_of::<alacritty_terminal::term::cell::Cell>(),
            extra_bytes: self.extra_bytes,
            pending_input_bytes: self.input.len(),
            pending_output_bytes: 0,
        }
    }
    pub(super) fn reservation(&self, size: Geometry) -> io::Result<usize> {
        size.cells(self.limits.scrollback_lines)?;
        Geometry {
            columns: self.peak.columns.max(size.columns),
            rows: self.peak.rows.max(size.rows),
        }
        .cells(self.limits.scrollback_lines)
    }
    pub(super) fn resize(&mut self, size: Geometry, cells: usize) -> io::Result<()> {
        if let Some(process) = &self.process {
            process.resize(size)?;
        }
        self.engine.term.resize(size);
        self.size = size;
        self.cells = cells;
        self.peak.columns = self.peak.columns.max(size.columns);
        self.peak.rows = self.peak.rows.max(size.rows);
        Ok(())
    }
    pub(super) fn queue(&mut self, bytes: &[u8]) -> io::Result<()> {
        if self.state != State::Running {
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "Terminal is not running",
            ));
        }
        if bytes.len()
            > self
                .limits
                .max_pending_input_bytes
                .saturating_sub(self.input.len())
        {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "Terminal input queue is full; input was not accepted",
            ));
        }
        self.paste = None;
        self.input.extend(bytes);
        self.rearm();
        Ok(())
    }
    pub(super) fn service(&mut self, budget: usize) -> (usize, bool) {
        let completed = self.poll_cleanup();
        if !matches!(self.state, State::Running | State::Draining) {
            return (0, completed);
        }
        let work = if self.state == State::Draining {
            self.read_output(budget)
        } else {
            self.pump(budget)
        };
        self.extra_bytes = self.engine.extra_bytes();
        if self.engine.rejected_controls() {
            self.diagnostic = Some("Oversized child control ignored through its terminator".into());
        }
        match work {
            Ok(bytes) => {
                self.finish_exit();
                if self.eof && self.state == State::Draining {
                    self.readiness.take();
                }
                self.rearm();
                (
                    bytes,
                    completed || bytes != 0 || self.state != State::Running,
                )
            }
            Err(error) => {
                self.diagnostic = Some(error.to_string());
                self.state = State::Failed(error.to_string());
                if let Err(cleanup) = self.start_cleanup(false) {
                    let message = format!("{error}; cleanup could not start: {cleanup}");
                    self.diagnostic = Some(message.clone());
                    self.state = State::Failed(message);
                }
                self.file.take();
                (0, true)
            }
        }
    }
    fn pump(&mut self, budget: usize) -> io::Result<usize> {
        let written = self.write_input(budget / 2)?;
        let read = self.read_output(budget - written)?;
        if !self.engine.advance(&[]) {
            return Err(io::Error::other(
                "Terminal retained extras or response queue exceeded its bound",
            ));
        }
        let replies = std::mem::take(&mut self.engine.effects().replies);
        if replies.len()
            > self
                .limits
                .max_pending_input_bytes
                .saturating_sub(self.input.len())
        {
            return Err(io::Error::other(
                "Terminal response exceeds pending input capacity",
            ));
        }
        if !replies.is_empty() {
            self.paste = None;
        }
        self.input.extend(replies);
        self.observe_exit()?;
        Ok(written + read)
    }
    fn write_input(&mut self, budget: usize) -> io::Result<usize> {
        let mut file = self.file.as_ref().expect("running terminal owns IO");
        let (bytes, _) = self.input.as_slices();
        let bytes = &bytes[..bytes.len().min(budget)];
        if bytes.is_empty() {
            return Ok(0);
        }
        match file.write(bytes) {
            Ok(count) => {
                if count > 0 {
                    self.paste = None;
                }
                self.input.drain(..count);
                Ok(count)
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                ) =>
            {
                Ok(0)
            }
            Err(error) if error.raw_os_error() == Some(5) => {
                self.paste = None;
                self.input.clear();
                Ok(0)
            }
            Err(error) => Err(error),
        }
    }
    fn read_output(&mut self, budget: usize) -> io::Result<usize> {
        let mut total = 0;
        let mut bytes = [0; 4096];
        while total < budget && !self.eof {
            let count = bytes
                .len()
                .min(budget - total)
                .min(self.limits.max_pending_output_bytes);
            let read = self
                .file
                .as_ref()
                .expect("terminal owns IO")
                .read(&mut bytes[..count]);
            let Some(count) = self.read_result(read)? else {
                break;
            };
            total += count;
            if !self.engine.advance(&bytes[..count]) {
                return Err(io::Error::other("Terminal emulator storage limit reached"));
            }
        }
        Ok(total)
    }
    fn read_result(&mut self, read: io::Result<usize>) -> io::Result<Option<usize>> {
        match read {
            Ok(0) => {
                self.eof = true;
                Ok(None)
            }
            Ok(count) => Ok(Some(count)),
            Err(error) if error.raw_os_error() == Some(5) => {
                self.eof = true;
                Ok(None)
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                ) =>
            {
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }
    fn observe_exit(&mut self) -> io::Result<()> {
        if self.process.as_ref().expect("running process").exited()? {
            self.start_cleanup(true)?;
            self.state = State::Draining;
            self.paste = None;
            self.input.clear();
        }
        Ok(())
    }
    fn start_cleanup(&mut self, exited: bool) -> io::Result<()> {
        if self.process.is_some() {
            let readiness = if exited { None } else { self.readiness.take() };
            self.cleanup = Some(Cleanup::start(
                &mut self.process,
                readiness,
                exited,
                self.ready.clone(),
                self.wake.clone(),
            )?);
        }
        Ok(())
    }
    fn poll_cleanup(&mut self) -> bool {
        let Some(result) = self.cleanup.as_ref().and_then(Cleanup::poll) else {
            return false;
        };
        self.complete_cleanup(result);
        true
    }
    fn complete_cleanup(&mut self, result: io::Result<ExitStatus>) {
        match result {
            Ok(status) => {
                self.pid = None;
                self.completed = Some(status);
                if self.state == State::Closing {
                    self.state = State::Closed;
                }
            }
            Err(error) => {
                self.process = self.cleanup.as_ref().and_then(Cleanup::process);
                self.diagnostic = Some(error.to_string());
                self.state = State::Failed(error.to_string());
                self.file.take();
                self.readiness.take();
            }
        }
        self.cleanup.take();
    }
    pub(super) fn wait_closed(&mut self) -> io::Result<()> {
        if let Some(cleanup) = &self.cleanup {
            let result = cleanup.wait()?;
            self.complete_cleanup(result);
        }
        if let State::Failed(message) = &self.state {
            return Err(io::Error::other(message.clone()));
        }
        Ok(())
    }
    fn finish_exit(&mut self) {
        if self.state == State::Draining
            && self.eof
            && let Some(status) = self.completed
        {
            self.state = State::Exited(status);
            self.pid = None;
            self.file.take();
            self.readiness.take();
        }
    }
    pub(super) fn rearm(&self) {
        if let Some(readiness) = &self.readiness {
            readiness.rearm(
                !self.input.is_empty(),
                self.state == State::Running,
                !self.eof,
            );
        }
    }
    /// Starts owned cleanup without waiting. Capacity and failure state stay
    /// published until service observes successful worker completion.
    pub(super) fn close(&mut self) -> io::Result<()> {
        if let Err(error) = self.start_cleanup(false) {
            self.state = State::Failed(error.to_string());
            self.diagnostic = Some(error.to_string());
            return Err(error);
        }
        self.readiness.take();
        self.file.take();
        self.paste = None;
        self.input.clear();
        self.state = if self.cleanup.is_some() {
            State::Closing
        } else {
            State::Closed
        };
        Ok(())
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        // Shutdown and error unwinding transfer ownership rather than waiting
        // for process discovery, signaling or upstream PTY destruction here.
        self.start_cleanup(false).expect("terminal shutdown worker");
    }
}
