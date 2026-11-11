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
    let fixture = Fixture::prepare(root, profile)?;
    let started = Instant::now();
    let mut terminal = Terminal::start_no_target(root, binary, wrapper)?;
    interact(root, profile, &mut terminal, &fixture)?;
    let label = tree_label(profile);
    terminal.finish()?;
    let first_bytes = terminal.bytes();
    drop(terminal);
    let retained = state(root)?;
    let file_bytes = restart_file(root, binary, wrapper)?;
    let directory = restart_directory(root, binary, wrapper, label)?;
    unchanged(root, &fixture, &retained)?;
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
    startup_colors(terminal, profile);
    dashboard_geometry(root, terminal)?;
    temporary_directory(root, profile, terminal, first)?;
    Ok(())
}

fn project_work(
    root: &Path,
    profile: &str,
    terminal: &mut Terminal,
    first: &Path,
) -> Result<rev::context::Project> {
    open_anchor(terminal, profile, first)?;
    let project = promote(root, profile, terminal)?;
    named(terminal, profile, "context-close")?;
    dashboard(terminal)?;
    project_directory(root, profile, terminal, first)?;
    named(terminal, profile, "context-close")?;
    dashboard(terminal)?;
    restore_anchor(terminal)?;
    Ok(project)
}

fn duplicate(
    root: &Path,
    profile: &str,
    terminal: &mut Terminal,
    first: &Path,
    project: &rev::context::Project,
) -> Result<rev::context::Workspace> {
    canceled_copy(root, profile, terminal, first)?;
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
    add_peer(root, profile, terminal, peer)?;
    saved_membership(root)?;
    blank_workspace(root, profile, terminal)?;
    remove_peer(root, profile, terminal)?;
    Ok(())
}

fn convert(
    root: &Path,
    profile: &str,
    terminal: &mut Terminal,
    project: &rev::context::Project,
    copy: &rev::context::Workspace,
) -> Result<()> {
    restore_project(terminal, profile)?;
    management(
        terminal,
        profile,
        "project-promote-to-workspace",
        "Converted",
    )?;
    terminal.frame(|f| visible(f, "Context metadata published"))?;
    converted_catalog(root, project, copy)?;
    named(terminal, profile, "context-close")?;
    dashboard(terminal)?;
    Ok(())
}

struct Fixture {
    config_path: std::path::PathBuf,
    config: String,
    first: std::path::PathBuf,
    peer: std::path::PathBuf,
}
impl Fixture {
    fn prepare(root: &Path, profile: &str) -> Result<Self> {
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
        Ok(Self {
            config_path,
            config,
            first,
            peer,
        })
    }
}

fn interact(root: &Path, profile: &str, terminal: &mut Terminal, fixture: &Fixture) -> Result<()> {
    startup(root, profile, terminal, &fixture.first)?;
    let project = project_work(root, profile, terminal, &fixture.first)?;
    let copy = duplicate(root, profile, terminal, &fixture.first, &project)?;
    membership(root, profile, terminal, &fixture.peer)?;
    convert(root, profile, terminal, &project, &copy)?;
    Ok(())
}

fn restart_file(root: &Path, binary: &Path, wrapper: &Path) -> Result<usize> {
    let mut file = Terminal::start(root, binary, wrapper, "beta/peer.txt")?;
    file.frame(|f| visible(f, "peer source") && !visible(f, "New Buffer"))?;
    file.idle()?;
    file.finish()?;
    let file_bytes = file.bytes();
    drop(file);
    Ok(file_bytes)
}

fn restart_directory(root: &Path, binary: &Path, wrapper: &Path, label: &str) -> Result<Terminal> {
    let mut directory = Terminal::start(root, binary, wrapper, "alpha")?;
    directory
        .frame(|f| visible(f, label) && !visible(f, "New Buffer") && !visible(f, "界e\u{301}"))?;
    directory.finish()?;
    Ok(directory)
}

fn unchanged(root: &Path, fixture: &Fixture, retained: &[u8]) -> Result<()> {
    assert_eq!(state(root)?, retained);
    assert_eq!(fs::read(&fixture.first)?, DOCUMENT.as_bytes());
    assert_eq!(fs::read(&fixture.peer)?, b"peer source\n");
    assert_eq!(fs::read_to_string(&fixture.config_path)?, fixture.config);
    assert!(!fixture.first.parent().unwrap().join(".rev").exists());
    Ok(())
}

fn startup_colors(terminal: &Terminal, profile: &str) {
    if profile == "non-modal" {
        assert!(terminal.contains("38;2;129;74;255m") || terminal.contains("48;2;129;74;255m"));
        assert!(terminal.contains("48;2;26;27;38m"));
    } else {
        assert!(!terminal.contains("38;2;") && !terminal.contains("48;2;"));
        assert!(terminal.contains("\x1b[7m"));
    }
}

