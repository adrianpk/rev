use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::{Command, GutterCommand as G, Split, TreeCommand};
use rev::config::{Snapshot, prepare};
use rev::gutter::{CurrentLine as C, Mode as M, Overrides, Settings, Side as S};
use rev::session::Session;
use rev::view::{Size, View};

use super::{Buffers, config_fixture::Fixture, relative::point};

fn editor(snapshot: Snapshot) -> Session {
    let mut buffers = Buffers::with_settings(snapshot.clone()).unwrap();
    let id = buffers.create("界\te\u{301}\r\nsecond\n").unwrap();
    let buffer = buffers.get(id).unwrap();
    let mut view = View::with_settings(buffer, &snapshot);
    point(&mut view, buffer, 1);
    Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap()
}

fn named(session: &mut Session, command: G) {
    let command = Command::Gutter(command);
    assert_eq!(Command::named(command.name()), Some(command));
    assert!(Command::matching("gutter-").contains(&command));
    session.execute(Command::ExecuteExtended);
    session.handle(Event::Paste(command.name().into()));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
}

// Every named operation has the accepted concrete state transition. Relative
// calculation, current forms and right placement are checked in real frame cells.
#[test]
fn native_operations() {
    let mut session = editor(Snapshot::default());
    let stamp = session.buffer().unwrap().stamp();
    let point = session.view().point;
    for (command, visible, mode, current, side, last, label) in [
        (G::Toggle, false, M::Absolute, C::Absolute, S::Left, "", ""),
        (
            G::CycleMode,
            false,
            M::Relative,
            C::Absolute,
            S::Left,
            "",
            "",
        ),
        (
            G::CycleCurrentLine,
            false,
            M::Relative,
            C::Zero,
            S::Left,
            "",
            "",
        ),
        (G::ToggleSide, false, M::Relative, C::Zero, S::Right, "", ""),
        (
            G::Absolute,
            true,
            M::Absolute,
            C::Zero,
            S::Right,
            "│   3 ",
            "│   0 ",
        ),
        (
            G::Relative,
            true,
            M::Relative,
            C::Zero,
            S::Right,
            "│   1 ",
            "│   0 ",
        ),
        (
            G::CycleCurrentLine,
            true,
            M::Relative,
            C::Hidden,
            S::Right,
            "│   1 ",
            "│     ",
        ),
        (
            G::CycleCurrentLine,
            true,
            M::Relative,
            C::Absolute,
            S::Right,
            "│   1 ",
            "│   2 ",
        ),
        (
            G::Reset,
            true,
            M::Absolute,
            C::Absolute,
            S::Left,
            "   3 │",
            "   2 │",
        ),
    ] {
        named(&mut session, command);
        assert_eq!(session.echo(), "");
        let expected = Settings {
            visible,
            mode,
            current_line: current,
            side,
            ..Settings::default()
        };
        assert_eq!(session.view().gutter.effective(), expected, "{command:?}");
        let frame = session.frame().unwrap();
        field(frame.rows()[2].text(), visible, side, last);
        field(frame.rows()[1].text(), visible, side, label);
        assert_eq!(session.view().point, point);
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert!(session.buffer().unwrap().history().is_empty());
        assert!(!session.buffer().unwrap().is_modified());
    }
    assert_eq!(session.view().gutter.overrides(), Overrides::default());
}

fn field(row: &str, visible: bool, side: S, expected: &str) {
    if !visible {
        assert!(!row.contains('│'));
        return;
    }
    match side {
        S::Left => assert!(row.starts_with(expected)),
        S::Right => assert!(row.ends_with(expected)),
    }
}

