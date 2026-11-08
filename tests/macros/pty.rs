use super::*;
use rustix::{
    event::{PollFd, PollFlags, Timespec, poll},
    fd::OwnedFd,
    process::{Pid, PidfdFlags, Signal, pidfd_open, pidfd_send_signal},
    pty::{OpenptFlags, grantpt, ioctl_tiocgptpeer, openpt, unlockpt},
    termios::{Termios, Winsize, tcgetattr, tcsetwinsize},
};
use std::io::Write;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command as Process, Stdio};

// Share the established source-frame projection and Linux CPU/RSS observations;
// PTY allocation, ownership and waiting use the already linked native primitives.
#[allow(dead_code)]
#[path = "../../examples/completion_acceptance/support.rs"]
mod support;
#[allow(dead_code)]
#[path = "../../examples/completion_acceptance/terminal.rs"]
mod terminal;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const LIMIT: usize = 1024 * 1024;
struct Pty {
    child: Child,
    exit: OwnedFd,
    master: OwnedFd,
    slave: OwnedFd,
    attributes: Termios,
    output: Vec<u8>,
    bytes: usize,
    deadline: Instant,
    finished: bool,
}
impl Pty {
    fn start(root: &Path) -> Result<Self> {
        let flags = OpenptFlags::RDWR | OpenptFlags::NOCTTY | OpenptFlags::CLOEXEC;
        let master = openpt(flags)?;
        let mode = rustix::fs::fcntl_getfl(&master)?;
        rustix::fs::fcntl_setfl(&master, mode | rustix::fs::OFlags::NONBLOCK)?;
        grantpt(&master)?;
        unlockpt(&master)?;
        let slave = ioctl_tiocgptpeer(&master, flags)?;
        tcsetwinsize(
            &slave,
            Winsize {
                ws_row: 24,
                ws_col: 100,
                ws_xpixel: 0,
                ws_ypixel: 0,
            },
        )?;
        let attributes = tcgetattr(&slave)?;
        let mut child = Process::new(std::env::current_exe()?)
            .args(["--exact", "pty::terminal_journeys", "--nocapture"])
            .env("REV_MACRO_PTY_CHILD", "1")
            .env("HOME", root.join("pty/home"))
            .env("XDG_CONFIG_HOME", root.join("pty/config"))
            .env("XDG_DATA_HOME", root.join("pty/data"))
            .env("XDG_STATE_HOME", root.join("pty/state"))
            .env("XDG_CACHE_HOME", root.join("pty/cache"))
            .env("TERM", "xterm-256color")
            .env("NO_COLOR", "1")
            .current_dir(root.join("workspace"))
            .stdin(Stdio::from(slave.try_clone()?))
            .stdout(Stdio::from(slave.try_clone()?))
            .stderr(Stdio::from(slave.try_clone()?))
            .spawn()?;
        let exit = match pidfd_open(Pid::from_child(&child), PidfdFlags::NONBLOCK) {
            Ok(exit) => exit,
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error.into());
            }
        };
        let mut pty = Self {
            child,
            exit,
            master,
            slave,
            attributes,
            output: Vec::new(),
            bytes: 0,
            deadline: Instant::now() + Duration::from_secs(8),
            finished: false,
        };
        pty.frame(|f| f.contains("notes.txt"))?;
        Ok(pty)
    }
    fn send(&mut self, bytes: &[u8]) -> Result<()> {
        self.output.clear();
        let mut input = std::fs::File::from(self.master.try_clone()?);
        input.write_all(bytes)?;
        input.flush()?;
        Ok(())
    }
    fn read(&mut self, wait: Duration) -> Result<bool> {
        let wait = wait.min(self.deadline.saturating_duration_since(Instant::now()));
        if wait.is_zero() {
            return Err("macro PTY exceeded eight seconds".into());
        }
        let mut fds = [PollFd::new(&self.master, PollFlags::IN)];
        let timeout = Timespec::try_from(wait)?;
        match poll(&mut fds, Some(&timeout)) {
            Ok(0) => return Ok(false),
            Err(rustix::io::Errno::INTR) => return Ok(true),
            Err(e) => return Err(e.into()),
            Ok(_) => {}
        }
        let mut chunk = [0; 8192];
        match rustix::io::read(&self.master, &mut chunk) {
            Ok(0) => return Err("owned PTY closed before completion".into()),
            Ok(n) => {
                self.bytes += n;
                if self.bytes > LIMIT {
                    return Err("macro PTY exceeded 1 MiB".into());
                }
                self.output.extend_from_slice(&chunk[..n]);
            }
            Err(rustix::io::Errno::AGAIN | rustix::io::Errno::INTR) => {}
            Err(e) => return Err(e.into()),
        }
        Ok(true)
    }
    fn frame(&mut self, predicate: impl Fn(&str) -> bool) -> Result<()> {
        loop {
            let text = String::from_utf8_lossy(&self.output);
            if let Some((_, tail)) = text.rsplit_once("\x1b[?25l")
                && let Some((completed, _)) = tail.rsplit_once("\x1b[?25h")
            {
                let visible = (1..=24)
                    .map(|r| terminal::row(completed, r))
                    .collect::<Vec<_>>()
                    .join("\n");
                if predicate(&visible) {
                    return Ok(());
                }
            }
            let wait = self.deadline.saturating_duration_since(Instant::now());
            if wait.is_zero() {
                let tail = text
                    .rsplit_once("\x1b[?25l")
                    .map_or(text.as_ref(), |(_, tail)| tail);
                let visible = (1..=24)
                    .map(|row| terminal::row(tail, row))
                    .collect::<Vec<_>>()
                    .join("\n");
                return Err(format!("macro PTY exceeded eight seconds; visible={visible}").into());
            }
            self.read(wait)?;
        }
    }
    fn step(&mut self, input: &[u8], expected: &str) -> Result<()> {
        self.send(input)?;
        self.frame(|f| f.contains(expected))
    }
    fn save(&mut self, root: &Path, expected: &str) -> Result<()> {
        self.send(b"\x18\x13")?;
        self.frame(|_| {
            std::fs::read_to_string(root.join("workspace/notes.txt")).is_ok_and(|s| s == expected)
        })?;
        Ok(())
    }
    fn idle(&mut self) -> Result<()> {
        // Drain queued redraws through a finite quiet barrier, then observe a
        // fresh quiet interval. Every byte still consumes the original cap.
        let settle = self.deadline.min(Instant::now() + Duration::from_secs(2));
        while self.read(Duration::from_millis(350))? {
            if Instant::now() >= settle {
                return Err("macro PTY did not become idle".into());
            }
        }
        let pid = self.child.id().to_string();
        let before = support::cpu_ticks(&pid)?;
        if self.deadline.saturating_duration_since(Instant::now()) < Duration::from_millis(350) {
            return Err("macro PTY has no remaining idle interval".into());
        }
        assert!(
            !self.read(Duration::from_millis(350))?,
            "idle macro owner emitted output"
        );
        let ticks = support::cpu_ticks(&pid)?.saturating_sub(before);
        assert!(ticks <= 1, "idle macro owner consumed {ticks} CPU ticks");
        println!(
            "MEASURE macro PTY idle_ms=350 cpu_ticks={ticks} peak_rss={}",
            support::peak_rss(&pid)?
        );
        Ok(())
    }
    fn finish(mut self) -> Result<()> {
        self.send(b"\x18\x03")?;
        let mut fds = [PollFd::new(&self.exit, PollFlags::IN)];
        let wait = self.deadline.saturating_duration_since(Instant::now());
        assert!(
            poll(&mut fds, Some(&Timespec::try_from(wait)?))? > 0,
            "macro PTY shutdown deadline"
        );
        assert!(self.child.wait()?.success());
        self.finished = true;
        // Drain the finite exit stream to verify native terminal restoration.
        let mut chunk = [0; 8192];
        loop {
            match rustix::io::read(&self.master, &mut chunk) {
                Ok(0) | Err(rustix::io::Errno::AGAIN) => break,
                Ok(n) => {
                    self.bytes += n;
                    assert!(self.bytes <= LIMIT);
                    self.output.extend_from_slice(&chunk[..n]);
                }
                Err(e) => return Err(e.into()),
            }
        }
        let output = String::from_utf8_lossy(&self.output);
        assert!(
            output.contains("\x1b[?1049l")
                && output.contains("\x1b[?2004l")
                && output.contains("\x1b[?25h")
        );
        let restored = tcgetattr(&self.slave)?;
        assert_eq!(restored.input_modes, self.attributes.input_modes);
        assert_eq!(restored.output_modes, self.attributes.output_modes);
        assert_eq!(restored.control_modes, self.attributes.control_modes);
        assert_eq!(restored.local_modes, self.attributes.local_modes);
        assert_eq!(restored.line_discipline, self.attributes.line_discipline);
        use rustix::termios::SpecialCodeIndex as C;
        for code in [
            C::VINTR,
            C::VQUIT,
            C::VERASE,
            C::VKILL,
            C::VEOF,
            C::VTIME,
            C::VMIN,
            C::VSWTC,
            C::VSTART,
            C::VSTOP,
            C::VSUSP,
            C::VEOL,
            C::VREPRINT,
            C::VDISCARD,
            C::VWERASE,
            C::VLNEXT,
            C::VEOL2,
        ] {
            assert_eq!(
                restored.special_codes[code],
                self.attributes.special_codes[code]
            );
        }
        assert_eq!(restored.input_speed(), self.attributes.input_speed());
        assert_eq!(restored.output_speed(), self.attributes.output_speed());
        println!(
            "MEASURE macro PTY bytes={} restored=true reaped=true",
            self.bytes
        );
        Ok(())
    }
}
impl Drop for Pty {
    fn drop(&mut self) {
        if !self.finished {
            let _ = pidfd_send_signal(&self.exit, Signal::KILL);
            let _ = self.child.wait();
        }
    }
}
fn fixture(profile: &str) -> fixture::Fixture {
    let f = fixture::Fixture::new();
    for path in [
        "pty/home",
        "pty/config/rev",
        "pty/data",
        "pty/state",
        "pty/cache",
        "workspace",
    ] {
        std::fs::create_dir_all(f.0.join(path)).unwrap();
    }
    f.write("pty/config/rev/config.toml",&format!("editor.interaction_profile='{profile}'\ntree.visible=false\ncompletion.automatic=false\n"));
    f.write("workspace/notes.txt", "");
    f
}
fn child() -> ! {
    rustix::process::setsid().unwrap();
    rustix::process::ioctl_tiocsctty(std::io::stdin()).unwrap();
    let error = Process::new(env!("CARGO_BIN_EXE_rev"))
        .arg("notes.txt")
        .exec();
    panic!("cannot exec native editor: {error}")
}

