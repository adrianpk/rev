use crate::{
    catalog::DOCUMENT,
    support::Result,
    terminal::{Terminal, row},
};
use rev::context::{Catalog, FileReference, SourcePosition, Store};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

fn visible(frame: &str, text: &str) -> bool {
    (1..=30).any(|n| row(frame, n).contains(text))
}
fn named(terminal: &mut Terminal, profile: &str, command: &str) -> Result<()> {
    let prefix = if profile == "modal" { ":" } else { "\x1bx" };
    terminal.send(format!("{prefix}{command}\r").as_bytes())
}
fn argument(terminal: &mut Terminal, label: &str, value: &str) -> Result<()> {
    replace_argument(terminal, label, "", value)
}
fn replace_argument(
    terminal: &mut Terminal,
    label: &str,
    initial: &str,
    value: &str,
) -> Result<()> {
    terminal.frame(|f| visible(f, label))?;
    terminal.send(format!("\x05{}{value}\r", "\x7f".repeat(initial.chars().count())).as_bytes())
}
fn pick(terminal: &mut Terminal, label: &str, value: &str) -> Result<()> {
    terminal.frame(|f| visible(f, label))?;
    terminal.send(format!("{value}\t\r").as_bytes())
}
fn path(
    terminal: &mut Terminal,
    profile: &str,
    command: &str,
    label: &str,
    value: &Path,
) -> Result<()> {
    named(terminal, profile, command)?;
    argument(
        terminal,
        label,
        value.to_str().ok_or("fixture path must be UTF-8")?,
    )
}
fn management(terminal: &mut Terminal, profile: &str, command: &str, name: &str) -> Result<()> {
    named(terminal, profile, command)?;
    pick(terminal, "Manage context:", "Atlas")?;
    replace_argument(terminal, "New context name:", "Atlas", name)
}
fn catalog(root: &Path) -> Result<Catalog> {
    Ok(Store::open(&root.join("pty/state/rev/contexts.toml"))?.1)
}
fn state(root: &Path) -> Result<Vec<u8>> {
    Ok(fs::read(root.join("pty/state/rev/contexts.toml"))?)
}
fn dashboard(terminal: &mut Terminal) -> Result<()> {
    terminal.frame(|f| visible(f, "New Buffer") && visible(f, "Open Workspace"))?;
    Ok(())
}

// Three serial starts form one journey per profile: no target, explicit file,
// explicit directory. Each keeps the shared eight-second/1-MiB editor limits.
pub(super) fn journey(root: &Path, binary: &Path, wrapper: &Path, profile: &str) -> Result<()> {
    assert!(matches!(profile, "non-modal" | "modal"));
    for directory in [
        "workspace/alpha",
        "workspace/beta",
        "pty/home",
        "pty/config/rev",
    ] {
        fs::create_dir_all(root.join(directory))?;
    }
    let config = format!(
        "editor.interaction_profile='{profile}'\ntree.visible=false\ntree.decorated={}\ngutter.nerd_fonts={}\ncompletion.automatic=false\n",
        profile == "non-modal",
        profile == "modal"
    );
    let config_path = root.join("pty/config/rev/config.toml");
    fs::write(&config_path, &config)?;
    let first = root.join("workspace/alpha/notes.txt");
    let peer = root.join("workspace/beta/peer.txt");
    fs::write(&first, DOCUMENT)?;
    fs::write(&peer, "peer source\n")?;
    let started = Instant::now();
    let mut terminal = Terminal::start_no_target(root, binary, wrapper)?;
    startup(root, profile, &mut terminal, &first)?;
    let project = project_work(root, profile, &mut terminal, &first)?;
    let copy = duplicate(root, profile, &mut terminal, &first, &project)?;
    membership(root, profile, &mut terminal, &peer)?;
    convert(root, profile, &mut terminal, &project, &copy)?;
    let label = tree_label(profile);
    terminal.finish()?;
    let first_bytes = terminal.bytes();
    drop(terminal);
    let retained = state(root)?;
    let mut file = Terminal::start(root, binary, wrapper, "beta/peer.txt")?;
    file.frame(|f| visible(f, "peer source") && !visible(f, "New Buffer"))?;
    file.idle()?;
    file.finish()?;
    let file_bytes = file.bytes();
    drop(file);
    let mut directory = Terminal::start(root, binary, wrapper, "alpha")?;
    directory
        .frame(|f| visible(f, label) && !visible(f, "New Buffer") && !visible(f, "界e\u{301}"))?;
    directory.finish()?;
    assert_eq!(state(root)?, retained);
    assert_eq!(fs::read(&first)?, DOCUMENT.as_bytes());
    assert_eq!(fs::read(&peer)?, b"peer source\n");
    assert_eq!(fs::read_to_string(config_path)?, config);
    assert!(!first.parent().unwrap().join(".rev").exists());
    assert!(started.elapsed() < Duration::from_secs(30));
    println!(
        "PASS Linux PTY {profile}: startup/actions/banner/resize, temporary matching directory, promotion, Unicode source anchor, Project/Workspace sessions, empty Workspace, duplicate/cancel/undo, membership, atomic conversion, explicit file/directory restart, unchanged source/configuration and restored TTYs; elapsed_ms={} output_bytes={first_bytes}+{file_bytes}+{}",
        started.elapsed().as_millis(),
        directory.bytes()
    );
    Ok(())
}

