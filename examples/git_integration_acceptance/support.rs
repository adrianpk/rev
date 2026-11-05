use crate::repository::Repository;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::{
    buffer::Buffers,
    command::Command,
    session::Session,
    view::{Size, View},
};
use std::time::{Duration, Instant};

pub(crate) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub(crate) use crate::metrics::{cpu_ticks, peak_rss};

// One native fixture owns its terminal children even when an assertion fails.
pub(crate) struct Editor(pub Session);
impl Drop for Editor {
    fn drop(&mut self) {
        self.0
            .shutdown_terminals()
            .expect("owned fixture terminals");
    }
}

pub(crate) fn editor(repository: &Repository, profile: &str) -> Result<Editor> {
    let terminal = repository.fixture.write(
        "terminal.sh",
        "printf 'TERM_READY\\n'\nwhile IFS= read -r line; do printf 'GOT:%s\\n' \"$line\"; done\n",
    );
    let snapshot = repository.snapshot(&format!(
        "[editor]\ninteraction_profile='{profile}'\n[tree]\nvisible=false\n[terminal]\nprogram='/bin/sh'\nargs=[{}]\n", serde_json::to_string(&terminal)?
    ));
    let mut buffers = Buffers::with_settings(snapshot.clone())?;
    let id = buffers.visit(&repository.root.join("source.txt"))?;
    let view = View::with_settings(buffers.get(id)?, &snapshot);
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 24,
        },
    )?;
    session.load(&repository.root)?;
    session.load(&repository.root.join("source.txt"))?;
    Ok(Editor(session))
}

pub(crate) fn key(editor: &mut Session, code: KeyCode, modifiers: KeyModifiers) {
    editor.handle(Event::Key(KeyEvent::new(code, modifiers)));
}

pub(crate) fn named(editor: &mut Session, name: &str) {
    editor.execute(Command::named(name).expect("native catalog command"));
}

pub(crate) fn plain(editor: &mut Session, character: char) {
    key(editor, KeyCode::Char(character), KeyModifiers::NONE);
}

pub(crate) fn surface(editor: &mut Session) -> String {
    editor
        .frame()
        .expect("native frame")
        .rows()
        .iter()
        .map(|r| r.text())
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn accept(editor: &mut Session) {
    key(editor, KeyCode::Char('c'), KeyModifiers::CONTROL);
    key(editor, KeyCode::Char('c'), KeyModifiers::CONTROL);
}

pub(crate) fn text(editor: &Session) -> String {
    let buffer = editor.buffer().expect("native buffer");
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

pub(crate) fn wait(editor: &mut Session, mut ready: impl FnMut(&mut Session) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        editor.expire_input(Instant::now());
        if ready(editor) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "native Git deadline: {}",
            editor.echo()
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}
