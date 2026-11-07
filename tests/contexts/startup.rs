use super::projects::{catalog, configured, prompt, ready};
use super::*;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use std::process::Command as Process;

fn show(s: &mut Session) {
    s.execute(Command::DashboardOpen);
    assert!(s.dashboard_visible(), "{}", s.echo());
}
fn select(s: &mut Session, index: usize) {
    for _ in 0..index {
        key(s, KeyCode::Down);
    }
}
fn action_row(s: &mut Session, label: &str) -> (u16, u16) {
    let frame = s.frame().unwrap();
    let row = frame
        .rows()
        .iter()
        .find(|r| r.text().contains(label))
        .unwrap();
    (row.text().find(label).unwrap() as u16 - 2, row.row())
}
fn mouse(s: &mut Session, kind: MouseEventKind, column: u16, row: u16) {
    s.handle(Event::Mouse(MouseEvent {
        kind,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }));
}

// The screen is passive retained presentation. Native profile navigation and
// command/picker cancellation cannot edit, reset or checkpoint the hidden source.
#[test]
fn retained_work_and_selectors() {
    for profile in ["non-modal", "modal"] {
        let f = fixture::Fixture::new();
        let source = f.write("source", "kept\n");
        let mut s = configured(&f, profile);
        s.load(&source).unwrap();
        s.insert('!');
        let stamp = s.buffer().unwrap().stamp();
        let point = s.view().point;
        let history = s.buffer().unwrap().history().len();
        let context = s.context().clone();
        let generation = s.context_generation();
        show(&mut s);
        s.handle(Event::Paste("must not edit".into()));
        key(&mut s, KeyCode::Char('x'));
        assert_eq!(s.buffer().unwrap().stamp(), stamp);
        assert_eq!(text(&s), "!kept\n");
        select(&mut s, 1);
        key(&mut s, KeyCode::Enter);
        assert!(prompt(&mut s, "Find file:"));
        cancel(&mut s);
        assert!(s.dashboard_visible());
        named(&mut s, profile, "project-list");
        ready(&mut s, |s| s.echo().contains("No registered Projects"));
        assert!(s.dashboard_visible());
        named(&mut s, profile, "workspace-list");
        ready(&mut s, |s| s.echo().contains("No registered Workspaces"));
        assert!(s.dashboard_visible());
        assert!(!f.0.join("state").exists());
        if profile == "modal" {
            key(&mut s, KeyCode::Char(':'));
        } else {
            s.execute(Command::ExecuteExtended);
        }
        assert!(prompt(&mut s, "M-x"));
        cancel(&mut s);
        assert!(s.dashboard_visible());
        cancel(&mut s);
        assert!(!s.dashboard_visible());
        assert_eq!(s.context(), &context);
        assert_eq!(s.context_generation(), generation);
        assert_eq!(s.buffer().unwrap().stamp(), stamp);
        assert_eq!(s.view().point, point);
        assert_eq!(s.buffer().unwrap().history().len(), history);
        assert_eq!(fs::read_to_string(&source).unwrap(), "kept\n");
    }
}

