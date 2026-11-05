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
pub(crate) struct Terminal {
    child: Child,
    input: Option<ChildStdin>,
    reader: Option<JoinHandle<()>>,
    received: Option<Receiver<Vec<u8>>>,
    output: String,
    undecoded: Vec<u8>,
    bytes: usize,
    deadline: Instant,
    editor: String,
    dimensions: Option<(usize, usize)>,
}

impl Terminal {
    pub(crate) fn start(root: &Path, binary: &Path, wrapper: &Path, name: &str) -> Result<Self> {
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
            .arg(root.join("workspace").join(name))
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
            dimensions: None,
        };
        terminal.wait(|text| text.contains("REV_EDITOR_PID=") && text.contains(name))?;
        terminal.editor = terminal
            .output
            .lines()
            .find_map(|line| line.strip_prefix("REV_EDITOR_PID="))
            .ok_or("missing owned editor PID")?
            .trim()
            .to_owned();
        terminal.dimensions = dimensions(&terminal.output);
        terminal.frame(|frame| frame.contains(name))?;
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

    pub(crate) fn frame(&mut self, predicate: impl Fn(&str) -> bool) -> Result<String> {
        let dimensions = self.dimensions;
        if let Err(error) = self.wait(|output| frame(output, dimensions).is_some_and(&predicate)) {
            return Err(format!(
                "PTY frame failed: {error}; bytes={} geometry={dimensions:?}; complete={} visible={}",
                self.bytes,
                frame(&self.output, dimensions).is_some(),
                diagnostic(&self.output, dimensions)
            )
            .into());
        }
        Ok(frame(&self.output, dimensions)
            .ok_or("missing completed frame")?
            .to_owned())
    }

    pub(crate) fn send(&mut self, input: &[u8]) -> Result<()> {
        self.output.clear();
        self.input
            .as_mut()
            .ok_or("closed PTY input")?
            .write_all(input)?;
        self.input.as_mut().ok_or("closed PTY input")?.flush()?;
        Ok(())
    }

    pub(crate) fn idle(&mut self) -> Result<()> {
        self.settle_output()?;
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

    // Establish a measurement epoch after queued command redraws. Every byte
    // still consumes the same stream cap; a busy renderer cannot pass either
    // this bounded 350-ms quiet barrier or the subsequent 350-ms silence assertion.
    fn settle_output(&mut self) -> Result<()> {
        let deadline = self.deadline.min(Instant::now() + Duration::from_secs(2));
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining < Duration::from_millis(350) {
                return Err("PTY did not become quiescent".into());
            }
            match self
                .received
                .as_ref()
                .ok_or("closed PTY reader")?
                .recv_timeout(remaining.min(Duration::from_millis(350)))
            {
                Ok(chunk) => self.append(chunk)?,
                Err(RecvTimeoutError::Timeout) => return Ok(()),
                Err(error) => return Err(error.into()),
            }
        }
    }

    // Mode/style bytes can precede a frame's hide-cursor delimiter. Keep their
    // observation on the same bounded stream used for restoration assertions.
    pub(crate) fn contains(&self, sequence: &str) -> bool {
        self.output.contains(sequence)
    }

    pub(crate) fn bytes(&self) -> usize {
        self.bytes
    }

    // Git navigation may hide the cursor; after a real resize its completed
    // frame must be recognized with the new geometry, under the same byte cap.
    #[allow(dead_code, reason = "Shared PTY callers opt into resize probes.")]
    pub(crate) fn resize(&mut self, rows: usize, columns: usize) -> Result<()> {
        self.send(b"")?;
        self.dimensions = Some((rows, columns));
        let tty = format!("/proc/{}/fd/0", self.editor);
        assert!(
            Command::new("stty")
                .args([
                    "-F",
                    &tty,
                    "rows",
                    &rows.to_string(),
                    "cols",
                    &columns.to_string()
                ])
                .status()?
                .success()
        );
        assert!(
            Command::new("kill")
                .args(["-WINCH", &self.editor])
                .status()?
                .success()
        );
        Ok(())
    }

    pub(crate) fn finish(&mut self) -> Result<()> {
        self.send(b"\x18\x03")?;
        self.wait(|text| text.contains("REV_TTY_RESTORED"))?;
        assert!(
            self.contains("\x1b[?1049l") && self.contains("\x1b[?2004l") && self.contains(SHOW)
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

// A child can legitimately hide the host cursor. Only the owned integrated
// fixture advertises geometry; older source journeys retain their Show footer.
fn dimensions(output: &str) -> Option<(usize, usize)> {
    let line = output
        .lines()
        .find_map(|line| line.strip_prefix("REV_EDITOR_SIZE="))?;
    let mut words = line.split_whitespace();
    Some((words.next()?.parse().ok()?, words.next()?.parse().ok()?))
}
fn frame(output: &str, dimensions: Option<(usize, usize)>) -> Option<&str> {
    let start = output.rfind(HIDE)?;
    let frame = &output[start..];
    if let Some(end) = frame.rfind(SHOW) {
        return Some(&frame[..end + SHOW.len()]);
    }
    let (rows, columns) = dimensions?;
    hidden_frame(frame, rows, columns)
}
fn hidden_frame(frame: &str, rows: usize, columns: usize) -> Option<&str> {
    use unicode_width::UnicodeWidthStr;
    let last = format!("\x1b[{rows};1H");
    let start = frame.rfind(&last)? + last.len();
    for (offset, _) in frame[start..].match_indices("\x1b[0m") {
        let end = start + offset + 4;
        if row(&frame[..end], rows).width() == columns {
            return Some(&frame[..end]);
        }
    }
    None
}

pub(crate) fn row(frame: &str, wanted: usize) -> String {
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

fn diagnostic(output: &str, dimensions: Option<(usize, usize)>) -> String {
    let frame = output.rsplit_once(HIDE).map_or(output, |(_, frame)| frame);
    let (rows, _) = dimensions.unwrap_or((24, 100));
    (1..=rows)
        .map(|number| format!("{number}: {}", row(frame, number)))
        .collect::<Vec<_>>()
        .join("\n")
        .chars()
        .take(4500)
        .collect()
}
