use super::*;
use super::{
    document::{argument, at},
    visibility::action,
};
use rev::command::{History, OrgCommand as C};

// Drafts and owned observations cannot follow a changed source/pane. Cancelling
// either prompt stage or active work leaves ordinary editing and idle readiness.
#[test]
fn document_draft_guards() {
    for (name, command, draft) in [
        ("entity", C::InsertEntity, "alpha"),
        ("emphasis", C::Emphasize, "*"),
        ("affiliated", C::EditAffiliated, "NAME"),
    ] {
        let (_f, mut s) = editor("", "notes.org", "Text.");
        action(&mut s, command);
        s.execute(Command::KeyboardQuit);
        settle(&mut s);
        assert_eq!(text(&s), "Text.", "{name}");
        assert!(s.buffer().unwrap().history().is_empty());
        action(&mut s, command);
        s.replace(0..0, "prefix ").unwrap();
        argument(&mut s, draft);
        assert!(
            s.echo().contains("source or view changed"),
            "{name}: {}",
            s.echo()
        );
        assert_eq!(text(&s), "prefix Text.");
        assert_eq!(s.buffer().unwrap().history().len(), 1);
    }
    let (_f, mut s) = editor("", "notes.org", "Text.");
    action(&mut s, C::EditAffiliated);
    argument(&mut s, "NAME");
    s.execute(Command::KeyboardQuit);
    assert_eq!(text(&s), "Text.");
    settle(&mut s);
    s.execute(Command::Org(C::FootnoteNew));
    s.execute(Command::KeyboardQuit);
    settle(&mut s);
    assert_eq!(text(&s), "Text.");
    assert!(s.buffer().unwrap().history().is_empty());
    assert!(s.remaining_input(Instant::now()).is_none());
}
// Read-only authority is enforced at real publication; navigation still works
// and neither failed edits nor successful following alter text/history.
#[test]
fn document_readonly() {
    let source = "Read [fn:8].\n\n[fn:8] body\n";
    for (name, command, drafts) in [
        ("new", C::FootnoteNew, vec![]),
        ("renumber", C::FootnoteRenumber, vec![]),
        ("entity", C::InsertEntity, vec!["alpha"]),
        ("emphasis", C::Emphasize, vec!["*"]),
        ("affiliated", C::EditAffiliated, vec!["NAME", "sample"]),
    ] {
        let (mut buffers, id) = super::folds::buffer(source);
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
        let before = s.view().clone();
        action(&mut s, command);
        for draft in drafts {
            argument(&mut s, draft);
        }
        assert!(s.echo().contains("read-only"), "{name}: {}", s.echo());
        assert_eq!(text(&s), source);
        assert_eq!(s.view(), &before);
        assert!(s.buffer().unwrap().history().is_empty());
        at(&mut s, 6);
        action(&mut s, C::FootnoteAction);
        assert_eq!(s.view().point.position.0, source.rfind("[fn:").unwrap());
    }
}
// Editing an existing affiliated line preserves all other source spellings and
// save format, including BOM, CRLF and literal block contents without EOF newline.
#[test]
fn affiliated_format() {
    let body = "#+ATTR_HTML: :class keep\r\n#+CAPTION: Old\r\n#+NAME: original\r\n#+begin_src text\r\n[fn:9] {{{unknown}}}\r\n#+end_src";
    let source = format!("\u{feff}{body}");
    let (f, mut s) = editor("", "notes.org", &source);
    action(&mut s, C::EditAffiliated);
    argument(&mut s, "CAPTION");
    argument(&mut s, "Wide 界");
    let expected = body.replace("CAPTION: Old", "CAPTION: Wide 界");
    assert_eq!(text(&s), expected, "{}", s.echo());
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    s.execute(Command::SaveBuffer);
    assert_eq!(
        std::fs::read(f.0.join("notes.org")).unwrap(),
        format!("\u{feff}{expected}").as_bytes()
    );
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), body);
    s.execute(Command::History(History::Redo));
    assert_eq!(text(&s), expected);
}
// Radio links reject larger words, literal occurrences, duplicate names and
// overlapping destinations, preserving the source point and history on failure.
#[test]
fn radio_boundaries() {
    let cases = [
        ("larger word", "<<<target>>>\n\nnotarget\n", "notarget"),
        ("code", "<<<target>>>\n\n~target~\n", "~target~"),
        ("macro", "<<<target>>>\n\n{{{target}}}\n", "{{{target}}}"),
        (
            "duplicate",
            "<<<Target>>> and <<<target>>>\n\ntarget\n",
            "\ntarget",
        ),
        (
            "overlap",
            "<<<wide target>>> and <<<target>>>\n\nwide target\n",
            "\nwide target",
        ),
    ];
    for (name, source, fragment) in cases {
        let (_f, mut s) = editor("", "notes.org", source);
        let byte = source.rfind(fragment).unwrap() + fragment.len() - 3;
        at(&mut s, source[..byte].chars().count());
        let before = s.view().clone();
        action(&mut s, C::OpenLink);
        assert_eq!(s.view(), &before, "{name}: {}", s.echo());
        assert_eq!(text(&s), source);
        assert!(s.buffer().unwrap().history().is_empty());
    }
}
// Invalid arguments and native observation bounds cannot create partial edits,
// truncate values or rewrite unrecognized syntax as an approximation.
#[test]
fn document_argument_bounds() {
    for name in ["unknown".into(), "x".repeat(65), "alpha\n".into()] {
        let (_f, mut s) = editor("", "notes.org", "Text.");
        action(&mut s, C::InsertEntity);
        argument(&mut s, &name);
        assert_eq!(text(&s), "Text.", "{}", s.echo());
        assert!(s.buffer().unwrap().history().is_empty());
    }
    let source = format!("Read [fn:{}:body].", "x".repeat(1025));
    let (_f, mut s) = editor("", "notes.org", &source);
    action(&mut s, C::FootnoteRenumber);
    assert!(s.echo().contains("1024-byte"), "{}", s.echo());
    assert_eq!(text(&s), source);
    let (_f, mut s) = editor("", "notes.org", "#+CAPTION: A\n#+CAPTION: B\nText.\n");
    action(&mut s, C::EditAffiliated);
    argument(&mut s, "CAPTION");
    argument(&mut s, "new");
    assert!(s.echo().contains("Repeated"), "{}", s.echo());
    assert!(s.buffer().unwrap().history().is_empty());
}