// Real function-key/C-x bytes traverse native input in both profiles. File
// assertions establish exactly-once effects; a fresh executable restores only
// the explicit saved snapshot. Query/cancel waits prove idle release on the PTY.
#[test]
fn terminal_journeys() -> Result<()> {
    if std::env::var_os("REV_MACRO_PTY_CHILD").is_some() {
        child();
    }
    let f = fixture("non-modal");
    let mut t = Pty::start(&f.0)?;
    t.step(b"\x1bOR", "Def")?;
    t.step(b"q", "q")?;
    t.step(b"\x1bOS", "recorded")?;
    t.step(b"\x1b3\x1bOS", "qqqq")?;
    t.save(&f.0, "qqqq")?;
    t.step(b"\x1f", "qqq")?;
    t.save(&f.0, "qqq")?;
    t.step(b"\x18e", "qqqq")?;
    t.step(b"e", "qqqqq")?;
    t.save(&f.0, "qqqqq")?;
    let library = f.0.join("pty/state/rev/macros/library.toml");
    assert!(!library.exists());
    t.step(b"\x18\x0bnsaved-part\r", "saved-part")?;
    t.step(b"\x1bxkmacro-save\r", "Named macro:")?;
    t.step(b"saved-part\r", "Save named macro")?;
    t.send(b"y")?;
    t.frame(|_| library.is_file())?;
    let disk = std::fs::read(&library)?;
    t.step(b"\x18\x0bnunsaved-part\r", "unsaved-part")?;
    t.idle()?;
    t.finish()?;
    assert_eq!(std::fs::read(&library)?, disk);
    let value: toml::Value = toml::from_str(std::str::from_utf8(&disk)?)?;
    assert_eq!(value["macros"].as_array().unwrap().len(), 1);
    assert_eq!(value["macros"][0]["name"].as_str(), Some("saved-part"));
    let mut restarted = Pty::start(&f.0)?;
    assert_eq!(
        std::fs::read_to_string(f.0.join("workspace/notes.txt"))?,
        "qqqqq"
    );
    restarted.step(b"\x1bxsaved-part\r", "qqqqqq")?;
    restarted.save(&f.0, "qqqqqq")?;
    restarted.idle()?;
    restarted.finish()?;
    assert_eq!(std::fs::read(&library)?, disk);

    let f = fixture("modal");
    let mut t = Pty::start(&f.0)?;
    t.step(b"\x18(", "Def")?;
    t.step(b"iM\x1b", "NORMAL")?;
    t.step(b"\x18)", "recorded")?;
    t.step(b"\x1b2\x18e", "MMM")?;
    t.save(&f.0, "MMM")?;
    t.step(b"u", "MM")?;
    t.save(&f.0, "MM")?;
    t.step(b"\x18(", "Def")?;
    t.step(b"i\x18qQ\x1b", "NORMAL")?;
    t.step(b"\x18)", "recorded")?;
    t.step(b"\x18e", "Macro query:")?;
    t.idle()?;
    t.send(b"\x07")?;
    t.step(b"\x1b", "NORMAL")?;
    t.save(&f.0, "MMQ")?;
    t.idle()?;
    t.step(b"\x18e", "Macro query:")?;
    t.step(b"y", "MMQQ")?;
    t.save(&f.0, "MMQQ")?;
    t.finish()?;
    Ok(())
}
