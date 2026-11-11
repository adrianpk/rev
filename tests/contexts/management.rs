use super::projects::{catalog, configured, prompt, ready};
use super::*;

fn command(s: &mut Session, profile: &str, name: &str) {
    if profile == "modal" && s.tree().focused() {
        cancel(s);
    }
    named(s, profile, name);
}
fn argument(s: &mut Session, text: &str) {
    key(s, KeyCode::End);
    for _ in 0..128 {
        key(s, KeyCode::Backspace);
    }
    s.handle(Event::Paste(text.into()));
    key(s, KeyCode::Enter);
}
fn select(s: &mut Session, profile: &str, cmd: &str, target: &str) {
    command(s, profile, cmd);
    ready(s, |s| prompt(s, "Manage context:"));
    s.handle(Event::Paste(target.into()));
    key(s, KeyCode::Tab);
    key(s, KeyCode::Enter);
    ready(s, |s| {
        prompt(s, "New context name:")
            || prompt(s, "Forget metadata?")
            || prompt(s, "Replacement root path:")
            || prompt(s, "Relocate root:")
    });
}
fn project_session(f: &fixture::Fixture, profile: &str) -> (Session, std::path::PathBuf) {
    let source = f.write("root/source", "a界e\u{301}\nline\n");
    let mut s = configured(f, profile);
    s.load(source.parent().unwrap()).unwrap();
    s.load(&source).unwrap();
    projects::promote(&mut s, profile, "Source");
    (s, source)
}
fn workspace_open(s: &mut Session, profile: &str, name: &str) {
    command(s, profile, "workspace-open");
    ready(s, |s| prompt(s, "Workspace:"));
    s.handle(Event::Paste(name.into()));
    key(s, KeyCode::Tab);
    key(s, KeyCode::Enter);
    ready(s, |s| matches!(s.context(), Context::Workspace(_)));
}

// Display-only edits preserve snapshot/root identity. Forgetting an active
// Project removes only its registration while dirty live directory work survives.
#[test]
fn project_labels_and_forget() {
    for profile in ["non-modal", "modal"] {
        let f = fixture::Fixture::new();
        let (mut s, source) = project_session(&f, profile);
        s.insert('!');
        let before = catalog(&f);
        let stamp = s.buffer().unwrap().stamp();
        let point = s.view().point;
        select(&mut s, profile, "project-rename", "Source");
        argument(&mut s, "  Renamed 界  ");
        ready(&mut s, |s| s.echo().contains("metadata published"));
        let renamed = catalog(&f);
        let p = &renamed.projects[0];
        assert_eq!(p.id, before.projects[0].id);
        assert_eq!(p.root, before.projects[0].root);
        assert_eq!(p.roots_revision, before.projects[0].roots_revision);
        assert_eq!(renamed.sessions, before.sessions);
        assert_eq!(p.name.as_str(), "Renamed 界");
        assert_eq!(s.buffer().unwrap().stamp(), stamp);
        assert_eq!(s.view().point, point);
        select(&mut s, profile, "project-forget", "Renamed");
        cancel(&mut s);
        assert_eq!(catalog(&f), renamed);
        select(&mut s, profile, "project-forget", "Renamed");
        argument(&mut s, "yes");
        ready(&mut s, |s| matches!(s.context(), Context::Directory(_)));
        assert!(catalog(&f).projects.is_empty());
        assert!(catalog(&f).sessions.is_empty());
        assert_eq!(s.buffer().unwrap().stamp(), stamp);
        assert_eq!(s.view().point, point);
        assert!(text(&s).starts_with('!'));
        assert_eq!(fs::read_to_string(source).unwrap(), "a界e\u{301}\nline\n");
    }
}

