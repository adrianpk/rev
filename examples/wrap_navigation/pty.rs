use std::error::Error;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{Receiver, RecvTimeoutError, sync_channel};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

const HIDE: &str = "\x1b[?25l";
const SHOW: &str = "\x1b[?25h";
const OUTPUT_LIMIT: usize = 1024 * 1024;

// util-linux script owns the controlling PTY; its wrapper times out the editor
// directly, so failure cannot strand it under an interrupted shell. The reader,
// channel and output retain at most 1 MiB plus one read chunk. Drop closes input,
// lets that owned child exit, and reaps both the process and reader on failure.
struct Terminal {
    child: Child,
    input: Option<ChildStdin>,
    reader: Option<JoinHandle<()>>,
    received: Option<Receiver<Vec<u8>>>,
    output: String,
    undecoded: Vec<u8>,
    bytes: usize,
    deadline: Instant,
    tty: PathBuf,
}

impl Terminal {
    fn start(root: &Path, binary: &Path, wrapper: &Path) -> Result<Self, Box<dyn Error>> {
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
            ])
            .arg("bash")
            .arg(wrapper)
            .arg("--pty-child")
            .arg(binary)
            .arg(root.join("workspace"))
            .current_dir(root.join("workspace"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let input = child.stdin.take();
        let mut output = child.stdout.take().unwrap();
        let (send, received) = sync_channel(256);
        let reader = std::thread::spawn(move || {
            let mut bytes = 0;
            while bytes <= OUTPUT_LIMIT {
                let mut chunk = vec![0; 8192];
                let count = match output.read(&mut chunk) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => n,
                };
                bytes += count;
                chunk.truncate(count);
                if send.send(chunk).is_err() {
                    break;
                }
            }
        });
        let mut terminal = Self {
            child,
            input,
            reader: Some(reader),
            received: Some(received),
            output: String::new(),
            undecoded: Vec::new(),
            bytes: 0,
            deadline: Instant::now() + Duration::from_secs(8),
            tty: PathBuf::new(),
        };
        terminal.wait(|s| s.contains("REV_PTY_TTY=") && s.contains('\n'))?;
        terminal.tty = terminal
            .output
            .lines()
            .find_map(|line| line.trim().strip_prefix("REV_PTY_TTY="))
            .unwrap()
            .into();
        terminal
            .frame(|f| f.contains("Tree *") && f.contains("alpha.rs") && f.contains("bravo.org"))?;
        let flags = Command::new("stty")
            .arg("-F")
            .arg(&terminal.tty)
            .arg("-a")
            .output()?;
        assert!(flags.status.success());
        let flags = String::from_utf8(flags.stdout)?;
        assert!(
            flags
                .split_whitespace()
                .any(|v| v.trim_end_matches(';') == "-icanon")
        );
        assert!(
            flags
                .split_whitespace()
                .any(|v| v.trim_end_matches(';') == "-echo")
        );
        Ok(terminal)
    }

    fn wait(&mut self, predicate: impl Fn(&str) -> bool) -> Result<(), Box<dyn Error>> {
        while !predicate(&self.output) {
            let duration = self.deadline.saturating_duration_since(Instant::now());
            if duration.is_zero() {
                return Err("PTY exceeded eight-second deadline".into());
            }
            let chunk = self.received.as_ref().unwrap().recv_timeout(duration)?;
            self.bytes += chunk.len();
            if self.bytes > OUTPUT_LIMIT {
                return Err("PTY exceeded 1 MiB output limit".into());
            }
            // ANSI and UTF-8 may cross reads. Retain an incomplete UTF-8 suffix
            // until its next bytes arrive; reject genuinely invalid output.
            self.undecoded.extend(chunk);
            let prefix = match std::str::from_utf8(&self.undecoded) {
                Ok(text) => text.len(),
                Err(error) if error.error_len().is_none() => error.valid_up_to(),
                Err(error) => return Err(error.into()),
            };
            self.output
                .push_str(std::str::from_utf8(&self.undecoded[..prefix])?);
            self.undecoded.drain(..prefix);
        }
        Ok(())
    }

    fn frame(&mut self, predicate: impl Fn(&str) -> bool) -> Result<String, Box<dyn Error>> {
        if let Err(error) = self.wait(|s| last_frame(s).is_some_and(&predicate)) {
            let tail: String = self
                .output
                .chars()
                .rev()
                .take(1600)
                .collect::<String>()
                .chars()
                .rev()
                .collect();
            return Err(format!("PTY frame wait failed: {error}; output tail: {tail:?}").into());
        }
        Ok(last_frame(&self.output).unwrap().to_owned())
    }

    fn send(&mut self, input: &[u8]) -> Result<(), Box<dyn Error>> {
        self.output.clear();
        self.input.as_mut().unwrap().write_all(input)?;
        self.input.as_mut().unwrap().flush()?;
        Ok(())
    }

    fn resize(&mut self, width: u16) -> Result<(), Box<dyn Error>> {
        self.output.clear();
        assert!(
            Command::new("stty")
                .arg("-F")
                .arg(&self.tty)
                .args(["rows", "24", "cols", &width.to_string()])
                .status()?
                .success()
        );
        Ok(())
    }

    fn finish(&mut self) -> Result<(), Box<dyn Error>> {
        self.send(b"\x18\x03")?;
        self.wait(|s| s.contains("REV_TTY_RESTORED"))?;
        assert!(self.output.contains("\x1b[?1049l"));
        assert!(self.output.contains("\x1b[?2004l"));
        assert!(self.output.contains(SHOW));
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

fn last_frame(output: &str) -> Option<&str> {
    let end = output.rfind(SHOW)? + SHOW.len();
    let start = output[..end].rfind(HIDE)?;
    Some(&output[start..end])
}

fn status(frame: &str, marker: &str) -> bool {
    frame
        .split_whitespace()
        .collect::<String>()
        .contains(&marker.split_whitespace().collect::<String>())
}

// Recover a tree row only from the completed redraw's absolute row moves.
// Labels help locate fixture cells; the editor resolves the actual hit identity.
fn tree_row(frame: &str, name: &str) -> u16 {
    let mut row = None;
    for part in frame.split("\x1b[") {
        let Some(end) = part.find(|c: char| c.is_ascii_alphabetic()) else {
            continue;
        };
        let (parameters, rest) = part.split_at(end);
        if rest.starts_with('H') {
            row = parameters
                .split(';')
                .next()
                .and_then(|value| value.parse().ok());
        }
        if rest[1..].split('│').next().unwrap().contains(name) {
            return row.expect("tree label has no absolute row");
        }
    }
    panic!("tree label absent from completed frame: {name}");
}

pub(super) fn run(root: &Path, binary: &Path, wrapper: &Path) -> Result<(), Box<dyn Error>> {
    let rust_path = root.join("workspace/alpha.rs");
    let org_path = root.join("workspace/bravo.org");
    let rust = std::fs::read(&rust_path)?;
    let org = std::fs::read(&org_path)?;
    let started = Instant::now();
    let mut terminal = Terminal::start(root, binary, wrapper)?;
    terminal.send(b"\x18\x06a\t")?;
    terminal.frame(|f| f.contains("Find file: alpha.rs") && f.contains("*scratch*"))?;
    terminal.send(b"\r")?;
    terminal.frame(|f| status(f, "> alpha.rs -- RW -- L1 C1 Rust"))?;
    terminal.send(b"\x18\x33")?;
    let frame = terminal.frame(|f| f.matches("alpha.rs -- RW").count() == 2)?;
    let row = tree_row(&frame, "bravo.org");
    terminal.send(format!("\x1b[<0;2;{row}M").as_bytes())?;
    terminal
        .frame(|f| status(f, "> bravo.org -- RW -- L1 C1 Org") && f.contains("alpha.rs -- RW"))?;
    terminal.send(b"\x18o")?;
    terminal.frame(|f| status(f, "> alpha.rs -- RW -- L1 C1 Rust"))?;
    terminal.send(b"\x0e")?;
    terminal.frame(|f| {
        status(f, "> alpha.rs -- RW -- L1 C") && !status(f, "> alpha.rs -- RW -- L1 C1 Rust")
    })?;
    terminal.send(b"\x0b")?;
    terminal.frame(|f| f.contains("alpha.rs ** RW"))?;
    terminal.send(b"\x18u")?;
    terminal.frame(|f| status(f, "> alpha.rs -- RW -- L1 C"))?;
    terminal.send(b"\x1b<\x13NEEDLE")?;
    terminal.frame(|f| f.contains("I-search:") && f.contains("NEEDLE"))?;
    terminal.send(b"\x07")?;
    terminal.frame(|f| status(f, "> alpha.rs -- RW -- L1 C1 Rust"))?;
    for width in [100, 140, 120] {
        terminal.resize(width)?;
        terminal.frame(|f| status(f, "> alpha.rs -- RW -- L1 C1 Rust"))?;
    }
    terminal.send(b"\x1bxvisual-line-mode\r\x05")?;
    terminal.frame(|f| status(f, &format!("> alpha.rs -- RW -- L1 C{} Rust", rust.len())))?;
    terminal.send(b"\x1bxvisual-line-mode\r\x18\x13")?;
    terminal.frame(|f| f.contains("Saved"))?;
    terminal.send(b"\x18\x06not-created\x07")?;
    terminal.frame(|f| f.contains("Quit") && !f.contains("Find file:"))?;
    // A settled session has no periodic redraw. The existing long-line probe
    // separately checks all pending native layout/input work has retired.
    terminal.output.clear();
    assert!(matches!(
        terminal
            .received
            .as_ref()
            .unwrap()
            .recv_timeout(Duration::from_millis(350)),
        Err(RecvTimeoutError::Timeout)
    ));
    terminal.finish()?;
    assert_eq!(std::fs::read(rust_path)?, rust);
    assert_eq!(std::fs::read(org_path)?, org);
    assert!(!root.join("workspace/not-created").exists());
    println!(
        "PASS Linux PTY joint journey: {}ms, {} output bytes; SGR click, splits, visual editing/search, resize, no-wrap, cancellation, exact save, idle silence and restored TTY",
        started.elapsed().as_millis(),
        terminal.bytes
    );
    Ok(())
}
