use crate::support::{Result, key, open, source};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::Command;
use rev::command::usage::{History, Store};
use rev::config::{Palette, prepare};
use rev::display::Band;
use rev::session::Session;
use std::path::Path;

// Literal promotions exercise the real frame projection as well as ranking.
// Policy variants stay native instead of multiplying expensive PTY journeys.
pub(super) fn run(root: &Path) -> Result<()> {
    let recent = [
        "find-file",
        "toggle-file-tree",
        "query-replace",
        "undo-redo",
    ];
    let frequent: Vec<_> = [
        ("undo", 8),
        ("save-buffer", 7),
        ("newline", 6),
        ("goto-line", 5),
    ]
    .into_iter()
    .flat_map(|(name, count)| std::iter::repeat_n(name, count))
    .chain(recent)
    .collect();
    let matches = vec![
        "kill-buffer",
        "kill-word",
        "backward-kill-word",
        "undo-redo",
    ];
    for (name, policy, query, events, expected) in [
        (
            "empty",
            "prefix-then-substring",
            "",
            vec![],
            vec!["ai-apply-result", "ai-cancel"],
        ),
        (
            "overlap",
            "prefix-then-substring",
            "",
            vec!["undo", "newline", "undo"],
            vec!["undo", "newline"],
        ),
        (
            "four-four",
            "prefix-then-substring",
            "",
            frequent,
            vec![
                "undo-redo",
                "query-replace",
                "toggle-file-tree",
                "find-file",
                "undo",
                "save-buffer",
                "newline",
                "goto-line",
            ],
        ),
        (
            "ties",
            "prefix-then-substring",
            "",
            vec![
                "write-file",
                "save-buffer",
                "newline",
                "goto-line",
                "revert-buffer",
                "mouse-mode",
                "undo",
                "undo-redo",
                "toggle-file-tree",
                "find-file",
            ],
            vec![
                "find-file",
                "toggle-file-tree",
                "undo-redo",
                "undo",
                "mouse-mode",
                "revert-buffer",
                "goto-line",
                "newline",
            ],
        ),
        (
            "prefix-first",
            "prefix-then-substring",
            "kill",
            matches.clone(),
            vec![
                "kill-word",
                "kill-buffer",
                "kill-line",
                "kill-rectangle",
                "kill-region",
                "kill-ring-save",
                "backward-kill-word",
            ],
        ),
        (
            "prefix-only",
            "prefix",
            "kill",
            matches.clone(),
            vec![
                "kill-word",
                "kill-buffer",
                "kill-line",
                "kill-rectangle",
                "kill-region",
                "kill-ring-save",
            ],
        ),
        (
            "substring",
            "substring",
            "kill",
            matches,
            vec!["backward-kill-word", "kill-word", "kill-buffer"],
        ),
        (
            "exact",
            "substring",
            "undo",
            vec!["undo-redo"],
            vec!["undo", "undo-redo"],
        ),
        (
            "removed",
            "prefix-then-substring",
            "",
            vec!["ai-removed", "find-file"],
            vec!["find-file"],
        ),
        (
            "registered",
            "prefix-then-substring",
            "ai-review",
            vec!["ai-review-more"],
            vec!["ai-review", "ai-review-more"],
        ),
    ] {
        let fixture = root.join(name);
        let path = fixture.join("history.toml");
        std::fs::create_dir_all(&fixture)?;
        if !events.is_empty() {
            let (mut store, _) = Store::open(&path)?;
            store.publish(&events.iter().map(|name| (*name).into()).collect::<Vec<_>>())?;
        }
        let mut editor = session(&fixture, &path, policy)?;
        editor.execute(Command::ExecuteExtended);
        editor.handle(Event::Paste(query.into()));
        let rows = names(&mut editor)?;
        assert_eq!(&rows[..expected.len()], expected, "{name}");
        assert!(rows.iter().all(|name| !name.contains("removed")), "{name}");
        assert_eq!(
            rows.iter().collect::<std::collections::BTreeSet<_>>().len(),
            rows.len()
        );
        if name == "empty" {
            assert!(rows.windows(2).all(|pair| pair[0] < pair[1]));
            key(&mut editor, KeyCode::Enter);
            assert_eq!(editor.echo(), "No AI result is ready");
        } else {
            // Inspect the writer's actual selected colors, not just row metadata.
            let frame = editor.frame()?;
            let row = frame
                .rows()
                .iter()
                .position(|row| row.band() == Band::Completion && row.selection().is_some())
                .ok_or("missing selected row")?;
            let cells = crate::surface::draw(&frame);
            assert_eq!(
                cells[&(row, 0)].background,
                Some(Palette::default().selection)
            );
            assert_eq!(
                cells[&(row, 0)].foreground,
                Some(Palette::default().selection_text)
            );
            cancel(&mut editor);
        }
        assert_eq!(source(&editor), "retained source\n");
        assert!(!editor.buffer()?.is_modified());
        if events.is_empty() {
            assert!(!path.exists());
        }
    }
    eviction(&root.join("eviction"))?;
    controls(&root.join("controls"))?;
    rejected_state(&root.join("rejected"))?;
    println!(
        "PASS native ranking: empty/overlap/four-four/ties, three match policies, exact/AI/removed names, selected surface, 256-event eviction, snapshot/resize/Tab/Return/accept-alias/cancel/history, rejected state and unchanged source"
    );
    Ok(())
}

