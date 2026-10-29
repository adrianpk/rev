use super::*;
use crate::fixture::Fixture;
use std::path::Path;
fn program(directory: &Path, script: &str, limit: usize) -> Program {
    Program {
        argv: vec!["/bin/sh".into(), "-c".into(), script.into()],
        directory: directory.into(),
        seconds: 1,
        output_bytes: limit,
    }
}
fn stopped(pid: &str) -> bool {
    let stat = std::fs::read_to_string(format!("/proc/{}/stat", pid.trim()));
    stat.is_err() || stat.unwrap().split_once(") ").unwrap().1.starts_with('Z')
}
// Real owned programs exercise the exact combined byte boundary, status and
// launch errors. Neither a successful exit nor truncated output hides failure.
#[test]
fn process_output() {
    let f = Fixture::new();
    let cancel = Cancellation::default();
    let out = program(&f.0, "printf abc; printf def >&2; exit 7", 6)
        .run(&cancel)
        .unwrap();
    assert_eq!(out.status.code(), Some(7));
    assert_eq!(out.stdout, b"abc");
    assert_eq!(out.stderr, b"def");
    for (name, script) in [
        ("combined", "printf abc; printf def >&2"),
        ("stdout", "printf abcdef"),
        ("stderr", "printf abcdef >&2"),
    ] {
        let error = program(&f.0, script, 5).run(&cancel).err().unwrap();
        assert!(error.contains("output limit"), "{name}: {error}");
    }
    let mut missing = program(&f.0, "", 6);
    missing.argv = vec![f.0.join("missing").to_str().unwrap().into()];
    assert!(
        missing
            .run(&cancel)
            .err()
            .unwrap()
            .contains("Start configured program")
    );
}
// Cancelling or timing out kills the whole owned group, drains bounded streams
// and reaps its direct child, even while a descendant holds both streams open.
#[test]
fn process_lifetime() {
    for (name, cancelled) in [("cancel", true), ("timeout", false)] {
        let f = Fixture::new();
        let cancel = Cancellation::default();
        let control = cancel.clone();
        let p = program(
            &f.0,
            "echo $$ > leader; sleep 30 & echo $! > descendant; wait",
            1024,
        );
        let path = f.0.clone();
        let job = std::thread::spawn(move || p.run(&control));
        let end = Instant::now() + Duration::from_secs(3);
        while !path.join("descendant").exists() {
            assert!(Instant::now() < end, "{name}: fixture did not start");
            std::thread::sleep(Duration::from_millis(2));
        }
        if cancelled {
            cancel.cancel();
        }
        let error = job.join().unwrap().err().unwrap();
        assert!(
            error.contains(if cancelled { "cancelled" } else { "timed out" }),
            "{error}"
        );
        assert!(
            Instant::now() < end,
            "{name}: cleanup exceeded fixture deadline"
        );
        let leader = std::fs::read_to_string(path.join("leader")).unwrap();
        assert!(
            !PathBuf::from(format!("/proc/{}", leader.trim())).exists(),
            "direct child was not reaped"
        );
        assert!(
            stopped(&std::fs::read_to_string(path.join("descendant")).unwrap()),
            "{name}: descendant still running"
        );
    }
}
// A leader's exit does not release lifetime ownership while a background child
// holds output open. Drop also cancels and reaps a launched direct child.
#[test]
fn process_descendants() {
    let f = Fixture::new();
    let p = program(&f.0, "sleep 30 & echo $! > descendant; printf done", 1024);
    let start = Instant::now();
    let out = p.run(&Cancellation::default()).unwrap();
    assert!(out.status.success());
    assert_eq!(out.stdout, b"done");
    assert!(start.elapsed() < Duration::from_secs(1));
    assert!(stopped(
        &std::fs::read_to_string(f.0.join("descendant")).unwrap()
    ));
    let owned = program(&f.0, "sleep 30", 1024).launch().unwrap();
    let pid = owned.pid;
    drop(owned);
    assert!(!PathBuf::from(format!("/proc/{pid}")).exists());
}
