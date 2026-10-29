use super::*;
use rev::command::{RegionCommand, Split, WindowCommand};

pub(super) fn roundtrip(s: &mut Session, source: &str, expected: &str) {
    assert_eq!(text(s), expected, "{}", s.echo());
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    s.execute(Command::History(History::Undo));
    assert_eq!(text(s), source);
    s.execute(Command::History(History::Redo));
    assert_eq!(text(s), expected);
}
// Structural row edits retain separators and original EOF spelling. Undo and
// redo restore the whole operation, including alignment, as one history unit.
#[test]
fn row_structure() {
    let cases = [
        (
            "up",
            C::TableMoveRowUp,
            21,
            "| A | B |\n| C | D |\n|---+---|\n",
        ),
        (
            "down",
            C::TableMoveRowDown,
            2,
            "|---+---|\n| A | B |\n| C | D |\n",
        ),
        ("kill data", C::TableKillRow, 2, "|---+---|\n| C | D |\n"),
        ("kill rule", C::TableKillRow, 10, "| A | B |\n| C | D |\n"),
    ];
    let source = "| A | B |\n|---+---|\n| C | D |\n";
    for (name, command, point, expected) in cases {
        let (_f, mut s) = editor("", source);
        at(&mut s, point);
        action(&mut s, command);
        assert!(s.echo().contains("Org table"), "{name}: {}", s.echo());
        roundtrip(&mut s, source, expected);
    }
    let source = "| A |\n| B |";
    let (_f, mut s) = editor("", source);
    at(&mut s, 2);
    action(&mut s, C::TableMoveRowDown);
    roundtrip(&mut s, source, "| B |\n| A |");
    let (_f, mut s) = editor("", "| A |");
    action(&mut s, C::TableKillRow);
    roundtrip(&mut s, "| A |", "");
}
// Every data row participates in column edits, including ragged/empty cells;
// hlines expand but remain separators rather than data that can be overwritten.
#[test]
fn column_structure() {
    let source = "| A | B |\n|---+---|\n| C | D |\n";
    let cases = [
        (
            "right",
            C::TableMoveColumnRight,
            2,
            "| B | A |\n|---+---|\n| D | C |\n",
        ),
        (
            "left",
            C::TableMoveColumnLeft,
            6,
            "| B | A |\n|---+---|\n| D | C |\n",
        ),
        (
            "insert",
            C::TableInsertColumn,
            6,
            "| A |   | B |\n|---+---+---|\n| C |   | D |\n",
        ),
        ("delete", C::TableDeleteColumn, 2, "| B |\n|---|\n| D |\n"),
    ];
    for (name, command, point, expected) in cases {
        let (_f, mut s) = editor("", source);
        at(&mut s, point);
        action(&mut s, command);
        assert!(s.echo().contains("Org table"), "{name}: {}", s.echo());
        roundtrip(&mut s, source, expected);
    }
    let source = "|A||C|\n|D|\n";
    let (_f, mut s) = editor("", source);
    at(&mut s, 4);
    action(&mut s, C::TableMoveColumnLeft);
    roundtrip(&mut s, source, "| A | C |   |\n| D |   |   |\n");
    let (_f, mut s) = editor("", "| A | B |\n");
    s.handle(Event::Key(KeyEvent::new(KeyCode::Right, KeyModifiers::ALT)));
    settle(&mut s);
    assert_eq!(text(&s), "| B | A |\n", "{}", s.echo());
    assert_eq!(s.view().point.position.0, 6);
}
// Stable sort is confined to the current separator group and moves the selected
// row/point, while invalid numeric keys reject the entire source transaction.
#[test]
fn section_sort() {
    let source = "|Name|Count|\n|--+--|\n|b|2|\n|A|3|\n|a|1|\n|--+--|\n|outside|0|\n";
    let (_f, mut s) = editor("", source);
    at(&mut s, source.find("|b|").unwrap() + 1);
    action(&mut s, C::TableSort);
    argument(&mut s, "a");
    roundtrip(
        &mut s,
        source,
        "| Name    | Count |\n|---------+-------|\n| A       |     3 |\n| a       |     1 |\n| b       |     2 |\n|---------+-------|\n| outside |     0 |\n",
    );
    let source = "| X | nope |\n| Y | 2    |\n";
    let (_f, mut s) = editor("", source);
    at(&mut s, 7);
    let before = s.view().clone();
    action(&mut s, C::TableSort);
    argument(&mut s, "n");
    assert!(s.echo().contains("finite numbers"), "{}", s.echo());
    assert_eq!(text(&s), source);
    assert_eq!(s.view(), &before);
    assert!(s.buffer().unwrap().history().is_empty());
}
// Checked one-column conversion retains literal item/cell values and surrounding
// source. Multiline/nested lists or separator-bearing tables are not flattened.
#[test]
fn list_conversion() {
    let source = "Before.\r\n- 界e\u{301}\r\n- *bold*\r\n\r\nEOF";
    let (_f, mut s) = editor("", source);
    at(&mut s, 9);
    action(&mut s, C::ListToTable);
    let expected = "Before.\r\n| 界e\u{301}    |\r\n| *bold* |\r\n\r\nEOF";
    roundtrip(&mut s, source, expected);
    action(&mut s, C::TableToList);
    assert_eq!(text(&s), source, "{}", s.echo());
    let (_f, mut s) = editor("", "- Root\n  - child\n");
    action(&mut s, C::ListToTable);
    assert!(s.echo().contains("flat single-line"), "{}", s.echo());
    assert_eq!(text(&s), "- Root\n  - child\n");
    assert!(s.buffer().unwrap().history().is_empty());
}
// Peers and a shared mark follow the actual moved value, independently of the
// initiating cursor. Undo/redo replay the same canonical source permutation.
#[test]
fn moved_shared_values() {
    let source = "|A|peer|\n|C|D|\n";
    for command in [C::TableMoveColumnLeft, C::TableMoveRowDown] {
        let (_f, mut s) = editor("", source);
        let point = source.find("peer").unwrap() + 2;
        at(&mut s, point);
        s.execute(Command::Region(RegionCommand::Set));
        s.execute(Command::KeyboardQuit);
        let peer = s.windows().selected_id();
        s.execute(Command::Split(Split::Right));
        s.execute(Command::Window(WindowCommand::Other));
        at(&mut s, source.find("peer").unwrap());
        action(&mut s, command);
        let after = text(&s);
        let moved = after.find("peer").unwrap() + 2;
        assert_eq!(
            s.windows().get(peer).unwrap().point.position.0,
            moved,
            "{}",
            s.echo()
        );
        assert_eq!(s.buffer().unwrap().mark().unwrap().0, moved);
        s.execute(Command::History(History::Undo));
        assert_eq!(text(&s), source);
        assert_eq!(s.windows().get(peer).unwrap().point.position.0, point);
        s.execute(Command::History(History::Redo));
        assert_eq!(s.windows().get(peer).unwrap().point.position.0, moved);
    }
}
// Region sorting uses the mark's column and both endpoint rows, leaving other
// rows fixed. Equal numeric keys retain source order; reverse sorting is stable.
#[test]
fn selected_sort() {
    let source = "|hdr|n|\n|-+-|\n|keep|9|\n|A|3|\n|B|1|\n|C|1|\n|tail|0|\n";
    for (mode, order) in [
        ("n", vec!["hdr", "keep", "B", "C", "A", "tail"]),
        ("N", vec!["hdr", "keep", "A", "B", "C", "tail"]),
    ] {
        let (_f, mut s) = editor("", source);
        at(&mut s, source.find("3|").unwrap());
        s.execute(Command::Region(RegionCommand::Set));
        at(&mut s, source.find("C|").unwrap());
        action(&mut s, C::TableSort);
        argument(&mut s, mode);
        assert_eq!(s.echo(), "Org table");
        let values = super::metadata::values(&s);
        assert_eq!(
            values
                .iter()
                .step_by(2)
                .map(String::as_str)
                .collect::<Vec<_>>(),
            order
        );
        let after = text(&s);
        assert!(after[s.view().point.position.0..].starts_with('C'));
        assert_eq!(s.buffer().unwrap().history().len(), 1);
        s.execute(Command::History(History::Undo));
        assert_eq!(text(&s), source);
    }
}
// Numeric zero includes signed zero and empty fields. Both sort directions
// retain their original relative order because their numeric keys are equal.
#[test]
fn equal_numeric_keys() {
    let source = "|minus|-0|\n|empty||\n|plus|0|\n";
    for mode in ["n", "N"] {
        let (_f, mut s) = editor("", source);
        at(&mut s, source.find("-0").unwrap());
        action(&mut s, C::TableSort);
        argument(&mut s, mode);
        assert_eq!(
            super::metadata::values(&s),
            vec!["minus", "-0", "empty", "", "plus", "0"],
            "{mode}: {}",
            s.echo()
        );
    }
}
