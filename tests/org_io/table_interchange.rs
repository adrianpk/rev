use super::*;
use rev::command::{History, Split, WindowCommand};
use std::{fs, os::unix::fs::PermissionsExt};
fn editor(source: &str) -> (Fixture, Session) {
    let fixture = Fixture::new();
    let path = fixture.write("source.org", source);
    let session = host(&fixture, Some(&path), false);
    (fixture, session)
}
fn select_line(session: &mut Session, line: &str) {
    session.execute(Command::GotoLine);
    argument(session, line);
}
fn export(session: &mut Session, path: &str) {
    action(session, C::TableExport);
    argument(session, path);
}
// Actual import/save preserves the existing source's BOM, CRLF, Unicode, EOF
// and peer position. The input file remains byte-identical through undo/redo.
#[test]
fn table_import_format() {
    let source = "\u{feff}* Keep 界\r\n\r\nEOF";
    let (fixture, mut session) = editor(source);
    let input = "A,B\r\n\"é,界\",2\r\n";
    fixture.write("input.csv", input);
    session.execute(Command::Split(Split::Right));
    let peer = session.windows().selected_id();
    select_line(&mut session, "3");
    session.execute(Command::Window(WindowCommand::Other));
    select_line(&mut session, "2");
    action(&mut session, C::TableImport);
    argument(&mut session, "input.csv");
    let expected = "* Keep 界\r\n| A    | B |\r\n| é,界 | 2 |\r\nEOF";
    assert_eq!(text(&session), expected, "{}", session.echo());
    assert!(session.buffer().unwrap().has_bom());
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    assert_eq!(
        fs::read_to_string(fixture.0.join("source.org")).unwrap(),
        source
    );
    session.select_view(peer).unwrap();
    assert_eq!(
        session.view().point.position.0,
        expected.chars().count() - 3
    );
    session.execute(Command::SaveBuffer);
    assert_eq!(
        fs::read_to_string(fixture.0.join("source.org")).unwrap(),
        format!("\u{feff}{expected}")
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), &source[3..]);
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), expected);
    assert_eq!(
        fs::read_to_string(fixture.0.join("input.csv")).unwrap(),
        input
    );
}

// Overwrite consent is bound to observed bytes. Decline, cancellation and a
// changed target preserve it; successful replacement preserves permissions.
#[test]
fn table_export_replacement() {
    let source = "#+NAME: data\n|A|B|\n|-+-|\n|界|2|\n#+TBLFM: $2=1/0\n";
    let (fixture, mut session) = editor(source);
    select_line(&mut session, "2");
    let output = fixture.write("output.csv", "prior\n");
    fs::set_permissions(&output, fs::Permissions::from_mode(0o640)).unwrap();
    export(&mut session, "output.csv");
    assert!(
        session.echo().contains("Confirm replacement"),
        "{}",
        session.echo()
    );
    argument(&mut session, "no");
    assert_eq!(fs::read_to_string(&output).unwrap(), "prior\n");
    export(&mut session, "output.csv");
    cancel(&mut session);
    assert_eq!(fs::read_to_string(&output).unwrap(), "prior\n");
    export(&mut session, "output.csv");
    fs::write(&output, "external replacement\n").unwrap();
    argument(&mut session, "yes");
    assert!(
        session.echo().contains("not published"),
        "{}",
        session.echo()
    );
    assert_eq!(
        fs::read_to_string(&output).unwrap(),
        "external replacement\n"
    );
    export(&mut session, "output.csv");
    argument(&mut session, "yes");
    assert!(session.echo().starts_with("Exported"), "{}", session.echo());
    assert_eq!(
        fs::read_to_string(&output).unwrap(),
        "\"A\",\"B\"\n\"界\",\"2\"\n"
    );
    assert_eq!(
        fs::metadata(&output).unwrap().permissions().mode() & 0o777,
        0o640
    );
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
    assert!(!session.buffer().unwrap().is_modified());
    assert_eq!(
        fs::read_to_string(fixture.0.join("source.org")).unwrap(),
        source
    );
    assert_eq!(fs::read_dir(&fixture.0).unwrap().count(), 3);
}

