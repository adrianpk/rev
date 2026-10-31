//! Bounded native-session latency probe; this is not the terminal input gate.
//! All cases use unsaved in-memory source, eight typed/reversed events and no IO.
use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{Command, Motion, Scroll};
use rev::coordinates::Scalar;
use rev::gutter::Change;
use rev::session::{Action, Session};
use rev::view::{Size, View};

type Error = Box<dyn std::error::Error>;

fn sample(session: &mut Session, action: impl FnOnce(&mut Session)) -> Result<Duration, Error> {
    let start = Instant::now();
    action(session);
    session.frame()?;
    Ok(start.elapsed())
}

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

fn case(name: &str, source: &str) -> Result<(), Error> {
    let mut buffers = Buffers::new()?;
    let id = buffers.create(source)?;
    let buffer = buffers.get(id)?;
    let original = Scalar(buffer.len_chars());
    let mut view = View::new(buffer);
    view.gutter.apply(Change::Visible(false));
    view.point.position = original;
    let start = Instant::now();
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )?;
    session.frame()?;
    let cold = start.elapsed();
    println!(
        "{name}: {} bytes, {} scalars, cold={}us",
        source.len(),
        original.0,
        cold.as_micros()
    );
    let mut motion = Duration::ZERO;
    for command in [Scroll::PreviousLine, Scroll::NextLine]
        .into_iter()
        .cycle()
        .take(8)
    {
        motion = motion.max(sample(&mut session, |s| {
            s.execute(Command::Scroll(command));
        })?);
    }
    assert_eq!(session.view().point.position, original);
    let mut resize = Duration::ZERO;
    for width in [48, 81, 52, 80] {
        resize = resize.max(sample(&mut session, |s| {
            s.handle(Event::Resize(width, 24));
        })?);
        assert_eq!(session.view().point.position, original);
    }
    let mut input = Duration::ZERO;
    for _ in 0..8 {
        input = input.max(sample(&mut session, |s| {
            s.handle(key(KeyCode::Char('x')));
        })?);
    }
    assert_eq!(session.view().point.position, Scalar(original.0 + 8));
    let mut reversal = Duration::ZERO;
    for _ in 0..8 {
        reversal = reversal.max(sample(&mut session, |s| {
            s.handle(key(KeyCode::Backspace));
        })?);
    }
    assert_eq!(session.view().point.position, original);
    let mut actual = Vec::new();
    rev::file::write(session.buffer()?, &mut actual)?;
    assert_eq!(actual, source.as_bytes());
    assert_eq!(session.remaining_input(Instant::now()), None);
    assert_eq!(
        session.expire_input(Instant::now() + Duration::from_secs(10)),
        Action::Wait
    );
    session.execute(Command::Motion(Motion::BufferEnd));
    println!(
        "{name}: max_motion={}us max_resize={}us max_input={}us max_reversal={}us; exact source restored, no idle deadline",
        motion.as_micros(),
        resize.as_micros(),
        input.as_micros(),
        reversal.as_micros()
    );
    Ok(())
}

fn main() -> Result<(), Error> {
    case("token", &"a".repeat(65_536))?;
    case("words", &"one two three four ".repeat(3_276))?;
    case("unicode", &"a\t界e\u{301} words ".repeat(4_096))?;
    Ok(())
}
