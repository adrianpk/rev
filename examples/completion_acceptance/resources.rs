use super::support::*;
use crossterm::event::KeyCode;
use rev::buffer::Buffers;
use rev::command::{Command, CompletionCommand as C};
use rev::config::Snapshot;
use rev::session::Session;
use rev::view::View;
use std::path::Path;
use std::time::{Duration, Instant};

pub(super) fn run(root: &Path, binary: &Path) -> Result<()> {
    inventory(root)?;
    limits(root)?;
    burst(root)?;
    let bytes = std::fs::metadata(binary)?.len();
    let baseline = 13_326_584_u64;
    println!(
        "MEASURE editor executable: bytes={bytes} baseline_bytes={baseline} delta_bytes={} ratio={:.6}; baseline uses the recorded normal release profile",
        bytes as i64 - baseline as i64,
        bytes as f64 / baseline as f64
    );
    println!("MEASURE native acceptance peak_rss={}", peak_rss("self")?);
    Ok(())
}

fn inventory(root: &Path) -> Result<()> {
    let mut session = inventory_session(root)?;
    let original = "fn main() { needle }\n";
    let started = Instant::now();
    complete(&mut session)?;
    let candidates = session
        .completion()
        .ok_or("missing bounded inventory")?
        .candidates();
    assert_eq!(
        candidates.len(),
        31,
        "current buffer plus at most 31 other observations"
    );
    assert!(
        candidates
            .iter()
            .any(|candidate| candidate.text() == "needle_39")
    );
    assert!(
        !candidates
            .iter()
            .any(|candidate| candidate.text() == "needle_00")
    );
    assert!(session.echo().contains("limited"));
    assert_eq!(source(&session), original);
    assert!(session.buffer()?.history().is_empty());
    println!(
        "MEASURE bounded observations: 40 open peers, 31 retained names, old peer evicted, request_us={}",
        started.elapsed().as_micros()
    );
    Ok(())
}

fn inventory_session(root: &Path) -> Result<Session> {
    let settings = Snapshot::default();
    let mut buffers = Buffers::with_settings(settings)?;
    for index in 0..40 {
        let path = root.join("workspace").join(format!("open-{index:02}.rs"));
        std::fs::write(&path, format!("fn needle_{index:02}() {{}}\n"))?;
        buffers.visit(&path)?;
    }
    let path = root.join("workspace/current.rs");
    let original = "fn main() { needle }\n";
    std::fs::write(&path, original)?;
    let id = buffers.visit(&path)?;
    let mut view = View::new(buffers.get(id)?);
    view.point.position.0 = original.find("needle").unwrap() + "needle".len();
    Ok(Session::new(buffers, view, SIZE)?)
}

fn limits(root: &Path) -> Result<()> {
    let oversized = format!("//{}\n|", "x".repeat(2 * 1024 * 1024));
    let mut session = open(root, "oversized.rs", &oversized, Snapshot::default())?;
    let stamp = session.buffer()?.stamp();
    complete(&mut session)?;
    assert!(session.completion().is_none());
    assert!(session.echo().contains("2 MiB"));
    assert_eq!(session.buffer()?.stamp(), stamp);
    assert!(session.buffer()?.history().is_empty());
    let mut symbols = String::new();
    for index in 0..4097 {
        symbols.push_str(&format!("fn symbol_{index:04}() {{}}\n"));
    }
    symbols.push_str("fn main() { symbol| }\n");
    let mut session = open(root, "symbols.rs", &symbols, Snapshot::default())?;
    let original = source(&session);
    complete(&mut session)?;
    assert!(session.completion().is_none());
    assert!(session.echo().contains("limit"));
    assert_eq!(source(&session), original);
    assert!(session.buffer()?.history().is_empty());
    println!(
        "PASS source admission: oversized >2 MiB and >4096 symbols report limits, preserve source/history and publish no partial popup"
    );
    Ok(())
}

fn burst(root: &Path) -> Result<()> {
    let mut document = String::new();
    for index in 0..1024 {
        document.push_str(&format!("fn burst_{index:04}() {{}}\n"));
    }
    document.push_str("fn main() { bur| }\n");
    let mut session = open(root, "burst.rs", &document, Snapshot::default())?;
    let original = source(&session);
    let started = Instant::now();
    let mut longest = Duration::ZERO;
    for code in std::iter::repeat_n(KeyCode::Char('x'), 64)
        .chain(std::iter::repeat_n(KeyCode::Backspace, 64))
    {
        let step = Instant::now();
        key(&mut session, code);
        session.poll_frame()?;
        longest = longest.max(step.elapsed());
    }
    settle(&mut session)?;
    assert_eq!(source(&session), original);
    session.execute(Command::Completion(C::Cancel));
    settle(&mut session)?;
    idle(&mut session);
    let burst = started.elapsed();
    let cpu = cpu_ticks("self")?;
    std::thread::sleep(Duration::from_millis(350));
    let idle_ticks = cpu_ticks("self")?.saturating_sub(cpu);
    assert!(
        idle_ticks <= 1,
        "idle native session consumed {idle_ticks} CPU ticks"
    );
    idle(&mut session);
    println!(
        "MEASURE burst: 1024 declarations, 128 edit/reversal events, total_us={} longest_event_frame_us={} idle_350ms_cpu_ticks={idle_ticks}; source exact and no idle deadline",
        burst.as_micros(),
        longest.as_micros()
    );
    Ok(())
}
