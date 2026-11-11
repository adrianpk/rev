use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{Command, CompletionCommand as C};
use rev::config::Snapshot;
use rev::coordinates::Scalar;
use rev::session::{Action, Session};
use rev::view::{Size, View};
use std::path::Path;
use std::time::{Duration, Instant};

pub(super) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub(super) const SIZE: Size = Size {
    width: 80,
    height: 24,
};

pub(super) fn source(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

pub(super) fn open(
    root: &Path,
    name: &str,
    document: &str,
    settings: &Snapshot,
) -> Result<Session> {
    let marker = document.find('|').ok_or("missing point marker")?;
    let text = document.replacen('|', "", 1);
    let path = root.join("workspace").join(name);
    std::fs::write(&path, text)?;
    let mut buffers = Buffers::with_settings(settings.clone())?;
    let id = buffers.visit(&path)?;
    let mut view = View::with_settings(buffers.get(id)?, settings);
    view.point.position = Scalar(document[..marker].chars().count());
    Ok(Session::new(buffers, view, SIZE)?)
}

pub(super) fn key(session: &mut Session, code: KeyCode) {
    session.handle(Event::Key(KeyEvent::new(code, KeyModifiers::NONE)));
}

pub(super) fn settle(session: &mut Session) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        session.expire_input(Instant::now());
        session.poll_frame()?;
        if session.remaining_input(Instant::now()).is_none() {
            return Ok(());
        }
        assert!(
            Instant::now() < deadline,
            "owned native work exceeded five seconds: {}",
            session.echo()
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

pub(super) fn complete(session: &mut Session) -> Result<()> {
    session.execute(Command::Completion(C::AtPoint));
    settle(session)
}

pub(super) fn select(session: &mut Session, wanted: &str) {
    let index = session
        .completion()
        .expect("native candidate popup")
        .candidates()
        .iter()
        .position(|candidate| candidate.text() == wanted)
        .unwrap_or_else(|| {
            panic!(
                "missing {wanted:?} in {:?}: {:?}; echo={}",
                session.buffer().unwrap().name(),
                session
                    .completion()
                    .unwrap()
                    .candidates()
                    .iter()
                    .map(|candidate| candidate.text())
                    .collect::<Vec<_>>(),
                session.echo()
            )
        });
    for _ in 0..index {
        session.execute(Command::Completion(C::Next));
    }
}

pub(super) fn idle(session: &mut Session) {
    assert!(session.remaining_input(Instant::now()).is_none());
    assert_eq!(
        session.expire_input(Instant::now() + Duration::from_secs(10)),
        Action::Wait
    );
}

pub(super) fn cpu_ticks(process: &str) -> Result<u64> {
    let text = std::fs::read_to_string(format!("/proc/{process}/stat"))?;
    let fields: Vec<_> = text
        .rsplit_once(')')
        .ok_or("invalid Linux stat")?
        .1
        .split_whitespace()
        .collect();
    Ok(fields[11].parse::<u64>()? + fields[12].parse::<u64>()?)
}

pub(super) fn peak_rss(process: &str) -> Result<String> {
    let status = std::fs::read_to_string(format!("/proc/{process}/status"))?;
    Ok(status
        .lines()
        .find_map(|line| line.strip_prefix("VmHWM:"))
        .ok_or("missing Linux peak RSS")?
        .trim()
        .to_owned())
}
