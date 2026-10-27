use std::collections::BTreeSet;

use crossterm::event::Event;
use rev::command::{Command, GutterCommand as G, Motion, Split, TreeCommand};
use rev::config::{Palette, Rgb};
use rev::display::Frame;
use rev::gutter::{Format, Policy, Settings};
use rev::panel::Composition;
use rev::session::Session;
use rev::view::{Size, View};
use rev::window::Area;
use unicode_width::UnicodeWidthStr;

use super::{Buffers, surface};

pub(super) fn editor(format: Format) -> Session {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers
        .create("\t界e\u{301}0123456789\r\n\nlast\n")
        .unwrap();
    let mut view = View::new(buffers.get(id).unwrap());
    view.gutter = Policy::new(Settings {
        format,
        ..Settings::default()
    });
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 120,
            height: 24,
        },
    )
    .unwrap();
    session.execute(Command::Split(Split::Right));
    session.execute(Command::Split(Split::Below));
    let order = session.windows().order();
    session.execute(Command::Gutter(G::Relative));
    session.select_view(order[1]).unwrap();
    session.execute(Command::Motion(Motion::BufferEnd));
    session.execute(Command::Gutter(G::ToggleSide));
    session.execute(Command::Gutter(G::CycleCurrentLine));
    session.select_view(order[2]).unwrap();
    session.execute(Command::Motion(Motion::BufferEnd));
    session.execute(Command::Gutter(G::Relative));
    session.execute(Command::Gutter(G::Toggle));
    session.select_view(order[0]).unwrap();
    session.execute(Command::Tree(TreeCommand::Toggle));
    session.execute(Command::KeyboardQuit);
    session.frame().unwrap();
    session
}

fn contains(area: Area, row: usize, column: usize) -> bool {
    (usize::from(area.row())..usize::from(area.row() + area.height())).contains(&row)
        && (usize::from(area.column())..usize::from(area.column() + area.width())).contains(&column)
}

fn lanes(session: &Session, size: Size, completion: bool) -> Vec<Area> {
    let composition = if completion {
        Composition::completion(session.windows(), session.tree(), size, 8)
    } else {
        Composition::workspace(session.windows(), session.tree(), size, 0)
    };
    let Some(composition) = composition else {
        return Vec::new();
    };
    let mut lanes = Vec::new();
    for (id, panel) in composition.panels() {
        let view = session.windows().get(*id).unwrap();
        let geometry = view
            .gutter_geometry(session.buffer().unwrap(), panel.content())
            .unwrap();
        assert!(geometry.text().width() >= 3);
        if let Some(numbers) = geometry.numbers() {
            assert_eq!(
                geometry.text().width() + numbers.width(),
                panel.content().width()
            );
            assert!(!contains(
                numbers,
                usize::from(panel.status().unwrap().row()),
                usize::from(numbers.column())
            ));
            lanes.push(numbers);
        } else {
            assert_eq!(geometry.text(), panel.content());
        }
    }
    lanes
}

fn painted(frame: Frame, lanes: &[Area]) {
    let mut palette = Palette::default();
    palette.gutter.background = Rgb(1, 2, 3);
    let frame = frame.with_palette(&palette);
    let cells = surface::draw(&frame);
    let actual: BTreeSet<_> = cells
        .iter()
        .filter(|(_, cell)| cell.background == Some(Rgb(1, 2, 3)))
        .map(|(position, _)| *position)
        .collect();
    let expected: BTreeSet<_> = lanes
        .iter()
        .flat_map(|area| {
            (area.row()..area.row() + area.height()).flat_map(move |row| {
                (area.column()..area.column() + area.width())
                    .map(move |column| (usize::from(row), usize::from(column)))
            })
        })
        .collect();
    assert_eq!(
        actual, expected,
        "gutter paint escaped its admitted number lanes"
    );
}

// Actual numbered cells occupy only admitted editing lanes across nested splits,
// independent policies, tree and completion coexistence. Whole fields yield at
// exact narrow thresholds and restore without changing point/policy/source.
#[test]
fn native_workspace() {
    for (format, completion) in [
        (Format::default(), false),
        (Format::default(), true),
        (Format::new(20, 4, 4).unwrap(), false),
        (Format::new(20, 4, 4).unwrap(), true),
    ] {
        workspace_case(format, completion);
    }
}

fn workspace_case(format: Format, completion: bool) {
    let mut session = editor(format);
    let before: Vec<_> = session
        .windows()
        .iter()
        .map(|(id, view)| (id, view.point, view.gutter))
        .collect();
    let source = (
        session.buffer().unwrap().stamp(),
        session.buffer().unwrap().state(),
    );
    if completion {
        session.execute(Command::ExecuteExtended);
        session.handle(Event::Paste("gutter-".into()));
    }
    for (width, height) in [
        (0, 0),
        (1, 1),
        (7, 6),
        (8, 13),
        (18, 13),
        (19, 13),
        (31, 8),
        (32, 8),
        (80, 24),
        (89, 24),
        (90, 24),
        (120, 24),
    ] {
        session.handle(Event::Resize(width, height));
        let size = Size { width, height };
        let frame = session.frame().unwrap();
        assert!(
            frame
                .rows()
                .iter()
                .all(|row| row.text().width() == usize::from(width))
        );
        assert!(
            frame
                .cursor()
                .is_none_or(|p| p.column < width && p.row < height)
        );
        painted(frame, &lanes(&session, size, completion));
        let current: Vec<_> = session
            .windows()
            .iter()
            .map(|(id, view)| (id, view.point, view.gutter))
            .collect();
        assert_eq!(current, before);
    }
    assert_eq!(
        lanes(
            &session,
            Size {
                width: 120,
                height: 24
            },
            completion
        )
        .len(),
        2
    );
    if completion {
        assert!(
            session
                .frame()
                .unwrap()
                .rows()
                .iter()
                .any(|row| row.text().starts_with("M-x gutter-"))
        );
        session.execute(Command::KeyboardQuit);
    }
    assert_eq!(
        (
            session.buffer().unwrap().stamp(),
            session.buffer().unwrap().state()
        ),
        source
    );
    assert!(session.buffer().unwrap().history().is_empty());
    assert!(!session.buffer().unwrap().is_modified());
}
