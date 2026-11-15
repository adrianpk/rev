use crossterm::event::{Event, KeyCode, KeyModifiers};
use rev::buffer::Mode;
use rev::command::{MacroCommand, ModalCommand, Motion, Split};
use rev::coordinates::Byte;
use rev::org::Source;
use rev::session::Session;
use rev::view::{Size, View};

use super::*;
use crate::editor::{character, key};

// Native inheritance exposes both new chords only in base editing and
// Insert/Replace. An explicit replacement retains the effective M-x hint.
#[test]
fn inheritance() {
    for state in [
        None,
        Some(ModalCommand::Normal),
        Some(ModalCommand::Insert),
        Some(ModalCommand::Replace),
        Some(ModalCommand::VisualCharacter),
        Some(ModalCommand::VisualLine),
        Some(ModalCommand::VisualBlock),
    ] {
        let config = if state.is_some() {
            "editor.interaction_profile='modal'"
        } else {
            ""
        };
        let mut e = Editor::build(config, "plain", "(|a)", true);
        if let Some(state) = state {
            e.session.execute(Command::Modal(state));
        }
        let admitted = matches!(
            state,
            None | Some(ModalCommand::Insert | ModalCommand::Replace)
        );
        keys(&mut e, "C-M-u");
        e.expect(if admitted { "|(a)" } else { "(|a)" });
        e.session.execute(Command::Motion(Motion::BufferStart));
        keys(&mut e, "C-M-d");
        e.expect(if admitted { "(|a)" } else { "|(a)" });
        assert!(e.session.buffer().unwrap().history().is_empty());
    }
    let mut e = Editor::new(
        "[keys.editor]\n'C-M-d'='forward-char'\n'M-j'='down-pair'",
        "plain",
        "|(a)",
    );
    keys(&mut e, "C-M-d");
    e.expect("(|a)");
    e.session.execute(Command::Motion(Motion::BufferStart));
    e.session.execute(Command::ExecuteExtended);
    e.session.handle(Event::Paste("down-pair".into()));
    assert!(echo(&mut e).contains("M-j"));
    e.session.execute(Command::KeyboardQuit);
    keys(&mut e, "M-j");
    e.expect("(|a)");
}

// Captured counts retain their nesting meaning during accepted macro replay,
// without source edits or a new undo item for either motion.
#[test]
fn macro_counts() {
    let mut e = Editor::new("", "plain", "|(a [b])");
    e.session.execute(Command::Macro(MacroCommand::Start));
    keys(&mut e, "C-u 2 C-M-d C-M-u");
    e.session.execute(Command::Macro(MacroCommand::End));
    e.expect("(a |[b])");
    e.session.execute(Command::Motion(Motion::BufferStart));
    e.session.execute(Command::Macro(MacroCommand::CallRepeat));
    settle(&mut e.session);
    e.expect("(a |[b])");
    assert!(e.session.buffer().unwrap().history().is_empty());
}

// Selected-view/source replacement and C-g retire the same pending group
// ticket. Later input runs only after the entire nesting target is published.
#[test]
fn lifecycle() {
    let source = format!("\"{}\" (b)", "a ".repeat(4096));
    for change in ["cancel", "source", "view"] {
        let mut e = Editor::new("", "plain", &marked(&source, 0));
        e.session.execute(Command::Split(Split::Right));
        e.session.execute(Command::DownPair);
        assert_eq!(e.session.view().point.position, Scalar(0));
        match change {
            "cancel" => {
                e.session
                    .handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
            }
            "source" => {
                e.session.replace(1..1, "changed").unwrap();
            }
            _ => {
                let current = e.session.windows().selected_id();
                let peer = e
                    .session
                    .windows()
                    .iter()
                    .find(|(id, _)| *id != current)
                    .unwrap()
                    .0;
                e.session.select_view(peer).unwrap();
            }
        }
        let point = e.session.view().point;
        settle(&mut e.session);
        assert_eq!(e.session.view().point, point, "{change}");
    }
    let mut e = Editor::new("", "plain", &marked(&source, 0));
    let started = Instant::now();
    e.session.handle(key(
        KeyCode::Char('d'),
        KeyModifiers::CONTROL | KeyModifiers::ALT,
    ));
    let dispatch = started.elapsed();
    e.session.handle(character('x'));
    settle(&mut e.session);
    eprintln!(
        "nesting native bytes={} dispatch_us={} completed_us={}",
        source.len(),
        dispatch.as_micros(),
        started.elapsed().as_micros()
    );
    assert_eq!(e.text(), source.replace("(b)", "(xb)"));
    assert_eq!(
        e.session.view().point.position.0,
        source.find("(b)").unwrap() + 2
    );
}

// Qualified targets use canonical source through native projection. Org reveal
// preserves peer folds; Markdown pretty headings retain their source positions.
#[test]
fn fold_reveal() {
    for (mode, heading) in [(Mode::Org, "*"), (Mode::Markdown, "#")] {
        let source = format!("{heading} Title\n(a\n[b]\n)\n{heading} Peer\nkeep\n");
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create(&source).unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        buffer.select_mode(mode).unwrap();
        buffer.set_readonly(true);
        let mut view = View::new(buffer);
        if mode == Mode::Org {
            let hidden = Source::new(buffer)
                .range(
                    Byte(source.find("(a").unwrap())
                        ..Byte(source.find(&format!("{heading} Peer")).unwrap()),
                )
                .unwrap();
            view.set_folds(buffer, &[hidden]).unwrap();
        }
        let mut session = Session::new(
            buffers,
            view,
            Size {
                width: 80,
                height: 24,
            },
        )
        .unwrap();
        settle(&mut session);
        session.execute(Command::Split(Split::Right));
        let selected = session.windows().selected_id();
        let peer = session
            .windows()
            .iter()
            .find(|(id, _)| *id != selected)
            .unwrap()
            .0;
        let stamp = session.buffer().unwrap().stamp();
        let peer_row = if mode == Mode::Org { None } else { Some(2) };
        assert_eq!(
            session
                .windows()
                .get(peer)
                .unwrap()
                .screen_row(session.buffer().unwrap(), 2),
            peer_row
        );
        session.execute(Command::PrefixArgument);
        session.handle(character('2'));
        session.handle(key(
            KeyCode::Char('d'),
            KeyModifiers::CONTROL | KeyModifiers::ALT,
        ));
        settle(&mut session);
        let buffer = session.buffer().unwrap();
        assert_eq!(
            session.view().point.position,
            Scalar(source.find("[b]").unwrap() + 1)
        );
        assert!(session.view().screen_row(buffer, 2).is_some());
        assert_eq!(
            session.windows().get(peer).unwrap().screen_row(buffer, 2),
            peer_row
        );
        assert_eq!(buffer.stamp(), stamp);
        assert!(buffer.history().is_empty());
        assert!(buffer.is_readonly());
        assert_eq!(
            buffer
                .slice(&buffer.span(0..buffer.len_chars()).unwrap())
                .unwrap()
                .to_string(),
            source
        );
        let frame = session.frame().unwrap();
        assert!(frame.rows().iter().any(|row| row.text().contains("Title")));
    }
}