// All four actions call the ordinary command owners. New Buffer starts fresh
// standalone work after accepted source decisions; files/directories stay
// temporary and typed catalog selection restores the independent artifact.
#[test]
fn action_dispatch() {
    for profile in ["non-modal", "modal"] {
        let f = fixture::Fixture::new();
        let source = f.write("root/source", "source");
        let mut s = configured(&f, profile);
        s.load(source.parent().unwrap()).unwrap();
        s.load(&source).unwrap();
        projects::promote(&mut s, profile, "Project");
        let saved = catalog(&f);
        show(&mut s);
        select(&mut s, 2);
        key(&mut s, KeyCode::Enter);
        ready(&mut s, |s| prompt(s, "Project:"));
        s.handle(Event::Paste("Project".into()));
        key(&mut s, KeyCode::Tab);
        key(&mut s, KeyCode::Enter);
        ready(&mut s, |s| !s.dashboard_visible());
        assert_eq!(catalog(&f), saved);
        show(&mut s);
        select(&mut s, 1);
        key(&mut s, KeyCode::Enter);
        s.handle(Event::Paste(source.to_str().unwrap().into()));
        key(&mut s, KeyCode::Enter);
        assert!(!s.dashboard_visible());
        assert!(matches!(s.context(), Context::Project(_)));
        show(&mut s);
        select(&mut s, 1);
        key(&mut s, KeyCode::Enter);
        assert!(prompt(&mut s, "Find file:"));
        s.handle(Event::Paste(
            source.parent().unwrap().to_str().unwrap().into(),
        ));
        key(&mut s, KeyCode::Enter);
        ready(&mut s, |s| matches!(s.context(), Context::Directory(_)));
        assert!(!s.dashboard_visible());
        s.load(&source).unwrap();
        s.insert('!');
        show(&mut s);
        key(&mut s, KeyCode::Enter);
        assert!(s.echo().contains("Leave context"));
        cancel(&mut s);
        assert!(s.dashboard_visible());
        assert!(text(&s).starts_with('!'));
        key(&mut s, KeyCode::Enter);
        key(&mut s, KeyCode::Char('n'));
        ready(&mut s, |s| {
            matches!(s.context(), Context::Standalone) && !s.dashboard_visible()
        });
        assert_eq!(text(&s), "");
        named(&mut s, profile, "workspace-create");
        ready(&mut s, |s| prompt(s, "Workspace name:"));
        s.handle(Event::Paste("Workspace".into()));
        key(&mut s, KeyCode::Enter);
        ready(&mut s, |s| matches!(s.context(), Context::Workspace(_)));
        show(&mut s);
        select(&mut s, 3);
        key(&mut s, KeyCode::Enter);
        ready(&mut s, |s| prompt(s, "Workspace:"));
        s.handle(Event::Paste("Workspace".into()));
        key(&mut s, KeyCode::Tab);
        key(&mut s, KeyCode::Enter);
        ready(&mut s, |s| !s.dashboard_visible());
        s.execute(Command::ContextClose);
        ready(&mut s, |s| s.dashboard_visible());
        assert!(matches!(s.context(), Context::Standalone));
        assert!(s.buffer().unwrap().file().is_none());
        assert_eq!(fs::read_to_string(source).unwrap(), "source");
    }
}

