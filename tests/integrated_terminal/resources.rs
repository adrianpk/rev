use super::*;
use rev::term::State;
use std::fs;

// Rejection is transactional: capacity and old geometry/session remain valid,
// and an input sequence is either fully queued or wholly rejected.
#[test]
fn atomic_admission() {
    let limits = Limits {
        max_sessions: 1,
        max_pending_input_bytes: 4096,
        ..Default::default()
    };
    let mut terminal = Terminal::new("stty -echo; printf READY; read -r action", limits);
    terminal.wait(|session| text(session).contains("READY"));
    let launch = Launch {
        program: "/bin/sh".into(),
        args: vec![],
        directory: terminal._fixture.0.clone(),
    };
    assert!(
        terminal
            .sessions
            .open(
                launch,
                Geometry {
                    columns: 80,
                    rows: 24
                }
            )
            .is_err()
    );
    let old = terminal.sessions.get(terminal.id).unwrap().geometry();
    for size in [
        Geometry {
            columns: 0,
            rows: 24,
        },
        Geometry {
            columns: 2,
            rows: 0,
        },
        Geometry {
            columns: u16::MAX,
            rows: u16::MAX,
        },
    ] {
        assert!(terminal.sessions.resize(terminal.id, size).is_err());
        assert_eq!(terminal.sessions.get(terminal.id).unwrap().geometry(), old);
    }
    terminal.send(&vec![b'x'; 4096]);
    assert!(terminal.sessions.queue(terminal.id, b"rejected").is_err());
    assert_eq!(
        terminal
            .sessions
            .get(terminal.id)
            .unwrap()
            .storage()
            .pending_input_bytes,
        4096
    );
    terminal.sessions.close(terminal.id).unwrap();
    assert!(terminal.sessions.get(terminal.id).is_none());
    assert!(terminal.sessions.queue(terminal.id, b"stale").is_err());
}

// Failed candidates never publish identities, spawn a shell, or retain PTY
// descriptors. Existing process and display state survive each launch failure.
#[test]
fn rejected_launches() {
    let fixture = Fixture::new();
    let mut sessions = Sessions::default();
    let baseline = fs::read_dir("/proc/self/fd").unwrap().count();
    let invalid = fixture.write(
        "invalid-executable",
        "#!/missing-terminal-interpreter\nexit 0\n",
    );
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&invalid, fs::Permissions::from_mode(0o755)).unwrap();
    let cases = [
        (
            "missing script interpreter",
            invalid.to_string_lossy().into_owned(),
            vec![],
            fixture.0.clone(),
        ),
        (
            "missing program",
            "/missing-terminal-program".to_owned(),
            vec![],
            fixture.0.clone(),
        ),
        (
            "bad directory",
            "/bin/sh".to_owned(),
            vec![],
            fixture.0.join("missing"),
        ),
        (
            "argument count",
            "/bin/sh".to_owned(),
            vec!["x".into(); 33],
            fixture.0.clone(),
        ),
        (
            "argument bytes",
            "/bin/sh".to_owned(),
            vec!["x".repeat(16385)],
            fixture.0.clone(),
        ),
        (
            "NUL argument",
            "/bin/sh".to_owned(),
            vec!["\0".into()],
            fixture.0.clone(),
        ),
    ];
    for (name, program, args, directory) in cases {
        assert!(
            sessions
                .open(
                    Launch {
                        program,
                        args,
                        directory
                    },
                    Geometry {
                        columns: 80,
                        rows: 24
                    }
                )
                .is_err(),
            "{name}"
        );
        assert_eq!(sessions.ids().count(), 0, "{name}");
        assert_eq!(
            fs::read_dir("/proc/self/fd").unwrap().count(),
            baseline,
            "{name}"
        );
    }
}