// Active conversion is one metadata publication with fresh Workspace/root
// identities and remapped file/active references, retaining dirty native buffers.
#[test]
fn active_conversion() {
    for profile in ["non-modal", "modal"] {
        let f = fixture::Fixture::new();
        let (mut s, source) = project_session(&f, profile);
        let loose = f.write("loose", "loose");
        s.load(&loose).unwrap();
        s.load(&source).unwrap();
        s.insert('!');
        let before = catalog(&f);
        let stamp = s.buffer().unwrap().stamp();
        let point = s.view().point;
        let history = s.buffer().unwrap().history().len();
        let focused = s.tree().focused();
        let visible = s.tree().visible();
        select(&mut s, profile, "project-promote-to-workspace", "Source");
        cancel(&mut s);
        assert_eq!(catalog(&f), before);
        select(&mut s, profile, "project-promote-to-workspace", "Source");
        argument(&mut s, "Source");
        ready(&mut s, |s| matches!(s.context(), Context::Workspace(_)));
        assert_eq!(s.buffer().unwrap().stamp(), stamp);
        assert_eq!(s.view().point, point);
        assert_eq!(s.buffer().unwrap().history().len(), history);
        assert_eq!(s.tree().focused(), focused);
        assert_eq!(s.tree().visible(), visible);
        let after = catalog(&f);
        assert!(after.projects.is_empty());
        assert_eq!(after.workspaces.len(), 1);
        let w = &after.workspaces[0];
        assert_ne!(w.roots[0].id, before.projects[0].root.id);
        assert_eq!(w.roots[0].path, before.projects[0].root.path);
        let snap = after
            .session(&w.session_ref)
            .unwrap()
            .snapshot
            .as_ref()
            .unwrap();
        assert_eq!(snap.files.len(), 2);
        assert!(
            matches!(&snap.files[0].file,FileReference::RootRelative{root,..} if root==&w.roots[0].id)
        );
        assert!(matches!(
            &snap.files[1].file,
            FileReference::LooseAbsolute(_)
        ));
        assert_eq!(snap.active_file, Some(snap.files[0].file.clone()));
        assert_eq!(fs::read_to_string(&source).unwrap(), "a界e\u{301}\nline\n");
        let mut restarted = configured(&f, profile);
        workspace_open(&mut restarted, profile, "Source");
        assert_eq!(text(&restarted), "a界e\u{301}\nline\n");
    }
}

// Inactive conversion and metadata edits cannot checkpoint or replace another
// context. Renaming and forgetting a Workspace leave its source directory intact.
#[test]
fn inactive_conversion_and_workspace_labels() {
    let profile = "non-modal";
    let f = fixture::Fixture::new();
    let (_source, source) = project_session(&f, profile);
    let outside = f.write("other/source", "other");
    let mut s = configured(&f, profile);
    s.load(&outside).unwrap();
    s.insert('!');
    let stamp = s.buffer().unwrap().stamp();
    let point = s.view().point;
    let context = s.context().clone();
    select(&mut s, profile, "project-promote-to-workspace", "Source");
    argument(&mut s, "Converted");
    ready(&mut s, |s| s.echo().contains("metadata published"));
    assert_eq!(s.context(), &context);
    assert_eq!(s.buffer().unwrap().stamp(), stamp);
    assert_eq!(s.view().point, point);
    let converted = catalog(&f);
    assert!(converted.projects.is_empty());
    let w = converted.workspaces[0].clone();
    select(&mut s, profile, "workspace-rename", "Converted");
    argument(&mut s, "Renamed");
    ready(&mut s, |s| s.echo().contains("metadata published"));
    let renamed = catalog(&f);
    assert_eq!(renamed.workspaces[0].id, w.id);
    assert_eq!(renamed.workspaces[0].roots, w.roots);
    assert_eq!(renamed.sessions, converted.sessions);
    select(&mut s, profile, "workspace-forget", "Renamed");
    argument(&mut s, "yes");
    ready(&mut s, |s| s.echo().contains("metadata published"));
    assert_eq!(s.context(), &context);
    assert_eq!(s.buffer().unwrap().stamp(), stamp);
    assert!(catalog(&f).workspaces.is_empty());
    assert!(source.exists());
    assert_eq!(fs::read_to_string(outside).unwrap(), "other");
}

