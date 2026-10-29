use super::*;
use rev::{command::Motion, file::Publication};
fn editor(source: &str) -> (Fixture, Session) {
    let f = Fixture::new();
    let path = f.write("source.org", source);
    let s = host(&f, Some(&path), false);
    (f, s)
}
// The real chord extracts unsaved canonical source, joins named fragments and
// creates only explicitly admitted parents. BOM, CRLF and source history survive.
#[test]
fn native_tangle() {
    let source = "\u{feff}#+NAME: fragment\r\n#+BEGIN_SRC sh\r\n  printf '界'\r\n#+END_SRC\r\n#+BEGIN_SRC sh :tangle \"out dir/one.sh\" :mkdirp yes :noweb yes\r\n<<fragment>>\r\n#+END_SRC\r\n#+BEGIN_SRC text :tangle two.txt\r\n  ,#+END_SRC\r\n#+END_SRC\r\n";
    let (f, mut s) = editor(source);
    s.execute(Command::Motion(Motion::BufferEnd));
    s.handle(Event::Paste("unsaved note".into()));
    let text_before = text(&s);
    let stamp = s.buffer().unwrap().stamp();
    let groups = s.buffer().unwrap().history().len();
    s.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('c'),
        KeyModifiers::CONTROL,
    )));
    s.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('v'),
        KeyModifiers::CONTROL,
    )));
    s.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('t'),
        KeyModifiers::NONE,
    )));
    settle(&mut s);
    assert_eq!(
        std::fs::read(f.0.join("out dir/one.sh")).unwrap(),
        "printf '界'\n".as_bytes()
    );
    assert_eq!(std::fs::read(f.0.join("two.txt")).unwrap(), b"#+END_SRC\n");
    assert_eq!(text(&s), text_before);
    assert_eq!(s.buffer().unwrap().stamp(), stamp);
    assert_eq!(s.buffer().unwrap().history().len(), groups);
    assert!(s.buffer().unwrap().has_bom());
    assert!(s.buffer().unwrap().is_modified());
    assert_eq!(
        std::fs::read(f.0.join("source.org")).unwrap(),
        source.as_bytes()
    );
    assert_eq!(s.tangling_results().len(), 2);
    assert!(
        s.tangling_results()
            .iter()
            .all(|r| r.publication == Publication::Complete),
        "{}",
        s.echo()
    );
}
// Expansion/path failures are discovered for the whole plan before replacement
// consent or any output write, including a valid earlier target in source order.
#[test]
fn tangle_preflight() {
    for (name, tail) in [
        (
            "unresolved",
            "#+BEGIN_SRC sh :tangle second.sh :noweb yes\n<<missing>>\n#+END_SRC\n",
        ),
        (
            "cycle",
            "#+NAME: loop\n#+BEGIN_SRC sh :tangle second.sh :noweb yes\n<<loop>>\n#+END_SRC\n",
        ),
        (
            "traversal",
            "#+BEGIN_SRC sh :tangle ../second.sh\nexit 99\n#+END_SRC\n",
        ),
        (
            "source target",
            "#+BEGIN_SRC sh :tangle source.org\nexit 99\n#+END_SRC\n",
        ),
        (
            "missing parent",
            "#+BEGIN_SRC sh :tangle missing/second.sh\nexit 99\n#+END_SRC\n",
        ),
    ] {
        let source = format!("#+BEGIN_SRC sh :tangle first.sh\nnew first\n#+END_SRC\n{tail}");
        let (f, mut s) = editor(&source);
        let first = f.write("first.sh", "old first");
        action(&mut s, C::Tangle);
        assert_eq!(
            std::fs::read_to_string(first).unwrap(),
            "old first",
            "{name}"
        );
        assert!(!f.0.join("second.sh").exists(), "{name}");
        assert!(!f.0.join("missing").exists(), "{name}");
        assert_eq!(text(&s), source, "{name}");
        assert!(s.tangling_results().is_empty(), "{name}");
    }
}
const TWO: &str = "#+BEGIN_SRC text :tangle first.txt\nfirst new\n#+END_SRC\n#+BEGIN_SRC text :tangle second.txt\nsecond new\n#+END_SRC\n";
// Replacement consent is independent for each file. Cancellation keeps actual
// earlier output and records the unperformed replacement without altering it.
#[test]
fn tangle_partial_cancel() {
    let (f, mut s) = editor(TWO);
    f.write("first.txt", "first old");
    f.write("second.txt", "second old");
    action(&mut s, C::Tangle);
    assert!(
        s.frame()
            .unwrap()
            .rows()
            .iter()
            .any(|r| r.text().contains("Replace tangle"))
    );
    argument(&mut s, "yes");
    assert_eq!(
        std::fs::read_to_string(f.0.join("first.txt")).unwrap(),
        "first new\n"
    );
    s.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('g'),
        KeyModifiers::CONTROL,
    )));
    settle(&mut s);
    assert_eq!(
        std::fs::read_to_string(f.0.join("second.txt")).unwrap(),
        "second old"
    );
    assert_eq!(text(&s), TWO);
    let outcomes = s.tangling_results();
    assert_eq!(outcomes.len(), 2);
    assert_eq!(outcomes[0].publication, Publication::Complete);
    assert_eq!(outcomes[1].publication, Publication::NotPublished);
    action(&mut s, C::TangleResults);
    argument(&mut s, "2");
    assert!(s.echo().contains("cancelled"), "{}", s.echo());
    assert_eq!(s.remaining_input(Instant::now()), None);
}
// An external change after planning rejects the stale replacement. A completed
// first output is retained and every file has its actual independent outcome.
#[test]
fn tangle_changed_target() {
    let (f, mut s) = editor(TWO);
    f.write("first.txt", "first old");
    let second = f.write("second.txt", "second old");
    action(&mut s, C::Tangle);
    std::fs::write(&second, "external change").unwrap();
    argument(&mut s, "yes");
    argument(&mut s, "yes");
    assert_eq!(
        std::fs::read_to_string(f.0.join("first.txt")).unwrap(),
        "first new\n"
    );
    assert_eq!(std::fs::read_to_string(second).unwrap(), "external change");
    assert_eq!(s.tangling_results().len(), 2);
    assert_eq!(s.tangling_results()[0].publication, Publication::Complete);
    assert_eq!(
        s.tangling_results()[1].publication,
        Publication::NotPublished
    );
    assert_eq!(text(&s), TWO);
}