fn tree_label(profile: &str) -> &'static str {
    if profile == "non-modal" {
        "▾ ◆ alpha/"
    } else {
        "v alpha/"
    }
}

fn startup(root: &Path, profile: &str, terminal: &mut Terminal, first: &Path) -> Result<()> {
    dashboard(terminal)?;
    if profile == "non-modal" {
        assert!(terminal.contains("38;2;129;74;255m") || terminal.contains("48;2;129;74;255m"));
        assert!(terminal.contains("48;2;0;0;0m"));
    } else {
        assert!(!terminal.contains("38;2;") && !terminal.contains("48;2;"));
        assert!(terminal.contains("\x1b[7m"));
    }
    assert!(!root.join("pty/state/rev/contexts.toml").exists());
    terminal.resize(12, 40)?;
    terminal.frame(|f| visible(f, "rev") && visible(f, "New Buffer") && !f.contains('▀'))?;
    terminal.resize(30, 120)?;
    dashboard(terminal)?;
    terminal.resize(24, 80)?;
    dashboard(terminal)?;
    terminal.idle()?;
    // This is the dashboard Open Dir action, rather than a command alias.
    terminal.send(b"\x1b[B\x1b[B\r")?;
    argument(
        terminal,
        "Open directory:",
        first.parent().unwrap().to_str().unwrap(),
    )?;
    let label = tree_label(profile);
    terminal.frame(|f| visible(f, label) && !visible(f, "New Buffer"))?;
    assert!(catalog(root)?.projects.is_empty());
    terminal.send(b"\x07")?;
    terminal.frame(|f| !visible(f, "Tree *"))?;
    Ok(())
}

fn project_work(
    root: &Path,
    profile: &str,
    terminal: &mut Terminal,
    first: &Path,
) -> Result<rev::context::Project> {
    let label = tree_label(profile);
    path(terminal, profile, "find-file", "Find file:", first)?;
    terminal.frame(|f| visible(f, "界e\u{301} line"))?;
    named(terminal, profile, "goto-line")?;
    argument(terminal, "Goto line:", "2")?;
    terminal.frame(|f| visible(f, "L2 C1"))?;
    terminal.send(b"\x1b[C\x1b[C")?;
    terminal.frame(|f| visible(f, "L2 C4"))?;
    named(terminal, profile, "project-promote")?;
    replace_argument(terminal, "Project name:", "alpha", "Atlas")?;
    terminal.frame(|f| visible(f, "Project registered"))?;
    let registered = catalog(root)?;
    let project = registered.projects[0].clone();
    assert_eq!(project.name.as_str(), "Atlas");
    let saved = registered
        .session(&project.session_ref)
        .unwrap()
        .snapshot
        .as_ref()
        .unwrap();
    assert_eq!(saved.files.len(), 1);
    assert_eq!(saved.files[0].cursor, SourcePosition { line: 2, column: 4 });
    named(terminal, profile, "context-close")?;
    dashboard(terminal)?;
    let retained = state(root)?;
    path(
        terminal,
        profile,
        "open-directory",
        "Open directory:",
        first.parent().unwrap(),
    )?;
    terminal.frame(|f| visible(f, label) && !visible(f, "界e\u{301}"))?;
    assert_eq!(state(root)?, retained);
    terminal.send(b"\x07")?;
    terminal.frame(|f| !visible(f, "Tree *"))?;
    named(terminal, profile, "context-close")?;
    dashboard(terminal)?;
    // The dashboard Open Project action restores the saved source anchor.
    terminal.send(b"\x1b[B\x1b[B\x1b[B\r")?;
    pick(terminal, "Project:", "Atlas")?;
    terminal.frame(|f| visible(f, "界e\u{301} line") && visible(f, "L2 C4"))?;
    Ok(project)
}