// Duplication resolves dirty departure before adding a destination. The original
// selected snapshot remains independent, while the new Workspace restores disk.
#[test]
fn duplicate_source_decisions() {
    for profile in ["non-modal", "modal"] {
        let f = fixture::Fixture::new();
        let (mut s, source) = project_session(&f, profile);
        s.insert('!');
        let before = catalog(&f);
        let stamp = s.buffer().unwrap().stamp();
        select(&mut s, profile, "project-duplicate-as-workspace", "Source");
        argument(&mut s, "Duplicate");
        assert!(s.echo().contains("Leave context"));
        cancel(&mut s);
        assert_eq!(catalog(&f), before);
        assert_eq!(s.buffer().unwrap().stamp(), stamp);
        select(&mut s, profile, "project-duplicate-as-workspace", "Source");
        argument(&mut s, "Duplicate");
        key(&mut s, KeyCode::Char('n'));
        ready(&mut s, |s| matches!(s.context(), Context::Workspace(_)));
        let after = catalog(&f);
        assert_eq!(after.projects, before.projects);
        assert_eq!(
            after.session(&before.projects[0].session_ref),
            before.session(&before.projects[0].session_ref)
        );
        assert_eq!(after.workspaces.len(), 1);
        assert_ne!(after.workspaces[0].roots[0].id, before.projects[0].root.id);
        assert_eq!(text(&s), fs::read_to_string(&source).unwrap());
        assert_ne!(s.buffer().unwrap().stamp().buffer, stamp.buffer);
        s.insert('?');
        command(&mut s, profile, "context-close");
        key(&mut s, KeyCode::Char('n'));
        ready(&mut s, |s| matches!(s.context(), Context::Standalone));
        assert_eq!(
            catalog(&f).session(&before.projects[0].session_ref),
            before.session(&before.projects[0].session_ref)
        );
    }
}

// Inactive duplication also resolves the current context before publication;
// cancellation creates nothing and does not consume the selected Project.
#[test]
fn duplicate_inactive_and_forget_source() {
    let profile = "non-modal";
    let f = fixture::Fixture::new();
    let (_project, source) = project_session(&f, profile);
    let before = catalog(&f);
    let outside = f.write("outside", "other");
    let mut s = configured(&f, profile);
    s.load(&outside).unwrap();
    s.insert('!');
    let stamp = s.buffer().unwrap().stamp();
    select(&mut s, profile, "project-duplicate-as-workspace", "Source");
    argument(&mut s, "Independent");
    cancel(&mut s);
    assert_eq!(catalog(&f), before);
    assert_eq!(s.buffer().unwrap().stamp(), stamp);
    select(&mut s, profile, "project-duplicate-as-workspace", "Source");
    argument(&mut s, "Independent");
    key(&mut s, KeyCode::Char('n'));
    ready(&mut s, |s| matches!(s.context(), Context::Workspace(_)));
    assert_eq!(s.buffer().unwrap().file().unwrap().path(), source);
    let w = catalog(&f).workspaces[0].clone();
    select(&mut s, profile, "project-forget", "Source");
    argument(&mut s, "yes");
    ready(&mut s, |s| s.echo().contains("metadata published"));
    assert!(catalog(&f).projects.is_empty());
    assert_eq!(catalog(&f).workspaces[0], w);
    assert_eq!(s.context(), &Context::Workspace(w.id));
    assert!(source.exists());
}

