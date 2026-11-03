use super::{Geometry, Launch};
use alacritty_terminal::tty::{self, ChildEvent, EventedPty};
use rustix::{
    event::{PollFd, PollFlags, Timespec, poll},
    fd::OwnedFd,
    process::{Pid, Signal},
};
use std::{
    collections::BTreeSet,
    fs::File,
    io::{self, Read},
    sync::Arc,
    time::{Duration, Instant},
};

pub(super) struct Process {
    pub pty: tty::Pty,
    pub exit: Arc<OwnedFd>,
    pub pid: Pid,
}
impl Process {
    pub fn launch(launch: &Launch, size: Geometry) -> io::Result<Self> {
        let config = tty::Options {
            shell: Some(tty::Shell::new(launch.program.clone(), launch.args.clone())),
            working_directory: Some(launch.directory.clone()),
            drain_on_exit: true,
            env: [
                ("TERM".into(), "xterm-256color".into()),
                ("COLORTERM".into(), "truecolor".into()),
            ]
            .into(),
        };
        let pty = tty::new(
            &config,
            alacritty_terminal::event::WindowSize {
                num_lines: size.rows,
                num_cols: size.columns,
                cell_width: 0,
                cell_height: 0,
            },
            0,
        )?;
        let pid = Pid::from_child(pty.child());
        let exit = match rustix::process::pidfd_open(pid, rustix::process::PidfdFlags::NONBLOCK) {
            Ok(exit) => exit,
            Err(error) => {
                let _ = rustix::process::kill_process_group(pid, Signal::KILL);
                return Err(error.into());
            }
        };
        Ok(Self {
            pty,
            exit: Arc::new(exit),
            pid,
        })
    }
    pub fn resize(&self, size: Geometry) -> io::Result<()> {
        // Upstream's OnResize exits the entire editor on ioctl failure. The
        // existing rustix boundary returns a checked error before publication.
        rustix::termios::tcsetwinsize(
            self.pty.file(),
            rustix::termios::Winsize {
                ws_row: size.rows,
                ws_col: size.columns,
                ws_xpixel: 0,
                ws_ypixel: 0,
            },
        )
        .map_err(Into::into)
    }
    pub fn exited(&self) -> io::Result<bool> {
        let mut fds = [PollFd::new(self.exit.as_ref(), PollFlags::IN)];
        poll(&mut fds, Some(&Timespec::default()))?;
        Ok(!fds[0].revents().is_empty())
    }
    pub fn status(&mut self) -> Option<std::process::ExitStatus> {
        match self.pty.next_child_event() {
            Some(ChildEvent::Exited(status)) => status,
            None => None,
        }
    }
    pub fn signal(&self, signal: Signal) -> io::Result<()> {
        let deadline = Instant::now() + Duration::from_millis(100);
        for group in groups(self.pid, deadline)? {
            match rustix::process::kill_process_group(
                Pid::from_raw(group).expect("observed process group"),
                signal,
            ) {
                Ok(()) | Err(rustix::io::Errno::SRCH) => {}
                Err(error) => return Err(error.into()),
            }
        }
        Ok(())
    }
    pub fn running(&self, deadline: Instant) -> io::Result<bool> {
        Ok(!groups(self.pid, deadline)?.is_empty())
    }
    pub fn stop(&mut self) -> io::Result<std::process::ExitStatus> {
        let start = Instant::now();
        self.signal(Signal::HUP)?;
        let grace = start + Duration::from_millis(500);
        self.wait_group(grace, start + Duration::from_secs(2))?;
        if self.running(start + Duration::from_secs(2))? {
            self.signal(Signal::KILL)?;
        }
        let deadline = start + Duration::from_secs(2);
        self.wait_group(deadline, deadline)?;
        if self.running(deadline)? {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Terminal process-group cleanup timed out",
            ));
        }
        self.reap(deadline)
    }
    fn reap(&mut self, deadline: Instant) -> io::Result<std::process::ExitStatus> {
        loop {
            if let Some(status) = self.status() {
                return Ok(status);
            }
            if Instant::now() >= deadline {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Terminal child reap timed out",
                ));
            }
            std::thread::sleep(Duration::from_millis(2));
        }
    }
    fn wait_group(&self, deadline: Instant, observation_deadline: Instant) -> io::Result<()> {
        while Instant::now() < deadline && self.running(observation_deadline)? {
            std::thread::sleep(Duration::from_millis(2));
        }
        Ok(())
    }
}

/// The PTY library creates a new Unix session. Shell job control can create
/// more than one process group in it; ownership follows that session, including
/// background groups that no longer hold an output descriptor. The leader is
/// not reaped until these groups stop, preventing session-ID reuse while scanning.
fn groups(session: Pid, deadline: Instant) -> io::Result<BTreeSet<i32>> {
    let mut groups = BTreeSet::new();
    let mut bytes = [0; 4097];
    for (index, entry) in std::fs::read_dir("/proc")?.enumerate() {
        if index >= 32768 || Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Terminal process observation exceeded its bound",
            ));
        }
        let entry = entry?;
        if !entry
            .file_name()
            .as_encoded_bytes()
            .iter()
            .all(u8::is_ascii_digit)
        {
            continue;
        }
        let mut path = entry.path();
        path.push("stat");
        if let Some(group) = member(&path, session, &mut bytes)? {
            groups.insert(group.as_raw_nonzero().get());
        }
        if groups.len() > 256 {
            return Err(io::Error::other(
                "Terminal process-group observation exceeds 256 groups",
            ));
        }
    }
    Ok(groups)
}
fn member(path: &std::path::Path, session: Pid, bytes: &mut [u8; 4097]) -> io::Result<Option<Pid>> {
    let size = match File::open(path).and_then(|mut file| read_stat(&mut file, bytes)) {
        Ok(size) => size,
        Err(error)
            if error.kind() == io::ErrorKind::NotFound || error.raw_os_error() == Some(3) =>
        {
            return Ok(None);
        }
        Err(error) => return Err(error),
    };
    if size > 4096 {
        return Err(io::Error::other("Terminal process stat exceeds 4 KiB"));
    }
    member_fields(&bytes[..size], session)
}

fn read_stat(file: &mut impl Read, bytes: &mut [u8; 4097]) -> io::Result<usize> {
    let mut size = 0;
    while size < bytes.len() {
        match file.read(&mut bytes[size..]) {
            Ok(0) => break,
            Ok(count) => size += count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }
    Ok(size)
}
fn member_fields(bytes: &[u8], session: Pid) -> io::Result<Option<Pid>> {
    let end = bytes
        .iter()
        .rposition(|b| *b == b')')
        .ok_or_else(|| io::Error::other("Invalid process stat"))?;
    let tail = std::str::from_utf8(&bytes[end + 1..]).map_err(io::Error::other)?;
    let mut fields = tail.split_whitespace();
    let state = field(&mut fields)?;
    let _parent = field(&mut fields)?;
    let group = field(&mut fields)?;
    let owner = field(&mut fields)?;
    if matches!(state, "Z" | "X" | "x")
        || owner.parse::<i32>().ok() != Some(session.as_raw_nonzero().get())
    {
        return Ok(None);
    }
    Ok(group.parse::<i32>().ok().and_then(Pid::from_raw))
}

fn field<'a>(fields: &mut impl Iterator<Item = &'a str>) -> io::Result<&'a str> {
    fields
        .next()
        .ok_or_else(|| io::Error::other("Incomplete process stat"))
}
