use rev::buffer::Buffers;
use rev::command::{Command, Whitespace};
use rev::coordinates::Coordinates;
use rev::session::Session;
use rev::view::{Size, View};

use super::{select, text};

pub(super) fn configured(source: &str, width: usize, readonly: bool) -> Session {
    let fixture = crate::config_fixture::Fixture::new();
    let startup =
        rev::config::prepare(&fixture.write("config.toml", &format!("editor.tab_width = {width}")));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.create(source).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.set_readonly(readonly);
    let view = View::new(buffer);
    Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 12,
        },
    )
    .unwrap()
}

fn columns(session: &Session) -> Vec<(usize, usize, String)> {
    let buffer = session.buffer().unwrap();
    let coordinates = Coordinates::new(buffer).unwrap();
    let mut result = Vec::new();
    for row in 0..buffer.len_lines() {
        for glyph in coordinates.glyphs(row).unwrap() {
            let glyph = glyph.unwrap();
            let source = buffer
                .span(glyph.source.start.0..glyph.source.end.0)
                .unwrap();
            let text = buffer.slice(&source).unwrap();
            if !matches!(text.as_str(), Some(" " | "\t")) {
                result.push((
                    row,
                    glyph.columns.start,
                    text.chars()
                        .filter(|character| !matches!(character, ' ' | '\t'))
                        .collect(),
                ));
            }
        }
    }
    result
}

// Explicit expected bytes distinguish actual prefix columns from region-local
// columns. Untouched tabs, graphemes, partial runs and line separators survive.
#[test]
fn selected_columns() {
    for (name, operation, width, source, region, expected) in [
        (
            "wide expansion",
            Whitespace::Untabify,
            8,
            "界e\u{301}\tZ\t",
            3..4,
            "界e\u{301}     Z\t",
        ),
        (
            "prefix spaces",
            Whitespace::Untabify,
            4,
            "x \tY",
            2..3,
            "x   Y",
        ),
        (
            "wide compression",
            Whitespace::Tabify,
            8,
            "界e\u{301}     Z",
            3..8,
            "界e\u{301}\tZ",
        ),
        (
            "internal runs",
            Whitespace::Tabify,
            4,
            "a   X    Y",
            1..9,
            "a\tX\t Y",
        ),
        ("mixed run", Whitespace::Tabify, 4, "a \t X", 1..4, "a\t X"),
        (
            "partial spaces",
            Whitespace::Tabify,
            4,
            "a    Z",
            2..5,
            "a \t Z",
        ),
        (
            "no shorter form",
            Whitespace::Tabify,
            4,
            "a  Z",
            1..3,
            "a  Z",
        ),
        (
            "one-cell stops",
            Whitespace::Tabify,
            1,
            "   \tZ",
            0..4,
            "   \tZ",
        ),
        ("one-cell tab", Whitespace::Untabify, 1, "a\tZ", 1..2, "a Z"),
        (
            "space grapheme",
            Whitespace::Tabify,
            4,
            " \u{301}    X",
            0..7,
            " \u{301}\t X",
        ),
        (
            "mixed separators",
            Whitespace::Untabify,
            4,
            "x\t\r\n界\tz\nend",
            1..6,
            "x   \r\n界  z\nend",
        ),
        (
            "unselected suffix",
            Whitespace::Untabify,
            4,
            "x\t ",
            1..2,
            "x    ",
        ),
        (
            "tab before combining",
            Whitespace::Untabify,
            4,
            "\t\u{301}X",
            0..1,
            "     \u{301}X",
        ),
    ] {
        let mut session = configured(source, width, false);
        let before = columns(&session);
        let policy = session.buffer().unwrap().policy().clone();
        select(&mut session, region.start, region.end);
        session.execute(Command::Whitespace(operation));
        assert_eq!(text(&session), expected, "{name}");
        assert_eq!(columns(&session), before, "{name}");
        assert_eq!(session.buffer().unwrap().policy(), &policy, "{name}");
    }
}

// Every non-whitespace glyph keeps its real column across tab widths, Unicode
// and mixed runs. Reapplying an already minimal/expanded run creates no edit.
#[test]
fn policy_grid() {
    for width in [1, 2, 4, 8, 16, 32] {
        for (prefix, run) in [("a", " "), ("界e\u{301}", "    "), ("\t界", " \t \t  ")] {
            for operation in [Whitespace::Tabify, Whitespace::Untabify] {
                policy_case(width, prefix, run, operation);
            }
        }
    }
}

fn policy_case(width: usize, prefix: &str, run: &str, operation: Whitespace) {
    let source = format!("{prefix}{run}終");
    let mut session = configured(&source, width, false);
    let before = columns(&session);
    let start = prefix.chars().count();
    select(&mut session, start, start + run.len());
    session.execute(Command::Whitespace(operation));
    let converted = text(&session);
    assert!(converted.starts_with(prefix) && converted.ends_with('終'));
    assert_eq!(
        columns(&session),
        before,
        "{width}: {prefix:?} {run:?} {operation:?}"
    );
    let end = converted.chars().count() - 1;
    select(&mut session, start, end);
    let stamp = session.buffer().unwrap().stamp();
    let history = session.buffer().unwrap().history().len();
    session.execute(Command::Whitespace(operation));
    assert_eq!(text(&session), converted);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(session.buffer().unwrap().history().len(), history);
}