// A distant definition and inline reference publish once to the shared canonical
// buffer. Peer points/marks map while stale fold revisions retire; a fresh
// independent visibility choice uses the current source observation.
#[test]
fn document_shared_history() {
    let source = "* Root\nRead text.\n* Peer\nend\n";
    let (_f, mut s) = editor("", "notes.org", source);
    at(&mut s, source.find("text.").unwrap() + 5);
    s.execute(Command::Split(Split::Right));
    super::headings::goto(&mut s, 3);
    action(&mut s, C::Overview);
    s.execute(Command::Region(rev::command::RegionCommand::Set));
    s.execute(Command::KeyboardQuit);
    let peer = s.view().point.position;
    let mark = s.buffer().unwrap().mark().unwrap();
    s.execute(Command::Window(WindowCommand::Other));
    action(&mut s, C::FootnoteNew);
    let expected = source.replacen("text.", "text.[fn:1]", 1) + "\n\n* Footnotes\n\n[fn:1] ";
    assert_eq!(text(&s), expected, "{}", s.echo());
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    s.execute(Command::Window(WindowCommand::Other));
    assert_eq!(s.view().point.position.0, peer.0 + 6);
    assert_eq!(s.buffer().unwrap().mark().unwrap().0, mark.0 + 6);
    assert_eq!(
        s.view().visible_lines(s.buffer().unwrap()),
        s.buffer().unwrap().len_lines()
    );
    action(&mut s, C::Overview);
    let visible = super::visibility::visible(&s);
    assert!(!visible.contains(&1));
    assert!(!visible.contains(&3));
    s.execute(Command::Window(WindowCommand::Other));
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), source);
    s.execute(Command::Window(WindowCommand::Other));
    assert_eq!(s.view().point.position, peer);
    assert_eq!(s.buffer().unwrap().mark(), Some(mark));
}

// Inline definitions are also references: marker-to-body and body-to-reference
// navigation is reversible for named/anonymous notes without an external record.
#[test]
fn inline_note_navigation() {
    for (name, source) in [
        ("anonymous", "Read [fn::body 界]."),
        ("named", "Read [fn:word:body 界]."),
    ] {
        let (_f, mut s) = editor("", "notes.org", source);
        at(&mut s, 6);
        action(&mut s, C::FootnoteAction);
        assert_eq!(
            s.view().point.position.0,
            source.find("body").unwrap(),
            "{name}: {}",
            s.echo()
        );
        action(&mut s, C::FootnoteAction);
        assert_eq!(s.view().point.position.0, 5, "{name}: {}", s.echo());
        assert_eq!(text(&s), source);
        assert!(s.buffer().unwrap().history().is_empty());
    }
}

// A new definition can precede another section without consuming its heading
// delimiter or rewriting the existing CRLF source and final newline state.
#[test]
fn footnote_existing_section() {
    let source = "Text [fn:2].\r\n* Footnotes\r\n[fn:2] old\r\n* Next\r\nbody";
    let (_f, mut s) = editor("", "notes.org", source);
    at(&mut s, 4);
    action(&mut s, C::FootnoteNew);
    let expected =
        "Text[fn:1] [fn:2].\r\n* Footnotes\r\n[fn:2] old\r\n\r\n\r\n[fn:1] \r\n* Next\r\nbody";
    assert_eq!(text(&s), expected, "{}", s.echo());
    assert!(expected[s.view().point.position.0..].starts_with("\r\n* Next"));
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), source);
}
// Whole-object removal retains combining characters and extended grapheme bytes;
// rejected whitespace boundaries leave the selected region and revision intact.
#[test]
fn emphasis_edit_edges() {
    for (name, source, marker, expected) in [
        ("remove literal", "~e\u{301} 👩‍🚀~", " ", "e\u{301} 👩‍🚀"),
        ("change", "*wide 界*", "/", "/wide 界/"),
        ("whitespace", " text ", "*", " text "),
    ] {
        let (_f, mut s) = editor("", "notes.org", source);
        s.execute(Command::Region(rev::command::RegionCommand::Set));
        s.execute(Command::Motion(rev::command::Motion::BufferEnd));
        let before = s.view().clone();
        action(&mut s, C::Emphasize);
        argument(&mut s, marker);
        assert_eq!(text(&s), expected, "{name}: {}", s.echo());
        if name == "whitespace" {
            assert_eq!(s.view(), &before);
            assert!(s.buffer().unwrap().history().is_empty());
        } else {
            assert_eq!(s.buffer().unwrap().history().len(), 1);
            s.execute(Command::History(History::Undo));
            assert_eq!(text(&s), source);
        }
    }
}