fn session(root: &Path, path: &Path, policy: &str) -> Result<Session> {
    std::fs::create_dir_all(root.join("workspace"))?;
    let config = root.join("config.toml");
    std::fs::write(
        &config,
        format!(
            "tree.visible=false\ncompletion.automatic=false\nminibuffer.match='{policy}'\n[keys.minibuffer]\n'M-a'='minibuffer-accept'\n[ai.commands.review]\nprompt='Review.'\ninput='buffer'\ncontext='input'\noutput='explanation'\n[ai.commands.review-more]\nprompt='Review more.'\ninput='buffer'\ncontext='input'\noutput='explanation'"
        ),
    )?;
    let startup = prepare(&config);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    Ok(
        open(root, "native.txt", "|retained source\n", startup.snapshot)?
            .with_command_usage(History::open(path)),
    )
}
fn names(editor: &mut Session) -> Result<Vec<String>> {
    // Compare rendered names, leaving shortcut and description columns to
    // presentation acceptance. Preserve the complete no-match notice.
    Ok(editor
        .frame()?
        .rows()
        .iter()
        .filter(|row| row.band() == Band::Completion)
        .map(|row| {
            row.text()
                .split("  ")
                .next()
                .unwrap_or_default()
                .trim()
                .to_owned()
        })
        .collect())
}
fn cancel(editor: &mut Session) {
    editor.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('g'),
        KeyModifiers::CONTROL,
    )));
}

fn eviction(root: &Path) -> Result<()> {
    std::fs::create_dir_all(root)?;
    let path = root.join("history.toml");
    let (mut store, _) = Store::open(&path)?;
    store.publish(&["undo".into()])?;
    let batch = std::iter::repeat_n("newline".into(), 252)
        .chain(
            [
                "find-file",
                "toggle-file-tree",
                "query-replace",
                "undo-redo",
            ]
            .map(String::from),
        )
        .collect::<Vec<_>>();
    let events = store.publish(&batch)?;
    assert_eq!(events.len(), 256);
    assert!(!events.iter().any(|name| name == "undo"));
    let mut editor = session(root, &path, "prefix-then-substring")?;
    editor.execute(Command::ExecuteExtended);
    let rows = names(&mut editor)?;
    assert_eq!(
        &rows[..5],
        [
            "undo-redo",
            "query-replace",
            "toggle-file-tree",
            "find-file",
            "newline"
        ]
    );
    assert!(rows[5..].windows(2).all(|pair| pair[0] < pair[1]));
    cancel(&mut editor);
    Ok(())
}

// A real publication while the selector is open must leave its selected identity
// stable. Only the final accepted name is appended; recall and internal calls add none.
fn controls(root: &Path) -> Result<()> {
    std::fs::create_dir_all(root)?;
    let path = root.join("history.toml");
    let (mut other, _) = Store::open(&path)?;
    other.publish(&["undo".into(), "undo-redo".into()])?;
    let mut editor = session(root, &path, "prefix-then-substring")?;
    editor.execute(Command::ExecuteExtended);
    key(&mut editor, KeyCode::Down);
    let before = names(&mut editor)?;
    other.publish(&["goto-line".into()])?;
    for (width, height) in [(4, 2), (80, 24)] {
        editor.handle(Event::Resize(width, height));
        editor.frame()?;
    }
    assert_eq!(names(&mut editor)?, before);
    key(&mut editor, KeyCode::Tab);
    assert!(
        editor
            .frame()?
            .rows()
            .iter()
            .any(|row| row.text().starts_with("M-x undo "))
    );
    editor.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('a'),
        KeyModifiers::ALT,
    )));
    editor.execute(Command::History(rev::command::History::Undo));
    editor.execute(Command::ExecuteExtended);
    editor.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('p'),
        KeyModifiers::ALT,
    )));
    assert!(
        editor
            .frame()?
            .rows()
            .iter()
            .any(|row| row.text().starts_with("M-x undo "))
    );
    cancel(&mut editor);
    editor
        .flush_command_usage()
        .map_err(std::io::Error::other)?;
    assert_eq!(
        Store::open(&path)?.1,
        ["undo", "undo-redo", "goto-line", "undo"]
    );
    assert!(editor.remaining_input(std::time::Instant::now()).is_none());
    assert_eq!(source(&editor), "retained source\n");
    Ok(())
}

// The native boundary checks rejected records without another terminal process.
// A malformed or over-byte-limit record stays intact while named input works.
fn rejected_state(root: &Path) -> Result<()> {
    for (name, record) in [
        ("malformed", "invalid-private-record".to_owned()),
        ("version", "version=2\ncommands=[]".to_owned()),
        ("oversized", " ".repeat(65537)),
    ] {
        let fixture = root.join(name);
        std::fs::create_dir_all(&fixture)?;
        let path = fixture.join("history.toml");
        std::fs::write(&path, &record)?;
        let mut editor = session(&fixture, &path, "prefix-then-substring")?;
        assert!(editor.echo().contains("Command usage:"));
        editor.execute(Command::ExecuteExtended);
        editor.handle(Event::Paste("not-a-command".into()));
        assert_eq!(names(&mut editor)?, ["No matching commands"]);
        key(&mut editor, KeyCode::Enter);
        assert_eq!(editor.echo(), "Unknown command");
        editor.execute(Command::ExecuteExtended);
        editor.handle(Event::Paste("goto-line".into()));
        key(&mut editor, KeyCode::Enter);
        cancel(&mut editor);
        editor.execute(Command::ExecuteExtended);
        assert_eq!(names(&mut editor)?[0], "goto-line");
        cancel(&mut editor);
        assert!(editor.flush_command_usage().is_err());
        assert_eq!(std::fs::read_to_string(&path)?, record);
        assert_eq!(source(&editor), "retained source\n");
    }
    Ok(())
}
