use super::*;
// Real local CSV/TSV files enter the selected canonical Org buffer through one
// native edit and leave through checked publication without acknowledging a save.
#[test]
fn interchange_roundtrip() {
    for (name, extension, input, expected) in [
        (
            "csv",
            "csv",
            "Name,Value\r\n\"Ada, 界\",\"He said \"\"Hi\"\"\"\r\n",
            "\"Name\",\"Value\"\n\"Ada, 界\",\"He said \"\"Hi\"\"\"\n",
        ),
        (
            "tsv",
            "tsv",
            "Name\tValue\n\"Ada\t界\"\t\"a\"\"b\"",
            "\"Name\"\t\"Value\"\n\"Ada\t界\"\t\"a\"\"b\"\n",
        ),
    ] {
        let (fixture, mut session) = editor("", "");
        fixture.write(&format!("input.{extension}"), input);
        action(&mut session, C::TableImport);
        argument(&mut session, &format!("input.{extension}"));
        let imported = text(&session);
        assert!(imported.starts_with("| Name"), "{name}: {}", session.echo());
        assert_eq!(session.buffer().unwrap().history().len(), 1, "{name}");
        assert_eq!(std::fs::read(fixture.0.join("notes.org")).unwrap(), b"");
        at(&mut session, 0);
        let before = session.view().clone();
        action(&mut session, C::TableExport);
        argument(&mut session, &format!("output.{extension}"));
        assert!(
            session.echo().starts_with("Exported"),
            "{name}: {}",
            session.echo()
        );
        assert_eq!(
            std::fs::read_to_string(fixture.0.join(format!("output.{extension}"))).unwrap(),
            expected,
            "{name}"
        );
        assert_eq!(session.view(), &before, "{name}");
        assert_eq!(text(&session), imported, "{name}");
        assert!(session.buffer().unwrap().is_modified(), "{name}");
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), "", "{name}");
        session.execute(Command::History(History::Redo));
        assert_eq!(text(&session), imported, "{name}");
    }
}

// Each late parse/representation failure leaves the complete canonical source,
// point and history intact; the earlier successful records cannot leak through.
#[test]
fn interchange_rejections() {
    for (name, input, diagnostic) in [
        ("open quote", "A,B\n\"unfinished", "Unterminated"),
        ("bare quote", "A,B\nab\"c,x", "Malformed"),
        ("closed tail", "A,B\n\"a\" x,b", "Malformed"),
        ("quoted LF", "A,B\n\"a\nb\",c", "line breaks"),
        ("quoted CR", "A,B\n\"a\rb\",c", "line breaks"),
        ("Unicode line", "A,B\n\"a\u{2028}b\",c", "line breaks"),
        ("Unicode paragraph", "A,B\n\"a\u{2029}b\",c", "line breaks"),
        ("record CR", "A,B\ra,b", "LF or CRLF"),
        ("leading space", "A,B\n\" a\",b", "whitespace"),
        ("trailing tab", "A,B\n\"a\t\",b", "whitespace"),
        ("literal bar", "A,B\na|b,c", "losslessly"),
        ("control", "A,B\na\u{1b}b,c", "losslessly"),
        ("ragged", "A,B\na", "Ragged"),
        ("empty", "", "empty"),
    ] {
        let (fixture, mut session) = editor("", "\n* Keep 界\nEOF");
        fixture.write("input.csv", input);
        let before = text(&session);
        let view = session.view().clone();
        action(&mut session, C::TableImport);
        argument(&mut session, "input.csv");
        assert!(
            session.echo().contains(diagnostic),
            "{name}: {}",
            session.echo()
        );
        assert_eq!(text(&session), before, "{name}");
        assert_eq!(session.view(), &view, "{name}");
        assert!(session.buffer().unwrap().history().is_empty(), "{name}");
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("input.csv")).unwrap(),
            input,
            "{name}"
        );
    }
}

