use super::links::editor;
use super::*;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
const SOURCE: &str = "\u{feff}#+TITLE: Source\r\n* Heading\r\nbody";
const PDF: &[u8] = b"%PDF-1.7\nfixture \xff\n%%EOF\n";
fn begin(s: &mut Session, command: C, path: &str) {
    action(s, command);
    for value in ["buffer", "toc:nil num:nil", path] {
        argument(s, value);
    }
}
fn policy(f: &Fixture, program: &str, limits: &str) -> String {
    // Fixture values use quoted TOML strings and never pass through a shell.
    let path = f.0.join("compiler.py");
    format!(
        "org.tools.pdf=[\"python3\",{path:?},\"{{input}}\",\"{{output}}\",{program:?}]\n{limits}"
    )
}
fn configured(program: &str, limits: &str) -> (Fixture, Session) {
    let f = Fixture::new();
    let settings = policy(&f, program, limits);
    let path = f.write("config.toml", &settings);
    let startup = prepare(&path);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let source = f.write("source.org", SOURCE);
    let id = buffers.visit(&source).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    let session = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 12,
        },
    )
    .unwrap()
    .with_org_clock(clock);
    f.write(
        "compiler.py",
        &format!(
            r#"import os, pathlib, sys, time
source, target, mode = map(str, sys.argv[1:])
root = pathlib.Path(__file__).parent
(root / 'called').write_text(str(os.getpid()))
(root / 'workspace').write_text(str(pathlib.Path.cwd()))
(root / 'input.tex').write_bytes(pathlib.Path(source).read_bytes())
(root / 'permissions').write_text(oct(pathlib.Path.cwd().stat().st_mode & 0o777))
if mode == 'status':
    print('fixture failure', file=sys.stderr)
    sys.exit(9)
if mode == 'noise':
    print('too much output', flush=True)
    time.sleep(30)
if mode == 'wait': time.sleep(30)
if mode == 'invalid': pathlib.Path(target).write_bytes(b'not PDF')
elif mode == 'oversize': pathlib.Path(target).write_bytes(b'%PDF-' + b'x' * 2000 + b'%%EOF')
elif mode == 'symlink': pathlib.Path(target).symlink_to(root / 'input.tex')
elif mode != 'missing': pathlib.Path(target).write_bytes({PDF:?})
"#
        )
        .replace(
            &format!("{PDF:?}"),
            "b'%PDF-1.7\\nfixture \\xff\\n%%EOF\\n'",
        ),
    );
    (f, session)
}
fn cleaned(f: &Fixture) {
    let root = std::fs::read_to_string(f.0.join("workspace")).unwrap();
    assert!(!Path::new(&root).exists(), "owned workspace leaked: {root}");
    let pid = std::fs::read_to_string(f.0.join("called")).unwrap();
    assert!(
        !Path::new(&format!("/proc/{pid}")).exists(),
        "direct child not reaped"
    );
}
// Native TeX commands and PDF discovery cannot invoke a configured program.
// PDF commands report an absent compiler rather than a false publication.
#[test]
fn tex_job_absent() {
    for command in [C::ExportLatexPdf, C::ExportBeamerPdf] {
        let (f, mut s) = editor("", SOURCE, "");
        f.write("output.pdf", "keep");
        begin(&mut s, command, "output.pdf");
        assert!(
            s.echo().contains("No PDF compiler configured"),
            "{}",
            s.echo()
        );
        assert_eq!(std::fs::read(f.0.join("output.pdf")).unwrap(), b"keep");
        assert_eq!(text(&s), SOURCE.trim_start_matches('\u{feff}'));
    }
    let (f, mut s) = configured("success", "");
    for (command, name, class) in [
        (C::ExportLatex, "article.tex", "article"),
        (C::ExportBeamer, "slides.tex", "beamer"),
    ] {
        begin(&mut s, command, name);
        assert!(s.echo().contains("Exported Org document"), "{}", s.echo());
        let tex = std::fs::read_to_string(f.0.join(name)).unwrap();
        assert!(tex.starts_with(&format!("\\documentclass{{{class}}}")));
    }
    assert!(!f.0.join("called").exists());
}
// An explicit PDF job requires its own confirmation. Its fixture verifies argv,
// private staging, exact binary publication and source state, not TeX semantics.
#[test]
fn tex_job_publication() {
    for command in [C::ExportLatexPdf, C::ExportBeamerPdf] {
        let (f, mut s) = configured("success", "");
        s.handle(Event::Paste("Unsaved ".into()));
        settle(&mut s);
        let before = text(&s);
        let stamp = s.buffer().unwrap().stamp();
        let view = s.view().clone();
        let history = s.buffer().unwrap().history().len();
        let output = f.write("output.pdf", "keep");
        std::fs::set_permissions(&output, std::fs::Permissions::from_mode(0o640)).unwrap();
        begin(&mut s, command, "output.pdf");
        assert!(prompt(&mut s).contains("Run PDF compiler"), "{}", s.echo());
        assert!(!f.0.join("called").exists());
        argument(&mut s, "no");
        assert!(!f.0.join("called").exists());
        begin(&mut s, command, "output.pdf");
        argument(&mut s, "yes");
        assert!(prompt(&mut s).contains("Replace export"), "{}", s.echo());
        assert_eq!(std::fs::read(&output).unwrap(), b"keep");
        argument(&mut s, "yes");
        assert!(s.echo().contains("Exported Org document"), "{}", s.echo());
        assert_eq!(std::fs::read(&output).unwrap(), PDF);
        assert_eq!(std::fs::metadata(&output).unwrap().mode() & 0o777, 0o640);
        assert_eq!(
            std::fs::read_to_string(f.0.join("permissions")).unwrap(),
            "0o700"
        );
        assert!(
            std::fs::read_to_string(f.0.join("input.tex"))
                .unwrap()
                .contains("Unsaved")
        );
        assert_eq!(text(&s), before);
        assert_eq!(s.buffer().unwrap().stamp(), stamp);
        assert_eq!(s.view(), &view);
        assert_eq!(s.buffer().unwrap().history().len(), history);
        assert!(s.buffer().unwrap().is_modified());
        assert_eq!(
            std::fs::read_to_string(f.0.join("source.org")).unwrap(),
            SOURCE
        );
        cleaned(&f);
    }
}

