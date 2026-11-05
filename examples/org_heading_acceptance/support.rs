use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::session::Session;
use std::path::Path;
use std::time::{Duration, Instant};

pub(crate) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub(crate) const MARKERS: [char; 8] = ['◉', '○', '✸', '✿', '◆', '▶', '◉', '○'];

pub(crate) fn document() -> String {
    (1..=8)
        .map(|level| format!("{} Heading {level}\n", "*".repeat(level)))
        .collect()
}

pub(crate) fn settings(root: &Path, profile: &str) -> Result<rev::config::Snapshot> {
    let path = root.join("headings.toml");
    std::fs::write(&path, config(profile))?;
    let startup = rev::config::prepare(&path);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    Ok(startup.snapshot)
}

pub(crate) fn config(profile: &str) -> String {
    format!(
        "[org]\nindent=false\n[editor]\ninteraction_profile='{profile}'\n[tree]\nvisible=false\n[syntax]\nenabled=false\n[completion]\nautomatic=false\n"
    )
}

pub(crate) fn open(root: &Path, profile: &str, text: &str) -> Result<Session> {
    let settings = settings(root, profile)?;
    let path = root.join("workspace/native.org");
    std::fs::write(&path, text)?;
    let mut buffers = rev::buffer::Buffers::with_settings(settings.clone())?;
    let id = buffers.visit(&path)?;
    let view = rev::view::View::with_settings(buffers.get(id)?, &settings);
    Ok(Session::new(
        buffers,
        view,
        rev::view::Size {
            width: 80,
            height: 24,
        },
    )?)
}

pub(crate) fn key(editor: &mut Session, code: KeyCode) {
    editor.handle(Event::Key(KeyEvent::new(code, KeyModifiers::NONE)));
}

pub(crate) fn named(editor: &mut Session, name: &str) {
    editor.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('x'),
        KeyModifiers::ALT,
    )));
    editor.handle(Event::Paste(name.into()));
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
    let status = std::fs::read_to_string(format!("/proc/{process}/status"))?;
    Ok(status
        .lines()
        .find_map(|line| line.strip_prefix("VmHWM:"))
        .ok_or("missing Linux peak RSS")?
        .trim()
        .to_owned())
}