fn dashboard_geometry(root: &Path, terminal: &mut Terminal) -> Result<()> {
    assert!(!root.join("pty/state/rev/contexts.toml").exists());
    terminal.resize(12, 40)?;
    terminal.frame(|f| visible(f, "rev") && visible(f, "New Buffer") && !f.contains('▀'))?;
    terminal.resize(30, 120)?;
    dashboard(terminal)?;
    terminal.resize(24, 80)?;
    dashboard(terminal)?;
    terminal.idle()?;
    Ok(())
}

fn temporary_directory(
    root: &Path,
    profile: &str,
    terminal: &mut Terminal,
    first: &Path,
) -> Result<()> {
    // The dashboard Open action accepts a directory through the path selector.
    terminal.send(b"\x1b[B\r")?;
    argument(
        terminal,
        "Find file:",
        first.parent().unwrap().to_str().unwrap(),
    )?;
    let label = tree_label(profile);
    terminal.frame(|f| visible(f, label) && !visible(f, "New Buffer"))?;
    assert!(catalog(root)?.projects.is_empty());
    terminal.send(b"\x07")?;
    terminal.frame(|f| !visible(f, "Tree *"))?;
    Ok(())
}

fn open_anchor(terminal: &mut Terminal, profile: &str, first: &Path) -> Result<()> {
    path(terminal, profile, "find-file", "Find file:", first)?;
    terminal.frame(|f| visible(f, "界e\u{301} line"))?;
    named(terminal, profile, "goto-line")?;
    argument(terminal, "Goto line:", "2")?;
    terminal.frame(|f| visible(f, "L2 C1"))?;
    terminal.send(b"\x1b[C\x1b[C")?;
    terminal.frame(|f| visible(f, "L2 C4"))?;
    Ok(())
}

fn promote(root: &Path, profile: &str, terminal: &mut Terminal) -> Result<rev::context::Project> {
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
    Ok(project)
}

fn project_directory(
    root: &Path,
    profile: &str,
    terminal: &mut Terminal,
    first: &Path,
) -> Result<()> {
    let label = tree_label(profile);
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
    Ok(())
}

fn restore_anchor(terminal: &mut Terminal) -> Result<()> {
    // The dashboard Open Project action restores the saved source anchor.
    terminal.send(b"\x1b[B\x1b[B\r")?;
    pick(terminal, "Project:", "Atlas")?;
    terminal.frame(|f| visible(f, "界e\u{301} line") && visible(f, "L2 C4"))?;
    Ok(())
}

fn canceled_copy(root: &Path, profile: &str, terminal: &mut Terminal, first: &Path) -> Result<()> {
    let retained = state(root)?;
    dirty_source(terminal, profile)?;
    cancel_copy(terminal, profile)?;
    assert!(catalog(root)?.workspaces.is_empty());
    assert_eq!(state(root)?, retained);
    assert_eq!(fs::read(first)?, DOCUMENT.as_bytes());
    undo_source(terminal, profile)?;
    Ok(())
}

fn dirty_source(terminal: &mut Terminal, profile: &str) -> Result<()> {
    if profile == "modal" {
        terminal.send(b"i!\x1b")?;
    } else {
        terminal.send(b"!")?;
    }
    terminal.frame(|f| visible(f, "界e\u{301}! line"))?;
    Ok(())
}

fn cancel_copy(terminal: &mut Terminal, profile: &str) -> Result<()> {
    management(
        terminal,
        profile,
        "project-duplicate-as-workspace",
        "Canceled",
    )?;
    terminal.frame(|f| visible(f, "Leave context:"))?;
    terminal.send(b"\x07")?;
    terminal.frame(|f| !visible(f, "Leave context:") && visible(f, "界e\u{301}! line"))?;
    Ok(())
}

fn undo_source(terminal: &mut Terminal, profile: &str) -> Result<()> {
    named(terminal, profile, "undo")?;
    terminal.frame(|f| visible(f, "界e\u{301} line") && !visible(f, "界e\u{301}!"))?;
    Ok(())
}

fn add_peer(root: &Path, profile: &str, terminal: &mut Terminal, peer: &Path) -> Result<()> {
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
    Ok(())
}

fn saved_membership(root: &Path) -> Result<()> {
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
    Ok(())
}

fn blank_workspace(root: &Path, profile: &str, terminal: &mut Terminal) -> Result<()> {
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
    Ok(())
}

fn remove_peer(root: &Path, profile: &str, terminal: &mut Terminal) -> Result<()> {
    terminal.send(b"\x1b[B\x1b[B\x1b[B\r")?;
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

fn restore_project(terminal: &mut Terminal, profile: &str) -> Result<()> {
    terminal.send(b"\x07")?;
    terminal.frame(|f| !visible(f, "Workspace *"))?;
    named(terminal, profile, "project-switch-project")?;
    pick(terminal, "Project:", "Atlas")?;
    terminal.frame(|f| visible(f, "界e\u{301} line") && !visible(f, "peer source"))?;
    Ok(())
}

fn converted_catalog(
    root: &Path,
    project: &rev::context::Project,
    copy: &rev::context::Workspace,
) -> Result<()> {
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
    Ok(())
}
