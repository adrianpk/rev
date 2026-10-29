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
