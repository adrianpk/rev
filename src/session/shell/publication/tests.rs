use crate::{
    buffer::Buffers,
    command::Command,
    fixture::Fixture,
    session::Session,
    view::{Size, View},
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::{
    fs,
    time::{Duration, Instant},
};

// The host can change readonly policy while an actual child owns a valid
// captured target. Completion retains stdout but cannot publish to that source.
#[test]
fn readonly_transition() {
    let f = Fixture::new();
    let source = f.write("source.txt", "source");
    let config = f.write("config.toml", "shell.program='/bin/sh'");
    let snapshot = crate::config::prepare(&config).snapshot;
    let mut buffers = Buffers::with_settings(snapshot.clone()).unwrap();
    let id = buffers.visit(&source).unwrap();
    let view = View::with_settings(buffers.get(id).unwrap(), &snapshot);
    let mut editor = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap();
    let before = editor.buffer().unwrap().stamp();
    editor.execute(Command::ShellCommandInsert);
    editor.handle(Event::Paste(
        "printf started > started; while test ! -f ready; do sleep 0.01; done; printf output"
            .into(),
    ));
    editor.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    let deadline = Instant::now() + Duration::from_secs(5);
    while !fs::metadata(f.0.join("started")).is_ok_and(|m| m.len() > 0) {
        assert!(Instant::now() < deadline, "shell fixture did not start");
        std::thread::sleep(Duration::from_millis(2));
    }
    editor.buffers.get_mut(id).unwrap().set_readonly(true);
    fs::write(f.0.join("ready"), "").unwrap();
    while editor.shell.operation.is_some() {
        editor.expire_input(Instant::now());
        assert!(Instant::now() < deadline, "shell completion did not arrive");
        std::thread::sleep(Duration::from_millis(2));
    }
    let buffer = editor.buffer().unwrap();
    assert_eq!(buffer.stamp(), before);
    assert!(buffer.is_readonly() && buffer.history().is_empty());
    assert_eq!(
        buffer
            .slice(&buffer.span(0..buffer.len_chars()).unwrap())
            .unwrap(),
        "source"
    );
    let output = editor
        .buffers
        .get(editor.shell.output.unwrap().buffer)
        .unwrap();
    assert_eq!(
        output
            .slice(&output.span(0..output.len_chars()).unwrap())
            .unwrap(),
        "output"
    );
    let diagnostics = editor
        .buffers
        .get(editor.shell.diagnostics.unwrap().buffer)
        .unwrap();
    assert!(
        diagnostics
            .slice(&diagnostics.span(0..diagnostics.len_chars()).unwrap())
            .unwrap()
            .to_string()
            .contains("read-only")
    );
    assert!(editor.shell_remaining().is_none());
}
