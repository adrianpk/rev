use super::support::*;
use crossterm::event::{Event, KeyCode};
use rev::command::{Command, CompletionCommand as C, History};
use rev::config::Snapshot;
use std::path::Path;

struct Case {
    file: &'static str,
    document: &'static str,
    mode: &'static str,
    token: &'static str,
    wanted: &'static str,
}

pub(super) fn run(root: &Path, libraries: &super::assets::Compiled) -> Result<()> {
    for case in cases() {
        language(root, case)?;
    }
    automatic(root)?;
    org(root)?;
    external(root, libraries)?;
    println!(
        "PASS native workflows: all ten bundled parser/query pairs, literal selection/cancel/accept/undo, Unicode, automatic debounce/paste, Org contexts, real external Python and bundled fallback"
    );
    Ok(())
}

fn cases() -> [Case; 11] {
    [
        Case {
            file: "sample.rs",
            document: "fn configuration() {}\nfn main() { conf| }",
            mode: "Rust",
            token: "conf",
            wanted: "configuration",
        },
        Case {
            file: "sample.c",
            document: "int configuration(void) { return 0; }\nint main(void) { conf|; }",
            mode: "C",
            token: "conf",
            wanted: "configuration",
        },
        Case {
            file: "sample.go",
            document: "package main\nfunc configuration() {}\nfunc main() { conf| }",
            mode: "Go",
            token: "conf",
            wanted: "configuration",
        },
        Case {
            file: "sample.js",
            document: "function configuration() {}\nconf|",
            mode: "JavaScript",
            token: "conf",
            wanted: "configuration",
        },
        Case {
            file: "sample.ts",
            document: "function configuration(): void {}\nconf|",
            mode: "TypeScript",
            token: "conf",
            wanted: "configuration",
        },
        Case {
            file: "sample.tsx",
            document: "function configuration() {}\nconst ui = <div/>;\nconf|",
            mode: "TSX",
            token: "conf",
            wanted: "configuration",
        },
        Case {
            file: "sample.html",
            document: "<custom-card></custom-card>\n<cus|>",
            mode: "HTML",
            token: "cus",
            wanted: "custom-card",
        },
        Case {
            file: "sample.css",
            document: ".custom-card {}\n.cus| {}",
            mode: "CSS",
            token: "cus",
            wanted: "custom-card",
        },
        Case {
            file: "sample.json",
            document: "{\"configuration\": 1, \"conf|\": 2}",
            mode: "JSON",
            token: "\"conf\"",
            wanted: "\"configuration\"",
        },
        Case {
            file: "sample.toml",
            document: "configuration = 1\nconf| = 2\n",
            mode: "TOML",
            token: "conf",
            wanted: "configuration",
        },
        Case {
            file: "unicode.rs",
            document: "fn 配置值() {}\nfn main() { 配| }",
            mode: "Rust",
            token: "配",
            wanted: "配置值",
        },
    ]
}

