use std::path::Path;
use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode};
use rev::bookmark::Resolution;
use rev::command::{BookmarkCommand as B, Command as C, Motion, Scroll};
use rev::coordinates::Scalar;
use rev::session::Session;

use super::support::*;

fn step(session: &mut Session, max_step: &mut Duration, operation: impl FnOnce(&mut Session)) {
    let started = Instant::now();
    operation(session);
    *max_step = (*max_step).max(started.elapsed());
}

// Reuse the native cooperative projection owner and its existing 512-batch
// fixture bound. Each observation records the longest individual event pass.
fn render(session: &mut Session, max_step: &mut Duration) -> Result<()> {
    for _ in 0..512 {
        let started = Instant::now();
        let ready = session.poll_frame()?.cursor().is_some();
        *max_step = (*max_step).max(started.elapsed());
        if ready && session.remaining_input(Instant::now()).is_none() {
            return Ok(());
        }
        step(session, max_step, |s| {
            s.expire_input(Instant::now());
        });
    }
    Err("native projection exceeded 512 batches".into())
}

fn sample(
    session: &mut Session,
    max_step: &mut Duration,
    operation: impl FnOnce(&mut Session),
) -> Result<Duration> {
    let started = Instant::now();
    step(session, max_step, operation);
    render(session, max_step)?;
    Ok(started.elapsed())
}

pub(super) fn run(root: &Path) -> Result<()> {
    let path = root.join("workspace/large.txt");
    let mut original = "one two three four ".repeat(4096) + "\n";
    for row in 0..40_000 {
        original.push_str(&format!(
            "line-{row:05} unique source words for checked bookmark context\n"
        ));
    }
    std::fs::write(&path, &original)?;
    let config = root.join("resource-config.toml");
    std::fs::write(
        &config,
        format!(
            "[bookmarks]\nfile='{}'\nsave='manual'\n",
            root.join("resource-data/bookmarks.toml").display()
        ),
    )?;
    let startup = rev::config::prepare(&config);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut session = configured(&startup.snapshot, Some(&path))?;
    let started = Instant::now();
    for row in 0..1024 {
        goto(&mut session, row + 2);
        bookmark(&mut session, B::Set, &format!("record-{row:04}"));
        assert_eq!(session.bookmarks().len(), row + 1);
    }
    let admission = started.elapsed();
    let target = session
        .bookmarks()
        .get("record-0512")
        .unwrap()
        .point()
        .position;
    bookmark(&mut session, B::Set, "overflow");
    assert!(session.echo().contains("collection is full"));
    session.execute(C::KeyboardQuit);
    assert_eq!(session.bookmarks().len(), 1024);
    let mut max_step = Duration::ZERO;
    let started = Instant::now();
    render(&mut session, &mut max_step)?;
    let cold = started.elapsed();
    let mut warm = Duration::ZERO;
    for _ in 0..16 {
        let started = Instant::now();
        render(&mut session, &mut max_step)?;
        warm = warm.max(started.elapsed());
    }
    let mut movement = Duration::ZERO;
    for command in [Scroll::PreviousLine, Scroll::NextLine]
        .into_iter()
        .cycle()
        .take(16)
    {
        movement = movement.max(sample(&mut session, &mut max_step, |s| {
            s.execute(C::Scroll(command));
        })?);
    }
    let started = Instant::now();
    for motion in [
        Motion::BufferEnd,
        Motion::BufferStart,
        Motion::BufferEnd,
        Motion::BufferStart,
    ] {
        step(&mut session, &mut max_step, |s| {
            s.execute(C::Motion(motion));
        });
    }
    render(&mut session, &mut max_step)?;
    let reversal = started.elapsed();
    assert_eq!(session.view().point.position, Scalar(0));
    let mut resize = Duration::ZERO;
    for width in [48, 96, 52, 72] {
        resize = resize.max(sample(&mut session, &mut max_step, |s| {
            s.handle(Event::Resize(width, 24));
        })?);
        assert_eq!(session.view().point.position, Scalar(0));
    }
    let started = Instant::now();
    for code in std::iter::repeat_n(KeyCode::Char('x'), 64)
        .chain(std::iter::repeat_n(KeyCode::Backspace, 64))
    {
        step(&mut session, &mut max_step, |s| {
            key(s, code);
        });
    }
    render(&mut session, &mut max_step)?;
    let input_reversal = started.elapsed();
    assert_eq!(source(&session), original.as_bytes());
    assert_eq!(session.bookmarks().len(), 1024);
    idle(&mut session);
    session.execute(C::Bookmark(B::Save));
    assert_eq!(session.echo(), "Bookmarks saved");
    assert!(!session.bookmarks().is_dirty());
    drop(session);

    std::fs::write(&path, format!("external prefix\n{original}"))?;
    let mut reopened = configured(&startup.snapshot, None)?;
    let origin = reopened.view().point;
    bookmark(&mut reopened, B::Jump, "record-0512");
    assert!(reopened.remaining_input(Instant::now()).is_some());
    reopened.execute(C::KeyboardQuit);
    assert_eq!(reopened.view().point, origin);
    assert_eq!(
        reopened
            .bookmarks()
            .get("record-0512")
            .unwrap()
            .resolution(),
        Resolution::Unresolved
    );
    let started = Instant::now();
    bookmark(&mut reopened, B::Jump, "record-0512");
    let mut passes = 0;
    // The approved search window is 262144 scalars and each pass consumes at
    // most 16384. This finite ceiling covers streaming context search and
    // native publication; cancellation above must retire the same pending work.
    while reopened.remaining_input(Instant::now()).is_some() {
        assert!(passes < 128, "bounded relocation failed to retire");
        step(&mut reopened, &mut max_step, |s| {
            s.expire_input(Instant::now());
        });
        passes += 1;
    }
    let relocation = started.elapsed();
    assert_eq!(
        reopened.view().point.position,
        Scalar(target.0 + "external prefix\n".chars().count())
    );
    assert!(reopened.echo().contains("checked context"));
    render(&mut reopened, &mut max_step)?;
    idle(&mut reopened);
    let status = std::fs::read_to_string("/proc/self/status")?;
    let peak = status
        .lines()
        .find_map(|line| line.strip_prefix("VmHWM:"))
        .ok_or("missing Linux peak memory observation")?
        .trim();
    println!(
        "PASS bookmark resources: {} bytes, 1024 records; admission={}us cold_frame={}us warm_frame_max={}us movement_max={}us far_reversal={}us resize_max={}us input_reversal={}us relocation={}us/{} passes max_step={}us peak_rss={peak}; cap rejected, source exact, canceled search retained origin, no idle deadline",
        original.len(),
        admission.as_micros(),
        cold.as_micros(),
        warm.as_micros(),
        movement.as_micros(),
        reversal.as_micros(),
        resize.as_micros(),
        input_reversal.as_micros(),
        relocation.as_micros(),
        passes,
        max_step.as_micros()
    );
    Ok(())
}
