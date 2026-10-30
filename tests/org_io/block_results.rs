use super::execution::configured;
use super::*;
use rev::{command::History, org::execution::State};
const PROFILE: &str =
    "[[org.source.programs]]\nname='shell'\nlanguage='sh'\nexecutable='/bin/sh'\ncwd='source'\n";
fn run(s: &mut Session) {
    action(s, C::ExecuteSrc);
    argument(s, "yes");
}
// Native publication replaces exactly the recognized result and is one undo
// unit. Unsaved output does not write the canonical file or change its BOM/EOL.
#[test]
fn result_transaction() {
    let source = "\u{feff}#+NAME: greet\r\n#+BEGIN_SRC sh\r\nprintf 'hello 界'\r\n#+END_SRC\r\n\r\n#+RESULTS[old]: greet\r\n: previous\r\n\r\n* Next\r\nretained\r\n";
    let (f, mut s) = configured(source, PROFILE);
    let before = text(&s);
    run(&mut s);
    let out = s.execution_result().unwrap();
    assert_eq!(out.state, State::Succeeded, "{}", out.detail);
    let expected = before
        .replace("#+RESULTS[old]:", "#+RESULTS:")
        .replace(": previous", ": hello 界");
    assert_eq!(text(&s), expected);
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    assert_eq!(
        std::fs::read(f.0.join("source.org")).unwrap(),
        source.as_bytes()
    );
    s.execute(Command::History(History::Undo));
    settle(&mut s);
    assert_eq!(text(&s), before);
    s.execute(Command::History(History::Redo));
    settle(&mut s);
    assert_eq!(text(&s), expected);
}
// Collection is declared by the chosen adapter; headers select bounded native
// table/wrapper/handling semantics. Invalid data leaves the old result intact.
#[test]
fn result_formats() {
    for (name, header, adapter, script, expected, state) in [
        (
            "insert",
            "",
            "output",
            "printf hello",
            "\n#+RESULTS:\n: hello\n",
            State::Succeeded,
        ),
        (
            "value",
            ":results value replace",
            "value",
            "printf 42",
            "#+RESULTS:\n: 42\n",
            State::Succeeded,
        ),
        (
            "table",
            ":results value table",
            "table",
            "printf 'a\\tb\\nc\\td\\n'",
            "#+RESULTS:\n| a | b |\n| c | d |\n",
            State::Succeeded,
        ),
        (
            "append",
            ":results output append",
            "output",
            "printf new",
            "#+RESULTS:\n: previous\n: new\n",
            State::Succeeded,
        ),
        (
            "prepend",
            ":results prepend",
            "output",
            "printf new",
            "#+RESULTS:\n: new\n: previous\n",
            State::Succeeded,
        ),
        (
            "drawer",
            ":results drawer",
            "output",
            "printf new",
            "#+RESULTS:\n:RESULTS:\nnew\n:END:\n",
            State::Succeeded,
        ),
        (
            "html",
            ":results html",
            "output",
            "printf '<b>new</b>'",
            "#+RESULTS:\n#+BEGIN_EXPORT html\n<b>new</b>\n#+END_EXPORT\n",
            State::Succeeded,
        ),
        (
            "silent",
            ":results silent",
            "output",
            "printf new",
            "#+RESULTS:\n: previous\n",
            State::Succeeded,
        ),
        (
            "bad table",
            ":results table",
            "table",
            "printf 'a\\tb\\nc\\n'",
            "#+RESULTS:\n: previous\n",
            State::Failed,
        ),
        (
            "failed",
            "",
            "output",
            "printf changed; exit 7",
            "#+RESULTS:\n: previous\n",
            State::Failed,
        ),
        (
            "invalid utf8",
            "",
            "output",
            "printf '\\377'",
            "#+RESULTS:\n: previous\n",
            State::Failed,
        ),
    ] {
        let end = if name == "insert" {
            ""
        } else {
            "#+RESULTS:\n: previous\n"
        };
        let source = format!("#+BEGIN_SRC sh {header}\n{script}\n#+END_SRC\n{end}");
        let config = format!("{PROFILE}result='{adapter}'");
        let (_f, mut s) = configured(&source, &config);
        run(&mut s);
        let out = s.execution_result().unwrap();
        assert_eq!(out.state, state, "{name}: {}", out.detail);
        let prefix = source.split("#+END_SRC").next().unwrap();
        assert_eq!(text(&s), format!("{prefix}#+END_SRC\n{expected}"), "{name}");
    }
}
// Calls override literal defaults after inside headers and place results at the
// call, preserving the named block and the real shell's argv/source boundaries.
#[test]
fn result_calls() {
    let source = "#+NAME: greet\n#+BEGIN_SRC sh :var n=2 :var text=\"default\"\nprintf '%s:%s' \"$n\" \"$text\"\n#+END_SRC\n\n#+CALL: greet[:var n=3](n=4,text=\"a, b; $(touch injected)\") :results drawer\n#+RESULTS: greet\n: previous\n";
    let (f, mut s) = configured(source, PROFILE);
    s.execute(Command::GotoLine);
    argument(&mut s, "6");
    action(&mut s, C::CtrlCCtrlC);
    assert!(!f.0.join("injected").exists());
    argument(&mut s, "yes");
    let out = s.execution_result().unwrap();
    assert_eq!(out.state, State::Succeeded, "{}", out.detail);
    assert_eq!(out.stdout, b"4:a, b; $(touch injected)");
    assert!(text(&s).ends_with("#+RESULTS: greet\n:RESULTS:\n4:a, b; $(touch injected)\n:END:\n"));
    assert!(text(&s).starts_with(source.split("#+CALL:").next().unwrap()));
    assert!(!f.0.join("injected").exists());
}
// Declared library files are observed only on explicit calls. Changed library
// text between preparation and consent prevents execution and retains results.
#[test]
fn result_library() {
    for stale in [false, true] {
        let (f, mut s) = configured(
            "#+CALL: greet()\n#+RESULTS: greet\n: previous\n",
            &format!("org.source.library_files=['lib.org']\n{PROFILE}"),
        );
        let library = f.write(
            "lib.org",
            "#+NAME: greet\n#+BEGIN_SRC sh\nprintf library > marker; printf result\n#+END_SRC\n",
        );
        action(&mut s, C::ExecuteSrc);
        assert!(!f.0.join("marker").exists());
        if stale {
            std::fs::write(&library, "changed library").unwrap();
        }
        argument(&mut s, "yes");
        if stale {
            assert!(s.execution_result().is_none());
            assert!(s.echo().contains("changed"), "{}", s.echo());
            assert!(!f.0.join("marker").exists());
            assert!(text(&s).ends_with(": previous\n"));
        } else {
            assert_eq!(s.execution_result().unwrap().state, State::Succeeded);
            assert!(text(&s).ends_with(": result\n"));
            assert_eq!(
                std::fs::read_to_string(&library).unwrap(),
                "#+NAME: greet\n#+BEGIN_SRC sh\nprintf library > marker; printf result\n#+END_SRC\n"
            );
        }
    }
}
// Restrictions belong to both the call context and referenced target; an
// override cannot erase no/never or query. Unsupported collection/expressions
// fail before any confirmed host operation can start.
#[test]
fn result_call_admission() {
    for (name, properties, target, call, expected) in [
        (
            "target never",
            "",
            ":eval never",
            "greet[:eval yes]()",
            "forbids execution",
        ),
        (
            "call context",
            "#+PROPERTY: header-args :eval no\n",
            "",
            "greet[:eval yes]()",
            "forbids execution",
        ),
        (
            "expression",
            "",
            ":var n=2",
            "greet(n=(system))",
            "unsupported",
        ),
        (
            "collection",
            "",
            ":results value",
            "greet() :results drawer",
            "matching declared",
        ),
        (
            "end hook",
            "",
            "",
            "greet() :results drawer\t:eval no",
            "end headers",
        ),
        ("comment", "* COMMENT Hidden\n", "", "greet()", "COMMENT"),
    ] {
        let source = format!(
            "{properties}#+NAME: greet\n#+BEGIN_SRC sh {target}\nprintf bad > marker\n#+END_SRC\n#+CALL: {call}\n#+RESULTS: greet\n: previous\n"
        );
        let (f, mut s) = configured(
            &source,
            &format!("org.source.confirm_execution=false\n{PROFILE}"),
        );
        let line = source
            .lines()
            .position(|l| l.starts_with("#+CALL:"))
            .unwrap()
            + 1;
        s.execute(Command::GotoLine);
        argument(&mut s, &line.to_string());
        action(&mut s, C::ExecuteSrc);
        assert!(s.echo().contains(expected), "{name}: {}", s.echo());
        assert!(s.execution_result().is_none(), "{name}");
        assert!(!f.0.join("marker").exists(), "{name}");
        assert_eq!(text(&s), source, "{name}");
    }
    let source = "#+NAME: greet\n#+BEGIN_SRC sh :eval query\nprintf bad > marker\n#+END_SRC\n#+CALL: greet[:eval yes]()\n";
    let (f, mut s) = configured(
        source,
        &format!("org.source.confirm_execution=false\n{PROFILE}"),
    );
    s.execute(Command::GotoLine);
    argument(&mut s, "5");
    action(&mut s, C::ExecuteSrc);
    assert!(s.execution_result().is_none());
    assert!(!f.0.join("marker").exists());
    argument(&mut s, "no");
    assert!(!f.0.join("marker").exists());
}
// A file changed by an actual running program invalidates the retained library
// observation after cleanup. Its output is inspectable but cannot replace text.
#[test]
fn result_library_stale() {
    let source = "#+CALL: greet()\n#+RESULTS: greet\n: previous\n";
    let (f, mut s) = configured(
        source,
        &format!("org.source.library_files=['lib.org']\n{PROFILE}"),
    );
    f.write(
        "lib.org",
        "#+NAME: greet\n#+BEGIN_SRC sh\nprintf changed > lib.org; printf result\n#+END_SRC\n",
    );
    run(&mut s);
    let out = s.execution_result().unwrap();
    assert_eq!(out.state, State::Stale, "{}", out.detail);
    assert_eq!(out.stdout, b"result");
    assert_eq!(text(&s), source);
    assert!(s.buffer().unwrap().history().is_empty());
}
// Canonical unsaved library text wins over disk. Observation and publication do
// not visit/save that library, and its retained source path stays inspectable.
#[test]
fn result_library_buffer() {
    let f = Fixture::new();
    let config = f.write(
        "config.toml",
        &format!("org.source.library_files=['lib.org']\n{PROFILE}"),
    );
    let snapshot = prepare(&config).snapshot;
    let library = f.write(
        "lib.org",
        "#+NAME: greet\n#+BEGIN_SRC sh\nprintf disk\n#+END_SRC\n",
    );
    let mut buffers = Buffers::with_settings(snapshot).unwrap();
    let id = buffers.visit(&library).unwrap();
    let draft = "#+NAME: greet\n#+BEGIN_SRC sh\nprintf unsaved\n#+END_SRC\n";
    let buffer = buffers.get_mut(id).unwrap();
    let edit = rev::buffer::edit::Replacement {
        source: buffer.span(0..buffer.len_chars()).unwrap(),
        text: draft.into(),
    };
    let prepared = buffer.prepare(&[edit], &[], None).unwrap();
    buffer.publish(prepared, &mut []).unwrap();
    let selected = buffers
        .visit(&f.write("source.org", "#+CALL: greet()\n"))
        .unwrap();
    let view = View::new(buffers.get(selected).unwrap());
    let mut s = Session::new(
        buffers,
        view,
        Size {
            width: 110,
            height: 16,
        },
    )
    .unwrap();
    run(&mut s);
    assert_eq!(s.execution_result().unwrap().state, State::Succeeded);
    assert_eq!(
        s.execution_result().unwrap().source_path.as_ref(),
        Some(&library)
    );
    assert!(text(&s).ends_with(": unsaved\n"));
    assert!(
        std::fs::read_to_string(&library)
            .unwrap()
            .contains("printf disk")
    );
    s.load(&library).unwrap();
    settle(&mut s);
    assert_eq!(text(&s), draft);
    assert!(s.buffer().unwrap().is_modified());
}
