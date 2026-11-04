use super::{metrics::threads, support::*};
use alacritty_terminal::grid::Dimensions;
use rev::term::{Geometry, Launch, Limits, SERVICE_BYTES, SessionId, Sessions, State};
use std::{
    fs,
    path::Path,
    sync::{
        Arc,
        mpsc::{self, Receiver, RecvTimeoutError},
    },
    time::{Duration, Instant},
};

struct Owned {
    sessions: Sessions,
    victim: SessionId,
    peer: SessionId,
    wake: Receiver<()>,
    bytes: usize,
    turns: usize,
}
impl Drop for Owned {
    fn drop(&mut self) {
        self.sessions.shutdown().expect("owned resource cleanup");
    }
}
impl Owned {
    fn tick(&mut self) {
        let work = self.sessions.service();
        assert!(work.bytes <= SERVICE_BYTES);
        self.bytes += work.bytes;
        self.turns += 1;
        for id in self.sessions.ids() {
            let s = self.sessions.get(id).expect("owned session");
            assert!(!matches!(s.state(), State::Failed(_)), "{:?}", s.state());
            assert!(s.storage().extra_bytes <= rev::term::EXTRA_BYTES);
            assert!(s.screen().grid().history_size() <= 64);
        }
    }
    fn wait(&mut self, ready: impl Fn(&Self) -> bool) -> Result<()> {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            self.tick();
            if ready(self) {
                return Ok(());
            }
            assert!(Instant::now() < deadline, "owned resource deadline");
            self.wake
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))?;
        }
    }
    fn text(&self, id: SessionId) -> String {
        screen(self.sessions.get(id).expect("owned session"))
    }
    fn pid(&self, id: SessionId) -> u32 {
        self.sessions
            .get(id)
            .expect("owned session")
            .process_id()
            .expect("owned PID")
    }
}
// Measure real bounded IO rather than a synthetic transcript or assumed rate.
// Exact FD/thread retirement remains after both workers have completed.
pub(crate) fn run(root: &Path, binary: &Path) -> Result<()> {
    let baseline = fds()?;
    let workers = threads()?;
    let mut owned = prepare(root)?;
    idle(&mut owned)?;
    flood(&mut owned)?;
    close(&mut owned)?;
    drop(owned);
    retired(baseline, workers)?;
    binary_size(binary)
}
fn prepare(root: &Path) -> Result<Owned> {
    let directory = root.join("resources");
    fs::create_dir_all(&directory)?;
    let (send, wake) = mpsc::sync_channel(1);
    let mut sessions = Sessions::new(
        Limits {
            scrollback_lines: 64,
            max_pending_input_bytes: 65536,
            ..Default::default()
        },
        Arc::new(move || {
            let _ = send.try_send(());
        }),
    )?;
    let victim = launch(
        &mut sessions,
        &directory,
        "stty -echo; printf VICTIM_READY; read -r action; stty raw -echo; printf FLOOD_READY; head -c 1048576 /dev/zero | tr '\\000' x; printf '\\r\\nFLOOD_DONE'; exec sleep 60",
    )?;
    let peer = launch(
        &mut sessions,
        &directory,
        "stty -echo; printf PEER_READY; read -r action; printf PEER_ACK; read -r end",
    )?;
    let mut owned = Owned {
        sessions,
        victim,
        peer,
        wake,
        bytes: 0,
        turns: 0,
    };
    owned.wait(|o| {
        o.text(o.victim).contains("VICTIM_READY") && o.text(o.peer).contains("PEER_READY")
    })?;
    Ok(owned)
}
fn launch(sessions: &mut Sessions, directory: &Path, script: &str) -> Result<SessionId> {
    Ok(sessions.open(
        Launch {
            program: "/bin/sh".into(),
            args: vec!["-c".into(), script.into()],
            directory: directory.to_owned(),
        },
        Geometry {
            columns: 80,
            rows: 24,
        },
    )?)
}
fn ticks(owned: &Owned) -> Result<u64> {
    Ok(cpu_ticks("self")?
        + cpu_ticks(&owned.pid(owned.victim).to_string())?
        + cpu_ticks(&owned.pid(owned.peer).to_string())?)
}
fn idle(owned: &mut Owned) -> Result<()> {
    owned.tick();
    while owned.wake.try_recv().is_ok() {}
    let before = ticks(owned)?;
    assert!(
        matches!(
            owned.wake.recv_timeout(Duration::from_millis(350)),
            Err(RecvTimeoutError::Timeout)
        ),
        "idle terminal woke without IO"
    );
    let cpu = ticks(owned)?.saturating_sub(before);
    assert!(cpu <= 1, "idle owner/shells consumed {cpu} ticks");
    println!(
        "MEASURE resources idle: interval_ms=350 owner_plus_shell_cpu_ticks={cpu} peak_rss={}",
        peak_rss("self")?
    );
    Ok(())
}
fn flood(owned: &mut Owned) -> Result<()> {
    owned.sessions.queue(owned.victim, b"start\n")?;
    owned.wait(|o| o.text(o.victim).contains("FLOOD_READY"))?;
    owned.sessions.queue(owned.victim, &vec![b'x'; 65536])?;
    assert!(owned.sessions.queue(owned.victim, b"overflow").is_err());
    assert_eq!(
        owned
            .sessions
            .get(owned.victim)
            .unwrap()
            .storage()
            .pending_input_bytes,
        65536
    );
    let before = ticks(owned)?;
    let started = Instant::now();
    owned.sessions.queue(owned.peer, b"ping\n")?;
    owned.wait(|o| o.text(o.peer).contains("PEER_ACK"))?;
    let peer = started.elapsed();
    owned.wait(|o| o.text(o.victim).contains("FLOOD_DONE"))?;
    active(owned, started, peer, before)
}
fn active(owned: &Owned, started: Instant, peer: Duration, before: u64) -> Result<()> {
    let cpu = ticks(owned)?.saturating_sub(before);
    let s = owned.sessions.get(owned.victim).unwrap();
    let storage = s.storage();
    assert!(storage.pending_input_bytes > 0);
    assert!(storage.pending_input_bytes <= 65536);
    assert!(storage.pending_output_bytes <= 262144);
    assert!(owned.bytes >= 1048576);
    println!(
        "MEASURE active: flood_bytes=1048576 service_bytes={} turns={} elapsed_us={} peer_response_us={} owner_plus_shell_cpu_ticks={cpu} retained_rows={} reserved_cells={} extra_bytes={} queued_input_bytes={} pending_output_bytes={} peak_rss={}",
        owned.bytes,
        owned.turns,
        started.elapsed().as_micros(),
        peer.as_micros(),
        s.screen().grid().history_size(),
        storage.reserved_cells,
        storage.extra_bytes,
        storage.pending_input_bytes,
        storage.pending_output_bytes,
        peak_rss("self")?
    );
    Ok(())
}
fn close(owned: &mut Owned) -> Result<()> {
    let pids = [owned.pid(owned.victim), owned.pid(owned.peer)];
    let start = Instant::now();
    owned.sessions.close(owned.victim)?;
    owned.sessions.close(owned.peer)?;
    while owned.sessions.ids().next().is_some() {
        owned.tick();
        if owned.sessions.ids().next().is_none() {
            break;
        }
        assert!(start.elapsed() < Duration::from_secs(2));
        owned
            .wake
            .recv_timeout(Duration::from_secs(2).saturating_sub(start.elapsed()))?;
    }
    for pid in pids {
        reaped(pid)?;
    }
    println!(
        "MEASURE owned cleanup: elapsed_us={} leaders_reaped=2",
        start.elapsed().as_micros()
    );
    Ok(())
}
fn fds() -> Result<usize> {
    Ok(fs::read_dir("/proc/self/fd")?.count())
}
fn retired(baseline: usize, workers: usize) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(3);
    while fds()? != baseline {
        assert!(Instant::now() < deadline, "descriptor retirement deadline");
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(fds()?, baseline);
    assert_eq!(threads()?, workers);
    println!(
        "MEASURE ownership: fds_before={baseline} fds_after={} threads_before={workers} threads_after={}",
        fds()?,
        threads()?
    );
    Ok(())
}
fn binary_size(binary: &Path) -> Result<()> {
    let bytes = fs::metadata(binary)?.len();
    let baseline = 21576728_u64;
    println!(
        "MEASURE release binary: bytes={bytes} recorded_pre_terminal_baseline_bytes={baseline} baseline_commit=507862798782d15235f3758868ab33cd3611c57e delta_bytes={} ratio={:.6}; baseline is the Slice 1 normal-release observation, not a fresh rebuild",
        bytes as i64 - baseline as i64,
        bytes as f64 / baseline as f64
    );
    Ok(())
}