// Missing retained roots can be relocated explicitly with stable identities.
// Relative references follow the new directory; loose and missing entries stay
// explicit. The old directory is never moved or recreated by the operation.
#[test]
fn inactive_relocation() {
    for kind in ["project", "workspace"] {
        let f = fixture::Fixture::new();
        let (mut source, old) = project_session(&f, "non-modal");
        let missing = f.write("root/missing", "missing");
        source.load(&missing).unwrap();
        command(&mut source, "non-modal", "context-close");
        ready(&mut source, |s| matches!(s.context(), Context::Standalone));
        if kind == "workspace" {
            select(
                &mut source,
                "non-modal",
                "project-promote-to-workspace",
                "Source",
            );
            argument(&mut source, "Source");
            ready(&mut source, |s| s.echo().contains("metadata published"));
        }
        let before = catalog(&f);
        let moved = f.0.join("moved");
        fs::rename(old.parent().unwrap(), &moved).unwrap();
        fs::remove_file(moved.join("missing")).unwrap();
        let mut s = configured(&f, "non-modal");
        select(
            &mut s,
            "non-modal",
            if kind == "project" {
                "project-set-root"
            } else {
                "workspace-set-root-path"
            },
            "Source",
        );
        if kind == "workspace" {
            s.handle(Event::Paste("1:".into()));
            key(&mut s, KeyCode::Tab);
            key(&mut s, KeyCode::Enter);
            assert!(prompt(&mut s, "Replacement root path:"));
        }
        argument(&mut s, moved.to_str().unwrap());
        ready(&mut s, |s| s.echo().contains("metadata published"));
        let after = catalog(&f);
        if kind == "project" {
            assert_eq!(after.projects[0].id, before.projects[0].id);
            assert_eq!(after.projects[0].root.id, before.projects[0].root.id);
            assert_eq!(after.projects[0].root.path.as_path(), moved);
            projects::choose(&mut s, "non-modal", "Source");
            ready(&mut s, |s| matches!(s.context(), Context::Project(_)));
        } else {
            assert_eq!(after.workspaces[0].id, before.workspaces[0].id);
            assert_eq!(
                after.workspaces[0].roots[0].id,
                before.workspaces[0].roots[0].id
            );
            assert_eq!(after.workspaces[0].roots[0].path.as_path(), moved);
            workspace_open(&mut s, "non-modal", "Source");
        }
        assert_eq!(after.sessions[0].snapshot, before.sessions[0].snapshot);
        assert!(s.echo().contains("1 unavailable"));
        assert!(!old.parent().unwrap().exists());
        assert!(!moved.join("missing").exists());
    }
}

