use super::Fixture;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::{
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

const CASE: &str = "REV_TEST_TERMINAL_WAKE";
const LIMIT: u64 = 1024 * 1024;

// Each fresh process owns the native event singleton and a real controlling
// PTY. Both readiness orders and a multi-read burst must survive one signal
// without a second input write supplying a replacement Mio edge.
#[test]
fn signal_keys() {
    if let Ok(case) = std::env::var(CASE) {
        probe(&case);
        return;
    }
    for case in ["signal-first", "input-first", "burst"] {
        run(case);
    }
}

struct Owned(Child);
impl Drop for Owned {
    fn drop(&mut self) {
        self.0.stdin.take();
        // The owned wrapper enforces three seconds plus one second to kill.
        self.0.wait().unwrap();
    }
}

fn run(case: &str) {
    let fixture = Fixture::new();
    let listener = UnixListener::bind(fixture.0.join("control")).unwrap();
    listener.set_nonblocking(true).unwrap();
    let mut child = Owned(
        Command::new("script")
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
            .args(["timeout", "--foreground", "--kill-after=1", "3"])
            .arg(std::env::current_exe().unwrap())
            .args(["--exact", "readiness::signal_keys", "--nocapture"])
            .env(CASE, case)
            .current_dir(&fixture.0)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let mut control = connect(&listener);
    let mut byte = [0];
    control.read_exact(&mut byte).unwrap();
    assert_eq!(&byte, b"R");
    let bytes = if case == "burst" {
        vec![b'a'; 1300]
    } else {
        b"\x1b[120;3u".to_vec()
    };
    let input = child.0.stdin.as_mut().unwrap();
    input.write_all(&bytes).unwrap();
    input.flush().unwrap();
    control.write_all(b"G").unwrap();
    let completed = control.read_exact(&mut byte);
    child.0.stdin.take();
    let mut output = Vec::new();
    child
        .0
        .stdout
        .take()
        .unwrap()
        .take(LIMIT + 1)
        .read_to_end(&mut output)
        .unwrap();
    assert!(output.len() as u64 <= LIMIT, "probe output exceeded 1 MiB");
    assert!(
        child.0.wait().unwrap().success(),
        "{case}: {}",
        String::from_utf8_lossy(&output)
    );
    completed.unwrap();
    assert_eq!(&byte, b"D");
}

fn connect(listener: &UnixListener) -> UnixStream {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                return stream;
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(Instant::now() < deadline, "probe did not connect");
                std::thread::sleep(Duration::from_millis(1));
            }
            Err(error) => panic!("probe control: {error}"),
        }
    }
}

fn probe(case: &str) {
    assert!(
        Command::new("stty")
            .args(["rows", "24", "cols", "80"])
            .status()
            .unwrap()
            .success()
    );
    let mut control = UnixStream::connect("control").unwrap();
    control
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    crossterm::terminal::enable_raw_mode().unwrap();
    assert!(!event::poll(Duration::ZERO).unwrap());
    if case != "input-first" {
        wake();
    }
    control.write_all(b"R").unwrap();
    control.read_exact(&mut [0]).unwrap();
    let count = if case == "burst" { 1300 } else { 8 };
    let deadline = Instant::now() + Duration::from_secs(2);
    while rustix::io::ioctl_fionread(std::io::stdin()).unwrap() < count {
        assert!(Instant::now() < deadline, "input did not reach owned PTY");
        std::thread::sleep(Duration::from_millis(1));
    }
    if case == "input-first" {
        wake();
    }
    observe(case);
    assert_eq!(rustix::io::ioctl_fionread(std::io::stdin()).unwrap(), 0);
    crossterm::terminal::disable_raw_mode().unwrap();
    control.write_all(b"D").unwrap();
}

fn wake() {
    rustix::process::kill_process(rustix::process::getpid(), rustix::process::Signal::WINCH)
        .unwrap();
}

fn observe(case: &str) {
    let expected = if case == "burst" { 1300 } else { 1 };
    let mut keys = 0;
    let mut resizes = 0;
    for _ in 0..=expected {
        assert!(
            event::poll(Duration::from_millis(100)).unwrap(),
            "{case}: lost queued native event after {keys} keys/{resizes} resizes"
        );
        match event::read().unwrap() {
            Event::Resize(width, height) => {
                assert_eq!((width, height), (80, 24));
                resizes += 1;
            }
            Event::Key(key) => {
                let code = if case == "burst" { 'a' } else { 'x' };
                let modifiers = if case == "burst" {
                    KeyModifiers::NONE
                } else {
                    KeyModifiers::ALT
                };
                assert_eq!((key.code, key.modifiers), (KeyCode::Char(code), modifiers));
                keys += 1;
            }
            event => panic!("unexpected native event: {event:?}"),
        }
    }
    assert_eq!((keys, resizes), (expected, 1));
}
