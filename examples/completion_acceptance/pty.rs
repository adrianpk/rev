use super::support::{Result, cpu_ticks, peak_rss};
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{Receiver, RecvTimeoutError, sync_channel};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

const HIDE: &str = "\x1b[?25l";
const SHOW: &str = "\x1b[?25h";
const LIMIT: usize = 1024 * 1024;

// Same util-linux controlling-PTY boundary and eight-second/1-MiB ownership
// limits as the existing native journeys. Drop reaps only this fixture.
pub(super) struct Terminal {
    child: Child,
    input: Option<ChildStdin>,
    reader: Option<JoinHandle<()>>,
    received: Option<Receiver<Vec<u8>>>,
    output: String,
    undecoded: Vec<u8>,
    bytes: usize,
    deadline: Instant,
    editor: String,
}

impl Terminal {
    pub(super) fn start(root: &Path, binary: &Path, wrapper: &Path) -> Result<Self> {
        let mut child = Command::new("script")
            .args([
                "-q",
                "-e",
                "-f",
                "-E",
                "never",
                "-o",
                "1048576",
                "/dev/null",
                "--",
                "bash",
            ])
            .arg(wrapper)
            .arg("--pty-child")
            .arg(binary)
            .arg(root.join("workspace/terminal.rs"))
            .env("HOME", root.join("pty/home"))
            .env("XDG_CONFIG_HOME", root.join("pty/config"))
            .env("XDG_DATA_HOME", root.join("pty/data"))
            .env("XDG_STATE_HOME", root.join("pty/state"))
            .env("XDG_CACHE_HOME", root.join("pty/cache"))
            .current_dir(root.join("workspace"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let input = child.stdin.take();
        let output = child.stdout.take().ok_or("missing PTY output")?;
        let (reader, received) = read_output(output);
        let mut terminal = Self {
            child,
            input,
            reader: Some(reader),
            received: Some(received),
            output: String::new(),
            undecoded: Vec::new(),
            bytes: 0,
            deadline: Instant::now() + Duration::from_secs(8),
            editor: String::new(),
        };
        terminal.wait(|text| text.contains("REV_EDITOR_PID=") && text.contains("terminal.rs"))?;
        terminal.editor = terminal
            .output
            .lines()
            .find_map(|line| line.strip_prefix("REV_EDITOR_PID="))
            .ok_or("missing owned editor PID")?
            .trim()
            .to_owned();
        terminal.frame(|frame| frame.contains("terminal.rs"))?;
        Ok(terminal)
    }

    fn wait(&mut self, predicate: impl Fn(&str) -> bool) -> Result<()> {
        while !predicate(&self.output) {
            let remaining = self.deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err("PTY exceeded eight-second deadline".into());
            }
            let chunk = self
                .received
                .as_ref()
                .ok_or("closed PTY reader")?
                .recv_timeout(remaining)?;
            self.append(chunk)?;
        }
        Ok(())
    }

    fn append(&mut self, chunk: Vec<u8>) -> Result<()> {
        self.bytes += chunk.len();
        if self.bytes > LIMIT {
            return Err("PTY exceeded 1 MiB output limit".into());
        }
        self.undecoded.extend(chunk);
        let prefix = match std::str::from_utf8(&self.undecoded) {
            Ok(text) => text.len(),
            Err(error) if error.error_len().is_none() => error.valid_up_to(),
            Err(error) => return Err(error.into()),
        };
        self.output
            .push_str(std::str::from_utf8(&self.undecoded[..prefix])?);
        self.undecoded.drain(..prefix);
        Ok(())
    }

    pub(super) fn frame(&mut self, predicate: impl Fn(&str) -> bool) -> Result<String> {
        if let Err(error) = self.wait(|output| frame(output).is_some_and(&predicate)) {
            return Err(format!(
                "PTY frame failed: {error}; tail={:?}",
                self.output.chars().rev().take(1500).collect::<String>()
            )
            .into());
        }
        Ok(frame(&self.output)
            .ok_or("missing completed frame")?
            .to_owned())
    }

    pub(super) fn send(&mut self, input: &[u8]) -> Result<()> {
        self.output.clear();
        self.input
            .as_mut()
            .ok_or("closed PTY input")?
            .write_all(input)?;
        self.input.as_mut().ok_or("closed PTY input")?.flush()?;
        Ok(())
    }

    pub(super) fn idle(&mut self) -> Result<()> {
        let ticks = cpu_ticks(&self.editor)?;
        assert!(
            matches!(
                self.received
                    .as_ref()
                    .ok_or("closed PTY reader")?
                    .recv_timeout(Duration::from_millis(350)),
                Err(RecvTimeoutError::Timeout)
            ),
            "idle editor emitted terminal output"
        );
        let ticks = cpu_ticks(&self.editor)?.saturating_sub(ticks);
        assert!(ticks <= 1, "idle editor consumed {ticks} CPU ticks");
        println!(
            "MEASURE Linux PTY idle: 350ms silence, cpu_ticks={ticks} peak_rss={}",
            peak_rss(&self.editor)?
        );
        Ok(())
    }

    pub(super) fn finish(&mut self) -> Result<()> {
        self.send(b"\x18\x03")?;
        self.wait(|text| text.contains("REV_TTY_RESTORED"))?;
        assert!(
            self.output.contains("\x1b[?1049l")
                && self.output.contains("\x1b[?2004l")
                && self.output.contains(SHOW)
        );
        self.input.take();
        assert!(self.child.wait()?.success());
        Ok(())
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.input.take();
        self.received.take();
        let _ = self.child.wait();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

fn frame(output: &str) -> Option<&str> {
    let start = output.rfind(HIDE)?;
    let frame = &output[start..];
    let end = frame.rfind(SHOW)?;
    Some(&frame[..end + SHOW.len()])
}

pub(super) fn row(frame: &str, wanted: usize) -> String {
    let mut row = None;
    let mut text = String::new();
    for part in frame.split("\x1b[").skip(1) {
        let Some(end) = part.find(|ch: char| ('@'..='~').contains(&ch)) else {
            continue;
        };
        if part.as_bytes()[end] == b'H' {
            row = part[..end]
                .split(';')
                .next()
                .and_then(|value| value.parse::<usize>().ok());
        }
        if row == Some(wanted) {
            text.push_str(&part[end + 1..]);
        }
    }
    text
}

pub(super) fn run(root: &Path, binary: &Path, wrapper: &Path) -> Result<()> {
    let original = "fn configuration() {}\nfn config_value() {}\ncon\n";
    let path = root.join("workspace/terminal.rs");
    std::fs::write(&path, original)?;
    let mut terminal = Terminal::start(root, binary, wrapper)?;
    manual_cancel(&mut terminal)?;
    manual_accept(&mut terminal)?;
    automatic(&mut terminal)?;
    terminal.idle()?;
    terminal.finish()?;
    assert_eq!(std::fs::read_to_string(path)?, original);
    println!(
        "PASS Linux PTY completion: {} output bytes; manual selection/cancel/accept, automatic debounce, literal one-step undo, source unchanged and TTY restored",
        terminal.bytes
    );
    Ok(())
}

fn read_output(mut output: std::process::ChildStdout) -> (JoinHandle<()>, Receiver<Vec<u8>>) {
    let (send, received) = sync_channel(256);
    let reader = std::thread::spawn(move || {
        let mut bytes = 0;
        while bytes <= LIMIT {
            let mut chunk = vec![0; 8192];
            let count = match output.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(count) => count,
            };
            bytes += count;
            chunk.truncate(count);
            if send.send(chunk).is_err() {
                break;
            }
        }
    });
    (reader, received)
}

fn manual_cancel(terminal: &mut Terminal) -> Result<()> {
    terminal.send(b"\x1bxgoto-line\r3\r\x05\x1bxcompletion-at-point\r")?;
    terminal.frame(popup)?;
    terminal.send(b"\x07")?;
    terminal.frame(|frame| source_is(frame, "con", false))?;
    Ok(())
}
fn manual_accept(terminal: &mut Terminal) -> Result<()> {
    terminal.send(b"\x1bxcompletion-at-point\r")?;
    terminal.frame(popup)?;
    terminal.send(b"\x0e\r")?;
    terminal.frame(|frame| source_is(frame, "configuration", true))?;
    terminal.send(b"\x18u")?;
    terminal.frame(|frame| source_is(frame, "con", false))?;
    Ok(())
}
fn automatic(terminal: &mut Terminal) -> Result<()> {
    terminal.send(b"f")?;
    terminal.frame(popup)?;
    terminal.send(b"\t")?;
    terminal.frame(|frame| source_is(frame, "config_value", true))?;
    terminal.send(b"\x18u")?;
    terminal.frame(|frame| source_is(frame, "conf", true))?;
    terminal.send(b"\x18u")?;
    terminal.frame(|frame| source_is(frame, "con", false))?;
    Ok(())
}

fn popup(frame: &str) -> bool {
    frame.contains('┌') && frame.contains("config_value") && frame.contains("configuration")
}
fn source_is(frame: &str, source: &str, modified: bool) -> bool {
    !frame.contains('┌')
        && row(frame, 3).contains(source)
        && frame.contains(if modified {
            "terminal.rs ** RW"
        } else {
            "terminal.rs -- RW"
        })
}
