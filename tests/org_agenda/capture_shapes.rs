use super::capture_publication::{draft, finalize};
use super::*;
use rev::command::{Motion, Split, WindowCommand};
// Actual finalize writes each selected native kind into its structural target;
// new files honor ordinary creation format and old source metadata stays exact.
#[test]
fn capture_shapes() {
    for (name, kind, target, source, payload, prepend, expected) in [
        (
            "outline",
            "entry",
            "{type='file+olp',file='destination.org',outline=['Projects','Tasks']}",
            "",
            "* Note\n** Child",
            false,
            "\u{feff}* Projects\r\n** Tasks\r\n*** Note\r\n**** Child\r\n",
        ),
        (
            "list",
            "checkitem",
            "{type='file+headline',file='destination.org',headline='Tasks'}",
            "* Tasks\n  1. Old\n    body\n\n* Peer\n",
            "- [ ] New\n  continued",
            false,
            "* Tasks\n  1. Old\n    body\n  2. [ ] New\n    continued\n\n* Peer\n",
        ),
        (
            "table",
            "table-line",
            "{type='file+headline',file='destination.org',headline='Tasks'}",
            "* Tasks\r\n#+NAME: keep\r\n| A | B |  \r\n| C | D |\r\n\r\n#+TBLFM: $1=2\r\n* Peer",
            "| E | F |",
            true,
            "* Tasks\r\n#+NAME: keep\r\n| E | F |\r\n| A | B |  \r\n| C | D |\r\n\r\n#+TBLFM: $1=2\r\n* Peer",
        ),
        (
            "plain",
            "plain",
            "{type='file+headline',file='destination.org',headline='Tasks'}",
            "* Tasks\n:PROPERTIES:\n:ID: keep\n:END:\nbody\n** Child\n",
            "literal",
            true,
            "* Tasks\n:PROPERTIES:\n:ID: keep\n:END:\nliteral\nbody\n** Child\n",
        ),
    ] {
        let config = format!(
            "files.bom=true\nfiles.newline='crlf'\norg.capture.templates=[{{key='t',description='Test',kind='{kind}',target={target},template='%?',prepend={prepend}}}]"
        );
        let (fixture, mut session) = editor(&config, "origin.txt", "home");
        let path = fixture.0.join("destination.org");
        if !source.is_empty() {
            std::fs::write(&path, source).unwrap()
        }
        draft(&mut session);
        session.handle(Event::Paste(payload.into()));
        finalize(&mut session);
        assert!(
            session.capture_draft().is_none(),
            "{name}: {}",
            session.echo()
        );
        assert_eq!(std::fs::read_to_string(path).unwrap(), expected, "{name}");
    }
}
// Renumbering uses distinct marker edits. A peer cursor inside an unchanged
// item retains that actual source text instead of collapsing to the list end.
#[test]
fn capture_list_peers() {
    let config = "org.capture.templates=[{key='t',description='List',kind='item',target={type='file+headline',file='destination.org',headline='Tasks'},template='%?',prepend=true}]";
    let (fixture, mut session) = editor(config, "origin.txt", "home");
    let path = fixture.write("destination.org", "* Tasks\n1. First\n2. Second\n");
    session.execute(Command::Split(Split::Right));
    let owner = session.windows().selected_id();
    let peer = session
        .windows()
        .iter()
        .find(|(id, _)| *id != owner)
        .unwrap()
        .0;
    session.select_view(peer).unwrap();
    session.load(&path).unwrap();
    goto(&mut session, 3);
    for _ in 0..3 {
        session.execute(Command::Motion(Motion::ForwardChar));
    }
    session.select_view(owner).unwrap();
    draft(&mut session);
    session.handle(Event::Paste("- New\n- Another".into()));
    finalize(&mut session);
    session.execute(Command::Window(WindowCommand::Other));
    assert_eq!(session.windows().selected_id(), peer);
    let point = session.view().point.position.0;
    assert!(
        text(&session)
            .chars()
            .skip(point)
            .collect::<String>()
            .starts_with("Second")
    );
    assert_eq!(
        text(&session),
        "* Tasks\n1. New\n2. Another\n3. First\n4. Second\n"
    );
    assert_eq!(session.buffer().unwrap().history().len(), 1);
}
