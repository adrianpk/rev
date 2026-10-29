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
    format!(
        "org.tools.pdf=[\"python3\",{},\"{{input}}\",\"{{output}}\",{}]\n{limits}",
        format!("{:?}", f.0.join("compiler.py").to_str().unwrap()),
        format!("{program:?}")
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
    print('too much output')
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