// An active root change resolves dirty source work, stages restored files and
// only then changes metadata. Cancel leaves the original root and text intact.
#[test]
fn active_relocation_and_workspace_forget() {
    for profile in ["non-modal", "modal"] {
        let f = fixture::Fixture::new();
        let (mut s, source) = project_session(&f, profile);
        let replacement = f.write("replacement/source", "replacement");
        s.insert('!');
        let before = catalog(&f);
        let stamp = s.buffer().unwrap().stamp();
        select(&mut s, profile, "project-set-root", "Source");
        argument(&mut s, replacement.parent().unwrap().to_str().unwrap());
        cancel(&mut s);
        assert_eq!(catalog(&f), before);
        assert_eq!(s.buffer().unwrap().stamp(), stamp);
        select(&mut s, profile, "project-set-root", "Source");
        argument(&mut s, replacement.parent().unwrap().to_str().unwrap());
        key(&mut s, KeyCode::Char('n'));
        ready(&mut s, |s| s.echo().starts_with("Project opened"));
        assert_eq!(text(&s), "replacement");
        assert_eq!(catalog(&f).projects[0].root.id, before.projects[0].root.id);
        assert!(source.exists());
        select(&mut s, profile, "project-promote-to-workspace", "Source");
        argument(&mut s, "Workspace");
        ready(&mut s, |s| matches!(s.context(), Context::Workspace(_)));
        s.insert('?');
        let relocated = f.write("workspace-new/source", "workspace relocated");
        let root_before = catalog(&f).workspaces[0].clone();
        select(&mut s, profile, "workspace-set-root-path", "Workspace");
        s.handle(Event::Paste("1:".into()));
        key(&mut s, KeyCode::Tab);
        key(&mut s, KeyCode::Enter);
        argument(&mut s, relocated.parent().unwrap().to_str().unwrap());
        cancel(&mut s);
        assert_eq!(catalog(&f).workspaces[0], root_before);
        select(&mut s, profile, "workspace-set-root-path", "Workspace");
        s.handle(Event::Paste("1:".into()));
        key(&mut s, KeyCode::Tab);
        key(&mut s, KeyCode::Enter);
        argument(&mut s, relocated.parent().unwrap().to_str().unwrap());
        key(&mut s, KeyCode::Char('n'));
        ready(&mut s, |s| s.echo().starts_with("Workspace opened"));
        assert_eq!(
            catalog(&f).workspaces[0].roots[0].id,
            root_before.roots[0].id
        );
        assert_eq!(text(&s), "workspace relocated");
        s.insert('?');
        let before = catalog(&f);
        let stamp = s.buffer().unwrap().stamp();
        select(&mut s, profile, "workspace-forget", "Workspace");
        argument(&mut s, "yes");
        cancel(&mut s);
        assert_eq!(catalog(&f), before);
        assert_eq!(s.buffer().unwrap().stamp(), stamp);
        select(&mut s, profile, "workspace-forget", "Workspace");
        argument(&mut s, "yes");
        key(&mut s, KeyCode::Char('n'));
        ready(&mut s, |s| matches!(s.context(), Context::Standalone));
        assert_eq!(s.tree().root(), None);
        assert!(catalog(&f).workspaces.is_empty());
        assert!(source.exists());
        assert_eq!(fs::read_to_string(replacement).unwrap(), "replacement");
    }
}

// A competing writer or damaged input at publication preserves the exact live
// owner and source text. Name/root conflicts cannot manufacture a second entry.
#[test]
fn publication_rejections() {
    for case in [
        "busy",
        "stale",
        "corrupt",
        "conversion busy",
        "conversion stale",
        "conversion corrupt",
        "name",
        "root",
    ] {
        let f = fixture::Fixture::new();
        let (mut s, source) = project_session(&f, "non-modal");
        s.insert('!');
        let stamp = s.buffer().unwrap().stamp();
        let path = f.0.join("state/contexts.toml");
        let before = catalog(&f);
        select(
            &mut s,
            "non-modal",
            if case.starts_with("conversion") {
                "project-promote-to-workspace"
            } else {
                "project-rename"
            },
            "Source",
        );
        let lock = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path.with_extension("toml.lock"))
            .unwrap();
        if case.ends_with("busy") {
            lock.try_lock().unwrap();
        }
        if competing_catalog(case, &f, &path, &before) {
            cancel(&mut s);
            select(
                &mut s,
                "non-modal",
                if case == "name" {
                    "project-rename"
                } else {
                    "project-set-root"
                },
                "Source",
            );
        }
        let bytes = fs::read(&path).unwrap();
        let other = f.0.join("other");
        argument(
            &mut s,
            if case == "name" {
                "Taken"
            } else if case == "root" {
                other.to_str().unwrap()
            } else {
                "Rejected"
            },
        );
        ready(&mut s, publication_failed);
        if case.ends_with("busy") {
            lock.unlock().unwrap();
        }
        assert_eq!(fs::read(path).unwrap(), bytes);
        assert_eq!(s.buffer().unwrap().stamp(), stamp);
        assert_eq!(
            s.context(),
            &Context::Project(before.projects[0].id.clone())
        );
        assert!(text(&s).starts_with('!'));
        assert_eq!(fs::read_to_string(source).unwrap(), "a界e\u{301}\nline\n");
    }
}

