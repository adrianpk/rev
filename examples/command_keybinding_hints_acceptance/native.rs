use crate::support::{Result, key, open};
use crossterm::event::{Event, KeyCode, KeyEvent};
use rev::command::Command;
use rev::config::{Snapshot, prepare};
use rev::display::Band;
use rev::input::keymap::Sequence;
use rev::input::{Input, OrgContext, Outcome};
use rev::session::Session;
use std::path::Path;

pub(super) fn settings(root: &Path, config: &str) -> Result<Snapshot> {
    std::fs::create_dir_all(root.join("workspace"))?;
    let path = root.join("config.toml");
    std::fs::write(
        &path,
        format!("tree.visible=false\ncompletion.automatic=false\n{config}"),
    )?;
    let startup = prepare(&path);
    assert!(
        startup.diagnostic.is_none(),
        "{config}: {:?}",
        startup.diagnostic
    );
    Ok(startup.snapshot)
}

pub(super) fn editor(root: &Path, config: &str) -> Result<Session> {
    open(
        root,
        "source.txt",
        "a|bc\nsecond line\n",
        settings(root, config)?,
    )
}

pub(super) fn send(editor: &mut Session, sequence: &str) -> Result<()> {
    for chord in Sequence::parse(sequence)?.chords() {
        editor.handle(Event::Key(KeyEvent::new(chord.code, chord.modifiers)));
    }
    Ok(())
}

pub(super) fn cancel(editor: &mut Session) -> Result<()> {
    send(editor, "C-g")
}

pub(super) fn row(editor: &mut Session, name: &str) -> Result<Vec<String>> {
    editor.execute(Command::ExecuteExtended);
    editor.handle(Event::Paste(name.into()));
    let frame = editor.frame()?;
    Ok(frame
        .rows()
        .iter()
        .find(|row| row.band() == Band::Completion && row.text().starts_with(name))
        .ok_or_else(|| format!("missing {name}: {}", editor.echo()))?
        .text()
        .split_whitespace()
        .map(str::to_owned)
        .collect())
}

pub(super) fn check_row(editor: &mut Session, name: &str, hint: Option<&str>) -> Result<()> {
    let hint = hint.map(|hint| format!("[{hint}]"));
    let expected: Vec<_> = std::iter::once(name)
        .chain(hint.as_deref().into_iter().flat_map(str::split_whitespace))
        .map(str::to_owned)
        .collect();
    assert_eq!(row(editor, name)?, expected, "{name}");
    cancel(editor)
}

// Every advertised label in each snapshot is replayed through ordinary input
// in the same origin. Literal expectations protect policy rather than mirror it.
pub(super) fn run(root: &Path) -> Result<()> {
    crate::presentation::run(&root.join("presentation"))?;
    for (name, profile, config, target, hint) in [
        ("factory", "non-modal", "", "save-buffer", Some("C-x C-s")),
        (
            "override",
            "non-modal",
            "[keys.editor]\n'C-x C-s'='newline'\n'C-c s'='save-buffer'",
            "save-buffer",
            Some("C-c s"),
        ),
        (
            "shortest",
            "non-modal",
            "[keys.editor]\n'M-z a'='save-buffer'\n'M-j'='save-buffer'",
            "save-buffer",
            Some("M-j"),
        ),
        (
            "lexical",
            "non-modal",
            "[keys.editor]\n'M-z'='save-buffer'\n'M-a'='save-buffer'",
            "save-buffer",
            Some("M-a"),
        ),
        (
            "Meta",
            "non-modal",
            "[keys.editor]\n'ESC j'='save-buffer'",
            "save-buffer",
            Some("M-j"),
        ),
        ("undo", "non-modal", "", "undo", Some("C-_")),
        (
            "eight",
            "non-modal",
            "[keys.editor]\n'M-j a b c d e f g'='mouse-mode'",
            "mouse-mode",
            Some("M-j a b c d e f g"),
        ),
        ("unbound", "non-modal", "", "mouse-mode", None),
        (
            "modal",
            "modal",
            "[keys.vim_normal]\n'M-j'='save-buffer'",
            "save-buffer",
            Some("M-j"),
        ),
        (
            "count-owned",
            "modal",
            "[keys.vim_normal]\n'1 a'='mouse-mode'",
            "mouse-mode",
            None,
        ),
    ] {
        let config = format!("editor.interaction_profile='{profile}'\n{config}");
        let snapshot = settings(&root.join(name), &config)?;
        inspect(&snapshot, None, target, hint)?;
    }
    for role in OrgContext::ALL {
        let snapshot = settings(&root.join(format!("org-{role:?}")), "")?;
        inspect(
            &snapshot,
            Some(role),
            "save-buffer",
            if matches!(
                role,
                OrgContext::Editing | OrgContext::Capture | OrgContext::SourceEdit
            ) {
                Some("C-x C-s")
            } else {
                None
            },
        )?;
    }
    for (name, parent, local, expected) in [
        ("inherited", "'M-j a'='mouse-mode'", "", Some("M-j a")),
        (
            "shadow-leaf",
            "'M-j a'='mouse-mode'",
            "'M-j'='newline'",
            None,
        ),
        (
            "shadow-prefix",
            "'M-j'='mouse-mode'",
            "'M-j a'='newline'",
            None,
        ),
        (
            "local",
            "'M-j'='mouse-mode'",
            "'M-j'='newline'\n'M-q'='mouse-mode'",
            Some("M-q"),
        ),
    ] {
        let snapshot = settings(
            &root.join(name),
            &format!(
                "editor.interaction_profile='modal'\n[keys.vim_normal]\n{parent}\n[keys.org]\n{local}"
            ),
        )?;
        inspect(&snapshot, Some(OrgContext::Editing), "mouse-mode", expected)?;
    }
    registered(&root.join("registered"))?;
    crate::contexts::run(&root.join("contexts"))?;
    println!(
        "PASS native hints: factory/configured/normalized/shortest/lexical/eight-chord/unbound/AI maps, all Org owners, inherited/shadowed leaves and prefixes, advertised dispatch agreement"
    );
    Ok(())
}

