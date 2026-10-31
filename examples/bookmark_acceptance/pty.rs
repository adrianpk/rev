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
    fn start(
        root: &Path,
        binary: &Path,
        wrapper: &Path,
        file: &str,
    ) -> Result<Self, Box<dyn Error>> {
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
            .arg(root.join("workspace").join(file))
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
        terminal.frame(|f| status(f, &format!("> {file} -- RW -- L1 C1 Text")))?;
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

// A read-only bookmark list intentionally has no visible cursor. Recognize
// its complete last screen row as well as the ordinary cursor-show trailer;
// never fall back to an older frame while a new redraw is incomplete.
fn last_frame(output: &str) -> Option<&str> {
    use unicode_width::UnicodeWidthStr;
    let start = output.rfind(HIDE)?;
    let frame = &output[start..];
    if let Some(end) = frame.rfind(SHOW) {
        return Some(&frame[..end + SHOW.len()]);
    }
    let width = screen_row(frame, 1).width();
    (width >= 3 && screen_row(frame, 24).width() == width && frame.ends_with("\x1b[0m"))
        .then_some(frame)
}

fn screen_row(frame: &str, wanted: u16) -> String {
    let mut row = None;
    let mut result = String::new();
    for part in frame.split("\x1b[").skip(1) {
        let Some(end) = part.find(|c: char| ('@'..='~').contains(&c)) else {
            continue;
        };
        if part.as_bytes()[end] == b'H' {
            row = part[..end]
                .split(';')
                .next()
                .and_then(|v| v.parse::<u16>().ok());
        }
        if row == Some(wanted) {
            result.push_str(&part[end + 1..]);
        }
    }
    result
}

fn status(frame: &str, marker: &str) -> bool {
    frame
        .split_whitespace()
        .collect::<String>()
        .contains(&marker.split_whitespace().collect::<String>())
}

pub(super) fn run(root: &Path, binary: &Path, wrapper: &Path) -> Result<(), Box<dyn Error>> {
    use unicode_width::UnicodeWidthStr;
    let path = root.join("workspace/alpha.txt");
    let original = std::fs::read(&path)?;
    let position = original.iter().position(|b| *b == b'\n').unwrap() + 1;
    let started = Instant::now();
    let mut first = Terminal::start(root, binary, wrapper, "alpha.txt")?;
    first.send(b"\x1bxgoto-line\r2\r\x18rmpty\r")?;
    first.frame(|f| {
        f.contains("Bookmark set")
            && f.contains('●')
            && status(f, "> alpha.txt -- RW -- L2 C1 Text")
    })?;
    first.send(b"\x18rMpty\r")?;
    first.frame(|f| f.contains("already exists"))?;
    first.send(b"\x07")?;
    first.frame(|f| f.contains("Quit") && status(f, "> alpha.txt -- RW -- L2 C1 Text"))?;
    first.send(b"\x1b<\x1b[200~temporary edit\n\x1b[201~")?;
    first.frame(|f| f.contains("alpha.txt ** RW"))?;
    first.send(b"\x18u")?;
    first.frame(|f| f.contains("alpha.txt -- RW") && !f.contains("temporary edit"))?;
    first.send(b"\x18rbpty\r\x18\x32")?;
    first.frame(|f| f.matches("alpha.txt -- RW").count() == 2 && f.matches('●').count() == 2)?;
    first.send(b"\x1bxbookmark-markers-mode\r")?;
    first.frame(|f| f.matches("alpha.txt -- RW").count() == 2 && f.matches('●').count() == 1)?;
    first.send(b"\x1bxgutter-toggle-line-numbers\r\x1bxbookmark-markers-mode\r")?;
    first.frame(|f| f.matches("alpha.txt -- RW").count() == 2 && f.matches('●').count() == 2)?;
    first.send(b"\x1bxgutter-set-line-numbers-relative\r\x1bxgutter-toggle-line-number-side\r")?;
    first.frame(|f| f.matches('●').count() == 2 && screen_row(f, 1).width() == 100)?;
    for width in [80, 120, 100] {
        first.resize(width)?;
        first.frame(|f| {
            screen_row(f, 1).width() == usize::from(width) && f.matches('●').count() == 2
        })?;
    }
    first.send(b"\x18rmpty second\r\x18rl")?;
    first.frame(|f| f.contains("*Bookmarks* [read-only] 2 entries") && f.contains("pty second"))?;
    first.send(b"\x1b[Bdx")?;
    first
        .frame(|f| f.contains("*Bookmarks* [read-only] 1 entries") && !f.contains("pty second"))?;
    first.send(b"q\x1bxbookmark-save\r")?;
    first.frame(|f| f.contains("Bookmarks saved") && f.matches('●').count() == 2)?;
    first.finish()?;
    let first_bytes = first.bytes;
    drop(first);

    let store = root.join("pty/data/rev/bookmarks.toml");
    let document: toml::Value = toml::from_str(&std::fs::read_to_string(&store)?)?;
    let records = document["bookmarks"].as_array().unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["name"].as_str(), Some("pty"));
    assert_eq!(records[0]["position"].as_integer(), Some(position as i64));
    assert_eq!(std::fs::read(&path)?, original);
    let mut second = Terminal::start(root, binary, wrapper, "origin.txt")?;
    second.send(b"\x18rbpty\r")?;
    second
        .frame(|f| status(f, "> alpha.txt -- RW -- L2 C1 Text") && f.matches('●').count() == 1)?;
    second.send(b"\x18rbmissing\r")?;
    second.frame(|f| f.contains("No bookmark") && f.contains("missing"))?;
    second.send(b"\x07")?;
    second.frame(|f| f.contains("Quit") && status(f, "> alpha.txt -- RW -- L2 C1 Text"))?;
    second.send(b"\x18rl")?;
    second.frame(|f| f.contains("*Bookmarks* [read-only] 1 entries"))?;
    second.send(b"dx")?;
    second.frame(|f| f.contains("*Bookmarks* [read-only] 0 entries"))?;
    second.send(b"q\x1bxbookmark-save\r")?;
    second.frame(|f| f.contains("Bookmarks saved") && !f.contains('●'))?;
    // Once native work has settled, idle input has no periodic redraw.
    assert!(matches!(
        second
            .received
            .as_ref()
            .unwrap()
            .recv_timeout(Duration::from_millis(350)),
        Err(RecvTimeoutError::Timeout)
    ));
    second.finish()?;
    let document: toml::Value = toml::from_str(&std::fs::read_to_string(&store)?)?;
    assert_eq!(document["version"].as_integer(), Some(1));
    assert!(document.get("bookmarks").is_none());
    let config = root.join("verify-empty.toml");
    std::fs::write(&config, format!("bookmarks.file='{}'\n", store.display()))?;
    let startup = rev::config::prepare(&config);
    assert!(startup.diagnostic.is_none());
    let loaded =
        super::support::configured(&startup.snapshot, Some(&root.join("workspace/origin.txt")))?;
    assert!(loaded.bookmarks().is_empty());
    assert_eq!(std::fs::read(path)?, original);
    println!(
        "PASS Linux PTY bookmark restart: {}ms, {first_bytes}/{} output bytes; set/refusal, edit/undo, splits, independent numbers/markers, resize, list/delete, persisted jump, missing-name cancellation, 350ms idle silence and restored TTY",
        started.elapsed().as_millis(),
        second.bytes
    );
    Ok(())
}
