use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::{Command, Motion};
use rev::session::Session;
use std::path::Path;
use std::time::{Duration, Instant};

pub(crate) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub(crate) const DOCUMENT: &str = include_str!("fixture.md");
pub(crate) const MARKERS: [char; 6] = ['◉', '○', '✸', '✿', '◆', '▶'];

pub(crate) fn document(crlf: bool) -> String {
    if crlf {
        format!("\u{feff}{}", DOCUMENT.replace('\n', "\r\n"))
    } else {
        DOCUMENT.into()
    }
}

pub(crate) fn config(profile: &str) -> String {
    format!(
        "editor.interaction_profile='{profile}'\ntree.visible=false\ncompletion.automatic=false\n"
    )
}

pub(crate) fn open(root: &Path, profile: &str, text: &str, extra: &str) -> Result<Session> {
    let path = root.join("native.toml");
    std::fs::write(&path, format!("{}{extra}", config(profile)))?;
    let startup = rev::config::prepare(&path);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let path = root.join("workspace/native.md");
    std::fs::write(&path, text)?;
    let mut buffers = rev::buffer::Buffers::with_settings(startup.snapshot.clone())?;
    let id = buffers.visit(&path)?;
    let view = rev::view::View::with_settings(buffers.get(id)?, &startup.snapshot);
    let mut session = Session::new(
        buffers,
        view,
        rev::view::Size {
            width: 80,
            height: 24,
        },
    )?;
    settle(&mut session)?;
    Ok(session)
}

pub(crate) fn key(editor: &mut Session, code: KeyCode) {
    editor.handle(Event::Key(KeyEvent::new(code, KeyModifiers::NONE)));
}

pub(crate) fn point(editor: &mut Session, needle: &str) {
    assert!(source(editor).contains(needle));
    editor.execute(Command::Motion(Motion::BufferStart));
    editor.execute(Command::SearchForward);
    editor.handle(Event::Paste(needle.into()));
    key(editor, KeyCode::Enter);
}

pub(crate) fn settle(editor: &mut Session) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        editor.expire_input(Instant::now());
        editor.frame()?;
        if editor.remaining_input(Instant::now()).is_none() {
            return Ok(());
        }
        assert!(
            Instant::now() < deadline,
            "native observation exceeded five seconds"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

pub(crate) fn source(editor: &Session) -> String {
    let buffer = editor.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

// Linux process observations belong to the shared bounded PTY idle check.
pub(crate) fn cpu_ticks(process: &str) -> Result<u64> {
    let text = std::fs::read_to_string(format!("/proc/{process}/stat"))?;
    let fields: Vec<_> = text
        .rsplit_once(')')
        .ok_or("invalid Linux stat")?
        .1
        .split_whitespace()
        .collect();
    Ok(fields[11].parse::<u64>()? + fields[12].parse::<u64>()?)
}

pub(crate) fn peak_rss(process: &str) -> Result<String> {
    let text = std::fs::read_to_string(format!("/proc/{process}/status"))?;
    Ok(text
        .lines()
        .find_map(|line| line.strip_prefix("VmHWM:"))
        .ok_or("missing Linux peak RSS")?
        .trim()
        .to_owned())
}
