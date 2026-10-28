use std::ops::Range;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::edit::{History, Replacement};
use rev::buffer::{Buffer, Buffers};
use rev::command::Command;
use rev::coordinates::Byte;
use rev::org::{Coverage, Limits, Scan, Source};
use rev::session::Session;
use rev::view::{Size, View};

use super::index::complete;

pub(super) fn replace(buffer: &mut Buffer, bytes: Range<usize>, text: &str) {
    let edits = [Replacement {
        source: buffer.byte_span(bytes).unwrap(),
        text: text.into(),
    }];
    let prepared = buffer.prepare(&edits, &[], None).unwrap();
    buffer.publish(prepared, &mut []).unwrap();
}

fn install(buffer: &mut Buffer) {
    let index = complete(Scan::new(buffer, Limits::default()).unwrap());
    buffer.install_org_index(index).unwrap();
}

fn resume(buffer: &mut Buffer) {
    let index = buffer.take_org_index().unwrap();
    let index = complete(Scan::resume(index, buffer).unwrap());
    buffer.install_org_index(index).unwrap();
}

fn rows(buffer: &Buffer) -> Vec<String> {
    let index = buffer.org_index().unwrap();
    (0..index.len())
        .map(|row| {
            let line = index.line(buffer, row).unwrap();
            Source::new(buffer).text(&line.source).unwrap().to_string()
        })
        .collect()
}

// Canonical edits retain only the unaffected prefix. Rebuilding a nested tail
// must recover its preceding ancestors without cloning or reparsing that prefix.
#[test]
fn retained_prefix() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers
        .create("* First\nbody\n* Second\nmore\n** Child\nlast\n")
        .unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    install(buffer);
    let old = buffer.org_index().unwrap().line(buffer, 0).unwrap().source;
    replace(buffer, 37..38, "changed");
    let index = buffer.org_index().unwrap();
    assert_eq!(index.stamp(), buffer.stamp());
    assert_eq!(index.coverage(), Coverage::Pending);
    assert_eq!(index.len(), 4);
    assert_eq!(index.covered_bytes(), Byte(27));
    assert!(Source::new(buffer).text(&old).is_err());
    assert_eq!(
        rows(buffer),
        ["* First\n", "body\n", "* Second\n", "more\n"]
    );
    resume(buffer);
    assert_eq!(
        rows(buffer),
        [
            "* First\n",
            "body\n",
            "* Second\n",
            "more\n",
            "** Child\n",
            "lchangedst\n",
            ""
        ]
    );
    let index = buffer.org_index().unwrap();
    assert_eq!(index.line(buffer, 4).unwrap().parent, Some(2));
    assert_eq!(index.line(buffer, 5).unwrap().parent, Some(4));
}

// Empty EOF rows, split/join seams and CRLF/Unicode edits must invalidate the
// correct source scope rather than retaining an obsolete or duplicate line.
#[test]
fn edit_seams() {
    let cases: [(&str, &str, Range<usize>, &str, &[&str]); 6] = [
        ("empty insertion", "", 0..0, "* Root", &["* Root"]),
        (
            "append after LF",
            "* Root\n",
            7..7,
            "tail",
            &["* Root\n", "tail"],
        ),
        (
            "split EOF",
            "* Root",
            6..6,
            "\r\nbody",
            &["* Root\r\n", "body"],
        ),
        ("join CRLF", "first\r\nsecond", 5..7, "", &["firstsecond"]),
        ("erase document", "* Root\r\nbody", 0..12, "", &[""]),
        (
            "UTF-8 seam",
            "* 界\r\nbody",
            2..5,
            "λ",
            &["* λ\r\n", "body"],
        ),
    ];
    let mut buffers = Buffers::new().unwrap();
    for (name, text, range, replacement, expected) in cases {
        let id = buffers.create(text).unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        install(buffer);
        replace(buffer, range, replacement);
        resume(buffer);
        assert_eq!(rows(buffer), expected, "{name}");
        assert_eq!(
            buffer.org_index().unwrap().coverage(),
            Coverage::Complete,
            "{name}"
        );
    }
}

