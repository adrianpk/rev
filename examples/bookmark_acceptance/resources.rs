use std::path::{Path, PathBuf};
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

struct Fixture {
    path: PathBuf,
    original: String,
    settings: rev::config::Snapshot,
}

impl Fixture {
    fn prepare(root: &Path) -> Result<Self> {
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
        Ok(Self {
            path,
            original,
            settings: startup.snapshot,
        })
    }
}

struct Measurements {
    admission: Duration,
    cold: Duration,
    warm: Duration,
    movement: Duration,
    reversal: Duration,
    resize: Duration,
    input_reversal: Duration,
    max_step: Duration,
}

pub(super) fn run(root: &Path) -> Result<()> {
    let fixture = Fixture::prepare(root)?;
    let mut session = configured(&fixture.settings, Some(&fixture.path))?;
    let (mut measurements, target) = measure(&mut session, &fixture.original)?;
    drop(session);
    let original = &fixture.original;
    std::fs::write(&fixture.path, format!("external prefix\n{original}"))?;
    let mut reopened = configured(&fixture.settings, None)?;
    canceled_search(&mut reopened)?;
    let (relocation, passes) = relocate(&mut reopened, &mut measurements.max_step, target)?;
    let peak = peak_memory()?;
    println!(
        "PASS bookmark resources: {} bytes, 1024 records; admission={}us cold_frame={}us warm_frame_max={}us movement_max={}us far_reversal={}us resize_max={}us input_reversal={}us relocation={}us/{} passes max_step={}us peak_rss={peak}; cap rejected, source exact, canceled search retained origin, no idle deadline",
        fixture.original.len(),
        measurements.admission.as_micros(),
        measurements.cold.as_micros(),
        measurements.warm.as_micros(),
        measurements.movement.as_micros(),
        measurements.reversal.as_micros(),
        measurements.resize.as_micros(),
        measurements.input_reversal.as_micros(),
        relocation.as_micros(),
        passes,
        measurements.max_step.as_micros()
    );
    Ok(())
}

fn admit(session: &mut Session) -> (Duration, Scalar) {
    let started = Instant::now();
    for row in 0..1024 {
        goto(session, row + 2);
        bookmark(session, B::Set, &format!("record-{row:04}"));
        assert_eq!(session.bookmarks().len(), row + 1);
    }
    let admission = started.elapsed();
    let target = session
        .bookmarks()
        .get("record-0512")
        .unwrap()
        .point()
        .position;
    bookmark(session, B::Set, "overflow");
    assert!(session.echo().contains("collection is full"));
    session.execute(C::KeyboardQuit);
    assert_eq!(session.bookmarks().len(), 1024);
    (admission, target)
}

fn measure(session: &mut Session, original: &str) -> Result<(Measurements, Scalar)> {
    let (admission, target) = admit(session);
    let mut max_step = Duration::ZERO;
    let started = Instant::now();
    render(session, &mut max_step)?;
    let cold = started.elapsed();
    let warm = warm_frames(session, &mut max_step)?;
    let movement = movement(session, &mut max_step)?;
    let reversal = far_reversal(session, &mut max_step)?;
    let resize = resizing(session, &mut max_step)?;
    let input_reversal = input_reversal(session, &mut max_step)?;
    assert_eq!(source(session), original.as_bytes());
    assert_eq!(session.bookmarks().len(), 1024);
    idle(session);
    session.execute(C::Bookmark(B::Save));
    assert_eq!(session.echo(), "Bookmarks saved");
    assert!(!session.bookmarks().is_dirty());
    Ok((
        Measurements {
            admission,
            cold,
            warm,
            movement,
            reversal,
            resize,
            input_reversal,
            max_step,
        },
        target,
    ))
}

fn warm_frames(session: &mut Session, max_step: &mut Duration) -> Result<Duration> {
    let mut warm = Duration::ZERO;
    for _ in 0..16 {
        let started = Instant::now();
        render(session, max_step)?;
        warm = warm.max(started.elapsed());
    }
    Ok(warm)
}

fn movement(session: &mut Session, max_step: &mut Duration) -> Result<Duration> {
    let mut movement = Duration::ZERO;
    for command in [Scroll::PreviousLine, Scroll::NextLine]
        .into_iter()
        .cycle()
        .take(16)
    {
        movement = movement.max(sample(session, max_step, |s| {
            s.execute(C::Scroll(command));
        })?);
    }
    Ok(movement)
}

fn far_reversal(session: &mut Session, max_step: &mut Duration) -> Result<Duration> {
    let started = Instant::now();
    for motion in [
        Motion::BufferEnd,
        Motion::BufferStart,
        Motion::BufferEnd,
        Motion::BufferStart,
    ] {
        step(session, max_step, |s| {
            s.execute(C::Motion(motion));
        });
    }
    render(session, max_step)?;
    let reversal = started.elapsed();
    assert_eq!(session.view().point.position, Scalar(0));
    Ok(reversal)
}

fn resizing(session: &mut Session, max_step: &mut Duration) -> Result<Duration> {
    let mut resize = Duration::ZERO;
    for width in [48, 96, 52, 72] {
        resize = resize.max(sample(session, max_step, |s| {
            s.handle(Event::Resize(width, 24));
        })?);
        assert_eq!(session.view().point.position, Scalar(0));
    }
    Ok(resize)
}

fn input_reversal(session: &mut Session, max_step: &mut Duration) -> Result<Duration> {
    let started = Instant::now();
    for code in std::iter::repeat_n(KeyCode::Char('x'), 64)
        .chain(std::iter::repeat_n(KeyCode::Backspace, 64))
    {
        step(session, max_step, |s| {
            key(s, code);
        });
    }
    render(session, max_step)?;
    let input_reversal = started.elapsed();
    Ok(input_reversal)
}

fn canceled_search(reopened: &mut Session) -> Result<()> {
    let origin = reopened.view().point;
    bookmark(reopened, B::Jump, "record-0512");
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
    Ok(())
}

fn relocate(
    reopened: &mut Session,
    max_step: &mut Duration,
    target: Scalar,
) -> Result<(Duration, usize)> {
    let started = Instant::now();
    bookmark(reopened, B::Jump, "record-0512");
    let mut passes = 0;
    // The approved search window is 262144 scalars and each pass consumes at
    // most 16384. This finite ceiling covers streaming context search and
    // native publication; cancellation above must retire the same pending work.
    while reopened.remaining_input(Instant::now()).is_some() {
        assert!(passes < 128, "bounded relocation failed to retire");
        step(reopened, max_step, |s| {
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
    render(reopened, max_step)?;
    idle(reopened);
    Ok((relocation, passes))
}

fn peak_memory() -> Result<String> {
    let status = std::fs::read_to_string("/proc/self/status")?;
    let peak = status
        .lines()
        .find_map(|line| line.strip_prefix("VmHWM:"))
        .ok_or("missing Linux peak memory observation")?
        .trim();
    Ok(peak.to_owned())
}