// Artwork follows the frame background without retaining a black rectangle or
// changing source palettes. Resized projections preserve action and hit geometry.
#[test]
fn artwork_and_resize() {
    if std::env::var_os("REV_STARTUP_MONO_CHILD").is_some() {
        return;
    }
    crossterm::style::force_color_output(true);
    let f = fixture::Fixture::new();
    let source = f.write("source", "source");
    let mut s = configured(&f, "non-modal");
    s.load(&source).unwrap();
    show(&mut s);
    s.handle(Event::Resize(120, 30));
    let frame = s.frame().unwrap();
    assert_eq!(frame.rows().len(), 30);
    assert_eq!(
        frame
            .rows()
            .iter()
            .filter(|r| r.text().contains('▀') || r.text().contains('▄'))
            .count(),
        8
    );
    for row in frame.rows() {
        assert_eq!(unicode_width::UnicodeWidthStr::width(row.text()), 120);
        assert!(!row.text().contains('\x1b'));
    }
    let banner_end = frame
        .rows()
        .iter()
        .rposition(|r| r.text().contains('▀'))
        .unwrap();
    let menu_start = frame
        .rows()
        .iter()
        .position(|r| r.text().contains("New Buffer"))
        .unwrap();
    assert_eq!(menu_start - banner_end - 1, 3);
    assert!(
        frame.rows()[banner_end + 1..menu_start]
            .iter()
            .all(|row| row.text().trim().is_empty())
    );
    let mut output = Vec::new();
    frame.write(&mut output).unwrap();
    let output = String::from_utf8(output).unwrap();
    assert!(output.contains("48;2;26;27;38m"));
    assert!(!output.contains("48;2;0;0;0m"));
    assert!(output.contains("38;2;129;74;255m") || output.contains("48;2;129;74;255m"));
    for (case, background, expected) in [
        ("custom", rev::config::Rgb(51, 34, 68), "48;2;51;34;68m"),
        ("black", rev::config::Rgb(0, 0, 0), "48;2;0;0;0m"),
    ] {
        let palette = rev::config::Palette {
            background,
            ..rev::config::Palette::default()
        };
        let mut output = Vec::new();
        s.frame()
            .unwrap()
            .with_palette(&palette)
            .write(&mut output)
            .unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains(expected), "{case}");
        if case == "custom" {
            assert!(!output.contains("48;2;0;0;0m"));
        }
        assert!(output.contains("38;2;129;74;255m") || output.contains("48;2;129;74;255m"));
    }
    select(&mut s, 3);
    for (width, height) in [
        (120, 20),
        (50, 10),
        (5, 3),
        (0, 0),
        (1, 1),
        (20, 1),
        (120, 30),
    ] {
        s.handle(Event::Resize(width, height));
        let frame = s.poll_frame().unwrap();
        assert_eq!(frame.rows().len(), usize::from(height));
        assert!(
            frame
                .rows()
                .iter()
                .all(|r| unicode_width::UnicodeWidthStr::width(r.text()) == usize::from(width))
        );
        if width > 0 && height > 0 {
            assert_eq!(
                frame
                    .rows()
                    .iter()
                    .filter(|r| r.selection().is_some())
                    .count(),
                1
            );
        }
        if width < 58 || height < 14 {
            assert!(
                frame
                    .rows()
                    .iter()
                    .all(|r| !r.text().contains('▀') && !r.text().contains('▄'))
            );
        }
        if width > 16 && height > 0 {
            let mut output = Vec::new();
            frame.write(&mut output).unwrap();
            let output = String::from_utf8(output).unwrap();
            assert!(
                output.contains("48;2;26;27;38m"),
                "{width}x{height}: {output:?}"
            );
        }
    }
    key(&mut s, KeyCode::Enter);
    ready(&mut s, |s| s.echo().contains("No registered Workspaces"));
    cancel(&mut s);
    assert!(!s.dashboard_visible());
    let mut output = Vec::new();
    s.frame().unwrap().write(&mut output).unwrap();
    assert!(String::from_utf8(output).unwrap().contains("source"));
    assert_eq!(text(&s), "source");
}

// Pointer admission shares current placement with painting. Banner clicks,
// off-row releases, resized gestures and disabled mouse input cannot activate.
#[test]
fn pointer_actions() {
    let f = fixture::Fixture::new();
    let mut s = configured(&f, "non-modal");
    show(&mut s);
    s.handle(Event::Resize(120, 30));
    if !s.mouse_enabled() {
        s.execute(Command::MouseMode);
    }
    let (x, y) = action_row(&mut s, "Open…");
    mouse(&mut s, MouseEventKind::Down(MouseButton::Left), 10, 5);
    mouse(&mut s, MouseEventKind::Up(MouseButton::Left), 10, 5);
    assert!(s.dashboard_visible());
    assert!(!prompt(&mut s, "Find file:"));
    mouse(&mut s, MouseEventKind::Down(MouseButton::Left), x, y);
    s.handle(Event::Resize(110, 25));
    s.handle(Event::Resize(120, 30));
    mouse(&mut s, MouseEventKind::Up(MouseButton::Left), x, y);
    assert!(!prompt(&mut s, "Find file:"));
    s.execute(Command::MouseMode);
    mouse(&mut s, MouseEventKind::Down(MouseButton::Left), x, y);
    mouse(&mut s, MouseEventKind::Up(MouseButton::Left), x, y);
    assert!(!prompt(&mut s, "Find file:"));
    s.execute(Command::MouseMode);
    mouse(&mut s, MouseEventKind::Down(MouseButton::Left), x, y);
    mouse(&mut s, MouseEventKind::Up(MouseButton::Left), x, y);
    assert!(prompt(&mut s, "Find file:"));
    cancel(&mut s);
    assert!(s.dashboard_visible());
}

