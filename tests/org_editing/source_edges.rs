use super::*;
use rev::command::{Motion, OrgCommand as C};
const SOURCE: &str = "#+BEGIN_SRC rust\nfn main() {}\n#+END_SRC\ntail\n";
fn action(s: &mut Session, c: C) {
    s.execute(Command::Org(c));
    settle(s);
}
// A stale canonical revision cannot overwrite a peer edit, even when unrelated
// to the block. The draft survives rejection and abort restores mapped source.
#[test]
fn stale_source_draft() {
    let (_f, mut s) = editor("", "notes.org", SOURCE);
    s.execute(Command::Split(Split::Right));
    let owner = s.windows().selected_id();
    let peer = s.windows().iter().find(|(id, _)| *id != owner).unwrap().0;
    action(&mut s, C::EditSpecial);
    let draft = s.buffer().unwrap().id();
    s.handle(Event::Paste("// retained\n".into()));
    let value = text(&s);
    s.select_view(peer).unwrap();
    s.execute(Command::Motion(Motion::BufferStart));
    s.handle(Event::Paste("prefix\n".into()));
    s.select_view(owner).unwrap();
    action(&mut s, C::EditSrcExit);
    assert_eq!(s.buffer().unwrap().id(), draft);
    assert_eq!(text(&s), value);
    assert!(s.echo().contains("changed"), "{}", s.echo());
    s.handle(control('g'));
    assert!(s.source_edit_draft().is_some());
    action(&mut s, C::EditSrcAbort);
    assert_eq!(text(&s), format!("prefix\n{SOURCE}"));
    assert_eq!(s.view().point.position.0, 7);
    assert_eq!(
        s.windows().get(peer).unwrap().point.stamp,
        s.buffer().unwrap().stamp()
    );
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    assert!(s.source_edit_draft().is_none());
}
// A successful publication maps both the hidden originating view and another
// pane, retains their identity and keeps source undo scoped to this operation.
#[test]
fn shared_source_return() {
    let (_f, mut s) = editor("", "notes.org", SOURCE);
    s.execute(Command::Split(Split::Right));
    let owner = s.windows().selected_id();
    let peer = s.windows().iter().find(|(id, _)| *id != owner).unwrap().0;
    s.select_view(peer).unwrap();
    s.execute(Command::Motion(Motion::BufferEnd));
    s.select_view(owner).unwrap();
    let before = SOURCE.find("main").unwrap();
    super::document::at(&mut s, before);
    action(&mut s, C::EditSpecial);
    s.handle(Event::Paste("// new\n".into()));
    action(&mut s, C::EditSrcExit);
    let returned = s.view().point.position.0;
    let expected = SOURCE.replace("fn main()", "// new\nfn main()");
    assert_eq!(text(&s), expected);
    let b = s.buffer().unwrap();
    let view = s.windows().get(peer).unwrap();
    assert_eq!(view.point.stamp, b.stamp());
    assert_eq!(view.point.position.0, expected.chars().count());
    assert_eq!(s.windows().selected_id(), owner);
    s.execute(Command::History(rev::command::History::Undo));
    assert_eq!(text(&s), SOURCE);
    assert_eq!(s.view().point.position.0, before);
    assert_eq!(
        s.windows().get(peer).unwrap().point.position.0,
        SOURCE.chars().count()
    );
    s.execute(Command::History(rev::command::History::Redo));
    assert_eq!(text(&s), expected);
    assert_eq!(s.view().point.position.0, returned);
}
// Cancellation of queued observation, malformed blocks and read-only source
// produce no draft, source edit, history or lingering owned-work deadline.
#[test]
fn source_entry_failures() {
    let (_f, mut s) = editor("", "notes.org", SOURCE);
    s.execute(Command::Org(C::EditSpecial));
    s.handle(control('g'));
    settle(&mut s);
    assert!(s.source_edit_draft().is_none());
    assert_eq!(text(&s), SOURCE);
    for (name, source) in [
        ("unterminated", "#+BEGIN_SRC rust\nfn main() {}"),
        (
            "malformed header",
            "#+BEGIN_SRC rust :var x=(]\nbody\n#+END_SRC\n",
        ),
    ] {
        let (_f, mut s) = editor("", "notes.org", source);
        action(&mut s, C::EditSpecial);
        assert!(s.source_edit_draft().is_none(), "{name}");
        assert_eq!(text(&s), source);
        assert!(s.buffer().unwrap().history().is_empty());
    }
    let (mut buffers, id) = super::folds::buffer(SOURCE);
    buffers.get_mut(id).unwrap().set_readonly(true);
    let view = View::new(buffers.get(id).unwrap());
    let mut s = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 12,
        },
    )
    .unwrap();
    for command in [C::EditSpecial, C::InsertHeaderArg] {
        action(&mut s, command);
        assert!(s.echo().contains("read-only"), "{}", s.echo());
        assert!(s.source_edit_draft().is_none());
    }
    action(&mut s, C::ViewSrcInfo);
    assert!(s.echo().contains("Source language rust"));
    assert_eq!(text(&s), SOURCE);
    assert!(s.buffer().unwrap().history().is_empty());
    assert_eq!(s.remaining_input(Instant::now()), None);
}