fn prompt(s: &mut Session) -> String {
    s.frame()
        .unwrap()
        .rows()
        .iter()
        .filter(|row| row.band() == rev::display::Band::Echo)
        .map(|row| row.text())
        .collect()
}

// A real missing executable, failed exit, missing/malformed PDF, nonregular
// output, output overflow and timeout never replace the selected destination.
#[test]
fn tex_job_failures() {
    for (name, limits, expected) in [
        ("status", "", "PDF compiler failed"),
        ("missing", "", "Read compiler PDF"),
        ("invalid", "", "no PDF envelope"),
        ("symlink", "", "regular file"),
        ("oversize", "org.limits.export_bytes=1000", "regular file"),
        ("noise", "org.limits.job_output_bytes=5", "output limit"),
    ] {
        let (f, mut s) = configured(name, limits);
        f.write("output.pdf", "keep");
        begin(&mut s, C::ExportLatexPdf, "output.pdf");
        argument(&mut s, "yes");
        assert!(s.echo().contains(expected), "{name}: {}", s.echo());
        assert_eq!(
            std::fs::read(f.0.join("output.pdf")).unwrap(),
            b"keep",
            "{name}"
        );
        assert_eq!(
            std::fs::read_to_string(f.0.join("source.org")).unwrap(),
            SOURCE,
            "{name}"
        );
        assert!(s.buffer().unwrap().history().is_empty(), "{name}");
        cleaned(&f);
    }
    let (f, mut s) = editor("org.tools.pdf=['./absent','{input}']", SOURCE, "");
    begin(&mut s, C::ExportLatexPdf, "output.pdf");
    argument(&mut s, "yes");
    assert!(
        s.echo().contains("Start configured program"),
        "{}",
        s.echo()
    );
    assert!(!f.0.join("output.pdf").exists());
}
fn launch(s: &mut Session, f: &Fixture) {
    begin(s, C::ExportLatexPdf, "output.pdf");
    s.handle(Event::Paste("yes".into()));
    s.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    let end = Instant::now() + Duration::from_secs(3);
    while !f.0.join("permissions").exists() {
        s.frame().unwrap();
        s.expire_input(Instant::now() + Duration::from_millis(10));
        assert!(Instant::now() < end, "{}", s.echo());
        std::thread::sleep(Duration::from_millis(2));
    }
}
// Both explicit cancellation and changed canonical source retain the worker
// until it reaps the compiler and removes only its owned private workspace.
#[test]
fn tex_job_cancellation() {
    for changed in [false, true] {
        let (f, mut s) = configured("wait", "");
        f.write("output.pdf", "keep");
        launch(&mut s, &f);
        if changed {
            s.insert('X');
        } else {
            cancel(&mut s);
        }
        let before = text(&s);
        settle(&mut s);
        assert_eq!(text(&s), before);
        assert_eq!(std::fs::read(f.0.join("output.pdf")).unwrap(), b"keep");
        assert!(s.remaining_input(Instant::now()).is_none());
        cleaned(&f);
    }
    let (f, mut s) = configured("wait", "org.limits.job_seconds=1");
    let start = Instant::now();
    begin(&mut s, C::ExportLatexPdf, "output.pdf");
    argument(&mut s, "yes");
    assert!(start.elapsed() < Duration::from_secs(4));
    assert!(
        s.echo().contains("timed out")
            || s.echo().contains("cancelled")
            || s.echo().contains("expired"),
        "{}",
        s.echo()
    );
    assert!(!f.0.join("output.pdf").exists());
    cleaned(&f);
}
// Confirmation cannot relabel stale canonical/disk input or changed target
// evidence. Every rejected case preserves the destination's current bytes.
#[test]
fn tex_job_freshness() {
    let (f, mut s) = configured("success", "");
    begin(&mut s, C::ExportLatexPdf, "output.pdf");
    s.insert('X');
    argument(&mut s, "yes");
    assert!(s.echo().contains("source or view changed"), "{}", s.echo());
    assert!(!f.0.join("called").exists());
    assert!(!f.0.join("output.pdf").exists());
    let (f, mut s) = configured("success", "");
    let target = f.write("output.pdf", "keep");
    begin(&mut s, C::ExportLatexPdf, "output.pdf");
    argument(&mut s, "yes");
    f.write("output.pdf", "external change");
    argument(&mut s, "yes");
    assert!(s.echo().contains("not published"), "{}", s.echo());
    assert_eq!(std::fs::read(target).unwrap(), b"external change");
    cleaned(&f);
    let (f, mut s) = configured("success", "");
    // Included files keep independent disk authority across compiler confirmation.
    s.handle(Event::Paste("#+INCLUDE: \"child.org\"\n".into()));
    settle(&mut s);
    f.write("child.org", "included\n");
    begin(&mut s, C::ExportLatexPdf, "output.pdf");
    f.write("child.org", "changed\n");
    argument(&mut s, "yes");
    assert!(s.echo().contains("disk source changed"), "{}", s.echo());
    assert!(!f.0.join("called").exists());
    assert!(!f.0.join("output.pdf").exists());
}
