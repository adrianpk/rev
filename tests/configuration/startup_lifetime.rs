use super::*;

// A source edit affects only a later startup. The original session also keeps
// its snapshot for buffers opened after the edit, without reloading on frames.
#[test]
fn restart_snapshot() {
    if isolated() {
        lifetime();
        return;
    }
    let fixture = Fixture::new();
    fixture.write("project/next.txt", "next");
    fixture.write("user/rev/config.toml", "include = ['editing.toml']");
    fixture.write("user/rev/editing.toml", "editor.tab_width = 8\neditor.indent_width = 2\npalette.foreground = '#010203'\nkeys.editor.M-j = 'newline'");
    child("startup::lifetime::restart_snapshot", &fixture);
}

fn lifetime() {
    let mut running = launch(None);
    let source = Roots::from_env().unwrap().config.join("editing.toml");
    std::fs::write(&source, "editor.tab_width = 2\neditor.indent_width = 6\npalette.foreground = '#040506'\nkeys.editor.M-j = 'insert-tab'").unwrap();
    let mut restarted = launch(None);
    running.handle(meta('j'));
    restarted.handle(meta('j'));
    assert_eq!(text(&running), "\n");
    assert_eq!(text(&restarted), "      ");
    assert_eq!(running.buffer().unwrap().policy().tab_width, 8);
    assert_eq!(restarted.buffer().unwrap().policy().tab_width, 2);
    let stamp = running.buffer().unwrap().stamp();
    assert!(
        wire(&mut running)
            .windows(b"\x1b[38;2;1;2;3m".len())
            .any(|part| part == b"\x1b[38;2;1;2;3m")
    );
    assert!(
        wire(&mut restarted)
            .windows(b"\x1b[38;2;4;5;6m".len())
            .any(|part| part == b"\x1b[38;2;4;5;6m")
    );
    assert_eq!(running.buffer().unwrap().stamp(), stamp);
    std::fs::remove_file(&source).unwrap();
    running.load(Path::new("next.txt")).unwrap();
    assert_eq!(running.buffer().unwrap().policy().tab_width, 8);
    running.handle(meta('j'));
    assert_eq!(text(&running), "\nnext");
    let fallback = launch(None);
    assert!(fallback.echo().starts_with("Configuration: "));
    assert_eq!(fallback.buffer().unwrap().policy().tab_width, 4);
}

fn wire(session: &mut Session) -> Vec<u8> {
    let mut output = Vec::new();
    session.frame().unwrap().write(&mut output).unwrap();
    output
}

// Startup supplies layout, syntax and retention owners, not just input policy.
// Geometry uses the chosen split share, prompt bounds reject whole input, and
// history retains the newest complete edit group under its configured cap.
#[test]
fn presentation_retention() {
    if isolated() {
        presentation();
        return;
    }
    let fixture = Fixture::new();
    fixture.write("project/code.rs", "fn main() {}\n");
    fixture.write("user/rev/config.toml", "tree.visible = true\ntree.width = 10\ntree.max_fraction = 0.5\nwindows.split_ratio = 0.25\nsyntax.enabled = false\nhistory.group_limit = 1\nminibuffer.input_limit = 3");
    child("startup::lifetime::presentation_retention", &fixture);
}

fn presentation() {
    let mut session = launch(Some(Path::new("code.rs")));
    assert!(session.tree().visible());
    assert!(!session.tree().focused());
    assert_eq!(
        session.tree().width(Size {
            width: 100,
            height: 12
        }),
        10
    );
    assert_eq!(session.buffer().unwrap().mode(), "Text");
    assert!(session.frame().unwrap().rows()[0].syntax().is_empty());
    session.execute(Command::Split(rev::command::Split::Right));
    let geometry = session
        .windows()
        .geometry(Size {
            width: 89,
            height: 11,
        })
        .unwrap();
    assert_eq!(geometry.panes()[0].1.width(), 22);
    session.handle(Event::Paste("a".into()));
    session.handle(Event::Paste("b".into()));
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(rev::command::History::Undo));
    assert_eq!(text(&session), "afn main() {}\n");
    session.execute(Command::ExecuteExtended);
    session.handle(Event::Paste("over".into()));
    assert_eq!(session.echo(), "Prompt input limit exceeded");
    session.execute(Command::KeyboardQuit);
    assert_eq!(text(&session), "afn main() {}\n");
}