// An open canonical input supplies unsaved data through a hard-link alias;
// export refuses those same owners rather than overwriting their disk backing.
#[test]
fn table_canonical_files() {
    let (fixture, mut session) = editor("");
    let input = fixture.write("input.csv", "disk\n");
    let alias = fixture.0.join("alias.csv");
    fs::hard_link(&input, &alias).unwrap();
    session.execute(Command::Split(Split::Right));
    session.load(&input).unwrap();
    session.handle(Event::Paste("unsaved,".into()));
    let live = text(&session);
    session.execute(Command::Window(WindowCommand::Other));
    action(&mut session, C::TableImport);
    argument(&mut session, "alias.csv");
    assert!(text(&session).contains("unsaved"), "{}", session.echo());
    assert!(text(&session).contains("disk"));
    for path in ["input.csv", "alias.csv"] {
        export(&mut session, path);
        assert!(
            session.echo().contains("live buffer"),
            "{path}: {}",
            session.echo()
        );
        assert_eq!(fs::read_to_string(&input).unwrap(), "disk\n");
    }
    session.execute(Command::Window(WindowCommand::Other));
    assert_eq!(text(&session), live);
    assert!(session.buffer().unwrap().is_modified());
}

// Invalid encoding/binary/non-regular/missing files and absent parents cannot
// import or publish partial data; failures leave no temporary files behind.
#[test]
fn table_file_failures() {
    for (name, bytes, diagnostic) in [
        ("encoding", vec![0xff], "UTF-8"),
        ("binary", vec![0], "NUL"),
    ] {
        let (fixture, mut session) = editor("");
        fs::write(fixture.0.join("input.csv"), bytes).unwrap();
        action(&mut session, C::TableImport);
        argument(&mut session, "input.csv");
        assert!(
            session.echo().contains(diagnostic),
            "{name}: {}",
            session.echo()
        );
        assert_eq!(text(&session), "");
        assert!(session.buffer().unwrap().history().is_empty());
    }
    for name in ["missing.csv", "directory.csv"] {
        let (fixture, mut session) = editor("");
        fs::create_dir(fixture.0.join("directory.csv")).unwrap();
        action(&mut session, C::TableImport);
        argument(&mut session, name);
        assert_eq!(text(&session), "", "{name}");
        assert!(session.buffer().unwrap().history().is_empty(), "{name}");
        assert!(!fixture.0.join("missing.csv").exists());
    }
    let (fixture, mut session) = editor("|A|\n");
    for path in ["absent/output.csv", "output.txt"] {
        export(&mut session, path);
        assert!(!fixture.0.join(path).exists(), "{path}");
        assert_eq!(text(&session), "|A|\n");
        assert!(session.buffer().unwrap().history().is_empty());
    }
    assert_eq!(fs::read_dir(&fixture.0).unwrap().count(), 2);
}

// The explicitly entered suffix selects the format while the ordinary file
// owner publishes to the observed symlink referent without replacing the link.
#[test]
fn table_export_alias() {
    let (fixture, mut session) = editor("|A|\n");
    let target = fixture.write("referent.data", "prior\n");
    let alias = fixture.0.join("alias.csv");
    std::os::unix::fs::symlink(&target, &alias).unwrap();
    export(&mut session, "alias.csv");
    argument(&mut session, "yes");
    assert!(session.echo().starts_with("Exported"), "{}", session.echo());
    assert!(fs::symlink_metadata(&alias).unwrap().is_symlink());
    assert_eq!(fs::read_to_string(&target).unwrap(), "\"A\"\n");
    assert_eq!(text(&session), "|A|\n");
    assert!(session.buffer().unwrap().history().is_empty());
}

// C-g cancels a real owned import before it can publish. Ordinary edits and
// source/view changes during owned work invalidate that captured request.
#[test]
fn table_import_cancel() {
    let (fixture, mut session) = editor("");
    fixture.write("input.csv", &format!("{}\n", "a".repeat(200)).repeat(10000));
    action(&mut session, C::TableImport);
    session.handle(Event::Paste("input.csv".into()));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    cancel(&mut session);
    settle(&mut session);
    assert_eq!(text(&session), "");
    assert!(session.buffer().unwrap().history().is_empty());
    assert_eq!(session.remaining_input(Instant::now()), None);
    session.execute(Command::Split(Split::Right));
    settle(&mut session);
    let selected = session.windows().selected_id();
    let other = session
        .windows()
        .iter()
        .find(|(id, _)| *id != selected)
        .unwrap()
        .0;
    action(&mut session, C::TableImport);
    session.handle(Event::Paste("input.csv".into()));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    session.select_view(other).unwrap();
    settle(&mut session);
    assert!(
        session.echo().contains("source or view changed"),
        "{}",
        session.echo()
    );
    assert_eq!(text(&session), "");
    assert!(session.buffer().unwrap().history().is_empty());
}
