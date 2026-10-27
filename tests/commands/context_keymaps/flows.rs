use super::*;

// Candidate and history aliases retain distinct targets. Custom movement and
// completion never rewrite history or invoke a command before acceptance.
#[test]
fn candidate_aliases() {
    let mut input = Input::with_settings(&settings(
        "[keys.minibuffer]\n'M-j' = 'minibuffer-next'\n'M-k' = 'minibuffer-previous'\n'M-h' = 'minibuffer-history-previous'\n'M-l' = 'minibuffer-history-next'\n'M-c' = 'minibuffer-complete'\n'M-a' = 'minibuffer-accept'",
    ));
    input.commands().unwrap();
    input.handle(Event::Paste("newline".into()));
    assert_eq!(input.handle(meta('a')), Outcome::Command(Command::Newline));
    input.commands().unwrap();
    input.handle(Event::Paste("split-window".into()));
    input.handle(meta('j'));
    assert_eq!(input.palette().unwrap().selected(), Some(1));
    assert_eq!(input.prompt().unwrap().text(), "split-window");
    input.handle(meta('k'));
    assert_eq!(input.palette().unwrap().selected(), Some(0));
    input.handle(meta('h'));
    assert_eq!(input.prompt().unwrap().text(), "newline");
    input.handle(meta('l'));
    assert_eq!(input.prompt().unwrap().text(), "split-window");
    input.handle(meta('j'));
    input.handle(meta('c'));
    assert_eq!(input.prompt().unwrap().text(), "split-window-right");
    assert_eq!(
        input.handle(meta('a')),
        Outcome::Command(Command::Split(rev::command::Split::Right))
    );
}

// A later selected row invokes the native file handler. Held accept cannot
// cross into its argument stage or publish a second save; fresh acceptance can.
#[test]
fn selected_arguments() {
    for (name, config, next, accept) in [
        ("native", "", key(KeyCode::Down), key(KeyCode::Enter)),
        (
            "aliases",
            "[keys.minibuffer]\n'M-j' = 'minibuffer-next'\n'M-a' = 'minibuffer-accept'",
            meta('j'),
            meta('a'),
        ),
    ] {
        let fixture = super::super::config_fixture::Fixture::new();
        let target = fixture.0.join("selected.txt");
        let mut session = editor(config, "retained界\n");
        let stamp = session.buffer().unwrap().stamp();
        session.execute(Command::ExecuteExtended);
        session.handle(Event::Paste("file".into()));
        for _ in 0..4 {
            session.handle(next.clone());
        }
        session.handle(accept.clone());
        for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
            assert_eq!(
                session.handle(phase(accept.clone(), kind)),
                rev::session::Action::Wait,
                "{name}"
            );
        }
        assert!(
            session
                .frame()
                .unwrap()
                .rows()
                .last()
                .unwrap()
                .text()
                .starts_with("Write file: "),
            "{name}"
        );
        session.handle(Event::Paste(target.to_str().unwrap().into()));
        session.handle(phase(accept.clone(), KeyEventKind::Repeat));
        assert!(!target.exists(), "{name}");
        session.handle(accept.clone());
        assert_eq!(
            std::fs::read(&target).unwrap(),
            "retained界\n".as_bytes(),
            "{name}"
        );
        assert_eq!(session.buffer().unwrap().stamp(), stamp, "{name}");
        assert!(session.buffer().unwrap().history().is_empty(), "{name}");
        assert_eq!(
            session.handle(phase(accept, KeyEventKind::Repeat)),
            rev::session::Action::Wait,
            "{name}"
        );
        assert_eq!(text(&session), "retained界\n", "{name}");
    }
}

