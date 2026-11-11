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
        git_index: None,
        workspace: None,
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

// An exited leader still owns descendants, even when they redirect both
// streams. Completion reaps the leader and observes descendant shutdown.
#[test]
fn exited_descendants() {
    for script in [
        "echo $$ > leader; sleep 30 & echo $! > descendant; printf done",
        "echo $$ > leader; sleep 30 >/dev/null 2>&1 & echo $! > descendant; printf done",
    ] {
        let fixture = Fixture::new();
        let start = Instant::now();
        let result = program(&fixture, script, 4096).run(&Cancellation::default(), |_, _| {});
        assert_eq!(result.end, End::Exited, "{:?}", result.diagnostic);
        assert!(result.status.unwrap().success());
        assert!(result.retained.is_none(), "{:?}", result.diagnostic);
        assert_eq!(result.stdout, b"done");
        assert!(start.elapsed() < Duration::from_secs(4));
        let leader = std::fs::read_to_string(fixture.0.join("leader")).unwrap();
        assert!(!PathBuf::from(format!("/proc/{}", leader.trim())).exists());
        let descendant = std::fs::read_to_string(fixture.0.join("descendant")).unwrap();
        let stat = std::fs::read_to_string(format!("/proc/{}/stat", descendant.trim()));
        assert!(stat.is_err() || stat.unwrap().split_once(") ").unwrap().1.starts_with('Z'));
    }
}

// A real group ignoring TERM must still receive KILL within the same cleanup
// budget, with the leader reaped and its descendant stopped before return.
#[test]
fn term_escalation() {
    use std::os::unix::process::ExitStatusExt;
    let fixture = Fixture::new();
    let mut program = program(
        &fixture,
        "trap '' TERM; echo $$ > leader; sleep 30 & echo $! > descendant; printf before; wait",
        4096,
    );
    let mut owned = program.launch(None, &Cancellation::default()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(4);
    while !fixture.0.join("descendant").exists() {
        assert!(Instant::now() < deadline, "TERM fixture did not start");
        std::thread::sleep(Duration::from_millis(2));
    }
    let leader = owned.child_id();
    let status = owned.stop(false).unwrap();
    assert_eq!(status.signal(), Some(9));
    owned.drain(4096).unwrap();
    assert_eq!(owned.streams.take(), (b"before".to_vec(), Vec::new()));
    assert!(Instant::now() < deadline);
    assert!(!PathBuf::from(format!("/proc/{leader}")).exists());
    let descendant = std::fs::read_to_string(fixture.0.join("descendant")).unwrap();
    let stat = std::fs::read_to_string(format!("/proc/{}/stat", descendant.trim()));
    assert!(stat.is_err() || stat.unwrap().split_once(") ").unwrap().1.starts_with('Z'));
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

// The same process owner keeps a private producer directory alive during a real
// child, clears its index override afterward and cleans only after cancellation.
#[test]
fn private_workspace() {
    let fixture = Fixture::new();
    let workspace = std::sync::Arc::new(
        crate::file::temporary::Directory::new(&fixture.0, vec!["index".into()]).unwrap(),
    );
    let root = std::fs::canonicalize(workspace.anchor()).unwrap();
    let mut owned = program(
        &fixture,
        "printf owned > \"$GIT_INDEX_FILE\"; printf before; sleep 30",
        4096,
    );
    owned.git_index = Some(workspace.entry("index").unwrap());
    owned.workspace = Some(workspace);
    let control = Cancellation::default();
    let cancel = control.clone();
    let worker = std::thread::spawn(move || owned.run(&control, |_, _| {}));
    let deadline = Instant::now() + Duration::from_secs(4);
    while !std::fs::read(root.join("index")).is_ok_and(|bytes| bytes == b"owned") {
        assert!(
            Instant::now() < deadline,
            "workspace producer did not start"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(std::fs::read(root.join("index")).unwrap(), b"owned");
    cancel.cancel();
    let result = worker.join().unwrap();
    assert_eq!(result.end, End::Cancelled);
    assert!(result.retained.is_none());
    assert!(Instant::now() < deadline);
    assert!(!root.exists());
    let output = program(&fixture, "printf '%s' \"${GIT_INDEX_FILE-unset}\"", 4096)
        .run(&Cancellation::default(), |_, _| {});
    assert_eq!(output.stdout, b"unset");
}
