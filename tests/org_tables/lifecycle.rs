use super::structure::roundtrip;
use super::*;
use rev::{
    command::{RegionCommand, Split, WindowCommand},
    coordinates::{Coordinates, Scalar},
};

// Real structural transactions retain Unicode/tab values, BOM, CRLF, metadata
// and EOF spelling through save and complete one-operation undo/redo.
#[test]
fn structural_format() {
    let source = "Before.\r\n#+NAME: stable\r\n|界e\u{301}|x\tq|\r\n|-+-|\r\n|*bold*|\\vert{}|\r\n#+TBLFM: $2=$1+1\r\n\r\nEOF";
    let (f, mut s) = editor("", &format!("\u{feff}{source}"));
    at(&mut s, source[..source.find('界').unwrap()].chars().count());
    action(&mut s, C::TableMoveColumnRight);
    let after = text(&s);
    assert_eq!(
        super::metadata::values(&s),
        vec!["x\tq", "界e\u{301}", "\\vert{}", "*bold*"]
    );
    assert!(after.starts_with("Before.\r\n#+NAME: stable\r\n"));
    assert!(after.ends_with("#+TBLFM: $1=$2+1\r\n\r\nEOF"));
    assert_eq!(
        after.matches("\r\n").count(),
        source.matches("\r\n").count()
    );
    assert!(
        Coordinates::new(s.buffer().unwrap())
            .unwrap()
            .is_boundary(s.view().point.position)
            .unwrap()
    );
    s.execute(Command::SaveBuffer);
    assert_eq!(
        std::fs::read(f.0.join("notes.org")).unwrap(),
        format!("\u{feff}{after}").as_bytes()
    );
    roundtrip(&mut s, source, &after);
}
// Identical rows keep distinct logical identities. Shared points and the mark
// follow the moved row even when its bytes equal its neighbor's bytes.
#[test]
fn identical_shared_rows() {
    let source = "| a |\n| a |\n| a |\n#+TBLFM: @3$1=@1$1+@2$1\n";
    let (_f, mut s) = editor("", source);
    at(&mut s, 2);
    s.execute(Command::Region(RegionCommand::Set));
    s.execute(Command::KeyboardQuit);
    let peer = s.windows().selected_id();
    s.execute(Command::Split(Split::Right));
    s.execute(Command::Window(WindowCommand::Other));
    action(&mut s, C::TableMoveRowDown);
    let expected = "| a |\n| a |\n| a |\n#+TBLFM: @3$1=@2$1+@1$1\n";
    assert_eq!(text(&s), expected, "{}", s.echo());
    for (history, point) in [
        (None, 8),
        (Some(History::Undo), 2),
        (Some(History::Redo), 8),
    ] {
        if let Some(history) = history {
            s.execute(Command::History(history));
        }
        assert_eq!(s.windows().get(peer).unwrap().point.position.0, point);
        assert_eq!(s.buffer().unwrap().mark(), Some(Scalar(point)));
        assert!(!s.buffer().unwrap().mark_active());
    }
    assert_eq!(text(&s), expected);
    assert_eq!(s.buffer().unwrap().history().len(), 1);
}
// Sorting retargets absolute row references together with values; an unsupported
// later formula line cannot publish an earlier rewrite or any body permutation.
#[test]
fn sorted_formula_transaction() {
    let source = "|B|2|\n|A|1|\n#+TBLFM: @1$2=@2$2+1\n";
    let (_f, mut s) = editor("", source);
    at(&mut s, 1);
    action(&mut s, C::TableSort);
    argument(&mut s, "a");
    roundtrip(
        &mut s,
        source,
        "| A | 1 |\n| B | 2 |\n#+TBLFM: @2$2=@1$2+1\n",
    );
    let source = format!("{source}#+TBLFM: $2=remote(other,$1)\n");
    let (_f, mut s) = editor("", &source);
    at(&mut s, 1);
    let before = s.view().clone();
    action(&mut s, C::TableMoveColumnRight);
    assert!(s.echo().contains("reference rewriting"), "{}", s.echo());
    assert_eq!(text(&s), source);
    assert_eq!(s.view(), &before);
    assert!(s.buffer().unwrap().history().is_empty());
}
// Sort prompts and workers obey the same cancellation/current-source guards as
// other table work, without publishing stale plans or leaving idle polling.
#[test]
fn sort_request_guards() {
    let source = "| B |\n| A |\n";
    let (_f, mut s) = editor("", source);
    at(&mut s, 2);
    let before = s.view().clone();
    action(&mut s, C::TableSort);
    s.execute(Command::KeyboardQuit);
    settle(&mut s);
    assert_eq!(text(&s), source);
    assert_eq!(s.view(), &before);
    assert!(s.buffer().unwrap().history().is_empty());
    assert!(s.remaining_input(Instant::now()).is_none());
    action(&mut s, C::TableSort);
    s.replace(0..0, "changed\n").unwrap();
    argument(&mut s, "a");
    assert!(s.echo().contains("source or view changed"), "{}", s.echo());
    assert_eq!(text(&s), format!("changed\n{source}"));
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    let (_f, mut s) = editor("", source);
    s.execute(Command::Org(C::TableMoveRowDown));
    s.execute(Command::KeyboardQuit);
    settle(&mut s);
    assert_eq!(text(&s), source);
    assert!(s.buffer().unwrap().history().is_empty());
    assert!(s.remaining_input(Instant::now()).is_none());
}
// An out-of-group region, invalid sort mode and bounded formula scans reject
// atomically, preserving source, the full selected view and history.
#[test]
fn structural_rejections() {
    let cases = [
        ("sort mode", String::from("$2=$1"), Some("x"), "Sort type"),
        (
            "depth",
            format!("$2={}1{}", "(".repeat(65), ")".repeat(65)),
            None,
            "depth 64",
        ),
        (
            "tokens",
            format!("$2={}1", "1+".repeat(50000)),
            None,
            "100000 tokens",
        ),
    ];
    for (name, formula, mode, diagnostic) in cases {
        let source = format!("| B | 2 |\n| A | 1 |\n#+TBLFM: {formula}\n");
        let (_f, mut s) = editor("", &source);
        at(&mut s, 2);
        let view = s.view().clone();
        if let Some(mode) = mode {
            action(&mut s, C::TableSort);
            argument(&mut s, mode);
        } else {
            action(&mut s, C::TableMoveColumnRight);
        }
        assert!(s.echo().contains(diagnostic), "{name}: {}", s.echo());
        assert_eq!(text(&s), source, "{name}");
        assert_eq!(s.view(), &view, "{name}");
        assert!(s.buffer().unwrap().history().is_empty());
    }
    let source = "| B |\n|---|\n| A |\n";
    let (_f, mut s) = editor("", source);
    at(&mut s, 2);
    s.execute(Command::Region(RegionCommand::Set));
    at(&mut s, 14);
    let view = s.view().clone();
    action(&mut s, C::TableSort);
    argument(&mut s, "a");
    assert!(s.echo().contains("separator group"), "{}", s.echo());
    assert_eq!(text(&s), source);
    assert_eq!(s.view(), &view);
    assert!(s.buffer().unwrap().mark_active());
    assert!(s.buffer().unwrap().history().is_empty());
}