// Every filesystem alias is rejected during whole-plan preflight, including
// aliases of live inputs and of an earlier output. The earlier file stays exact.
#[test]
fn tangle_alias_preflight() {
    use std::os::unix::fs::symlink;
    for mode in [
        "leaf",
        "parent",
        "source hardlink",
        "output hardlink",
        "live output",
    ] {
        let tail = if mode == "parent" {
            "alias/second.txt"
        } else {
            "second.txt"
        };
        let source = TWO.replace("second.txt", tail);
        let (f, mut s) = editor(&source);
        let first = f.write("first.txt", "first old");
        let other = f.write("unowned.txt", "unowned");
        let second = f.0.join("second.txt");
        match mode {
            "leaf" => symlink(&other, &second).unwrap(),
            "parent" => symlink(&f.0, f.0.join("alias")).unwrap(),
            "source hardlink" => std::fs::hard_link(f.0.join("source.org"), &second).unwrap(),
            "output hardlink" => std::fs::hard_link(&first, &second).unwrap(),
            "live output" => {
                f.write("second.txt", "live bytes");
                s.load(&second).unwrap();
                s.load(&f.0.join("source.org")).unwrap();
            }
            _ => unreachable!(),
        }
        action(&mut s, C::Tangle);
        assert_eq!(
            std::fs::read_to_string(first).unwrap(),
            "first old",
            "{mode}"
        );
        assert_eq!(std::fs::read_to_string(other).unwrap(), "unowned", "{mode}");
        assert_eq!(
            std::fs::read_to_string(f.0.join("source.org")).unwrap(),
            source,
            "{mode}"
        );
        assert!(s.tangling_results().is_empty(), "{mode}: {}", s.echo());
        assert!(
            !s.echo().contains("Tangle finished"),
            "{mode}: {}",
            s.echo()
        );
    }
}
// Swapping a checked ancestor for a symlink cannot redirect publication. Earlier
// output remains complete and the blocked second output retains its failure.
#[test]
fn tangle_parent_swap() {
    let source = TWO.replace("second.txt", "out/second.txt");
    let (f, mut s) = editor(&source);
    f.write("first.txt", "first old");
    f.write("out/second.txt", "second old");
    f.write("unowned/second.txt", "unowned");
    action(&mut s, C::Tangle);
    std::fs::rename(f.0.join("out"), f.0.join("held")).unwrap();
    std::os::unix::fs::symlink(f.0.join("unowned"), f.0.join("out")).unwrap();
    argument(&mut s, "yes");
    argument(&mut s, "yes");
    assert_eq!(
        std::fs::read_to_string(f.0.join("first.txt")).unwrap(),
        "first new\n"
    );
    assert_eq!(
        std::fs::read_to_string(f.0.join("held/second.txt")).unwrap(),
        "second old"
    );
    assert_eq!(
        std::fs::read_to_string(f.0.join("unowned/second.txt")).unwrap(),
        "unowned"
    );
    assert_eq!(s.tangling_results().len(), 2);
    assert_eq!(s.tangling_results()[0].publication, Publication::Complete);
    assert_eq!(
        s.tangling_results()[1].publication,
        Publication::NotPublished
    );
}

