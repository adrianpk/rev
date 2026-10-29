use super::*;
use rev::{
    config::{ExportBackend as B, Org},
    org::{
        Cancellation,
        export::{Request, Scope, Work},
    },
};
fn package(source: &str, limit: usize) -> Result<Vec<u8>, String> {
    let f = Fixture::new();
    let path = f.write("source.org", source);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let mut policy = Org::default();
    policy.limits.export_bytes = limit;
    let request = Request {
        backend: B::Odt,
        scope: Scope::Buffer,
        options: None,
        time: clock(&policy.time).unwrap(),
    };
    let snapshot = Work::new(&buffers, id, request, &policy)?.run(&Cancellation::default())?;
    Ok(snapshot.package(&Cancellation::default())?.bytes().into())
}
// Python's independent ZIP/CRC and namespace-aware XML readers check the actual
// binary output, including the special first uncompressed mimetype record.
#[test]
fn odt_package_envelope() {
    let f = Fixture::new();
    let source = "#+TITLE: Notes & plans\nText <tag> 界 and é\n";
    let bytes = package(source, 16777216).unwrap();
    assert_eq!(bytes, package(source, 16777216).unwrap());
    let path = f.0.join("document.odt");
    std::fs::write(&path, bytes).unwrap();
    verify(&path, false);
    assert!(!f.0.join("marker").exists());
}
// XML-invalid scalars and a finite final package budget reject output rather
// than substituting data or returning a partial or malformed archive.
#[test]
fn odt_package_bounds() {
    for source in ["invalid \u{1} scalar\n", "invalid \u{fffe} scalar\n"] {
        assert!(
            package(source, 16777216)
                .unwrap_err()
                .contains("ODT XML cannot represent")
        );
    }
    assert!(package("body\n", 100).is_err());
}

fn verify(path: &Path, fixture: bool) {
    let mut command = std::process::Command::new("python3");
    command.args([
        "scripts/configuration_ui/odt_document.py",
        path.to_str().unwrap(),
    ]);
    if fixture {
        command.arg("--fixture");
    }
    let check = command.output().unwrap();
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );
}
// A representative whole document is checked by independent namespace-aware
// XML semantics: headings, list starts, cell values, styles, links and notes.
#[test]
fn odt_native_forms() {
    let f = Fixture::new();
    let bytes = package(include_str!("../fixtures/org-odt/document.org"), 16777216).unwrap();
    let path = f.0.join("document.odt");
    std::fs::write(&path, bytes).unwrap();
    verify(&path, true);
    assert!(!f.0.join("marker").exists());
}
// The ODT command publishes current unsaved source using binary file ownership
// without changing source association, format, view, revision or undo history.
#[test]
fn odt_file_publication() {
    let f = Fixture::new();
    let source = "\u{feff}* Heading 界\r\nBody";
    let path = f.write("source.org", source);
    let mut s = host(&f, Some(&path), false);
    s.handle(Event::Paste("#+TITLE: Unsaved\r\n".into()));
    settle(&mut s);
    let before = text(&s);
    let stamp = s.buffer().unwrap().stamp();
    let history = s.buffer().unwrap().history().len();
    let view = s.view().clone();
    action(&mut s, C::ExportOdt);
    for value in ["buffer", "toc:nil num:nil", "document.odt"] {
        argument(&mut s, value);
    }
    assert!(s.echo().contains("Exported Org document"), "{}", s.echo());
    verify(&f.0.join("document.odt"), false);
    assert_eq!(text(&s), before);
    assert_eq!(s.buffer().unwrap().stamp(), stamp);
    assert_eq!(s.buffer().unwrap().history().len(), history);
    assert_eq!(s.view(), &view);
    assert!(s.buffer().unwrap().is_modified());
    assert_eq!(std::fs::read_to_string(path).unwrap(), source);
}