// C-g retires candidate navigation and its draft, returning to the same tree
// selection or editor view without changing text, history or file identity.
#[test]
fn selected_cancellation() {
    for from_tree in [false, true] {
        let (_fixture, mut session) = tree_editor("[keys.minibuffer]\n'M-j' = 'minibuffer-next'");
        if !from_tree {
            session.handle(key(KeyCode::Tab));
        }
        session.frame().unwrap();
        let view = session.view().clone();
        let stamp = session.buffer().unwrap().stamp();
        let selected = session.tree().selected().unwrap().path().to_owned();
        session.execute(Command::ExecuteExtended);
        session.handle(Event::Paste("file".into()));
        session.handle(meta('j'));
        session.frame().unwrap();
        session.handle(control('g'));
        session.frame().unwrap();
        assert_eq!(session.tree().focused(), from_tree);
        assert_eq!(session.tree().selected().unwrap().path(), selected);
        assert_eq!(*session.view(), view);
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert!(session.buffer().unwrap().history().is_empty());
        assert_eq!(text(&session), "retained");
    }
}

// A real Unicode path is repaired through configured prompt handlers. Accept
// opens that file once; completion guidance and editing never touch source text.
#[test]
fn find_file_aliases() {
    let fixture = super::super::config_fixture::Fixture::new();
    let path = fixture.write("e\u{301}界.txt", "opened");
    let mut session = editor(
        "[keys.minibuffer]\n'M-b' = 'backward-char'\n'M-d' = 'delete-char'\n'M-a' = 'minibuffer-accept'\n'M-c' = 'minibuffer-complete'",
        "retained",
    );
    let stamp = session.buffer().unwrap().stamp();
    session.execute(Command::FindFile);
    session.handle(Event::Paste(format!("{}BAD", path.display())));
    for _ in 0..3 {
        session.handle(meta('b'));
    }
    for _ in 0..3 {
        session.handle(meta('d'));
    }
    session.handle(meta('c'));
    assert_eq!(session.echo(), "");
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    session.handle(meta('a'));
    session.handle(phase(meta('a'), KeyEventKind::Repeat));
    assert_eq!(text(&session), "opened");
    assert_eq!(session.buffer().unwrap().file().unwrap().path(), path);
    assert!(session.buffer().unwrap().history().is_empty());
    session.handle(character('界'));
    assert_eq!(text(&session), "界opened");
    assert_eq!(std::fs::read(path).unwrap(), b"opened");
}

// A command accept alias can acquire the next argument without a held chord
// submitting it. Cancellation restores the original view and retires all text.
#[test]
fn command_continuation() {
    let mut session = editor("[keys.minibuffer]\n'M-a' = 'minibuffer-accept'", "retained");
    let view = session.view().clone();
    session.execute(Command::ExecuteExtended);
    session.handle(Event::Paste("find-file".into()));
    session.handle(meta('a'));
    session.handle(phase(meta('a'), KeyEventKind::Repeat));
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .find(|row| row.band() == rev::display::Band::Echo)
            .unwrap()
            .text()
            .starts_with("Find file: ")
    );
    session.handle(character('é'));
    session.handle(control('g'));
    assert_eq!(session.view(), &view);
    assert_eq!(text(&session), "retained");
    session.handle(character('a'));
    assert_eq!(text(&session), "aretained");
}

// The same prefix owner is bounded in prompts. Paste is literal even for bound
// characters; mismatches, acceptance and cancellation leave no delayed suffix.
#[test]
fn prompt_prefixes() {
    let mut input = Input::with_settings(&settings(
        "[keys.minibuffer]\n'M-z a b c d e f g' = 'minibuffer-accept'\n'a' = 'minibuffer-accept'",
    ));
    input.argument("Text: ", "").unwrap();
    input.handle(meta('z'));
    input.handle(Event::Paste("aé".into()));
    assert_eq!(input.prefix(), None);
    assert_eq!(input.prompt().unwrap().text(), "aé");
    input.handle(meta('z'));
    assert_eq!(
        input.handle(character('x')),
        Outcome::Message("Undefined key sequence")
    );
    assert_eq!(input.prompt().unwrap().text(), "aé");
    input.handle(meta('z'));
    for value in ['a', 'b', 'c', 'd', 'e', 'f'] {
        assert!(matches!(input.handle(character(value)), Outcome::Prefix(_)));
    }
    assert_eq!(input.prefix_label().as_deref(), Some("M-z a b c d e f"));
    assert_eq!(input.handle(character('g')), Outcome::Argument("aé".into()));
    assert_eq!(input.prefix(), None);
    assert_eq!(
        input.handle(phase(character('g'), KeyEventKind::Repeat)),
        Outcome::Ignored
    );
}