// Executable startup is isolated from real user roots. Only absent target opens
// the screen; relative files, dot, absent files and invalid targets keep their
// native meaning without creating context state or reading project config.
#[test]
fn executable_targets() {
    if std::env::var_os("REV_STARTUP_TARGET_CHILD").is_none() {
        let f = fixture::Fixture::new();
        f.write("project/source", "source");
        f.write("project/.rev/config.toml", "invalid local configuration");
        let output = Process::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "editor::startup::executable_targets",
                "--nocapture",
            ])
            .current_dir(f.0.join("project"))
            .env("REV_STARTUP_TARGET_CHILD", "1")
            .env("HOME", f.0.join("home"))
            .env("XDG_CONFIG_HOME", f.0.join("config"))
            .env("XDG_STATE_HOME", f.0.join("state"))
            .env("XDG_DATA_HOME", f.0.join("data"))
            .env("XDG_CACHE_HOME", f.0.join("cache"))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!f.0.join("state/rev/contexts.toml").exists());
        return;
    }
    let size = Size {
        width: 100,
        height: 25,
    };
    let s = Session::startup(None, size).unwrap();
    assert!(s.dashboard_visible());
    assert!(matches!(s.context(), Context::Standalone));
    let s = Session::startup(Some(Path::new("source")), size).unwrap();
    assert!(!s.dashboard_visible());
    assert_eq!(text(&s), "source");
    assert!(s.buffer().unwrap().file().unwrap().path().is_absolute());
    let s = Session::startup(Some(Path::new(".")), size).unwrap();
    assert!(matches!(s.context(), Context::Directory(_)));
    assert!(!s.dashboard_visible());
    let mut s = Session::startup(Some(Path::new("absent")), size).unwrap();
    assert!(!s.dashboard_visible());
    assert!(!s.buffer().unwrap().file().unwrap().is_present());
    assert_eq!(text(&s), "");
    s.insert('!');
    assert!(s.buffer().unwrap().is_modified());
    assert!(!Path::new("absent").exists());
    assert!(Session::startup(Some(Path::new("source/child")), size).is_err());
}

// NO_COLOR is checked in an isolated process so native renderer caches cannot
// leak from another test. Selection remains visible without flattening artwork.
#[test]
fn monochrome_startup() {
    if std::env::var_os("REV_STARTUP_MONO_CHILD").is_none() {
        let output = Process::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "editor::startup::monochrome_startup",
                "--nocapture",
            ])
            .env("REV_STARTUP_MONO_CHILD", "1")
            .env("NO_COLOR", "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let f = fixture::Fixture::new();
    let mut s = configured(&f, "modal");
    show(&mut s);
    s.handle(Event::Resize(120, 30));
    let frame = s.frame().unwrap();
    assert!(frame.rows().iter().any(|r| r.text().trim() == "rev"));
    assert!(
        frame
            .rows()
            .iter()
            .all(|r| !r.text().contains('▀') && !r.text().contains('▄'))
    );
    let mut output = Vec::new();
    frame.write(&mut output).unwrap();
    let output = String::from_utf8(output).unwrap();
    assert!(!output.contains("38;2;") && !output.contains("48;2;"));
    assert!(output.contains("\x1b[7m"));
    key(&mut s, KeyCode::Char('j'));
    key(&mut s, KeyCode::Enter);
    assert!(prompt(&mut s, "Find file:"));
    cancel(&mut s);
    assert!(s.dashboard_visible());
}

