use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::{
    buffer::Buffers,
    command::{BufferCommand, Command, Motion, RegionCommand},
    session::{Action, Session},
    view::{Size, View},
};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

pub(crate) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub(crate) fn open(
    root: &Path,
    profile: &str,
    name: &str,
    text: &str,
    readonly: bool,
) -> Result<Session> {
    let directory = root.join("workspace").join(format!("{name}-{profile}"));
    fs::create_dir_all(&directory)?;
    let path = directory.join("source.txt");
    fs::write(&path, text)?;
    let settings = settings(&directory, profile)?;
    let mut buffers = Buffers::with_settings(settings.clone())?;
    let id = buffers.visit(&path)?;
    let buffer = buffers.get_mut(id)?;
    buffer.set_readonly(readonly);
    let view = View::with_settings(buffer, &settings);
    Ok(Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )?)
}

fn settings(directory: &Path, profile: &str) -> Result<rev::config::Snapshot> {
    let path = directory.join("config.toml");
    fs::write(
        &path,
        format!(
            "editor.interaction_profile='{profile}'\ntree.visible=false\nshell.program='/bin/sh'\nshell.timeout_seconds=3\nfiles.external_change_debounce_ms=0\n"
        ),
    )?;
    let prepared = rev::config::prepare(&path);
    assert!(prepared.diagnostic.is_none(), "{:?}", prepared.diagnostic);
    Ok(prepared.snapshot)
}

pub(crate) fn key(editor: &mut Session, code: KeyCode, modifiers: KeyModifiers) {
    editor.handle(Event::Key(KeyEvent::new(code, modifiers)));
}
pub(crate) fn answer(editor: &mut Session, text: &str) {
    editor.handle(Event::Paste(text.into()));
    key(editor, KeyCode::Enter, KeyModifiers::NONE);
}
pub(crate) fn named(editor: &mut Session, profile: &str, name: &str) {
    if profile == "modal" {
        key(editor, KeyCode::Char(':'), KeyModifiers::NONE);
    } else {
        key(editor, KeyCode::Char('x'), KeyModifiers::ALT);
    }
    answer(editor, name);
}
pub(crate) fn whole(editor: &mut Session) {
    editor.execute(Command::Motion(Motion::BufferStart));
    editor.execute(Command::Region(RegionCommand::Set));
    editor.execute(Command::Motion(Motion::BufferEnd));
}
pub(crate) fn source(editor: &Session) -> String {
    let buffer = editor.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}
pub(crate) fn settle(editor: &mut Session) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        editor.expire_input(Instant::now());
        editor.poll_frame()?;
        if editor.remaining_input(Instant::now()).is_none() {
            return Ok(());
        }
        assert!(
            Instant::now() < deadline,
            "native work exceeded five seconds: {}",
            editor.echo()
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}
pub(crate) fn output(editor: &mut Session) -> Result<String> {
    settle(editor)?;
    editor.execute(Command::ShellCommandOutput);
    let buffer = editor.buffer()?;
    assert!(buffer.is_readonly() && !buffer.is_org());
    assert!(buffer.name().starts_with("*Shell"));
    Ok(source(editor))
}
pub(crate) fn switch(editor: &mut Session, name: &str) {
    editor.execute(Command::Buffer(BufferCommand::Switch));
    answer(editor, name);
}
pub(crate) fn idle(editor: &mut Session) -> Result<()> {
    settle(editor)?;
    assert!(editor.remaining_input(Instant::now()).is_none());
    assert_eq!(
        editor.expire_input(Instant::now() + Duration::from_secs(10)),
        Action::Wait
    );
    Ok(())
}
pub(crate) fn ready(path: &Path) -> Result<String> {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Ok(text) = fs::read_to_string(path)
            && !text.is_empty()
        {
            return Ok(text);
        }
        assert!(
            Instant::now() < deadline,
            "owned child did not report readiness: {}",
            path.display()
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}
pub(crate) fn reaped(pid: &str) -> Result<()> {
    let path = format!("/proc/{}", pid.trim().parse::<u32>()?);
    let state = fs::read_to_string(format!("{path}/stat"));
    assert!(
        state.is_err()
            || state?
                .rsplit_once(')')
                .ok_or("invalid process stat")?
                .1
                .trim_start()
                .starts_with('Z'),
        "owned process remains live"
    );
    Ok(())
}
