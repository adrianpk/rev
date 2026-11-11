#![forbid(unsafe_code)]

//! Bounded native-session latency probe; this is not the terminal input gate.
//! All cases use unsaved in-memory source and finite native event sequences.
use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{Command, Motion, Scroll};
use rev::coordinates::Scalar;
use rev::gutter::Change;
use rev::session::{Action, Session};
use rev::view::{Size, View};

type Error = Box<dyn std::error::Error>;

fn render(session: &mut Session, max_step: &mut Duration) -> Result<(), Error> {
    let start = Instant::now();
    let mut ready = session.poll_frame()?.cursor().is_some();
    *max_step = (*max_step).max(start.elapsed());
    for _ in 0..512 {
        if ready {
            return Ok(());
        }
        assert_eq!(
            session.remaining_input(Instant::now()),
            Some(Duration::ZERO)
        );
        let start = Instant::now();
        let action = session.expire_input(Instant::now());
        *max_step = (*max_step).max(start.elapsed());
        if action == Action::Redraw {
            let start = Instant::now();
            ready = session.poll_frame()?.cursor().is_some();
            *max_step = (*max_step).max(start.elapsed());
        }
    }
    assert!(ready, "native projection did not settle within 512 batches");
    Ok(())
}

fn sample(
    session: &mut Session,
    max_step: &mut Duration,
    action: impl FnOnce(&mut Session),
) -> Result<Duration, Error> {
    let start = Instant::now();
    step(session, max_step, action);
    render(session, max_step)?;
    Ok(start.elapsed())
}

fn step(session: &mut Session, max_step: &mut Duration, action: impl FnOnce(&mut Session)) {
    let step = Instant::now();
    action(session);
    *max_step = (*max_step).max(step.elapsed());
}

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

#[derive(Default)]
struct Timings {
    cold: Duration,
    motion: Duration,
    queued: Duration,
    far: Duration,
    resize: Duration,
    input: Duration,
    reversal: Duration,
    step: Duration,
}

struct Probe {
    session: Session,
    original: Scalar,
    times: Timings,
}

impl Probe {
    fn new(name: &str, source: &str) -> Result<Self, Error> {
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
        let mut times = Timings::default();
        render(&mut session, &mut times.step)?;
        times.cold = start.elapsed();
        println!(
            "{name}: {} bytes, {} scalars, cold={}us",
            source.len(),
            original.0,
            times.cold.as_micros()
        );
        Ok(Self {
            session,
            original,
            times,
        })
    }

    fn scrolling(&mut self) -> Result<(), Error> {
        for command in [Scroll::PreviousLine, Scroll::NextLine]
            .into_iter()
            .cycle()
            .take(8)
        {
            self.times.motion =
                self.times
                    .motion
                    .max(sample(&mut self.session, &mut self.times.step, |s| {
                        s.execute(Command::Scroll(command));
                    })?);
        }
        assert_eq!(self.session.view().point.position, self.original);
        Ok(())
    }

    fn queued_reversal(&mut self) -> Result<(), Error> {
        // Queue edits while C-e is waiting on a cold resized projection. The
        // native owner must retain their order and stop after the reversal burst.
        let started = Instant::now();
        step(&mut self.session, &mut self.times.step, |s| {
            s.handle(Event::Resize(47, 24));
        });
        step(&mut self.session, &mut self.times.step, |s| {
            s.execute(Command::Motion(Motion::LineEnd));
        });
        for code in std::iter::repeat_n(KeyCode::Char('x'), 64)
            .chain(std::iter::repeat_n(KeyCode::Backspace, 64))
        {
            step(&mut self.session, &mut self.times.step, |s| {
                s.handle(key(code));
            });
        }
        render(&mut self.session, &mut self.times.step)?;
        self.times.queued = started.elapsed();
        assert_eq!(self.session.view().point.position, self.original);
        Ok(())
    }

    fn far_reversal(&mut self) -> Result<(), Error> {
        let started = Instant::now();
        for motion in [Motion::BufferStart, Motion::BufferEnd, Motion::BufferStart] {
            step(&mut self.session, &mut self.times.step, |s| {
                s.execute(Command::Motion(motion));
            });
        }
        render(&mut self.session, &mut self.times.step)?;
        self.times.far = started.elapsed();
        assert_eq!(self.session.view().point.position, Scalar(0));
        sample(&mut self.session, &mut self.times.step, |s| {
            s.execute(Command::Motion(Motion::BufferEnd));
        })?;
        assert_eq!(self.session.view().point.position, self.original);
        Ok(())
    }

    fn resizing(&mut self) -> Result<(), Error> {
        for width in [48, 81, 52, 80] {
            self.times.resize =
                self.times
                    .resize
                    .max(sample(&mut self.session, &mut self.times.step, |s| {
                        s.handle(Event::Resize(width, 24));
                    })?);
            assert_eq!(self.session.view().point.position, self.original);
        }
        Ok(())
    }

    fn editing(&mut self) -> Result<(), Error> {
        for _ in 0..8 {
            self.times.input =
                self.times
                    .input
                    .max(sample(&mut self.session, &mut self.times.step, |s| {
                        s.handle(key(KeyCode::Char('x')));
                    })?);
        }
        assert_eq!(
            self.session.view().point.position,
            Scalar(self.original.0 + 8)
        );
        for _ in 0..8 {
            self.times.reversal =
                self.times
                    .reversal
                    .max(sample(&mut self.session, &mut self.times.step, |s| {
                        s.handle(key(KeyCode::Backspace));
                    })?);
        }
        assert_eq!(self.session.view().point.position, self.original);
        Ok(())
    }

    fn restored(&mut self, source: &str) -> Result<(), Error> {
        let mut actual = Vec::new();
        rev::file::write(self.session.buffer()?, &mut actual)?;
        assert_eq!(actual, source.as_bytes());
        assert_eq!(self.session.remaining_input(Instant::now()), None);
        assert_eq!(
            self.session
                .expire_input(Instant::now() + Duration::from_secs(10)),
            Action::Wait
        );
        self.session.execute(Command::Motion(Motion::BufferEnd));
        Ok(())
    }

    fn report(&self, name: &str) {
        println!(
            "{name}: max_motion={}us max_resize={}us max_input={}us max_reversal={}us queued_reversal={}us far_reversal={}us max_step={}us; exact source restored, no idle deadline",
            self.times.motion.as_micros(),
            self.times.resize.as_micros(),
            self.times.input.as_micros(),
            self.times.reversal.as_micros(),
            self.times.queued.as_micros(),
            self.times.far.as_micros(),
            self.times.step.as_micros()
        );
    }
}

fn case(name: &str, source: &str) -> Result<(), Error> {
    let mut probe = Probe::new(name, source)?;
    probe.scrolling()?;
    probe.queued_reversal()?;
    probe.far_reversal()?;
    probe.resizing()?;
    probe.editing()?;
    probe.restored(source)?;
    probe.report(name);
    Ok(())
}

fn main() -> Result<(), Error> {
    case("token", &"a".repeat(65_536))?;
    case("words", &"one two three four ".repeat(3_276))?;
    case("unicode", &"a\t界e\u{301} words ".repeat(4_096))?;
    case("regional", &"🇦🇧".repeat(8_192))?;
    case("many-lines", &"short source line\n".repeat(40_000))?;
    Ok(())
}
