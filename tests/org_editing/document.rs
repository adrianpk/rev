use super::visibility::action;
use super::*;
use rev::command::{History, Motion, OrgCommand as C};
pub(super) fn at(s: &mut Session, n: usize) {
    s.execute(Command::Motion(Motion::BufferStart));
    for _ in 0..n {
        s.execute(Command::Motion(Motion::ForwardChar));
    }
}
pub(super) fn argument(s: &mut Session, value: &str) {
    s.handle(Event::Paste(value.into()));
    s.handle(key(KeyCode::Enter));
    settle(s);
}
// Footnote creation changes two distant ranges in one real buffer transaction.
// The definition cursor, numeric graph and ordinary undo/redo remain coherent.
#[test]
fn footnote_create_follow() {
    let source = "Text.";
    let (_fixture, mut s) = editor("", "notes.org", source);
    at(&mut s, 5);
    action(&mut s, C::FootnoteAction);
    let expected = "Text.[fn:1]\n\n* Footnotes\n\n[fn:1] ";
    assert_eq!(text(&s), expected, "{}", s.echo());
    assert_eq!(s.view().point.position.0, expected.chars().count());
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    action(&mut s, C::FootnoteAction);
    assert_eq!(s.view().point.position.0, 5, "{}", s.echo());
    action(&mut s, C::FootnoteAction);
    assert_eq!(s.view().point.position.0, expected.rfind("[fn:").unwrap());
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), source);
    s.execute(Command::History(History::Redo));
    assert_eq!(text(&s), expected);
}
// Numeric labels follow first reference order; repeated/named/inline notes keep
// their associations, definition text and unrelated spelling across one undo.
#[test]
fn footnote_renumber() {
    let cases = [
        (
            "separate",
            "Read [fn:8] and [fn:3] then [fn:8].\n\n[fn:3] three\n[fn:8] eight\n",
            "Read [fn:1] and [fn:2] then [fn:1].\n\n[fn:2] three\n[fn:1] eight\n",
        ),
        (
            "inline",
            "Read [fn:9:wide 界] and [fn:9] and [fn:word:body].",
            "Read [fn:1:wide 界] and [fn:1] and [fn:word:body].",
        ),
    ];
    for (name, source, expected) in cases {
        let (_f, mut s) = editor("", "notes.org", source);
        action(&mut s, C::FootnoteRenumber);
        assert_eq!(text(&s), expected, "{name}: {}", s.echo());
        assert_eq!(s.buffer().unwrap().history().len(), 1);
        s.execute(Command::History(History::Undo));
        assert_eq!(text(&s), source, "{name}");
    }
}
// Native markup uses active scalar regions while entity/affiliated prompts insert
// literal compatible source. Existing unrelated metadata is never reconstructed.
#[test]
fn inline_document_edits() {
    let (_f, mut s) = editor("", "notes.org", "text");
    action(&mut s, C::Emphasize);
    argument(&mut s, "*");
    assert_eq!(text(&s), "**text", "{}", s.echo());
    assert_eq!(s.view().point.position.0, 1);
    s.execute(Command::History(History::Undo));
    action(&mut s, C::InsertEntity);
    argument(&mut s, "alpha");
    assert_eq!(text(&s), "\\alpha{}text", "{}", s.echo());
    s.execute(Command::History(History::Undo));
    action(&mut s, C::EditAffiliated);
    argument(&mut s, "NAME");
    argument(&mut s, "sample");
    assert_eq!(text(&s), "#+NAME: sample\ntext", "{}", s.echo());
    action(&mut s, C::EditAffiliated);
    argument(&mut s, "CAPTION");
    argument(&mut s, "A caption");
    assert_eq!(
        text(&s),
        "#+CAPTION: A caption\n#+NAME: sample\ntext",
        "{}",
        s.echo()
    );
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), "#+NAME: sample\ntext");
}

// Active Unicode regions become one compatible emphasis object; whole-object
// selection can change or remove markers without changing its content.
#[test]
fn emphasis_regions() {
    for (name, marker) in [
        ("bold", "*"),
        ("italic", "/"),
        ("underline", "_"),
        ("strike", "+"),
        ("verbatim", "="),
        ("code", "~"),
    ] {
        let (_f, mut s) = editor("", "notes.org", "wide 界");
        s.execute(Command::Region(rev::command::RegionCommand::Set));
        s.execute(Command::Motion(Motion::BufferEnd));
        action(&mut s, C::Emphasize);
        argument(&mut s, marker);
        assert_eq!(
            text(&s),
            format!("{marker}wide 界{marker}"),
            "{name}: {}",
            s.echo()
        );
        s.execute(Command::History(History::Undo));
        assert_eq!(text(&s), "wide 界");
    }
}