// Idle readiness parks in poll. The native shell and notifier consume no CPU
// ticks during a bounded receive timeout, and queueing input wakes without a timer.
#[test]
fn idle_readiness() {
    let mut terminal = Terminal::new(
        "stty -echo; printf READY; read -r action; printf DONE",
        Limits::default(),
    );
    terminal.wait(|session| text(session).contains("READY"));
    terminal.sessions.service();
    while terminal.wake.try_recv().is_ok() {}
    let pid = terminal
        .sessions
        .get(terminal.id)
        .unwrap()
        .process_id()
        .unwrap();
    let before = ticks(pid);
    assert!(
        terminal
            .wake
            .recv_timeout(Duration::from_millis(100))
            .is_err()
    );
    let after = ticks(pid);
    assert_eq!(after, before);
    println!(
        "native shell idle_ticks={} observation_ms=100",
        after - before
    );
    terminal.send(b"finish\n");
    terminal.wait(|session| matches!(session.state(), State::Exited(_)));
    assert!(terminal.text().contains("DONE"));
}
fn ticks(pid: u32) -> u64 {
    let text = fs::read_to_string(format!("/proc/{pid}/stat")).unwrap();
    text.rsplit_once(") ")
        .unwrap()
        .1
        .split_whitespace()
        .skip(11)
        .take(2)
        .map(|value| value.parse::<u64>().unwrap())
        .sum()
}

// Owned closure kills shell job-control groups and reaps the leader. It also
// releases notification threads and descriptors on explicit close and Drop.
#[test]
fn process_cleanup() {
    let baseline = fs::read_dir("/proc/self/fd").unwrap().count();
    for explicit in [true, false] {
        let mut terminal = Terminal::new(
            "set -m; sleep 60 & printf '%s' $! > child.pid; printf READY; read -r action",
            Limits::default(),
        );
        terminal.wait(|session| text(session).contains("READY"));
        let parent = terminal
            .sessions
            .get(terminal.id)
            .unwrap()
            .process_id()
            .unwrap();
        let child = fs::read_to_string(terminal._fixture.0.join("child.pid"))
            .unwrap()
            .parse::<u32>()
            .unwrap();
        if explicit {
            terminal.sessions.close(terminal.id).unwrap();
        }
        drop(terminal);
        assert!(!fs::exists(format!("/proc/{parent}")).unwrap());
        assert!(stopped(child));
        assert_eq!(fs::read_dir("/proc/self/fd").unwrap().count(), baseline);
    }
}
fn stopped(pid: u32) -> bool {
    fs::read_to_string(format!("/proc/{pid}/stat")).map_or(true, |text| {
        text.rsplit_once(") ").unwrap().1.starts_with('Z')
    })
}

// Session ownership must retain every owned job-control group while
// leaving another live session usable. Both PTYs release their descriptors.
#[test]
fn session_isolation() {
    let baseline = fs::read_dir("/proc/self/fd").unwrap().count();
    let mut peer = Terminal::new(
        "stty -echo; printf PEER; read -r line; printf ALIVE; read -r end",
        Limits::default(),
    );
    peer.wait(|s| text(s).contains("PEER"));
    let mut victim = Terminal::new("stty -echo; printf READY; read -r line", Limits::default());
    victim.wait(|s| text(s).contains("READY"));
    victim.sessions.close(victim.id).unwrap();
    peer.send(b"ping\n");
    peer.wait(|s| text(s).contains("ALIVE"));
    assert_eq!(*peer.sessions.get(peer.id).unwrap().state(), State::Running);
    peer.sessions.close(peer.id).unwrap();
    drop(victim);
    drop(peer);
    assert_eq!(fs::read_dir("/proc/self/fd").unwrap().count(), baseline);
}

// The instance cell ledger includes both grids and upstream row caches. A
// rejected third session cannot consume a descriptor or disturb either owner.
#[test]
fn aggregate_cells() {
    let fixture = Fixture::new();
    let mut sessions = Sessions::new(
        Limits {
            scrollback_lines: 0,
            ..Default::default()
        },
        Arc::new(|| {}),
    )
    .unwrap();
    let launch = Launch {
        program: "/bin/sh".into(),
        args: vec!["-c".into(), "read -r action".into()],
        directory: fixture.0.clone(),
    };
    let size = Geometry {
        columns: 400,
        rows: 24,
    };
    let first = sessions.open(launch.clone(), size).unwrap();
    let second = sessions.open(launch.clone(), size).unwrap();
    let before = fs::read_dir("/proc/self/fd").unwrap().count();
    assert!(sessions.open(launch, size).is_err());
    assert_eq!(fs::read_dir("/proc/self/fd").unwrap().count(), before);
    assert_eq!(sessions.ids().collect::<Vec<_>>(), vec![first, second]);
    assert!(
        sessions
            .ids()
            .map(|id| sessions.get(id).unwrap().storage().reserved_cells)
            .sum::<usize>()
            <= rev::term::EDITOR_CELLS
    );
    sessions.close(first).unwrap();
    sessions.close(second).unwrap();
}