fn language(root: &Path, case: Case) -> Result<()> {
    let mut session = open(root, case.file, case.document, Snapshot::default())?;
    assert_eq!(session.buffer()?.mode(), case.mode);
    let original = source(&session);
    let point = session.view().point;
    let offset = original.rfind(case.token).expect("replacement token");
    let mut expected = original.clone();
    expected.replace_range(offset..offset + case.token.len(), case.wanted);
    complete(&mut session)?;
    select(&mut session, case.wanted);
    assert_eq!(source(&session), original);
    assert_eq!(session.view().point, point);
    assert!(session.buffer()?.history().is_empty());
    session.handle(Event::Resize(32, 12));
    assert!(
        session
            .frame()?
            .rows()
            .iter()
            .any(|row| row.text().contains('┌'))
    );
    session.execute(Command::Completion(C::Cancel));
    assert!(session.completion().is_none());
    assert!(session.buffer()?.history().is_empty());
    complete(&mut session)?;
    select(&mut session, case.wanted);
    session.execute(Command::Completion(C::Accept));
    assert_eq!(source(&session), expected, "{}", case.file);
    assert_eq!(session.buffer()?.history().len(), 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(source(&session), original);
    session.execute(Command::History(History::Redo));
    assert_eq!(source(&session), expected);
    assert_eq!(
        std::fs::read_to_string(root.join("workspace").join(case.file))?,
        original
    );
    Ok(())
}

fn automatic(root: &Path) -> Result<()> {
    let mut session = open(
        root,
        "automatic.rs",
        "fn configuration() {}\nfn main() { c| }",
        Snapshot::default(),
    )?;
    key(&mut session, KeyCode::Char('o'));
    assert!(session.completion().is_none());
    settle(&mut session)?;
    assert!(session.completion().is_some());
    let original = source(&session);
    session.execute(Command::Completion(C::Cancel));
    idle(&mut session);
    assert_eq!(source(&session), original);
    session.handle(Event::Paste("nf".into()));
    assert!(session.completion().is_none());
    idle(&mut session);
    session.execute(Command::History(History::Undo));
    assert_eq!(source(&session), original);
    Ok(())
}

fn org(root: &Path) -> Result<()> {
    for (document, wanted) in [
        ("* T| task\n", "TODO"),
        ("* Task :release:\n* Peer :rel|:\n", "release"),
        ("#+TI|\n", "TITLE:"),
        (
            "* One\n:PROPERTIES:\n:OWNER: Ada\n:END:\n* Two\n:PROPERTIES:\n:OW|\n:END:\n",
            "OWNER:",
        ),
        ("See [[fi|]]", "file:"),
        ("* Target heading\n[[*Tar|]]\n", "Target%20heading"),
    ] {
        let mut session = open(root, "notes.org", document, Snapshot::default())?;
        let original = source(&session);
        complete(&mut session)?;
        select(&mut session, wanted);
        session.execute(Command::Completion(C::Accept));
        assert_ne!(source(&session), original);
        session.execute(Command::History(History::Undo));
        assert_eq!(source(&session), original);
    }
    let mut session = open(root, "prose.org", "ordinary pro|se\n", Snapshot::default())?;
    complete(&mut session)?;
    assert!(session.completion().is_none());
    assert!(session.buffer()?.history().is_empty());
    Ok(())
}

fn external(root: &Path, libraries: &super::assets::Compiled) -> Result<()> {
    let highlights = root.join("external-highlights.scm");
    let completions = root.join("external-completions.scm");
    std::fs::write(&highlights, "(identifier) @function")?;
    std::fs::write(
        &completions,
        "(function_definition name: (identifier) @function)",
    )?;
    let config = format!(
        "[language.python.external]\n{}",
        super::assets::registration(
            libraries.parser("python"),
            "tree_sitter_python",
            &highlights,
            &completions
        )
    );
    let config_path = root.join("external.toml");
    std::fs::write(&config_path, config)?;
    let settings = rev::config::prepare(&config_path);
    assert!(settings.diagnostic.is_none());
    let mut session = open(
        root,
        "external.py",
        "def configuration():\n    pass\nconf|\n",
        settings.snapshot,
    )?;
    super::assets::settle(&mut session);
    assert_eq!(session.buffer()?.mode(), "External");
    complete(&mut session)?;
    select(&mut session, "configuration");
    let original = source(&session);
    session.execute(Command::Completion(C::Accept));
    assert!(source(&session).ends_with("configuration\n"));
    session.execute(Command::History(History::Undo));
    assert_eq!(source(&session), original);
    fallback(root, &highlights, &completions)
}

fn fallback(root: &Path, highlights: &Path, completions: &Path) -> Result<()> {
    let config = format!(
        "[language.rust.external]\n{}",
        super::assets::registration(
            &root.join("missing.so"),
            "tree_sitter_rust",
            highlights,
            completions
        )
    );
    let config_path = root.join("fallback.toml");
    std::fs::write(&config_path, config)?;
    let settings = rev::config::prepare(&config_path);
    let mut fallback = open(
        root,
        "fallback.rs",
        "fn configuration() {}\nfn main() { conf| }",
        settings.snapshot,
    )?;
    settle(&mut fallback)?;
    assert!(
        fallback
            .buffer()?
            .external_grammar_status()
            .unwrap()
            .is_err()
    );
    assert_eq!(fallback.buffer()?.mode(), "Rust");
    complete(&mut fallback)?;
    select(&mut fallback, "configuration");
    Ok(())
}