// The real prefix selects one block on read-only source. Default language paths
// and ordinary replacement permissions work without visiting or saving output.
#[test]
fn tangle_readonly_selection() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let source = "#+BEGIN_SRC text :tangle other.txt\nuntouched\n#+END_SRC\n#+BEGIN_SRC python :tangle yes\nprint('界')\n#+END_SRC\n";
    let f = Fixture::new();
    let path = f.write("source.org", source);
    let output = f.write("source.py", "old code");
    f.write("other.txt", "other old");
    std::fs::set_permissions(&output, std::fs::Permissions::from_mode(0o640)).unwrap();
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    buffers.get_mut(id).unwrap().set_readonly(true);
    let view = View::new(buffers.get(id).unwrap());
    let mut s = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 12,
        },
    )
    .unwrap();
    s.execute(Command::GotoLine);
    argument(&mut s, "5");
    let stamp = s.buffer().unwrap().stamp();
    for (value, modifiers) in [
        ('u', KeyModifiers::CONTROL),
        ('c', KeyModifiers::CONTROL),
        ('v', KeyModifiers::CONTROL),
        ('t', KeyModifiers::NONE),
    ] {
        s.handle(Event::Key(KeyEvent::new(KeyCode::Char(value), modifiers)));
    }
    settle(&mut s);
    assert_eq!(std::fs::read_to_string(&output).unwrap(), "old code");
    argument(&mut s, "yes");
    assert_eq!(std::fs::read_to_string(&output).unwrap(), "print('界')\n");
    assert_eq!(std::fs::metadata(&output).unwrap().mode() & 0o777, 0o640);
    assert_eq!(
        std::fs::read_to_string(f.0.join("other.txt")).unwrap(),
        "other old"
    );
    assert_eq!(text(&s), source);
    assert_eq!(s.buffer().unwrap().stamp(), stamp);
    assert!(s.buffer().unwrap().is_readonly());
    assert!(s.buffer().unwrap().history().is_empty());
    assert!(!s.buffer().unwrap().is_modified());
    assert_eq!(s.tangling_results().len(), 1);
    assert_eq!(s.tangling_results()[0].publication, Publication::Complete);
}
// Invalid consent keeps the same prepared target; declining one replacement
// continues to the next file without applying global or remembered consent.
#[test]
fn tangle_independent_consent() {
    let (f, mut s) = editor(TWO);
    f.write("first.txt", "first old");
    f.write("second.txt", "second old");
    action(&mut s, C::Tangle);
    argument(&mut s, "maybe");
    assert!(s.tangling_results().is_empty());
    argument(&mut s, "no");
    assert_eq!(
        std::fs::read_to_string(f.0.join("first.txt")).unwrap(),
        "first old"
    );
    assert_eq!(
        std::fs::read_to_string(f.0.join("second.txt")).unwrap(),
        "second old"
    );
    argument(&mut s, "yes");
    assert_eq!(s.tangling_results().len(), 2);
    assert_eq!(
        s.tangling_results()[0].publication,
        Publication::NotPublished
    );
    assert_eq!(s.tangling_results()[1].publication, Publication::Complete);
    assert_eq!(
        std::fs::read_to_string(f.0.join("second.txt")).unwrap(),
        "second new\n"
    );
    action(&mut s, C::Tangle);
    assert!(s.tangling_results().is_empty());
    cancel(&mut s);
    settle(&mut s);
    assert_eq!(s.tangling_results().len(), 2);
    assert!(
        s.tangling_results()
            .iter()
            .all(|r| r.publication == Publication::NotPublished)
    );
    assert_eq!(
        std::fs::read_to_string(f.0.join("second.txt")).unwrap(),
        "second new\n"
    );
}
// Queued preflight owns source input and is cancellable before any output.
// Once reaped, ordinary editing resumes and the idle session has no wakeups.
#[test]
fn tangle_queued_cancel() {
    let (f, mut s) = editor(TWO);
    let stamp = s.buffer().unwrap().stamp();
    s.execute(Command::Org(C::Tangle));
    s.handle(Event::Paste("blocked".into()));
    s.execute(Command::DeleteChar);
    assert!(s.load(&f.0.join("source.org")).is_err());
    assert_eq!(text(&s), TWO);
    assert_eq!(s.buffer().unwrap().stamp(), stamp);
    cancel(&mut s);
    settle(&mut s);
    assert!(!f.0.join("first.txt").exists());
    assert!(!f.0.join("second.txt").exists());
    assert_eq!(s.remaining_input(Instant::now()), None);
    s.handle(Event::Paste("resumed ".into()));
    assert!(text(&s).starts_with("resumed "));
}
