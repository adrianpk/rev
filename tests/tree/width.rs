use std::path::Path;

use rev::buffer::Buffers;
use rev::command::{ModalCommand, RegionCommand};
use rev::config::prepare;
use rev::view::View;

use super::*;

pub(super) fn configured(fixture: &config_fixture::Fixture, config: &str) -> Session {
    let file = fixture.write("root/main.txt", "retained");
    let startup = prepare(&fixture.write("config.toml", config));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
    let id = buffers.visit(&file).unwrap();
    let view = View::with_settings(buffers.get(id).unwrap(), &startup.snapshot);
    Session::new(
        buffers,
        view,
        Size {
            width: 200,
            height: 8,
        },
    )
    .unwrap()
}

pub(super) fn choose(session: &mut Session, path: &Path) {
    tree(session, TreeCommand::Focus);
    let index = session
        .tree()
        .entries()
        .iter()
        .position(|e| e.path() == path)
        .unwrap();
    tree(session, TreeCommand::Previous);
    while session.tree().selected().unwrap().path() != path {
        let current = session
            .tree()
            .entries()
            .iter()
            .position(|e| e.path() == session.tree().selected().unwrap().path())
            .unwrap();
        tree(
            session,
            if current < index {
                TreeCommand::Next
            } else {
                TreeCommand::Previous
            },
        );
    }
}

pub(super) fn prefix(session: &mut Session, suffix: char) {
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('x'),
        KeyModifiers::CONTROL,
    )));
    session.handle(key(KeyCode::Char('t')));
    session.handle(key(KeyCode::Char(suffix)));
}

// Expanded offscreen rows contribute once at publication. Folded descendants
// do not; folding and refresh removal retain the mark until explicit reset.
#[test]
fn automatic_observations() {
    let fixture = config_fixture::Fixture::new();
    for index in 0..16 {
        fixture.write(&format!("root/a{index:02}"), "");
    }
    let long = "z-name-longer-than-the-preferred-width.txt";
    fixture.write(&format!("root/{long}"), "");
    let hidden = format!("root/folded/{}", "hidden".repeat(15));
    fixture.write(&hidden, "");
    let mut session = configured(&fixture, "tree.visible=true");
    let initial = u16::try_from(long.width() + 6).unwrap();
    assert_eq!(session.tree().desired_width(), initial);
    assert!(session.tree().auto_width());
    assert!(
        !session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|r| r.text().contains(long))
    );
    choose(&mut session, &fixture.0.join("root/folded"));
    assert_eq!(session.tree().desired_width(), initial);
    tree(&mut session, TreeCommand::Expand);
    let expanded = session.tree().desired_width();
    assert_eq!(expanded, 98);
    tree(&mut session, TreeCommand::Collapse);
    tree(&mut session, TreeCommand::Refresh);
    assert_eq!(session.tree().desired_width(), expanded);
    tree(&mut session, TreeCommand::ResetWidth);
    assert_eq!(session.tree().desired_width(), initial);
    std::fs::remove_file(fixture.0.join("root").join(long)).unwrap();
    tree(&mut session, TreeCommand::Refresh);
    assert_eq!(session.tree().desired_width(), initial);
    tree(&mut session, TreeCommand::ResetWidth);
    assert_eq!(session.tree().desired_width(), 24);
}

// Both symbol catalogs and minimal rows measure sanitized terminal cells,
// including whole combining clusters and wide characters, rather than bytes.
#[test]
fn measured_cells() {
    for (decorated, nerd_fonts, expected) in [
        (false, false, 29),
        (false, true, 29),
        (true, false, 31),
        (true, true, 31),
    ] {
        let fixture = config_fixture::Fixture::new();
        let file = fixture.write("root/abcdefghijklmnop界é\x1b.txt", "retained");
        let mut session = configured(
            &fixture,
            &format!(
                "tree.visible=true\ntree.decorated={decorated}\ngutter.nerd_fonts={nerd_fonts}"
            ),
        );
        assert_eq!(session.tree().desired_width(), expected);
        choose(&mut session, &file);
        let frame = session.frame().unwrap();
        assert!(frame.rows().iter().any(|r| r.text().contains("界é^[.txt│")));
        assert_eq!(text(&session), "retained");
    }
}