fn competing_catalog(case: &str, f: &fixture::Fixture, path: &Path, before: &Catalog) -> bool {
    if case.ends_with("stale") {
        let mut next = before.clone();
        next.projects[0].name = Name::new("Other writer").unwrap();
        next.projects[0].revision = next.projects[0].revision.next().unwrap();
        Store::open(path)
            .unwrap()
            .0
            .publish(&before.prepare(next).unwrap())
            .unwrap();
    }
    if case.ends_with("corrupt") {
        fs::write(path, "broken = [").unwrap();
    }
    if case == "name" || case == "root" {
        let mut next = before.clone();
        let target = f.write("other/source", "other");
        super::super::project(&mut next, target.parent().unwrap(), "Taken");
        Store::open(path)
            .unwrap()
            .0
            .publish(&before.prepare(next).unwrap())
            .unwrap();
        return true;
    }
    false
}

fn publication_failed(s: &mut Session) -> bool {
    s.echo().contains("Contexts:")
        || s.echo().contains("already registered")
        || s.echo().contains("conflict")
}

// Replacement paths are identities observed before source decisions. Replacing
// that directory while the prompt is open cannot retarget the prepared change.
#[test]
fn root_substitution_during_departure() {
    let f = fixture::Fixture::new();
    let (mut s, source) = project_session(&f, "non-modal");
    s.insert('!');
    let replacement = f.write("replacement/source", "selected");
    let before = catalog(&f);
    let stamp = s.buffer().unwrap().stamp();
    select(&mut s, "non-modal", "project-set-root", "Source");
    argument(&mut s, replacement.parent().unwrap().to_str().unwrap());
    fs::rename(replacement.parent().unwrap(), f.0.join("retained")).unwrap();
    f.write("replacement/source", "substitute");
    key(&mut s, KeyCode::Char('n'));
    ready(&mut s, |s| s.echo().contains("Selected root changed"));
    assert_eq!(catalog(&f), before);
    assert_eq!(s.buffer().unwrap().stamp(), stamp);
    assert_eq!(fs::read_to_string(source).unwrap(), "a界e\u{301}\nline\n");
    assert_eq!(fs::read_to_string(replacement).unwrap(), "substitute");
}

// Invalid selected sessions reject conversion without resetting their bytes.
// Their typed registrations can still be renamed or explicitly forgotten.
#[test]
fn damaged_session_management() {
    let f = fixture::Fixture::new();
    let (_old, source) = project_session(&f, "non-modal");
    let path = f.0.join("state/contexts.toml");
    let bytes = fs::read_to_string(&path).unwrap();
    let broken = bytes.replace("line = 1", "line = 0");
    assert_ne!(bytes, broken);
    fs::write(&path, &broken).unwrap();
    let mut s = configured(&f, "non-modal");
    s.load(&source).unwrap();
    s.insert('!');
    let stamp = s.buffer().unwrap().stamp();
    select(
        &mut s,
        "non-modal",
        "project-promote-to-workspace",
        "Source",
    );
    argument(&mut s, "Blocked");
    assert!(s.echo().contains("Damaged context session"));
    assert_eq!(fs::read_to_string(&path).unwrap(), broken);
    assert_eq!(s.buffer().unwrap().stamp(), stamp);
    select(&mut s, "non-modal", "project-rename", "Source");
    argument(&mut s, "Recoverable");
    ready(&mut s, |s| s.echo().contains("metadata published"));
    assert!(catalog(&f).sessions[0].snapshot.is_err());
    assert!(fs::read_to_string(&path).unwrap().contains("line = 0"));
    select(&mut s, "non-modal", "project-forget", "Recoverable");
    argument(&mut s, "yes");
    ready(&mut s, |s| s.echo().contains("metadata published"));
    assert!(catalog(&f).projects.is_empty());
    assert!(catalog(&f).sessions.is_empty());
    assert_eq!(s.buffer().unwrap().stamp(), stamp);
    assert!(source.exists());
}
