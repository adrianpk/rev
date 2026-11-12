use super::width::{choose, configured, prefix};
use super::*;
use rev::command::{ModalCommand, RegionCommand};

fn pan(session: &mut Session, right: bool, count: u16) {
    for _ in 0..count {
        tree(
            session,
            if right {
                TreeCommand::ScrollRight
            } else {
                TreeCommand::ScrollLeft
            },
        );
    }
}

fn row(session: &mut Session, path: &std::path::Path) -> String {
    choose(session, path);
    let frame = session.frame().unwrap();
    frame
        .rows()
        .iter()
        .find(|r| r.selection().is_some())
        .unwrap()
        .text()
        .split('│')
        .next()
        .unwrap()
        .to_owned()
}

// Every style shares one logical cell displacement, including partial wide
// edges, combining marks and sanitized controls. Even blank rows remain hits.
#[test]
fn shared_cells() {
    for (decorated, nerd) in [(false, false), (false, true), (true, false), (true, true)] {
        let f = config_fixture::Fixture::new();
        let file = f.write("root/abc界é\x1b-tail-ending.txt", "retained");
        let mut s = configured(
            &f,
            &format!(
                "tree.visible=true\ntree.auto_width=false\ntree.width=12\ntree.decorated={decorated}\ngutter.nerd_fonts={nerd}"
            ),
        );
        let initial = if decorated {
            "    ≡ abc界é"
        } else {
            "    abc界é^["
        };
        let expected = if nerd {
            initial.replace('≡', "\u{f0219}")
        } else {
            initial.into()
        };
        assert_eq!(row(&mut s, &file), expected);
        let hit = s.tree().hit(2);
        let edge = if decorated { 10 } else { 8 };
        pan(&mut s, true, edge);
        assert_eq!(s.tree().horizontal_offset(), edge);
        assert_eq!(row(&mut s, &file), " é^[-tail-en");
        assert_eq!(s.tree().hit(2), hit);
        assert!(
            s.frame().unwrap().rows()[0]
                .text()
                .starts_with("            │")
        );
        pan(&mut s, false, 1);
        assert_eq!(row(&mut s, &file), "界é^[-tail-e");
        pan(&mut s, true, 100);
        let end = s.tree().horizontal_offset();
        assert_eq!(row(&mut s, &file), "l-ending.txt");
        pan(&mut s, true, 2);
        assert_eq!(s.tree().horizontal_offset(), end);
        pan(&mut s, false, 100);
        assert_eq!(s.tree().horizontal_offset(), 0);
        assert_eq!(row(&mut s, &file), expected);
    }
}

// Real arrows pan only in either profile. Directory operations, prefix cancel,
// focus/visibility/vertical retention and dirty source identity remain separate.
#[test]
fn input_routes() {
    for profile in ["non-modal", "modal"] {
        let f = config_fixture::Fixture::new();
        let folder = f.write(
            "root/folder/nested/very-long-name-with-an-ending.txt",
            "child",
        );
        let mut s = configured(
            &f,
            &format!(
                "editor.interaction_profile='{profile}'\ntree.visible=true\ntree.auto_width=false\ntree.width=12"
            ),
        );
        if profile == "modal" {
            s.execute(Command::Modal(ModalCommand::Insert));
        }
        s.insert('!');
        s.execute(Command::Region(RegionCommand::Set));
        let stamp = s.buffer().unwrap().stamp();
        let point = s.view().point;
        let mark = s.buffer().unwrap().mark();
        let history = s.buffer().unwrap().history().len();
        let window = s.windows().selected_id();
        choose(&mut s, folder.parent().unwrap().parent().unwrap());
        let count = s.tree().entries().len();
        s.handle(key(KeyCode::Right));
        assert_eq!(s.tree().horizontal_offset(), 1);
        assert_eq!(s.tree().entries().len(), count);
        assert!(!s.tree().selected().unwrap().expanded());
        s.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('f'),
            KeyModifiers::CONTROL,
        )));
        assert!(s.tree().selected().unwrap().expanded());
        s.handle(key(KeyCode::Enter));
        assert!(!s.tree().selected().unwrap().expanded());
        s.handle(key(KeyCode::Left));
        assert_eq!(s.tree().horizontal_offset(), 0);
        s.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('f'),
            KeyModifiers::CONTROL,
        )));
        choose(&mut s, folder.parent().unwrap());
        s.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('f'),
            KeyModifiers::CONTROL,
        )));
        pan(&mut s, true, 3);
        for command in [
            TreeCommand::Next,
            TreeCommand::Previous,
            TreeCommand::Refresh,
            TreeCommand::Toggle,
            TreeCommand::Toggle,
        ] {
            tree(&mut s, command);
            s.frame().unwrap();
            assert_eq!(s.tree().horizontal_offset(), 3);
        }
        s.handle(key(KeyCode::Tab));
        let frame = s.frame().unwrap();
        let active_row = frame
            .rows()
            .iter()
            .find(|r| r.selection().is_some())
            .unwrap()
            .row();
        let expected = s
            .tree()
            .entries()
            .iter()
            .position(|e| e.path() == f.0.join("root/main.txt"))
            .unwrap();
        assert_eq!(
            s.tree().hit(active_row),
            s.tree().hit(u16::try_from(expected).unwrap())
        );
        assert_eq!(s.tree().horizontal_offset(), 3);
        tree(&mut s, TreeCommand::Focus);
        assert_eq!(s.tree().horizontal_offset(), 3);
        s.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('x'),
            KeyModifiers::CONTROL,
        )));
        s.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('g'),
            KeyModifiers::CONTROL,
        )));
        assert_eq!(s.tree().horizontal_offset(), 3);
        tree(&mut s, TreeCommand::Focus);
        s.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('b'),
            KeyModifiers::CONTROL,
        )));
        assert!(!s.tree().selected().unwrap().expanded());
        prefix(&mut s, '=');
        assert_eq!(s.tree().horizontal_offset(), 0);
        assert_eq!(s.buffer().unwrap().stamp(), stamp);
        assert_eq!(s.view().point, point);
        assert_eq!(s.buffer().unwrap().mark(), mark);
        assert_eq!(s.buffer().unwrap().history().len(), history);
        assert_eq!(s.windows().selected_id(), window);
        assert_eq!(text(&s), "!retained");
        assert!(s.buffer().unwrap().is_modified());
    }
}