// Workspace roots bound each source consumer independently. AI's directory
// prompt, explicit terminal cwd and disabled LSP retain their owners without
// importing peer roots, local config, or activating any service during selection.
#[test]
fn shared_consumers() {
    use rev::command::{LspCommand, TerminalCommand};
    let f = fixture::Fixture::new();
    let left = f.write("left/nested/source.rs", "left");
    let right = f.write("right/source.rs", "right");
    let loose = f.write("loose/source.rs", "loose");
    f.write("right/.rev/config.toml", "invalid local configuration");
    let script = f.write("terminal.sh", "printf READY\nexec /bin/cat\n");
    let settings = prepare(&f.write(
        "services.toml",
        &format!(
            "lsp.enabled=false\nterminal.program='/bin/sh'\nterminal.args=[{}]",
            toml_edit::Value::from(script.to_str().unwrap())
        ),
    ));
    assert!(settings.diagnostic.is_none());
    let buffers = Buffers::with_settings(settings.snapshot).unwrap();
    let view = View::new(buffers.get(buffers.initial()).unwrap());
    let mut s = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 20,
        },
    )
    .unwrap();
    let path = f.0.join("state/contexts.toml");
    s.set_context_storage(path.clone()).unwrap();
    let (store, empty) = Store::open(&path).unwrap();
    let mut candidate = empty.clone();
    let roots = vec![
        super::super::root(&f.0.join("left")),
        super::super::root(left.parent().unwrap()),
        super::super::root(right.parent().unwrap()),
    ];
    super::super::workspace(&mut candidate, "Peers", roots);
    store.publish(&empty.prepare(candidate).unwrap()).unwrap();
    show(&mut s);
    select(&mut s, 4);
    key(&mut s, KeyCode::Enter);
    ready(&mut s, |s| prompt(s, "Workspace:"));
    s.handle(Event::Paste("Peers".into()));
    key(&mut s, KeyCode::Tab);
    key(&mut s, KeyCode::Enter);
    ready(&mut s, |s| matches!(s.context(), Context::Workspace(_)));
    assert!(!s.dashboard_visible());
    assert!(matches!(s.lsp_status(), rev::lsp::Status::Disabled));
    assert_eq!(s.terminals().ids().count(), 0);
    for source in [&right, &left, &loose] {
        s.load(source).unwrap();
        let stamp = s.buffer().unwrap().stamp();
        s.execute(Command::AiSetDirectory);
        assert!(
            prompt(&mut s, source.parent().unwrap().to_str().unwrap()),
            "{}",
            s.echo()
        );
        cancel(&mut s);
        assert_eq!(s.buffer().unwrap().stamp(), stamp);
        assert!(matches!(s.lsp_status(), rev::lsp::Status::Disabled));
    }
    s.load(&right).unwrap();
    s.execute(Command::Terminal(TerminalCommand::NewBuffer));
    let mut pid = None;
    ready(&mut s, |s| {
        pid = s
            .terminals()
            .ids()
            .find_map(|id| s.terminals().get(id).unwrap().process_id());
        pid.is_some()
    });
    let pid = pid.unwrap();
    assert_eq!(
        fs::read_link(format!("/proc/{pid}/cwd")).unwrap(),
        right.parent().unwrap()
    );
    show(&mut s);
    s.execute(Command::ExecuteExtended);
    s.handle(Event::Paste("dashboard-open".into()));
    key(&mut s, KeyCode::Enter);
    assert!(s.dashboard_visible());
    cancel(&mut s);
    assert_eq!(s.terminals().ids().count(), 1);
    assert!(Path::new(&format!("/proc/{pid}")).exists());
    s.execute(Command::ContextClose);
    assert!(s.echo().contains("terminate"));
    key(&mut s, KeyCode::Char('y'));
    ready(&mut s, |s| {
        s.dashboard_visible() && matches!(s.context(), Context::Standalone)
    });
    assert!(!Path::new(&format!("/proc/{pid}")).exists());
    // An explicit host command can return from presentation to its existing
    // service handler; context selection itself preserved the disabled policy.
    s.execute(Command::Lsp(LspCommand::Mode));
    assert!(!s.dashboard_visible());
    assert!(!matches!(s.lsp_status(), rev::lsp::Status::Disabled));
    assert_eq!(fs::read_to_string(&left).unwrap(), "left");
    assert_eq!(fs::read_to_string(&right).unwrap(), "right");
}