// Escape expiry clears an entire minibuffer sequence while retaining its draft.
// Compatible Escape delivery before expiry resolves the same configured alias.
#[test]
fn prompt_meta() {
    use std::time::Instant;
    let now = Instant::now();
    let mut input = Input::with_settings(&settings(
        "[keys.minibuffer]\n'M-z M-a' = 'minibuffer-accept'",
    ));
    input.argument("Text: ", "draft").unwrap();
    input.handle_at(meta('z'), now);
    input.handle_at(key(KeyCode::Esc), now);
    input.expire(now + rev::input::META_TIMEOUT);
    assert_eq!(input.prefix(), None);
    assert_eq!(input.focus(), Focus::Argument);
    assert_eq!(input.prompt().unwrap().text(), "draft");
    input.handle_at(meta('z'), now + rev::input::META_TIMEOUT);
    input.handle_at(Event::Resize(1, 1), now + rev::input::META_TIMEOUT);
    input.handle_at(key(KeyCode::Esc), now + rev::input::META_TIMEOUT);
    assert_eq!(
        input.handle_at(character('a'), now + rev::input::META_TIMEOUT),
        Outcome::Argument("draft".into())
    );
}

// Search aliases repeat within the retained search owner rather than opening a
// new search or modifying text. Aliased acceptance commits the selected result.
#[test]
fn search_aliases() {
    let mut session = editor(
        "[keys.minibuffer]\n'M-s' = 'isearch-forward'\n'M-r' = 'isearch-backward'\n'M-a' = 'minibuffer-accept'",
        "cat cat",
    );
    let stamp = session.buffer().unwrap().stamp();
    session.execute(Command::SearchForward);
    session.handle(Event::Paste("cat".into()));
    assert_eq!(session.view().point.position, rev::coordinates::Scalar(3));
    session.handle(meta('s'));
    assert_eq!(session.view().point.position, rev::coordinates::Scalar(7));
    session.handle(meta('r'));
    assert_eq!(session.view().point.position, rev::coordinates::Scalar(0));
    session.handle(meta('a'));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .all(|row| row.search().is_none())
    );
    session.handle(character('!'));
    assert_eq!(text(&session), "!cat cat");
}

// Completion in literal query/replacement stages inserts a tab through their
// existing owner. It neither invokes ordinary completion nor interprets text.
#[test]
fn replacement_tabs() {
    let mut session = editor(
        "[keys.minibuffer]\n'M-c' = 'minibuffer-complete'\n'M-a' = 'minibuffer-accept'",
        "cat\tdog",
    );
    session.execute(Command::QueryReplace);
    session.handle(Event::Paste("cat".into()));
    session.handle(meta('c'));
    session.handle(Event::Paste("dog".into()));
    session.handle(meta('a'));
    session.handle(meta('c'));
    session.handle(meta('a'));
    assert_eq!(text(&session), "cat\tdog");
    session.handle(character('y'));
    assert_eq!(text(&session), "\t");
    assert_eq!(session.buffer().unwrap().history().len(), 1);
}

// Reassignment replaces only the tree chord. Acquiring/canceling M-x clears a
// tree prefix; returning to editing also retires its suffix and held navigation.
#[test]
fn tree_transitions() {
    let (_fixture, mut session) = tree_editor(
        "[keys.tree]\n'C-n' = 'tree-previous-line'\n'M-z a' = 'tree-open'\n'M-a' = 'execute-extended-command'\n'M-e' = 'insert-tab'",
    );
    let root = session.tree().root().unwrap().to_path_buf();
    session.execute(Command::Tree(TreeCommand::Expand));
    session.handle(key(KeyCode::Down));
    session.handle(control('n'));
    assert_eq!(session.tree().selected().unwrap().path(), root);
    session.handle(key(KeyCode::Down));
    session.handle(meta('z'));
    session.execute(Command::ExecuteExtended);
    session.handle(character('a'));
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .find(|row| row.band() == rev::display::Band::Echo)
            .unwrap()
            .text()
            .starts_with("M-x a")
    );
    session.handle(control('g'));
    assert!(session.tree().focused());
    assert_eq!(text(&session), "retained");
    session.handle(meta('a'));
    session.handle(control('g'));
    session.handle(meta('z'));
    session.execute(Command::InsertTab);
    assert!(!session.tree().focused());
    session.handle(phase(character('a'), KeyEventKind::Repeat));
    assert_eq!(text(&session), "retained");
    session.handle(character('a'));
    assert_eq!(text(&session), "aretained");
}