// Inspect reports effective values and each leaf's startup/view provenance.
// An equal-value explicit selection is still an override; reset uses the frozen
// startup snapshot after its source is changed, without writing that source.
#[test]
fn inspect_reset() {
    let fixture = Fixture::new();
    let root = fixture.write("config.toml", "[gutter]\nvisible = false\nmode = 'relative'\ncurrent_line = 'zero'\nside = 'right'\nminimum_digits = 6\nleft_padding = 2\nright_padding = 3");
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none());
    let initial = startup.snapshot.settings().gutter;
    let mut session = editor(startup.snapshot);
    named(&mut session, G::Inspect);
    assert_eq!(
        session.echo(),
        "Gutter: visible=false(startup) mode=Relative(startup) current_line=Zero(startup) side=Right(startup) digits=6 padding=2/3(startup)"
    );
    named(&mut session, G::Relative);
    named(&mut session, G::Inspect);
    assert!(
        session
            .echo()
            .contains("visible=true(view) mode=Relative(view)")
    );
    assert!(
        session
            .echo()
            .contains("current_line=Zero(startup) side=Right(startup)")
    );
    std::fs::write(&root, "gutter.mode = 'absolute'").unwrap();
    named(&mut session, G::Reset);
    assert_eq!(session.view().gutter.effective(), initial);
    assert_eq!(session.view().gutter.overrides(), Overrides::default());
    assert_eq!(
        std::fs::read_to_string(root).unwrap(),
        "gutter.mode = 'absolute'"
    );
    assert!(session.buffer().unwrap().history().is_empty());
    assert!(!session.buffer().unwrap().is_modified());
}

// Native commands inherit through splits and diverge in only the addressed view.
// A file rebind retains that view's complete policy and independent peer state.
#[test]
fn shared_controls() {
    let fixture = Fixture::new();
    let path = fixture.write("other.txt", "other\r\n");
    let mut session = editor(Snapshot::default());
    named(&mut session, G::Relative);
    let initial = session.view().clone();
    session.execute(Command::Split(Split::Right));
    let original = session.windows().selected_id();
    let peer = session.windows().next_view().unwrap();
    assert_eq!(session.windows().get(peer).unwrap(), &initial);
    named(&mut session, G::Absolute);
    assert_eq!(session.windows().get(peer).unwrap(), &initial);
    let original_view = session.view().clone();
    session.select_view(peer).unwrap();
    named(&mut session, G::ToggleSide);
    named(&mut session, G::CycleCurrentLine);
    named(&mut session, G::CycleCurrentLine);
    assert_eq!(session.windows().get(original).unwrap(), &original_view);
    let peer_policy = session.view().gutter;
    session.load(&path).unwrap();
    assert_eq!(session.view().gutter, peer_policy);
    assert_eq!(session.windows().get(original).unwrap(), &original_view);
    session.select_view(original).unwrap();
    named(&mut session, G::Reset);
    assert_eq!(session.view().gutter.effective(), Settings::default());
    assert_eq!(session.windows().get(peer).unwrap().gutter, peer_policy);
    assert_eq!(session.view().point, original_view.point);
}

// All identities are valid configurable editor targets and rejected in prompt/
// tree maps. Their direct dispatch respects transient focus and C-g cancellation.
#[test]
fn context_admission() {
    for command in Command::matching("gutter-") {
        let fixture = Fixture::new();
        let root = fixture.write(
            "config.toml",
            &format!("keys.editor.M-j = '{}'", command.name()),
        );
        let startup = prepare(&root);
        assert!(startup.diagnostic.is_none());
        let mut session = editor(startup.snapshot);
        session.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('j'),
            KeyModifiers::ALT,
        )));
        assert!(!session.echo().contains("Command is not a text edit"));
        for context in ["minibuffer", "tree"] {
            let root = fixture.write(
                "config.toml",
                &format!("keys.{context}.M-j = '{}'", command.name()),
            );
            assert!(prepare(&root).diagnostic.is_some());
        }
    }
    let mut session = editor(Snapshot::default());
    let before = session.view().clone();
    session.execute(Command::ExecuteExtended);
    session.handle(Event::Paste("gutter-".into()));
    session.execute(Command::Gutter(G::Relative));
    assert_eq!(session.echo(), "Gutter commands require an editing view");
    assert_eq!(session.view(), &before);
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|row| row.text().starts_with("M-x gutter-"))
    );
    session.execute(Command::KeyboardQuit);
    assert_eq!(session.view(), &before);
    session.execute(Command::Tree(TreeCommand::Focus));
    named(&mut session, G::Relative);
    assert_eq!(session.echo(), "File tree owns input focus");
    assert_eq!(session.view(), &before);
}
