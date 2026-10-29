use super::links::{editor, line};
use super::*;
use rev::command::{History, Motion, Split, WindowCommand};
use std::os::unix::fs::PermissionsExt;
fn unchanged(s: &mut Session, command: C, expected: &str) {
    let source = text(s);
    let view = s.view().clone();
    let history = s.buffer().unwrap().history().len();
    action(s, command);
    assert!(s.echo().to_lowercase().contains(expected), "{}", s.echo());
    assert_eq!(text(s), source);
    assert_eq!(s.view(), &view);
    assert_eq!(s.buffer().unwrap().history().len(), history);
}
// Ambiguity and malformed/unsupported destinations never choose a convenient
// first match or replace the canonical view. Literal blocks stay inert.
#[test]
fn links_rejections() {
    for (name, source, expected) in [
        ("targets", "[[spot]]\n<<spot>>\n<<spot>>\n", "ambiguous"),
        ("headings", "[[*Same]]\n* Same\n* Same\n", "ambiguous"),
        (
            "custom ids",
            "[[#same]]\n* A\n:PROPERTIES:\n:CUSTOM_ID: same\n:END:\n* B\n:PROPERTIES:\n:CUSTOM_ID: same\n:END:\n",
            "ambiguous",
        ),
        ("absent", "[[unknown]]\n* Root\n", "unresolved"),
        ("encoding", "[[file:bad%GG.org]]\n", "percent"),
        ("utf8", "[[file:bad%FF.org]]\n", "utf-8"),
        ("control", "[[file:bad%00.org]]\n", "control"),
        ("shell", "[[shell:touch-owned]]\n", "not admitted"),
        ("elisp", "[[elisp:form]]\n", "not admitted"),
        ("line", "[[file:destination.org::0]]\n", "outside"),
    ] {
        let (_f, mut s) = editor("", source, "* Existing\n");
        unchanged(&mut s, C::OpenLink, expected);
        assert_eq!(text(&s), source, "{name}");
    }
    let (_f, mut s) = editor(
        "",
        "#+begin_example\n[[https://example.invalid/]]\n#+end_example\n",
        "",
    );
    line(&mut s, "2");
    unchanged(&mut s, C::OpenLink, "no supported link");
}
// ID resolution requires a complete explicit source set. Duplicate IDs across
// real documents, or an unreadable selected source, cannot certify uniqueness.
#[test]
fn links_id_completeness() {
    for (name, second, expected) in [
        ("duplicate", true, "ambiguous"),
        ("missing", false, "missing"),
        ("unreadable", true, "permission"),
    ] {
        let (f, mut s) = editor(
            "org.agenda.sources=['destination.org','other.org']",
            "[[id:stable]]\n",
            "* One\n:PROPERTIES:\n:ID: stable\n:END:\n",
        );
        if second {
            f.write("other.org", "* Two\n:PROPERTIES:\n:ID: stable\n:END:\n");
        }
        if name == "unreadable" {
            std::fs::set_permissions(
                f.0.join("other.org"),
                std::fs::Permissions::from_mode(0o000),
            )
            .unwrap();
        }
        unchanged(&mut s, C::OpenLink, expected);
        assert_eq!(
            std::fs::read_to_string(f.0.join("source.org")).unwrap(),
            text(&s),
            "{name}"
        );
    }
}
// Attachment traversal and symlink escape are rejected against the actual DIR
// root, while missing local documents are never created by following a link.
#[test]
fn links_path_boundaries() {
    for (name, target, expected) in [
        ("parent", "attachment:../destination.org", "inside"),
        ("symlink", "attachment:escape.org", "escapes"),
        ("missing", "file:absent.org", "does not exist"),
    ] {
        let source = format!("* Home\n:PROPERTIES:\n:DIR: assets\n:END:\n[[{target}]]\n");
        let (f, mut s) = editor("", &source, "* Outside\n");
        std::fs::create_dir(f.0.join("assets")).unwrap();
        std::os::unix::fs::symlink(f.0.join("destination.org"), f.0.join("assets/escape.org"))
            .unwrap();
        line(&mut s, "5");
        unchanged(&mut s, C::OpenLink, expected);
        assert!(!f.0.join("absent.org").exists(), "{name}");
    }
}
// Existing dirty canonical target text wins over disk. Read-only target/source
// buffers remain visitable and storable, but insertion never gains edit rights.
#[test]
fn links_canonical_readonly() {
    let f = Fixture::new();
    let source = f.write("source.org", "[[file:destination.org::*Unsaved]]\n");
    let target = f.write("destination.org", "* Disk\n");
    let mut buffers = Buffers::new().unwrap();
    let dest = buffers.visit(&target).unwrap();
    let b = buffers.get_mut(dest).unwrap();
    let edit = rev::buffer::edit::Replacement {
        source: b.span(0..b.len_chars()).unwrap(),
        text: "* Unsaved\n".into(),
    };
    let prepared = b.prepare(&[edit], &[], None).unwrap();
    b.publish(prepared, &mut []).unwrap();
    b.set_readonly(true);
    let id = buffers.visit(&source).unwrap();
    buffers.get_mut(id).unwrap().set_readonly(true);
    let view = View::new(buffers.get(id).unwrap());
    let mut s = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 14,
        },
    )
    .unwrap();
    unchanged(&mut s, C::InsertLink, "read-only");
    action(&mut s, C::StoreLink);
    assert!(s.echo().contains("stored"));
    action(&mut s, C::OpenLink);
    assert!(s.echo().contains("Org link destination"), "{}", s.echo());
    assert_eq!(text(&s), "* Unsaved\n");
    assert!(s.buffer().unwrap().is_readonly());
    unchanged(&mut s, C::InsertLink, "read-only");
    assert_eq!(std::fs::read_to_string(target).unwrap(), "* Disk\n");
    action(&mut s, C::LinkBack);
    assert_eq!(s.buffer().unwrap().id(), id);
}
// Accepted insertion maps the actual shared views in one undo group. Bracket
// escaping survives a second edit without changing the literal file target.
#[test]
fn links_edit_shared() {
    let (f, mut s) = editor("", "body 界\n", "");
    f.write("two[one].org", "* Literal\n");
    s.execute(Command::Split(Split::Right));
    let owner = s.windows().selected_id();
    let peer = s.windows().iter().find(|(id, _)| *id != owner).unwrap().0;
    s.execute(Command::Motion(Motion::BufferEnd));
    action(&mut s, C::InsertLink);
    argument(&mut s, "file:two[one].org");
    argument(&mut s, "Literal 界");
    assert!(
        text(&s).contains("[[file:two\\[one\\].org][Literal 界]]"),
        "{}",
        text(&s)
    );
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    assert_eq!(
        s.windows().get(peer).unwrap().point.stamp,
        s.buffer().unwrap().stamp()
    );
    line(&mut s, "2");
    let before = text(&s);
    action(&mut s, C::InsertLink);
    argument(&mut s, "");
    argument(&mut s, "");
    assert_eq!(text(&s), before);
    line(&mut s, "2");
    action(&mut s, C::OpenLink);
    assert_eq!(text(&s), "* Literal\n", "{}", s.echo());
    action(&mut s, C::LinkBack);
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), before);
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), "body 界\n");
    s.select_view(peer).unwrap();
    assert_eq!(text(&s), "body 界\n");
    s.execute(Command::Window(WindowCommand::Keep));
}
// Source edits invalidate a retained prompt, and failed return points cannot
// acquire authority over newer text. Boundary navigation wraps only on repeat.
#[test]
fn links_stale_and_motion() {
    let (_f, mut s) = editor("", "[[*Target]]\n* Target\nbody\n", "");
    action(&mut s, C::OpenLink);
    s.insert('界');
    let view = s.view().clone();
    unchanged(&mut s, C::LinkBack, "source changed");
    assert_eq!(s.view(), &view);
    action(&mut s, C::InsertLink);
    s.insert('X');
    let changed = text(&s);
    argument(&mut s, "*Target");
    assert!(s.echo().contains("source or view changed"), "{}", s.echo());
    assert_eq!(text(&s), changed);
    let (_f, mut s) = editor("", "[[*One]]\n[[*Two]]\n* One\n* Two\n", "");
    action(&mut s, C::NextLink);
    assert_eq!(s.view().point.position.0, 9);
    unchanged(&mut s, C::NextLink, "repeat to wrap");
    action(&mut s, C::NextLink);
    assert_eq!(s.view().point.position.0, 0);
}
