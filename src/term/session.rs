use super::{
    Geometry, Launch, Limits, Waker, engine::Engine, process::Process, readiness::Readiness,
};
use alacritty_terminal::{Term, event::EventListener};
use std::{
    collections::VecDeque,
    io::{self, Read, Write},
    process::ExitStatus,
    sync::{Arc, atomic::AtomicBool},
    time::{Duration, Instant},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    Running,
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
    readiness: Option<Readiness>,
    engine: Engine,
    size: Geometry,
    peak: Geometry,
    limits: Limits,
    cells: usize,
    input: VecDeque<u8>,
    state: State,
    eof: bool,
    exit_started: Option<Instant>,
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
            readiness: None,
            engine,
            size,
            peak: size,
            limits,
            cells,
            input: VecDeque::new(),
            state: State::Running,
            eof: false,
            exit_started: None,
            diagnostic: None,
            extra_bytes: 0,
        };
        let process = session.process.as_ref().expect("prepared terminal process");
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
        self.process
            .as_ref()
            .map(|p| p.pid.as_raw_nonzero().get() as u32)
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
        self.input.extend(bytes);
        self.rearm();
        Ok(())
    }
    pub(super) fn service(&mut self, budget: usize) -> (usize, bool) {
        if self.state != State::Running {
            return (0, false);
        }
        match self.pump(budget) {
            Ok(bytes) => {
                self.rearm();
                (bytes, bytes != 0 || self.state != State::Running)
            }
            Err(error) => {
                self.diagnostic = Some(error.to_string());
                self.state = State::Failed(error.to_string());
                self.readiness.take();
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
        self.input.extend(replies);
        self.extra_bytes = self.engine.extra_bytes();
        if self.engine.rejected_controls() {
            self.diagnostic = Some("Oversized child control ignored through its terminator".into());
        }
        self.observe_exit()?;
        Ok(written + read)
    }
    fn write_input(&mut self, budget: usize) -> io::Result<usize> {
        let process = self
            .process
            .as_ref()
            .expect("running terminal owns process");
        let (bytes, _) = self.input.as_slices();
        let bytes = &bytes[..bytes.len().min(budget)];
        if bytes.is_empty() {
            return Ok(0);
        }
        match process.pty.file().write(bytes) {
            Ok(count) => {
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
            let process = self
                .process
                .as_ref()
                .expect("running terminal owns process");
            let read = process.pty.file().read(&mut bytes[..count]);
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
        let process = self
            .process
            .as_ref()
            .expect("running terminal owns process");
        if !process.exited()? {
            return Ok(());
        }
        if self.exit_started.is_none() {
            self.exit_started = Some(Instant::now());
            process.signal(rustix::process::Signal::KILL)?;
        }
        let deadline =
            self.exit_started.expect("exit observation started") + Duration::from_secs(2);
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Terminal exit cleanup timed out",
            ));
        }
        if !self.eof || process.running(deadline)? {
            return Ok(());
        }
        if let Some(status) = self.process.as_mut().expect("owned process").status() {
            self.state = State::Exited(status);
            self.input.clear();
            self.readiness.take();
            self.process.take();
        }
        Ok(())
    }
    fn rearm(&self) {
        if let Some(readiness) = &self.readiness {
            readiness.rearm(!self.input.is_empty(), self.exit_started.is_none());
        }
    }
    /// The caller supplies termination authority. Failure retains the process,
    /// failed status and capacity so the operation cannot claim clean closure.
    pub(super) fn close(&mut self) -> io::Result<()> {
        if let Some(process) = &mut self.process {
            if let Err(error) = process.stop() {
                self.state = State::Failed(error.to_string());
                self.diagnostic = Some(error.to_string());
                return Err(error);
            }
        }
        self.readiness.take();
        self.process.take();
        self.input.clear();
        self.state = State::Closed;
        Ok(())
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        if self.process.is_none() {
            return;
        }
        if let Err(error) = self.close() {
            // Keep ownership off the editor thread if the kernel cannot finish
            // a killed child inside the explicit close deadline.
            eprintln!("Terminal cleanup remains pending: {error}");
            self.readiness.take();
            if let Some(process) = self.process.take() {
                std::thread::spawn(move || {
                    let _ = process.signal(rustix::process::Signal::KILL);
                    drop(process);
                });
            }
        }
    }
}
