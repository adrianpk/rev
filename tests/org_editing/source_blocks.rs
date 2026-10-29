use super::*;
use rev::command::{History, Motion, OrgCommand as C};
fn action(s: &mut Session, command: C) {
    s.execute(Command::Org(command));
    settle(s);
}
fn special(s: &mut Session) {
    s.handle(control('c'));
    s.handle(key(KeyCode::Char('\'')));
    settle(s);
}
// The native editing chord enters a real transient buffer, reuses installed Rust
// syntax and returns one checked source change with ordinary undo and redo.
#[test]
fn source_edit_roundtrip() {
    let source = "#+NAME: sample\r\n#+BEGIN_SRC rust :exports code\r\n\tfn main() {}\r\n#+END_SRC\r\nunchanged";
    let (f, mut s) = editor("", "notes.org", &format!("\u{feff}{source}"));
    let original = s.buffer().unwrap().id();
    special(&mut s);
    let b = s
        .source_edit_draft()
        .unwrap_or_else(|| panic!("{}", s.echo()));
    assert!(b.is_transient());
    assert!(!b.is_org());
    assert!(b.file().is_none());
    assert!(!b.has_bom());
    assert_eq!(b.mode(), "Org Src Rust");
    assert_eq!(text(&s), "fn main() {}\n");
    assert!(!b.syntax().captures(b, 0..b.len_bytes()).is_empty());
    s.handle(Event::Paste("// 界\n".into()));
    let draft = s.buffer().unwrap().id();
    for command in [
        Command::SaveBuffer,
        Command::WriteFile,
        Command::FindFile,
        Command::Quit,
    ] {
        s.execute(command);
        assert_eq!(s.buffer().unwrap().id(), draft);
        assert!(s.echo().contains("draft retained"));
    }
    special(&mut s);
    assert!(s.source_edit_draft().is_none(), "{}", s.echo());
    assert_eq!(s.buffer().unwrap().id(), original);
    let expected = source.replace("\tfn main()", "\t// 界\r\n\tfn main()");
    assert_eq!(text(&s), expected);
    assert!(s.buffer().unwrap().has_bom());
    assert!(s.buffer().unwrap().is_modified());
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    assert_eq!(
        std::fs::read(f.0.join("notes.org")).unwrap(),
        format!("\u{feff}{source}").as_bytes()
    );
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), source);
    s.execute(Command::History(History::Redo));
    assert_eq!(text(&s), expected);
}
// Aborting or returning an unchanged draft preserves the exact source revision,
// view and history. Org-looking embedded content remains literal editable data.
#[test]
fn source_edit_cancellation() {
    let source = "#+BEGIN_SRC org\n  ,* body\n  ,#+END_SRC\n#+END_SRC\n";
    for abort in [true, false] {
        let (_f, mut s) = editor("", "notes.org", source);
        let home = s.view().clone();
        let stamp = s.buffer().unwrap().stamp();
        special(&mut s);
        assert!(!s.buffer().unwrap().is_org());
        assert_eq!(s.buffer().unwrap().mode(), "Org Src");
        assert_eq!(text(&s), "* body\n#+END_SRC\n");
        if abort {
            s.handle(Event::Paste("discard".into()));
            s.handle(control('c'));
            s.handle(control('k'));
        } else {
            special(&mut s);
        }
        settle(&mut s);
        assert!(s.source_edit_draft().is_none(), "{}", s.echo());
        assert_eq!(text(&s), source);
        assert_eq!(s.view(), &home);
        assert_eq!(s.buffer().unwrap().stamp(), stamp);
        assert!(s.buffer().unwrap().history().is_empty());
    }
}
// Header prompts are cancellable; the admitted argument replaces only the
// opening line in one undo step and inspection never mutates source.
#[test]
fn source_header_prompt() {
    let source = "#+BEGIN_SRC sh :results output\nexit 99\n#+END_SRC\n";
    let (_f, mut s) = editor("", "notes.org", source);
    action(&mut s, C::ViewSrcInfo);
    assert!(s.echo().contains(":results output"));
    assert_eq!(text(&s), source);
    action(&mut s, C::InsertHeaderArg);
    s.handle(control('g'));
    settle(&mut s);
    assert_eq!(text(&s), source);
    action(&mut s, C::InsertHeaderArg);
    s.handle(Event::Paste(":exports both".into()));
    s.handle(key(KeyCode::Enter));
    settle(&mut s);
    assert_eq!(
        text(&s),
        source.replace(":results output", ":results output :exports both")
    );
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), source);
    s.execute(Command::Motion(Motion::BufferEnd));
}