// Configured create and rename acquire their native filesystem captures. A
// canceled argument leaves disk unchanged; a later accepted name publishes once.
#[test]
fn filesystem_arguments() {
    for (operation, alias) in [("create", 'n'), ("rename", 'r')] {
        let (fixture, mut session) = tree_editor(
            "[keys.tree]\n'M-n' = 'tree-create-file'\n'M-r' = 'tree-rename'\n[keys.minibuffer]\n'M-a' = 'minibuffer-accept'\n'M-z a' = 'minibuffer-accept'",
        );
        session.execute(Command::Tree(TreeCommand::Expand));
        if operation == "rename" {
            session.handle(key(KeyCode::Down));
        }
        let stamp = session.buffer().unwrap().stamp();
        session.handle(meta(alias));
        session.handle(Event::Paste("new-é.txt".into()));
        session.handle(meta('z'));
        session.handle(control('g'));
        assert!(!fixture.0.join("root/new-é.txt").exists(), "{operation}");
        assert!(fixture.0.join("root/a.txt").exists(), "{operation}");
        assert_eq!(session.buffer().unwrap().stamp(), stamp, "{operation}");
        assert!(session.tree().focused());
        session.handle(meta(alias));
        session.handle(Event::Paste("new-é.txt".into()));
        session.handle(meta('a'));
        session.handle(phase(meta('a'), KeyEventKind::Repeat));
        assert!(fixture.0.join("root/new-é.txt").is_file(), "{operation}");
        assert_eq!(fixture.0.join("root/a.txt").exists(), operation == "create");
        assert_eq!(text(&session), "retained");
    }
}

// Recursive deletion still requires a second fresh decision. A mapped accept
// cannot substitute for it, and C-g discards both filesystem and prefix state.
#[test]
fn recursive_consent() {
    let (fixture, mut session) = tree_editor(
        "[keys.tree]\n'M-d' = 'tree-delete'\n[keys.minibuffer]\n'M-a' = 'minibuffer-accept'",
    );
    fixture.write("root/dir/item", "child");
    session.execute(Command::Tree(TreeCommand::Expand));
    session.handle(key(KeyCode::Down));
    session.handle(meta('d'));
    session.handle(character('y'));
    assert!(session.echo().contains("recursively delete"));
    session.handle(phase(character('y'), KeyEventKind::Repeat));
    session.handle(meta('a'));
    assert!(fixture.0.join("root/dir/item").exists());
    session.handle(control('g'));
    assert!(session.tree().focused());
    session.handle(meta('d'));
    session.handle(character('y'));
    session.handle(character('y'));
    assert!(!fixture.0.join("root/dir").exists());
    assert_eq!(text(&session), "retained");
}

// A configured tree-open prefix reaches the ordinary buffer visit owner once;
// a held suffix cannot reopen a file or become document input after dispatch.
#[test]
fn tree_open_sequence() {
    let (_fixture, mut session) = tree_editor("[keys.tree]\n'M-z a' = 'tree-open'");
    session.execute(Command::Tree(TreeCommand::Expand));
    session.handle(key(KeyCode::Down));
    session.handle(key(KeyCode::Down));
    let original = session.buffer().unwrap().id();
    session.handle(meta('z'));
    assert_eq!(text(&session), "retained");
    session.handle(character('a'));
    let opened = session.buffer().unwrap().id();
    assert_ne!(opened, original);
    assert_eq!(text(&session), "other");
    session.handle(phase(character('a'), KeyEventKind::Repeat));
    assert_eq!(session.buffer().unwrap().id(), opened);
    assert_eq!(text(&session), "other");
    assert!(session.buffer().unwrap().history().is_empty());
}