// Structural contexts never turn source-looking notes or commands in literal,
// unknown or dynamic forms into edits. Ordinary surrounding notes still work.
#[test]
fn document_literal_contexts() {
    let cases = [
        (
            "source",
            "#+begin_src text\n[fn:9] literal\n#+end_src\n",
            16,
        ),
        (
            "example",
            "#+begin_example\n[fn:9] literal\n#+end_example\n",
            16,
        ),
        ("dynamic", "#+begin: unknown\n[fn:9] literal\n#+end:\n", 18),
        (
            "unknown",
            "#+begin_mystery\n[fn:9] literal\n#+end_mystery\n",
            17,
        ),
        ("macro", "text {{{unknown([fn:9])}}} after\n", 12),
        ("code", "text ~[fn:9]~ after\n", 8),
        ("verbatim", "text =[fn:9]= after\n", 8),
    ];
    for (name, source, point) in cases {
        for command in [
            C::FootnoteAction,
            C::InsertEntity,
            C::Emphasize,
            C::CtrlCCtrlC,
            C::OpenLink,
        ] {
            let (_f, mut s) = editor("", "notes.org", source);
            at(&mut s, point);
            let before = s.view().clone();
            action(&mut s, command);
            assert_eq!(text(&s), source, "{name}/{command:?}: {}", s.echo());
            assert_eq!(s.view(), &before, "{name}/{command:?}: {}", s.echo());
            assert!(s.buffer().unwrap().history().is_empty());
            s.handle(Event::Paste("z".into()));
            assert_eq!(
                text(&s).chars().count(),
                source.chars().count() + 1,
                "{name}/{command:?}: {}",
                s.echo()
            );
        }
    }
}
// Contextual footnote actions reach real definitions/references, while radio
// following is case-insensitive, Unicode-safe and excludes larger words/code.
#[test]
fn document_context_follow() {
    for command in [C::CtrlCCtrlC, C::OpenLink] {
        let source = "Read [fn:x].\n\n[fn:x] body\n";
        let (_f, mut s) = editor("", "notes.org", source);
        at(&mut s, 6);
        action(&mut s, command);
        assert_eq!(
            s.view().point.position.0,
            source.rfind("[fn:").unwrap(),
            "{}",
            s.echo()
        );
        action(&mut s, command);
        assert_eq!(s.view().point.position.0, 5);
        assert_eq!(text(&s), source);
        assert!(s.buffer().unwrap().history().is_empty());
    }
    let source = "<<<Wide 界>>>\n\nA wide 界 here.\nNot awide 界, nor ~wide 界~.\n";
    let (_f, mut s) = editor("", "notes.org", source);
    let byte = source.find("wide 界 here").unwrap();
    at(&mut s, source[..byte].chars().count() + 2);
    let before = s.view().point;
    action(&mut s, C::OpenLink);
    assert_eq!(s.view().point.position.0, 0, "{}", s.echo());
    action(&mut s, C::LinkBack);
    assert_eq!(s.view().point, before);
    assert_eq!(text(&s), source);
    assert!(s.buffer().unwrap().history().is_empty());
}

// Missing, duplicate and malformed references never authorize a partial label
// rewrite. Literal/unknown lookalikes are excluded from the same note graph.
#[test]
fn footnote_graph_rejection() {
    let cases = [
        ("missing", "Read [fn:9].\n"),
        ("duplicate", "Read [fn:9].\n\n[fn:9] one\n[fn:9] two\n"),
        ("malformed", "Read [fn:bad label].\n"),
        ("unclosed", "Read [fn::unfinished\n"),
        ("orphan", "[fn:9] orphan\n"),
    ];
    for (name, source) in cases {
        let (_f, mut s) = editor("", "notes.org", source);
        let before = s.view().clone();
        action(&mut s, C::FootnoteRenumber);
        assert_eq!(text(&s), source, "{name}: {}", s.echo());
        assert_eq!(s.view(), &before);
        assert!(s.buffer().unwrap().history().is_empty());
    }
    let source = "Read [fn:9], ~[fn:bad]~ and {{{unknown([fn:x])}}}.\n\n#+begin_mystery\n[fn:x] untouched\n#+end_mystery\n\n[fn:9] body\n";
    let (_f, mut s) = editor("", "notes.org", source);
    action(&mut s, C::FootnoteRenumber);
    assert_eq!(text(&s), source.replace("fn:9", "fn:1"), "{}", s.echo());
}