fn inspect(
    snapshot: &Snapshot,
    role: Option<OrgContext>,
    target: &str,
    expected: Option<&str>,
) -> Result<()> {
    let mut input = Input::with_settings(snapshot);
    input.set_org_context(role);
    input.commands()?;
    let palette = input.palette().ok_or("missing palette")?;
    assert_eq!(
        palette.hint(Command::named(target).ok_or("unknown target")?),
        expected,
        "{role:?}: {target}"
    );
    let mut dispatch = Input::with_settings(snapshot);
    dispatch.set_org_context(role);
    for command in palette.candidates() {
        if let Some(hint) = palette.hint(command) {
            dispatch.cancel();
            let mut outcome = Outcome::Ignored;
            for chord in Sequence::parse(hint)?.chords() {
                outcome = dispatch.handle(Event::Key(KeyEvent::new(chord.code, chord.modifiers)));
            }
            assert_eq!(outcome, Outcome::Command(command), "{role:?}: {hint}");
        }
    }
    input.handle(Event::Paste("C-x C-s".into()));
    assert!(
        input
            .palette()
            .ok_or("missing palette")?
            .candidates()
            .is_empty()
    );
    input.set_org_context(if role.is_some() {
        None
    } else {
        Some(OrgContext::Agenda)
    });
    let retired = input.palette().ok_or("missing retired palette")?;
    // The hint-only query above has no rows; inspect identities rather than
    // making retirement vacuously pass over that empty filtered result.
    assert!(
        Command::ALL
            .into_iter()
            .all(|command| retired.hint(command).is_none()),
        "changed origin must retire hints"
    );
    input.cancel();
    input.set_org_context(role);
    input.commands()?;
    assert_eq!(
        input
            .palette()
            .ok_or("missing palette")?
            .hint(Command::named(target).ok_or("unknown target")?),
        expected
    );
    Ok(())
}

fn registered(root: &Path) -> Result<()> {
    let snapshot = settings(
        root,
        "[ai.commands.review]\nprompt='Review text.'\ninput='buffer'\ncontext='input'\noutput='explanation'",
    )?;
    let mut input = Input::with_settings(&snapshot);
    input.commands()?;
    input.handle(Event::Paste("ai-review".into()));
    let command = input.palette().ok_or("missing palette")?.candidates()[0];
    assert_eq!(command.name(), "ai-review");
    assert_eq!(
        input.palette().ok_or("missing palette")?.hint(command),
        None
    );
    input.handle(Event::Key(KeyEvent::new(
        KeyCode::Tab,
        crossterm::event::KeyModifiers::NONE,
    )));
    assert_eq!(input.prompt().ok_or("missing prompt")?.text(), "ai-review");
    assert_eq!(
        input.handle(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            crossterm::event::KeyModifiers::NONE
        ))),
        Outcome::Command(command)
    );
    let mut editor = editor(&root.join("session"), "")?;
    editor.execute(Command::ExecuteExtended);
    key(&mut editor, KeyCode::Enter);
    assert_eq!(editor.echo(), "No command entered");
    Ok(())
}
