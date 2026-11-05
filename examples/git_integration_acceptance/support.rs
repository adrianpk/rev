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
    let snapshot = repository.snapshot(&format!(
        "[editor]\ninteraction_profile='{profile}'\n[tree]\nvisible=false\n"
    ));
    let mut buffers = Buffers::with_settings(snapshot.clone())?;
    let id = buffers.visit(&repository.root.join("source.txt"))?;
    let view = View::with_settings(buffers.get(id)?, &snapshot);
    Ok(Editor(Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 24,
        },
    )?))
}

pub(crate) fn key(editor: &mut Session, code: KeyCode, modifiers: KeyModifiers) {
    editor.handle(Event::Key(KeyEvent::new(code, modifiers)));
}

pub(crate) fn named(editor: &mut Session, name: &str) {
    editor.execute(Command::named(name).expect("native catalog command"));
}

pub(crate) fn text(editor: &Session) -> String {
    let buffer = editor.buffer().expect("native buffer");
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

pub(crate) fn wait(editor: &mut Session, ready: impl Fn(&Session) -> bool) {
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
