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
    assert_eq!(
        *terminal.sessions.get(terminal.id).unwrap().state(),
        State::Closing
    );
    assert_eq!(terminal.sessions.ids().count(), 1);
    assert!(
        terminal
            .sessions
            .open(
                Launch {
                    program: "/bin/sh".into(),
                    args: vec![],
                    directory: terminal._fixture.0.clone(),
                },
                old
            )
            .is_err()
    );
    terminal.close();
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
        "stty -echo; printf READY; while read -r action; do case \"$action\" in finish) printf DONE; exit 0;; esac; done",
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
            .recv_timeout(Duration::from_millis(2100))
            .is_err()
    );
    let after = ticks(pid);
    assert_eq!(after, before);
    println!(
        "native shell idle_ticks={} observation_ms=2100",
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
    for signal in ["", "trap '' HUP;"] {
        for explicit in [true, false] {
            let mut terminal = Terminal::new(
                &format!(
                    "{signal} set -m; sleep 60 & printf '%s' $! > child.pid; printf READY; read -r action"
                ),
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
            let start = Instant::now();
            if explicit {
                terminal.sessions.close(terminal.id).unwrap();
                assert!(
                    start.elapsed() < Duration::from_millis(100),
                    "close blocked caller"
                );
                assert_eq!(
                    *terminal.sessions.get(terminal.id).unwrap().state(),
                    State::Closing
                );
                terminal.close();
            } else {
                // Exercise the registry destructor itself, without the shared
                // fixture completion helper taking explicit-close authority.
                let sessions = std::mem::take(&mut terminal.sessions);
                drop(sessions);
                assert!(
                    start.elapsed() < Duration::from_millis(100),
                    "Drop blocked caller"
                );
            }
            drop(terminal);
            wait_fds(baseline);
            assert!(start.elapsed() < Duration::from_secs(2), "close deadline");
            if !signal.is_empty() {
                assert!(start.elapsed() >= Duration::from_millis(500), "HUP grace");
            }
            assert!(!fs::exists(format!("/proc/{parent}")).unwrap());
            assert!(stopped(child));
            assert_eq!(fs::read_dir("/proc/self/fd").unwrap().count(), baseline);
        }
    }
}
pub(super) fn stopped(pid: u32) -> bool {
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
    victim.close();
    peer.send(b"ping\n");
    peer.wait(|s| text(s).contains("ALIVE"));
    assert_eq!(*peer.sessions.get(peer.id).unwrap().state(), State::Running);
    peer.close();
    drop(victim);
    drop(peer);
    wait_fds(baseline);
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
    let deadline = Instant::now() + Duration::from_secs(3);
    while sessions.ids().next().is_some() {
        sessions.service();
        assert!(Instant::now() < deadline, "aggregate cleanup deadline");
        std::thread::sleep(Duration::from_millis(1));
    }
}

// One flooded PTY with a blocked input queue must not starve another PTY in
// the same registry. Kernel backpressure retains unread input; every fair turn
// remains below the shared service budget and the finite flood completes.
#[test]
fn flood_fairness() {
    let baseline = fs::read_dir("/proc/self/fd").unwrap().count();
    let limits = Limits {
        max_pending_input_bytes: 65536,
        scrollback_lines: 0,
        ..Default::default()
    };
    let mut terminal = Terminal::new(
        "stty raw -echo; printf READY; while [ ! -f start ]; do sleep 0.01; done; head -c 1048576 /dev/zero | tr '\\000' f; printf FLOOD_DONE; while :; do sleep 60; done",
        limits,
    );
    terminal.wait(|s| text(s).contains("READY"));
    let peer = terminal.sessions.open(Launch {
        program: "/bin/sh".into(),
        args: vec!["-c".into(), "stty raw -echo; printf PEER_READY; dd bs=1 count=1 >/dev/null 2>&1; printf PEER_ACK; read -r end".into()],
        directory: terminal._fixture.0.clone(),
    }, Geometry { columns: 80, rows: 24 }).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !text(terminal.sessions.get(peer).unwrap()).contains("PEER_READY") {
        terminal.sessions.service();
        if text(terminal.sessions.get(peer).unwrap()).contains("PEER_READY") {
            break;
        }
        assert!(Instant::now() < deadline);
        terminal
            .wake
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .expect("peer ready deadline");
    }
    terminal.send(&vec![b'x'; 65536]);
    assert!(terminal.sessions.queue(terminal.id, b"overflow").is_err());
    assert_eq!(
        terminal
            .sessions
            .get(terminal.id)
            .unwrap()
            .storage()
            .pending_input_bytes,
        65536
    );
    terminal.sessions.queue(peer, b"p").unwrap();
    terminal._fixture.write("start", "ready");
    let mut bytes = 0;
    loop {
        let work = terminal.sessions.service();
        assert!(work.bytes <= rev::term::SERVICE_BYTES);
        bytes += work.bytes;
        let victim = terminal.sessions.get(terminal.id).unwrap();
        assert_eq!(*victim.state(), State::Running);
        let storage = victim.storage();
        assert!(storage.pending_input_bytes <= 65536);
        assert!(storage.pending_output_bytes <= 262144);
        assert!(storage.extra_bytes <= rev::term::EXTRA_BYTES);
        if text(terminal.sessions.get(peer).unwrap()).contains("PEER_ACK")
            && text(victim).contains("FLOOD_DONE")
        {
            assert!(
                storage.pending_input_bytes > 0,
                "child unexpectedly drained blocked input"
            );
            break;
        }
        assert!(Instant::now() < deadline, "fair flood deadline");
        terminal
            .wake
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .expect("fair readiness deadline");
    }
    assert!(bytes >= 1048576, "finite flood did not complete");
    terminal.sessions.close(peer).unwrap();
    terminal.close();
    terminal.sessions.shutdown().unwrap();
    drop(terminal);
    wait_fds(baseline);
    assert_eq!(fs::read_dir("/proc/self/fd").unwrap().count(), baseline);
}

// An unterminated oversized control stays bounded during repeated valid and
// rejected resizes. Repeated closure still retires the same owned child and
// every descriptor without requiring a control terminator or an input drain.
#[test]
fn incomplete_close() {
    let baseline = fs::read_dir("/proc/self/fd").unwrap().count();
    let mut terminal = Terminal::new(
        "stty raw -echo; printf 'READY\\033]2;'; i=0; while [ $i -lt 5000 ]; do printf x; i=$((i+1)); done; while :; do sleep 60; done",
        Limits::default(),
    );
    terminal.wait(|s| s.diagnostic().is_some_and(|d| d.contains("Oversized")));
    let pid = terminal
        .sessions
        .get(terminal.id)
        .unwrap()
        .process_id()
        .unwrap();
    for step in 0..64 {
        let size = Geometry {
            columns: 80 + step % 2,
            rows: 24 + step % 3,
        };
        terminal.sessions.resize(terminal.id, size).unwrap();
        let storage = terminal.sessions.get(terminal.id).unwrap().storage();
        assert!(storage.reserved_cells <= rev::term::SESSION_CELLS);
        assert!(
            terminal
                .sessions
                .resize(
                    terminal.id,
                    Geometry {
                        columns: u16::MAX,
                        rows: u16::MAX
                    }
                )
                .is_err()
        );
        assert_eq!(terminal.sessions.get(terminal.id).unwrap().geometry(), size);
        assert_eq!(
            terminal.sessions.get(terminal.id).unwrap().process_id(),
            Some(pid)
        );
    }
    assert!(terminal.text().contains("READY"));
    assert!(!terminal.text().contains("xxxx"));
    assert_eq!(terminal.sessions.get(terminal.id).unwrap().title(), None);
    let start = Instant::now();
    terminal.sessions.close(terminal.id).unwrap();
    terminal.sessions.close(terminal.id).unwrap();
    terminal.close();
    assert!(start.elapsed() < Duration::from_secs(2));
    assert!(!fs::exists(format!("/proc/{pid}")).unwrap());
    drop(terminal);
    wait_fds(baseline);
    assert_eq!(fs::read_dir("/proc/self/fd").unwrap().count(), baseline);
}
