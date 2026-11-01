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
    let owned = program(&f.0, "sleep 30", 1024).launch(None, None).unwrap();
    let pid = owned.pid;
    drop(owned);
    assert!(!PathBuf::from(format!("/proc/{pid}")).exists());
}

// Observe real EOF before cancelling an owned launch, then use the production
// collection/cleanup path. This orders cancellation before its collection
// deadline without racing a test worker's readiness polling against that timer.
// Redirected descendants must already be dead when the completed run returns.
#[test]
fn process_closed_output() {
    for attempt in 0..32 {
        let f = Fixture::new();
        let cancel = Cancellation::default();
        let p = program(
            &f.0,
            "exec >/dev/null 2>&1; i=0; while test $i -lt 8; do sleep 30 & echo $! >> descendants; i=$((i+1)); done; : > ready; wait",
            1024,
        );
        let path = f.0.clone();
        let start = Instant::now();
        let mut owned = p.launch(None, None).unwrap();
        let leader = owned.pid;
        while !path.join("ready").exists() || !owned.streams.closed() {
            owned.streams.read(p.output_bytes).unwrap();
            assert!(
                start.elapsed() < Duration::from_secs(1),
                "fixture did not start"
            );
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(owned.child.as_mut().unwrap().try_wait().unwrap().is_none());
        cancel.cancel();
        let error = owned.run(&p, &cancel).err().unwrap();
        assert!(error.contains("cancelled"), "attempt {attempt}: {error}");
        assert!(start.elapsed() < Duration::from_secs(1));
        assert!(
            !PathBuf::from(format!("/proc/{leader}")).exists(),
            "attempt {attempt}: direct child was not reaped"
        );
        let pids = std::fs::read_to_string(path.join("descendants")).unwrap();
        assert_eq!(pids.lines().count(), 8);
        for pid in pids.lines() {
            assert!(pid.parse::<u32>().is_ok());
            assert!(
                stopped(pid),
                "attempt {attempt}: descendant {pid} still running"
            );
        }
    }
}

// Closed output does not establish shutdown. Observing a live owned group must
// time out without signalling it; stop reaps that group and leaves another
// owned group alive. Real executable names cover comm delimiters and UTF-8
// truncation without interpreting the name bytes as process-state fields.
#[test]
fn process_observation() {
    let f = Fixture::new();
    let mut unrelated = program(&f.0, "sleep 30", 1024).launch(None, None).unwrap();
    for name in ["plain", "sleep ) name", "abcdefghijklmn\u{9149}"] {
        let executable = f.0.join(name);
        std::fs::copy("/bin/sleep", &executable).unwrap();
        let mut p = program(&f.0, "", 1024);
        p.argv = vec![
            "/bin/sh".into(),
            "-c".into(),
            "exec >/dev/null 2>&1; exec \"$1\" 30".into(),
            "fixture".into(),
            executable.to_str().unwrap().into(),
        ];
        let mut owned = p.launch(None, None).unwrap();
        let start = Instant::now();
        while !owned.streams.closed() {
            owned.streams.read(1024).unwrap();
            assert!(start.elapsed() < Duration::from_secs(1));
            std::thread::sleep(Duration::from_millis(2));
        }
        let error = owned
            .wait_group(Instant::now() + Duration::from_millis(20))
            .unwrap_err();
        assert!(error.contains("shutdown timed out"), "{name}: {error}");
        assert!(owned.child.as_mut().unwrap().try_wait().unwrap().is_none());
        let pid = owned.pid;
        owned.stop().unwrap();
        assert!(!PathBuf::from(format!("/proc/{pid}")).exists());
        assert!(
            unrelated
                .child
                .as_mut()
                .unwrap()
                .try_wait()
                .unwrap()
                .is_none()
        );
    }
}
