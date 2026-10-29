use super::*;
use rev::{
    command::{RegionCommand, Split, WindowCommand},
    coordinates::Scalar,
};

// A real file keeps BOM, CRLF, EOF, metadata and unrelated Unicode source.
// Canonical affinity maps a peer view and inactive mark through calculation,
// ordinary undo and redo; saving uses the existing file owner.
#[test]
fn calculation_file_views() {
    let source = "Before 界.\r\n#+NAME: stable\r\n|Input|Result|\r\n|-----+------|\r\n|2|old|\r\n|3|old|\r\n#+TBLFM: $2=$1*2\r\n\r\nEOF";
    let expected = source
        .replacen("|old|", "|4|", 1)
        .replacen("|old|", "|6|", 1);
    let (f, mut s) = editor("", &format!("\u{feff}{source}"));
    at(
        &mut s,
        source[..source.find("EOF").unwrap()].chars().count(),
    );
    s.execute(Command::Region(RegionCommand::Set));
    s.execute(Command::KeyboardQuit);
    s.execute(Command::Split(Split::Right));
    let peer = s.windows().selected_id();
    s.execute(Command::Window(WindowCommand::Other));
    at(
        &mut s,
        source[..source.find("|2|").unwrap()].chars().count() + 1,
    );
    let origin = s.view().point.position;
    action(&mut s, C::TableRecalculate);
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    for (operation, current) in [
        (None, expected.as_str()),
        (Some(History::Undo), source),
        (Some(History::Redo), expected.as_str()),
    ] {
        if let Some(operation) = operation {
            s.execute(Command::History(operation));
        }
        assert_eq!(text(&s), current, "{}", s.echo());
        let eof = Scalar(current[..current.find("EOF").unwrap()].chars().count());
        assert_eq!(s.windows().get(peer).unwrap().point.position, eof);
        assert_eq!(s.buffer().unwrap().mark(), Some(eof));
        assert!(!s.buffer().unwrap().mark_active());
        assert_eq!(s.view().point.position, origin);
    }
    s.execute(Command::SaveBuffer);
    assert_eq!(
        std::fs::read(f.0.join("notes.org")).unwrap(),
        format!("\u{feff}{expected}").as_bytes()
    );
    assert!(!s.buffer().unwrap().is_modified());
}

// Cancellation and source/view changes invalidate the actual asynchronous
// request. Only the deliberate intervening edit may enter source history.
#[test]
fn calculation_request_guards() {
    let source = "|2|old|\n#+TBLFM: $2=$1*2\n";
    for change in ["cancel", "source", "view"] {
        let (_f, mut s) = editor("", source);
        let before = s.view().clone();
        s.execute(Command::Org(C::TableRecalculate));
        match change {
            "cancel" => {
                s.execute(Command::KeyboardQuit);
            }
            "source" => {
                s.replace(0..0, "changed\n").unwrap();
            }
            "view" => {
                s.execute(Command::Motion(Motion::BufferEnd));
            }
            _ => unreachable!(),
        }
        settle(&mut s);
        let expected = if change == "source" {
            format!("changed\n{source}")
        } else {
            source.into()
        };
        assert_eq!(text(&s), expected, "{change}: {}", s.echo());
        assert_eq!(
            s.buffer().unwrap().history().len(),
            usize::from(change == "source"),
            "{change}"
        );
        if change == "cancel" {
            assert_eq!(s.view(), &before);
        }
        assert!(s.remaining_input(Instant::now()).is_none(), "{change}");
    }
}

// Missing/unsupported precise contexts do not select a neighboring field or
// fall through to a destructive ancestor. The view and history remain intact.
#[test]
fn calculation_target_guards() {
    for (name, source, command, needle, diagnostic) in [
        (
            "no formula",
            "|2|old|\n",
            C::TableRecalculate,
            "|2|",
            "No TBLFM",
        ),
        (
            "header",
            "|A|B|\n|-+-|\n|2|old|\n#+TBLFM: $2=$1*2\n",
            C::TableRecalculateCell,
            "B",
            "No formula targets",
        ),
        (
            "separator",
            "|A|B|\n|-+-|\n|2|old|\n#+TBLFM: $2=$1*2\n",
            C::TableRecalculateColumn,
            "|-+-|",
            "Select an existing data field",
        ),
        (
            "unassigned",
            "|2|old|\n#+TBLFM: $2=$1*2\n",
            C::TableRecalculateCell,
            "2",
            "No formula targets",
        ),
        (
            "formula cell",
            "|2|old|\n#+TBLFM: $2=$1*2\n",
            C::TableRecalculateCell,
            "#+TBLFM",
            "Select an existing data field",
        ),
        (
            "literal",
            "* Parent\n#+begin_example\n|2|old|\n#+TBLFM: $2=$1*2\n#+end_example\n",
            C::TableRecalculate,
            "|2|",
            "No native Org table",
        ),
    ] {
        let (_f, mut s) = editor("", source);
        at(
            &mut s,
            source[..source.find(needle).unwrap()].chars().count(),
        );
        let view = s.view().clone();
        action(&mut s, command);
        assert!(s.echo().contains(diagnostic), "{name}: {}", s.echo());
        assert_eq!(text(&s), source, "{name}");
        assert_eq!(s.view(), &view, "{name}");
        assert!(s.buffer().unwrap().history().is_empty(), "{name}");
    }
}
