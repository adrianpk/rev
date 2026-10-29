use super::properties::argument;
use super::*;
use rev::command::{History, OrgCommand as C};
fn configured(template: &str) -> String {
    format!(
        "org.capture.templates=[{{key='t',description='Test',kind='plain',target={{type='file',file='destination.org'}},template='''{template}'''}}]"
    )
}
fn begin(session: &mut Session) {
    session.execute(Command::Org(C::Capture));
    argument(session, "t");
}
// Every preparation stage can cancel without creating a target or modifying
// the origin. The queued/read stages reap only their own session job.
#[test]
fn preparation_cancel() {
    for stage in ["template", "file", "prompt", "quantum", "queued", "running"] {
        let config = match stage {
            "template" | "file" => String::new(),
            "prompt" => configured("%^{Name}"),
            "quantum" => configured(&"%?".repeat(256)),
            _ => configured("%?"),
        };
        let (fixture, mut session) = editor(&config, "origin.txt", "界e\u{301}\r\nlast");
        let home = session.view().point;
        session.execute(Command::Org(C::Capture));
        if stage != "template" {
            argument(&mut session, "t")
        }
        if stage == "running" {
            session.expire_input(Instant::now());
        }
        session.handle(control('g'));
        settle(&mut session);
        assert!(session.capture_draft().is_none(), "{stage}");
        assert_eq!(session.view().point, home, "{stage}");
        assert_eq!(text(&session), "界e\u{301}\r\nlast", "{stage}");
        assert!(session.buffer().unwrap().history().is_empty(), "{stage}");
        assert!(!fixture.0.join("destination.org").exists(), "{stage}");
        assert_eq!(session.remaining_input(Instant::now()), None, "{stage}");
    }
}
// Static malformed instructions reject before file input or a destination read;
// an actual malformed destination also cannot replace the original editor.
#[test]
fn failed_preparation() {
    for (name, template, destination) in [
        ("Lisp", "%(message)", "* Target\r\n"),
        ("file escape", "%[unread]", "* Target\r\n"),
        ("time format", "%^{Name}%<%Q>", "* Target\r\n"),
        ("structure", "%?", "#+begin_src\n* raw heading\n#+end_src\n"),
    ] {
        let (fixture, mut session) = editor(&configured(template), "origin.txt", "untouched");
        let path = fixture.write("destination.org", destination);
        let home = session.view().point;
        begin(&mut session);
        settle(&mut session);
        assert!(session.capture_draft().is_none(), "{name}");
        assert_eq!(session.view().point, home, "{name}");
        assert_eq!(text(&session), "untouched", "{name}");
        assert_eq!(
            std::fs::read_to_string(path).unwrap(),
            destination,
            "{name}"
        );
        assert_eq!(session.remaining_input(Instant::now()), None, "{name}");
    }
}
// A checked cursor marker can fall inside a newly joined grapheme. Presentation
// snaps it to the following boundary; edit and undo never split that cluster.
#[test]
fn joined_cursor() {
    let (_fixture, mut session) = editor(&configured("e%?\u{301}界"), "origin.txt", "home");
    begin(&mut session);
    settle(&mut session);
    assert_eq!(text(&session), "e\u{301}界");
    assert_eq!(session.view().point.position.0, 2);
    session.frame().unwrap();
    session.execute(Command::BackwardDeleteChar);
    assert_eq!(text(&session), "界");
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "e\u{301}界");
    session.execute(Command::Org(C::CaptureKill));
    assert_eq!(text(&session), "home");
}
// A caller can change canonical text while expansion is queued. A stale origin
// rejects publication instead of installing a draft against old source identity.
#[test]
fn stale_origin() {
    let (fixture, mut session) = editor(&configured(&"%?".repeat(256)), "origin.txt", "home");
    begin(&mut session);
    session.replace(0..0, "new ").unwrap();
    settle(&mut session);
    assert!(session.capture_draft().is_none());
    assert!(session.echo().contains("origin changed"));
    assert_eq!(text(&session), "new home");
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    assert!(!fixture.0.join("destination.org").exists());
}
// Read-only destination bytes and source association remain untouched through
// editable drafts and ordinary C-g. A named abort works from another pane.
#[test]
fn readonly_destination() {
    use std::os::unix::fs::PermissionsExt;
    let (fixture, mut session) = editor(&configured("* Draft %?"), "origin.txt", "home");
    let path = fixture.write("destination.org", "\u{feff}* Existing\r\ntail");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o444)).unwrap();
    let before = std::fs::read(&path).unwrap();
    session.execute(Command::Split(Split::Right));
    let owner = session.windows().selected_id();
    let peer = session
        .windows()
        .iter()
        .find(|(id, _)| *id != owner)
        .unwrap()
        .0;
    begin(&mut session);
    settle(&mut session);
    assert_eq!(
        session.capture_target().unwrap().source.version,
        rev::org::io::DestinationVersion::Disk(rev::file::observe(&path).unwrap().unwrap())
    );
    session.insert('界');
    session.handle(control('g'));
    assert!(session.capture_draft().is_some());
    session.select_view(peer).unwrap();
    named(&mut session, "org-capture-kill");
    settle(&mut session);
    assert!(session.capture_draft().is_none());
    assert_eq!(session.windows().selected_id(), owner);
    assert_eq!(text(&session), "home");
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert_eq!(
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
        0o444
    );
}

// Canonical read-only ownership is separate from disk permissions. A capture
// targeting the actual origin owner can edit/abort only its transient text.
#[test]
fn canonical_readonly() {
    let fixture = Fixture::new();
    let startup = prepare(&fixture.write("config.toml", "org.capture.default_file='origin.org'"));
    assert!(startup.diagnostic.is_none());
    let path = fixture.write("origin.org", "\u{feff}* Tasks\r\nunchanged");
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.visit(&path).unwrap();
    buffers.get_mut(id).unwrap().set_readonly(true);
    let view = View::new(buffers.get(id).unwrap());
    let home = view.point;
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 12,
        },
    )
    .unwrap();
    begin(&mut session);
    settle(&mut session);
    let target = session.capture_target().unwrap();
    assert_eq!(
        target.source.version,
        rev::org::io::DestinationVersion::Buffer(home.stamp)
    );
    assert!(target.source.readonly);
    session.insert('!');
    session.execute(Command::Org(C::CaptureKill));
    assert_eq!(session.view().point, home);
    assert!(session.buffer().unwrap().is_readonly());
    assert!(session.buffer().unwrap().history().is_empty());
    assert_eq!(text(&session), "* Tasks\r\nunchanged");
    assert_eq!(
        std::fs::read(path).unwrap(),
        "\u{feff}* Tasks\r\nunchanged".as_bytes()
    );
}
