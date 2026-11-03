use super::*;
use crate::filters::settle;

fn painted(editor: &mut Session) -> String {
    editor
        .frame()
        .unwrap()
        .rows()
        .iter()
        .map(|r| r.text())
        .collect::<Vec<_>>()
        .join("\n")
}

// Startup and command discovery remain idle. A host can observe real child
// completion through frame production alone; no post-job shell wake is retained.
#[test]
fn frame_completion() {
    let f = Fixture::new();
    let (mut editor, path) = editor(&f, "");
    assert!(editor.remaining_input(Instant::now()).is_none());
    assert!(!painted(&mut editor).contains("<Shell"));
    editor.execute(Command::ExecuteExtended);
    editor.handle(Event::Paste("shell-command".into()));
    assert!(editor.remaining_input(Instant::now()).is_none());
    editor.handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
    editor.execute(Command::ShellCommandInsert);
    answer(
        &mut editor,
        "printf started > started; while test ! -f ready; do sleep 0.01; done; printf done",
    );
    let directory = path.parent().unwrap();
    ready(&directory.join("started"));
    let point = editor.view().point;
    let frame = painted(&mut editor);
    assert!(frame.contains("<Shell running>"), "{frame}");
    assert!(editor.remaining_input(Instant::now()).is_some());
    fs::write(directory.join("ready"), "").unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let frame = painted(&mut editor);
        if editor.buffer().unwrap().stamp().revision != point.stamp.revision {
            assert!(!frame.contains("<Shell"));
            break;
        }
        assert!(
            Instant::now() < deadline,
            "frame did not observe the completed job"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(
        text(editor.buffer().unwrap()),
        "doneunsaved input is data; never a command\n"
    );
    assert_eq!(editor.buffer().unwrap().history().len(), 1);
    settle(&mut editor);
    assert!(editor.remaining_input(Instant::now()).is_none());
    for _ in 0..3 {
        assert_eq!(editor.expire_input(Instant::now()), Action::Wait);
        assert!(!painted(&mut editor).contains("<Shell"));
        assert!(editor.remaining_input(Instant::now()).is_none());
    }
}

// Shell and timer activity share one status annotation, preserve ordinary
// narrow-pane rendering and both disappear when their owners have stopped.
#[test]
fn shared_status() {
    let f = Fixture::new();
    let (mut editor, path) = editor(&f, "");
    editor.execute(Command::Org(rev::command::OrgCommand::TimerStart));
    start(&mut editor, "printf started > started; sleep 30");
    ready(&path.parent().unwrap().join("started"));
    let frame = painted(&mut editor);
    assert!(frame.contains("<T+"), "{frame}");
    assert!(frame.contains("<Shell running>"), "{frame}");
    editor.handle(Event::Resize(12, 8));
    assert!(editor.frame().is_ok());
    editor.handle(Event::Resize(80, 24));
    editor.execute(Command::ShellCommandCancel);
    settle(&mut editor);
    let frame = painted(&mut editor);
    assert!(!frame.contains("<Shell"));
    assert!(frame.contains("<T+"));
    editor.execute(Command::Org(rev::command::OrgCommand::TimerStop));
    assert!(!painted(&mut editor).contains("<T+"));
    assert!(editor.remaining_input(Instant::now()).is_none());
}