fn duplicate(
    root: &Path,
    profile: &str,
    terminal: &mut Terminal,
    first: &Path,
    project: &rev::context::Project,
) -> Result<rev::context::Workspace> {
    let retained = state(root)?;
    if profile == "modal" {
        terminal.send(b"i!\x1b")?;
    } else {
        terminal.send(b"!")?;
    }
    terminal.frame(|f| visible(f, "界e\u{301}! line"))?;
    management(
        terminal,
        profile,
        "project-duplicate-as-workspace",
        "Canceled",
    )?;
    terminal.frame(|f| visible(f, "Leave context:"))?;
    terminal.send(b"\x07")?;
    terminal.frame(|f| !visible(f, "Leave context:") && visible(f, "界e\u{301}! line"))?;
    assert!(catalog(root)?.workspaces.is_empty());
    assert_eq!(state(root)?, retained);
    assert_eq!(fs::read(first)?, DOCUMENT.as_bytes());
    named(terminal, profile, "undo")?;
    terminal.frame(|f| visible(f, "界e\u{301} line") && !visible(f, "界e\u{301}!"))?;
    management(terminal, profile, "project-duplicate-as-workspace", "Copy")?;
    terminal.frame(|f| visible(f, "Workspace opened"))?;
    let copied = catalog(root)?;
    assert_eq!(copied.projects[0].id, project.id);
    let copy = copied.workspaces[0].clone();
    assert_eq!(copy.name.as_str(), "Copy");
    assert_ne!(copy.roots[0].id, project.root.id);
    Ok(copy)
}

fn membership(root: &Path, profile: &str, terminal: &mut Terminal, peer: &Path) -> Result<()> {
    path(
        terminal,
        profile,
        "workspace-add-directory",
        "Open directory:",
        peer.parent().unwrap(),
    )?;
    terminal.frame(|f| visible(f, "Workspace roots published"))?;
    assert_eq!(catalog(root)?.workspaces[0].roots.len(), 2);
    path(terminal, profile, "find-file", "Find file:", peer)?;
    terminal.frame(|f| visible(f, "peer source"))?;
    named(terminal, profile, "context-close")?;
    dashboard(terminal)?;
    let two = catalog(root)?;
    let saved = two
        .session(&two.workspaces[0].session_ref)
        .unwrap()
        .snapshot
        .as_ref()
        .unwrap();
    assert_eq!(saved.files.len(), 2);
    assert_eq!(
        saved.active_file.as_ref(),
        saved.files.last().map(|f| &f.file)
    );
    let project_session = two
        .session(&two.projects[0].session_ref)
        .unwrap()
        .snapshot
        .as_ref()
        .unwrap();
    assert_eq!(project_session.files.len(), 1);
    named(terminal, profile, "workspace-create")?;
    argument(terminal, "Workspace name:", "Blank")?;
    terminal.frame(|f| visible(f, "Workspace opened"))?;
    let created = catalog(root)?;
    let blank = created
        .workspaces
        .iter()
        .find(|w| w.name.as_str() == "Blank")
        .unwrap();
    assert!(blank.roots.is_empty());
    assert!(
        created
            .session(&blank.session_ref)
            .unwrap()
            .snapshot
            .as_ref()
            .unwrap()
            .files
            .is_empty()
    );
    terminal.send(b"\x07")?;
    terminal.frame(|f| !visible(f, "Workspace *"))?;
    named(terminal, profile, "context-close")?;
    dashboard(terminal)?;
    terminal.send(b"\x1b[B\x1b[B\x1b[B\x1b[B\r")?;
    pick(terminal, "Workspace:", "Copy")?;
    terminal.frame(|f| visible(f, "peer source"))?;
    named(terminal, profile, "workspace-remove-root")?;
    pick(terminal, "Remove Workspace root:", "beta")?;
    terminal.frame(|f| visible(f, "Workspace roots published"))?;
    let removed = catalog(root)?;
    let copy = removed
        .workspaces
        .iter()
        .find(|w| w.name.as_str() == "Copy")
        .unwrap();
    assert_eq!(copy.roots.len(), 1);
    assert_eq!(
        removed
            .session(&copy.session_ref)
            .unwrap()
            .snapshot
            .as_ref()
            .unwrap()
            .files
            .len(),
        1
    );
    Ok(())
}

fn convert(
    root: &Path,
    profile: &str,
    terminal: &mut Terminal,
    project: &rev::context::Project,
    copy: &rev::context::Workspace,
) -> Result<()> {
    terminal.send(b"\x07")?;
    terminal.frame(|f| !visible(f, "Workspace *"))?;
    named(terminal, profile, "project-switch-project")?;
    pick(terminal, "Project:", "Atlas")?;
    terminal.frame(|f| visible(f, "界e\u{301} line") && !visible(f, "peer source"))?;
    management(
        terminal,
        profile,
        "project-promote-to-workspace",
        "Converted",
    )?;
    terminal.frame(|f| visible(f, "Context metadata published"))?;
    let converted = catalog(root)?;
    assert!(converted.projects.is_empty());
    let target = converted
        .workspaces
        .iter()
        .find(|w| w.name.as_str() == "Converted")
        .unwrap();
    assert_ne!(target.id, copy.id);
    assert_ne!(target.roots[0].id, project.root.id);
    let saved = converted
        .session(&target.session_ref)
        .unwrap()
        .snapshot
        .as_ref()
        .unwrap();
    assert!(
        matches!(&saved.files[0].file, FileReference::RootRelative {root,..} if root==&target.roots[0].id)
    );
    named(terminal, profile, "context-close")?;
    dashboard(terminal)?;
    Ok(())
}
