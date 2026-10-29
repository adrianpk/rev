use super::*;
use rev::{
    command::{RegionCommand, Split, WindowCommand},
    coordinates::{Coordinates, Scalar},
};

fn pipes(s: &Session) -> Vec<Vec<usize>> {
    let source = text(s);
    let coordinates = Coordinates::new(s.buffer().unwrap()).unwrap();
    let mut columns = vec![Vec::new(); source.lines().count()];
    for (index, _) in source.chars().enumerate().filter(|(_, ch)| *ch == '|') {
        let cell = coordinates.to_cell(Scalar(index)).unwrap();
        columns[cell.row].push(cell.column);
    }
    columns.into_iter().filter(|r| r.len() > 2).collect()
}
// Aligned delimiters must occupy identical real canvas columns with internal
// tabs, wide/combined clusters, ZWJ emoji, controls and a tab-indented prefix.
#[test]
fn displayed_columns() {
    let source = "\t|界e\u{301}|x\tq|\n\t|-+-|\n\t|👩\u{200d}💻|\u{1}z|\n\t|\u{301}|end|\n";
    for width in [4, 8] {
        let (_f, mut s) = editor(&format!("editor.tab_width={width}\n"), source);
        let before = super::metadata::values(&s);
        action(&mut s, C::TableAlign);
        let columns = pipes(&s);
        assert_eq!(columns.len(), 3, "{}", s.echo());
        assert!(columns.windows(2).all(|p| p[0] == p[1]), "{columns:?}");
        assert_eq!(super::metadata::values(&s), before);
        let aligned = text(&s);
        action(&mut s, C::TableAlign);
        assert_eq!(text(&s), aligned);
        let end_field = aligned[..aligned.rfind("end").unwrap()].chars().count();
        at(&mut s, end_field);
        action(&mut s, C::TablePreviousField);
        assert_eq!(s.echo(), "Org table");
        assert!(
            Coordinates::new(s.buffer().unwrap())
                .unwrap()
                .is_boundary(s.view().point.position)
                .unwrap()
        );
        assert_eq!(s.buffer().unwrap().history().len(), 1);
        s.execute(Command::History(History::Undo));
        assert_eq!(text(&s), source);
    }
}
// Canonical padding edits retain BOM, mixed native line endings and EOF bytes
// through actual save, undo and redo, rather than only a rendered comparison.
#[test]
fn source_format() {
    let body = "Before.\r\n|界|x\te\u{301}|\r\n|-+-|\n|Long|👩\u{200d}💻|\r\n\r\nEOF";
    let (f, mut s) = editor("", &format!("\u{feff}{body}"));
    at(&mut s, 9);
    action(&mut s, C::TableAlign);
    let aligned = text(&s);
    assert_ne!(aligned, body, "{}", s.echo());
    assert_eq!(
        aligned.matches("\r\n").count(),
        body.matches("\r\n").count()
    );
    assert_eq!(aligned.matches('\n').count(), body.matches('\n').count());
    assert!(aligned.ends_with("\r\n\r\nEOF"));
    s.execute(Command::SaveBuffer);
    assert_eq!(
        std::fs::read(f.0.join("notes.org")).unwrap(),
        format!("\u{feff}{aligned}").as_bytes()
    );
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), body);
    s.execute(Command::History(History::Redo));
    assert_eq!(text(&s), aligned);
    assert_eq!(s.buffer().unwrap().history().len(), 1);
}
// Shared points inside original values map with their retained source, and
// undo restores each pane's source coordinate without replacing either view.
#[test]
fn shared_positions() {
    let source = "|short|value|\n|longer|peer|\n";
    let (_f, mut s) = editor("", source);
    let peer_at = source.find("peer").unwrap() + 2;
    at(&mut s, peer_at);
    s.execute(Command::Region(RegionCommand::Set));
    s.execute(Command::KeyboardQuit);
    let peer = s.windows().selected_id();
    s.execute(Command::Split(Split::Right));
    s.execute(Command::Window(WindowCommand::Other));
    at(&mut s, 2);
    action(&mut s, C::TableAlign);
    let aligned = text(&s);
    assert_eq!(
        s.windows().get(peer).unwrap().point.position.0,
        aligned.find("peer").unwrap() + 2
    );
    assert_eq!(
        s.buffer().unwrap().mark().unwrap().0,
        aligned.find("peer").unwrap() + 2
    );
    assert!(!s.buffer().unwrap().mark_active());
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), source);
    assert_eq!(s.buffer().unwrap().mark(), Some(Scalar(peer_at)));
    assert_eq!(s.windows().get(peer).unwrap().point.position.0, peer_at);
}
// Editing authority is checked by the real transaction. Read-only motion over
// aligned source still succeeds without history; padding and insertions reject.
#[test]
fn readonly_tables() {
    for source in ["| A | B |\n", "|A|B|\n"] {
        let f = Fixture::new();
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.visit(&f.write("notes.org", source)).unwrap();
        buffers.get_mut(id).unwrap().set_readonly(true);
        let view = View::new(buffers.get(id).unwrap());
        let mut s = Session::new(
            buffers,
            view,
            Size {
                width: 80,
                height: 12,
            },
        )
        .unwrap();
        at(&mut s, 2);
        let before = s.view().clone();
        action(&mut s, C::TableNextField);
        if source.contains(" A ") {
            assert_eq!(s.view().point.position.0, 6, "{}", s.echo());
        } else {
            assert!(s.echo().contains("read-only"), "{}", s.echo());
            assert_eq!(s.view(), &before);
        }
        let before = s.view().clone();
        action(&mut s, C::TableInsertRow);
        assert!(s.echo().contains("read-only"), "{}", s.echo());
        assert_eq!(s.view(), &before);
        assert_eq!(text(&s), source);
        assert!(s.buffer().unwrap().history().is_empty());
    }
}
// A stale create draft or C-g during a prompt/worker cannot publish an old
// table plan. The session returns to ordinary editing and stops idle polling.
#[test]
fn request_guards() {
    let (_f, mut s) = editor("", "\n");
    action(&mut s, C::TableCreate);
    s.replace(0..0, "changed").unwrap();
    argument(&mut s, "2x2");
    assert!(s.echo().contains("source or view changed"), "{}", s.echo());
    assert_eq!(text(&s), "changed\n");
    let (_f, mut s) = editor("", "\n");
    action(&mut s, C::TableCreate);
    s.execute(Command::KeyboardQuit);
    settle(&mut s);
    assert_eq!(text(&s), "\n");
    assert!(s.remaining_input(Instant::now()).is_none());
    let source = "|A|B|\n";
    let (_f, mut s) = editor("", source);
    s.execute(Command::Org(C::TableAlign));
    s.execute(Command::KeyboardQuit);
    settle(&mut s);
    assert_eq!(text(&s), source);
    assert!(s.buffer().unwrap().history().is_empty());
    assert!(s.remaining_input(Instant::now()).is_none());
    s.handle(Event::Paste("ok ".into()));
    assert_eq!(text(&s), "ok |A|B|\n");
}
// Invalid source and bounded output reject before any point/history changes;
// oversized dimensions are rejected before allocating a table's cells.
#[test]
fn rejection_bounds() {
    let cases = [
        ("bad rule", "|A|\n|--oops|\n".into(), "separator"),
        (
            "conflicting cookies",
            "|<l>|\n|<r>|\n".into(),
            "Conflicting",
        ),
        ("wide cell", format!("|{}|\n", "界".repeat(2049)), "4096"),
        ("columns", format!("|{}\n", "x|".repeat(257)), "256 columns"),
    ];
    for (name, source, diagnostic) in cases {
        let (_f, mut s) = editor("", &source);
        let before = s.view().clone();
        action(&mut s, C::TableAlign);
        assert!(s.echo().contains(diagnostic), "{name}: {}", s.echo());
        assert_eq!(text(&s), source, "{name}");
        assert_eq!(s.view(), &before, "{name}");
        assert!(s.buffer().unwrap().history().is_empty());
    }
    for dimensions in [
        "257x1",
        "1x10000",
        "256x9999",
        "1x0",
        "9".repeat(33).as_str(),
    ] {
        let (_f, mut s) = editor("", "");
        action(&mut s, C::TableCreate);
        argument(&mut s, dimensions);
        assert_eq!(text(&s), "", "{dimensions}: {}", s.echo());
        assert!(s.buffer().unwrap().history().is_empty());
    }
}
// RET at table line boundaries and inside literal lookalikes keeps ordinary
// newline semantics. Empty separator-only tables can become usable data rows.
#[test]
fn return_and_empty_rows() {
    for (source, point, expected) in [
        ("text", 2, "te\nxt"),
        ("| A |\n", 0, "\n| A |\n"),
        ("| A |\n", 5, "| A |\n\n"),
        (
            "#+begin_example\n|A|\n#+end_example",
            17,
            "#+begin_example\n|\nA|\n#+end_example",
        ),
    ] {
        let (_f, mut s) = editor("", source);
        at(&mut s, point);
        action(&mut s, C::Return);
        assert_eq!(text(&s), expected, "{}", s.echo());
        s.execute(Command::History(History::Undo));
        assert_eq!(text(&s), source);
    }
    let (_f, mut s) = editor("", "|-+-|\n");
    action(&mut s, C::TableNextField);
    assert_eq!(text(&s), "|---+---|\n|   |   |\n", "{}", s.echo());
    assert_eq!(s.view().point.position.0, 12);
}
// A real table edit retires old fold ranges in every shared view. Undo/redo
// restore canonical source and points without reviving obsolete fold authority.
#[test]
fn fold_publication() {
    let source = "|A|B|\n\n* H\nbody\n* Next\n";
    let f = Fixture::new();
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&f.write("notes.org", source)).unwrap();
    let buffer = buffers.get(id).unwrap();
    let mut view = View::new(buffer);
    let hidden = rev::org::Source::new(buffer)
        .range(rev::coordinates::Byte(11)..rev::coordinates::Byte(16))
        .unwrap();
    view.set_folds(buffer, &[hidden]).unwrap();
    let mut s = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 18,
        },
    )
    .unwrap();
    assert!(s.view().visible_lines(s.buffer().unwrap()) < s.buffer().unwrap().len_lines());
    s.execute(Command::Split(Split::Right));
    action(&mut s, C::TableAlign);
    let aligned = text(&s);
    assert_ne!(aligned, source, "{}", s.echo());
    for history in [None, Some(History::Undo), Some(History::Redo)] {
        if let Some(history) = history {
            s.execute(Command::History(history));
        }
        let buffer = s.buffer().unwrap();
        for (_, view) in s.windows().iter() {
            assert_eq!(view.point.stamp, buffer.stamp());
            assert_eq!(view.visible_lines(buffer), buffer.len_lines());
        }
    }
    assert_eq!(text(&s), aligned);
    assert_eq!(s.buffer().unwrap().history().len(), 1);
}