// Cached extent shrinks on folding/removal; only excess is clamped by wider
// geometry or width ownership. Lost directories prove pan/resize never discover.
#[test]
fn extent_transitions() {
    let f = config_fixture::Fixture::new();
    let long = f.write("root/folder/deep/nested-with-a-long-ending.txt", "");
    let mut s = configured(
        &f,
        "tree.visible=true\ntree.auto_width=false\ntree.width=12",
    );
    choose(&mut s, long.parent().unwrap().parent().unwrap());
    tree(&mut s, TreeCommand::Expand);
    choose(&mut s, long.parent().unwrap());
    tree(&mut s, TreeCommand::Expand);
    pan(&mut s, true, 100);
    assert_eq!(row(&mut s, &long), "g-ending.txt");
    let end = s.tree().horizontal_offset();
    tree(&mut s, TreeCommand::Widen);
    assert_eq!(s.tree().horizontal_offset(), end - 4);
    tree(&mut s, TreeCommand::Narrow);
    assert_eq!(s.tree().horizontal_offset(), end - 4);
    choose(&mut s, long.parent().unwrap().parent().unwrap());
    tree(&mut s, TreeCommand::Collapse);
    assert_eq!(s.tree().horizontal_offset(), 2); // main.txt is the longest remaining row
    tree(&mut s, TreeCommand::AutoWidthMode);
    assert_eq!(s.tree().horizontal_offset(), 0);
    tree(&mut s, TreeCommand::Narrow);
    pan(&mut s, true, 100);
    tree(&mut s, TreeCommand::ResetWidth);
    assert_eq!(s.tree().horizontal_offset(), 0);
    s.load(long.parent().unwrap().parent().unwrap()).unwrap();
    assert_eq!(s.tree().horizontal_offset(), 0);
    choose(&mut s, long.parent().unwrap());
    tree(&mut s, TreeCommand::Expand);
    tree(&mut s, TreeCommand::Widen);
    tree(&mut s, TreeCommand::Widen);
    s.handle(Event::Resize(32, 8));
    pan(&mut s, true, 100);
    let narrow_end = s.tree().horizontal_offset();
    s.handle(Event::Resize(200, 8));
    assert_eq!(s.tree().horizontal_offset(), narrow_end - 10);
    let retained = s.tree().horizontal_offset();
    assert!(retained > 0);
    std::fs::rename(f.0.join("root"), f.0.join("moved")).unwrap();
    for columns in [31, 0, 40, 200] {
        s.handle(Event::Resize(columns, 8));
        s.frame().unwrap();
        assert_eq!(s.tree().horizontal_offset(), retained);
    }
}
