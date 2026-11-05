use super::*;
use crate::fixture::Fixture;

fn program(fixture: &Fixture, script: &str, output_bytes: usize) -> Program {
    Program {
        argv: vec!["/bin/sh".into(), "-c".into(), script.into()],
        directory: fixture.0.clone(),
        seconds: 1,
        output_bytes,
        input: None,
        editors: Editors::Disabled,
    }
}

// Real OS processes establish exact argv bytes, full draining after exit and
// retained failure prefixes. No Git output is synthesized by these tests.
#[test]
fn retained_capture() {
    let fixture = Fixture::new();
    let bytes = vec![b'a'; 262144];
    std::fs::write(fixture.0.join("output"), &bytes).unwrap();
    for (name, script, limit, expected) in [
        ("full drain", "cat output", 262144, End::Exited),
        ("overflow", "cat output", 4096, End::OutputLimit),
        (
            "failed exit",
            "printf before; printf diagnostic >&2; exit 7",
            4096,
            End::Exited,
        ),
    ] {
        let result = program(&fixture, script, limit).run(&Cancellation::default(), |_, _| {});
        assert_eq!(result.end, expected, "{name}: {:?}", result.diagnostic);
        assert!(result.retained.is_none(), "{name}");
        assert!(result.status.is_some(), "{name}");
        assert!(result.stdout.len() + result.stderr.len() <= limit, "{name}");
        match name {
            "full drain" => assert_eq!(result.stdout, bytes),
            "overflow" => assert_eq!(result.stdout.len(), limit),
            _ => {
                assert_eq!(result.status.unwrap().code(), Some(7));
                assert_eq!(result.stdout, b"before");
                assert_eq!(result.stderr, b"diagnostic");
            }
        }
    }
    use std::os::unix::ffi::OsStringExt;
    let argument = OsString::from_vec(b"-raw\n\xff".to_vec());
    let mut exact = program(&fixture, "printf '%s' \"$1\"", 4096);
    exact
        .argv
        .extend([OsString::from("fixture"), argument.clone()]);
    let result = exact.run(&Cancellation::default(), |_, _| {});
    assert_eq!(result.stdout, argument.into_vec());
}

// Cancellation and timeout retain diagnostics, reap the leader and stop its
// owned descendants through the capture owner's process group.
#[test]
fn bounded_cleanup() {
    for cancelled in [true, false] {
        let fixture = Fixture::new();
        let control = Cancellation::default();
        let cancel = control.clone();
        let program = program(
            &fixture,
            "printf before; printf diagnostic >&2; echo $$ > leader; sleep 30 & echo $! > descendant; wait",
            4096,
        );
        let worker = std::thread::spawn(move || program.run(&control, |_, _| {}));
        let deadline = Instant::now() + Duration::from_secs(4);
        while !fixture.0.join("descendant").exists() {
            assert!(Instant::now() < deadline, "capture fixture did not start");
            std::thread::sleep(Duration::from_millis(2));
        }
        if cancelled {
            cancel.cancel();
        }
        let result = worker.join().unwrap();
        assert_eq!(
            result.end,
            if cancelled {
                End::Cancelled
            } else {
                End::TimedOut
            }
        );
        assert!(Instant::now() < deadline);
        assert!(result.retained.is_none());
        assert_eq!(result.stdout, b"before");
        assert_eq!(result.stderr, b"diagnostic");
        let leader = std::fs::read_to_string(fixture.0.join("leader")).unwrap();
        assert!(!PathBuf::from(format!("/proc/{}", leader.trim())).exists());
        let descendant = std::fs::read_to_string(fixture.0.join("descendant")).unwrap();
        let stat = std::fs::read_to_string(format!("/proc/{}/stat", descendant.trim()));
        assert!(stat.is_err() || stat.unwrap().split_once(") ").unwrap().1.starts_with('Z'));
    }
}

// NUL and aggregate argv overflow fail before process launch. A pre-cancelled
// request likewise cannot create a child or claim an exit status.
#[test]
fn launch_admission() {
    let fixture = Fixture::new();
    for name in ["NUL", "aggregate", "cancelled"] {
        let cancel = Cancellation::default();
        let mut p = program(&fixture, "exit 0", 4096);
        match name {
            "NUL" => p.argv.push(OsString::from("\0")),
            "aggregate" => p.argv.extend(vec![OsString::from("a".repeat(4096)); 16]),
            _ => cancel.cancel(),
        }
        let mut launched = false;
        let result = p.run(&cancel, |_, _| launched = true);
        assert!(!launched, "{name}");
        assert!(result.status.is_none(), "{name}");
        assert!(result.diagnostic.is_some(), "{name}");
    }
}