// Local and prefixed keys share explicit ownership in both profiles. Width
// operations preserve dirty source, point, mark, history and ordinary input.
#[test]
fn width_routes() {
    for profile in ["non-modal", "modal"] {
        let fixture = config_fixture::Fixture::new();
        let mut session = configured(
            &fixture,
            &format!("editor.interaction_profile='{profile}'\ntree.visible=true"),
        );
        if profile == "modal" {
            session.execute(Command::Modal(ModalCommand::Insert));
        }
        session.handle(key(KeyCode::Char('<')));
        session.handle(key(KeyCode::Char('>')));
        assert_eq!(text(&session), "<>retained");
        session.execute(Command::Region(RegionCommand::Set));
        let stamp = session.buffer().unwrap().stamp();
        let point = session.view().point;
        let mark = session.buffer().unwrap().mark();
        let history = session.buffer().unwrap().history().len();
        prefix(&mut session, '<');
        assert_eq!(session.tree().desired_width(), 20);
        tree(&mut session, TreeCommand::Focus);
        session.handle(key(KeyCode::Char('>')));
        assert_eq!(session.tree().desired_width(), 24);
        session.handle(key(KeyCode::Char('<')));
        for command in [
            TreeCommand::Next,
            TreeCommand::Refresh,
            TreeCommand::Toggle,
            TreeCommand::Toggle,
        ] {
            tree(&mut session, command);
            assert_eq!(session.tree().desired_width(), 20);
        }
        assert!(!session.tree().auto_width());
        prefix(&mut session, '=');
        assert!(session.tree().auto_width());
        assert_eq!(session.tree().desired_width(), 24);
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert_eq!(session.view().point, point);
        assert_eq!(session.buffer().unwrap().mark(), mark);
        assert_eq!(session.buffer().unwrap().history().len(), history);
        assert!(session.buffer().unwrap().is_modified());
        assert_eq!(text(&session), "<>retained");
    }
}

// Fixed configuration is restored without rewriting TOML. Runtime mode changes
// and manual widths survive directory replacement, then reset fits current rows.
#[test]
fn ownership_transitions() {
    for automatic in [true, false] {
        let fixture = config_fixture::Fixture::new();
        let config = format!("tree.visible=true\ntree.auto_width={automatic}\ntree.width=30");
        let other = fixture.write("other/a-name-that-needs-more-than-thirty-columns.txt", "");
        let mut session = configured(&fixture, &config);
        tree(&mut session, TreeCommand::Narrow);
        session.load(other.parent().unwrap()).unwrap();
        assert_eq!(session.tree().desired_width(), 26);
        assert!(!session.tree().auto_width());
        tree(&mut session, TreeCommand::AutoWidthMode);
        let fitted = session.tree().desired_width();
        assert!(session.tree().auto_width() && fitted > 30);
        session.load(&fixture.0.join("root")).unwrap();
        assert!(session.tree().auto_width());
        assert_eq!(session.tree().desired_width(), 30);
        session.load(other.parent().unwrap()).unwrap();
        assert_eq!(session.tree().desired_width(), fitted);
        tree(&mut session, TreeCommand::AutoWidthMode);
        session.load(&fixture.0.join("root")).unwrap();
        assert_eq!(session.tree().desired_width(), fitted);
        tree(&mut session, TreeCommand::ResetWidth);
        assert_eq!(session.tree().auto_width(), automatic);
        assert_eq!(session.tree().desired_width(), 30);
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("config.toml")).unwrap(),
            config
        );
    }
}

// Lost paths prove redraw, selection and resize consume only retained state.
// Temporary frame caps restore the same desired columns when room returns.
#[test]
fn retained_geometry() {
    let fixture = config_fixture::Fixture::new();
    fixture.write(&format!("root/{}", "long".repeat(30)), "");
    let mut session = configured(&fixture, "tree.visible=true");
    let desired = session.tree().desired_width();
    let root = session.tree().root().unwrap().to_path_buf();
    std::fs::rename(&root, fixture.0.join("moved")).unwrap();
    for columns in [31, 32, 80, 200] {
        session.handle(Event::Resize(columns, 8));
        tree(&mut session, TreeCommand::Next);
        let frame = session.frame().unwrap();
        assert_eq!(session.tree().desired_width(), desired);
        assert_eq!(
            session.tree().width(Size {
                width: columns,
                height: 8
            }),
            if columns < 32 {
                0
            } else {
                desired.min(columns / 3)
            }
        );
        assert!(
            frame
                .rows()
                .iter()
                .all(|r| r.text().width() == usize::from(columns))
        );
    }
    tree(&mut session, TreeCommand::Narrow);
    let manual = desired - 4;
    for columns in [31, 200] {
        session.handle(Event::Resize(columns, 8));
        session.frame().unwrap();
        assert_eq!(session.tree().desired_width(), manual);
    }
}

// Desired steps saturate at the native minimum even when the configured fixed
// preference is already one cell; a subsequent widen still advances four cells.
#[test]
fn minimum_width() {
    let fixture = config_fixture::Fixture::new();
    let mut session = configured(
        &fixture,
        "tree.visible=true\ntree.auto_width=false\ntree.width=1",
    );
    tree(&mut session, TreeCommand::Narrow);
    assert_eq!(session.tree().desired_width(), 1);
    tree(&mut session, TreeCommand::Widen);
    assert_eq!(session.tree().desired_width(), 5);
    tree(&mut session, TreeCommand::ResetWidth);
    assert_eq!(session.tree().desired_width(), 1);
}