// Parser edits arrive in descending byte order. Invalidation must use the
// earliest base mutation rather than retaining stale rows between two changes.
#[test]
fn disjoint_edits() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("* One\nx\n* Two\ny\n* Three\nz\n").unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    install(buffer);
    let edits = [
        Replacement {
            source: buffer.byte_span(6..7).unwrap(),
            text: "X".into(),
        },
        Replacement {
            source: buffer.byte_span(24..25).unwrap(),
            text: "Z".into(),
        },
    ];
    let prepared = buffer.prepare(&edits, &[], None).unwrap();
    buffer.publish(prepared, &mut []).unwrap();
    assert!(buffer.org_index().unwrap().is_empty());
    resume(buffer);
    assert_eq!(
        rows(buffer),
        ["* One\n", "X\n", "* Two\n", "y\n", "* Three\n", "Z\n", ""]
    );
}

// Native typing can merge history records, but each exact publication must
// invalidate a current index independently. Undo/redo retain ordinary grouping.
#[test]
fn typing_history() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.initial();
    install(buffers.get_mut(id).unwrap());
    let view = View::new(buffers.get(id).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap();
    for character in "* Title".chars() {
        session.handle(Event::Key(KeyEvent::new(
            KeyCode::Char(character),
            KeyModifiers::NONE,
        )));
        let buffer = session.buffer().unwrap();
        assert_eq!(buffer.org_index().unwrap().stamp(), buffer.stamp());
        assert_eq!(buffer.org_index().unwrap().coverage(), Coverage::Pending);
    }
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    assert!(!session.buffer().unwrap().is_modified());
    session.execute(Command::History(History::Redo));
    let buffer = session.buffer().unwrap();
    assert_eq!(buffer.org_index().unwrap().stamp(), buffer.stamp());
    let text = buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap();
    assert_eq!(text.to_string(), "* Title");
}

// Analyze the selected 1MiB/10000-heading fixture, replace a large tail and
// replay it through history. Only the affected suffix loses index coverage.
#[test]
fn large_edit() {
    let entry = format!("* Heading\n{}\n", "x".repeat(90));
    let mut text = entry.repeat(10_000);
    text.extend(std::iter::repeat_n('x', 1_048_576 - text.len()));
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(&text).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    install(buffer);
    assert_eq!(buffer.org_index().unwrap().len(), 20_001);
    assert_eq!(buffer.org_index().unwrap().covered_bytes(), Byte(1_048_576));
    let start = entry.len() * 9900;
    replace(buffer, start..text.len(), "* Tail\n");
    assert_eq!(buffer.org_index().unwrap().len(), 19_798);
    resume(buffer);
    assert_eq!(buffer.org_index().unwrap().len(), 19_802);
    let expected = format!("{}* Tail\n", &text[..start]);
    assert_eq!(rows(buffer).concat(), expected);
    buffer.navigate(History::Undo, &mut [], None).unwrap();
    resume(buffer);
    assert_eq!(rows(buffer).concat(), text);
    buffer.navigate(History::Redo, &mut [], None).unwrap();
    resume(buffer);
    assert_eq!(rows(buffer).concat(), expected);
}

// A snapshot may outlive edits, but neither index publication nor resume can
// relabel its locations as belonging to another document or newer revision.
#[test]
fn stale_index() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("* Old\n").unwrap();
    let other = buffers.create("* Old\n").unwrap();
    let index = complete(Scan::new(buffers.get(id).unwrap(), Limits::default()).unwrap());
    assert!(
        buffers
            .get_mut(other)
            .unwrap()
            .install_org_index(index)
            .is_err()
    );
    let buffer = buffers.get_mut(id).unwrap();
    let scan = Scan::new(buffer, Limits::default()).unwrap();
    let publication = Scan::new(buffer, Limits::default()).unwrap();
    replace(buffer, 2..5, "New");
    let old = complete(scan);
    assert!(old.line(buffer, 0).is_err());
    assert!(Scan::resume(old, buffer).is_err());
    assert!(buffer.install_org_index(complete(publication)).is_err());
    assert!(buffer.org_index().is_none());
}