// Row, column, grid and display limits are exercised through real native file
// imports, retaining ordinary editing after rejection rather than truncation.
#[test]
fn interchange_limits() {
    for (name, input, diagnostic) in [
        ("rows", "a\n".repeat(10001), "10000 rows"),
        ("columns", vec!["a"; 257].join(","), "256 columns"),
        (
            "cells",
            (vec!["a"; 251].join(",") + "\n").repeat(400),
            "100000 cells",
        ),
        ("width", "a".repeat(4097), "4096 displayed"),
    ] {
        let (fixture, mut session) = editor("", "");
        fixture.write("input.csv", &input);
        action(&mut session, C::TableImport);
        argument(&mut session, "input.csv");
        assert!(
            session.echo().contains(diagnostic),
            "{name}: {}",
            session.echo()
        );
        assert_eq!(text(&session), "", "{name}");
        assert!(session.buffer().unwrap().history().is_empty(), "{name}");
        session.handle(Event::Paste("ordinary".into()));
        assert_eq!(text(&session), "ordinary", "{name}");
    }
}

// The parsed native model supplies an independent observation of imported
// values, including empty fields, literal quotes, non-ASCII spaces and markup.
#[test]
fn interchange_values() {
    let (fixture, mut session) = editor("", "");
    let input =
        "\u{feff},\"\"\"\",\"a,b\",\"a\tb\",\"\u{a0}界\u{a0}\",\"*bold*\",---,\"e\u{301}\"\n";
    fixture.write("input.csv", input);
    action(&mut session, C::TableImport);
    argument(&mut session, "input.csv");
    assert_eq!(
        super::metadata::values(&session),
        [
            "",
            "\"",
            "a,b",
            "a\tb",
            "\u{a0}界\u{a0}",
            "*bold*",
            "---",
            "e\u{301}"
        ],
        "{}",
        session.echo()
    );
    at(&mut session, 0);
    action(&mut session, C::TableExport);
    argument(&mut session, "output.csv");
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("output.csv")).unwrap(),
        "\"\",\"\"\"\",\"a,b\",\"a\tb\",\"\u{a0}界\u{a0}\",\"*bold*\",\"---\",\"e\u{301}\"\n"
    );
}

// A one-column data file participates in both existing native conversions.
// Multi-column, separator and formula-bearing transformations remain rejected.
#[test]
fn interchange_conversion() {
    let (fixture, mut session) = editor("", "");
    fixture.write("input.csv", "\"a,b\"\n\"界\"\n");
    action(&mut session, C::TableImport);
    argument(&mut session, "input.csv");
    at(&mut session, 0);
    action(&mut session, C::TableToList);
    assert_eq!(text(&session), "- a,b\n- 界");
    action(&mut session, C::ListToTable);
    action(&mut session, C::TableExport);
    argument(&mut session, "converted.csv");
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("converted.csv")).unwrap(),
        "\"a,b\"\n\"界\"\n"
    );
    for (name, source) in [
        ("columns", "|a|b|\n"),
        ("separator", "|a|\n|---|\n"),
        ("formula", "|a|\n#+TBLFM: $1=2\n"),
    ] {
        let (_fixture, mut session) = editor("", source);
        action(&mut session, C::TableToList);
        assert_eq!(text(&session), source, "{name}");
        assert!(session.buffer().unwrap().history().is_empty(), "{name}");
    }
}

// Canonical write authority rejects import even after valid file preparation;
// a read-only table can still export data without changing its state/history.
#[test]
fn interchange_readonly() {
    let fixture = Fixture::new();
    let source = "\n|A|\n";
    fixture.write("input.csv", "X\n");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&fixture.write("notes.org", source)).unwrap();
    buffers.get_mut(id).unwrap().set_readonly(true);
    let view = View::new(buffers.get(id).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 18,
        },
    )
    .unwrap();
    action(&mut session, C::TableImport);
    argument(&mut session, "input.csv");
    assert!(session.echo().contains("read-only"), "{}", session.echo());
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
    at(&mut session, 1);
    action(&mut session, C::TableExport);
    argument(&mut session, "output.csv");
    assert!(session.echo().starts_with("Exported"), "{}", session.echo());
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("output.csv")).unwrap(),
        "\"A\"\n"
    );
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
    assert!(!session.buffer().unwrap().is_modified());
}
