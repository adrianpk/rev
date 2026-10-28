use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use rev::session::{Action, Session};

use super::{Result, emit, require};

const TICK: Duration = Duration::from_millis(100);

fn mouse(kind: MouseEventKind, row: u16) -> Event {
    Event::Mouse(MouseEvent {
        kind,
        column: 8,
        row,
        modifiers: KeyModifiers::NONE,
    })
}

fn tick(session: &mut Session, now: Instant) -> Result<Duration> {
    let top = session.view().top;
    let started = Instant::now();
    let action = session.expire_input(now);
    require(
        action == Action::Redraw,
        &format!(
            "drag tick missing: {action:?}, top={top}, echo={}",
            session.echo()
        ),
    )?;
    session.frame()?.write(&mut io::sink())?;
    let elapsed = started.elapsed();
    require(session.view().top == top + 3, "drag exceeded scroll step")?;
    require(
        session.expire_input(now) == Action::Wait,
        "drag caught up multiple ticks",
    )?;
    require(
        session.remaining_input(now) == Some(TICK),
        "drag tick cadence changed",
    )?;
    Ok(elapsed)
}

fn idle(session: &mut Session, now: Instant) -> Result<()> {
    session.handle_at(mouse(MouseEventKind::Up(MouseButton::Left), 22), now);
    let view = session.view().clone();
    let mut samples = Vec::new();
    for index in 1..=64 {
        let at = now + TICK * index;
        let started = Instant::now();
        require(
            session.expire_input(at) == Action::Wait,
            "released drag ticked",
        )?;
        require(
            session.remaining_input(at).is_none(),
            "idle drag retained deadline",
        )?;
        samples.push(started.elapsed());
    }
    require(
        session.view() == &view,
        "released drag moved point or viewport",
    )?;
    emit(
        "drag_idle",
        samples,
        "\"iterations\":64,\"ticks\":0,\"deadlines\":0",
    )
}

pub(super) fn run(session: &mut Session) -> Result<()> {
    let now = Instant::now();
    session.handle_at(mouse(MouseEventKind::Down(MouseButton::Left), 0), now);
    session.handle_at(mouse(MouseEventKind::Drag(MouseButton::Left), 22), now);
    let mut samples = Vec::new();
    for index in 1..=64 {
        samples.push(tick(session, now + TICK * index)?);
    }
    require(session.view().top == 192, "drag iteration count changed")?;
    emit(
        "drag_active",
        samples,
        "\"iterations\":64,\"ticks\":64,\"tick_ms\":100,\"step_rows\":3,\"scrolled_rows\":192",
    )?;
    idle(session, now + TICK * 64)
}
