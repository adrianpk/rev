use super::*;
use rev::{
    config::{ExportBackend as B, Org},
    org::{
        Cancellation,
        export::{Request, Scope, Work},
    },
};
fn archive(source: &str, limit: usize) -> Result<rev::org::export::Archive, String> {
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
    snapshot.package(&Cancellation::default())
}
fn package(source: &str, limit: usize) -> Result<Vec<u8>, String> {
    Ok(archive(source, limit)?.bytes().into())
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
    let bytes = package("body\n", 16777216).unwrap();
    assert_eq!(package("body\n", bytes.len()).unwrap(), bytes);
    assert!(
        package("body\n", bytes.len() - 1)
            .unwrap_err()
            .contains("ODT package output byte limit")
    );
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

fn begin(s: &mut Session, scope: &str, options: &str, path: &str) {
    action(s, C::ExportOdt);
    for value in [scope, options, path] {
        argument(s, value);
    }
}
// Unsupported assets, raw export and entity/math syntax remain readable source
// with diagnostics. XML characters cannot inject executable package members.
#[test]
fn odt_rich_diagnostics() {
    let source = "#+ATTR_ODT: :onclick execute\nText </text:p> & \\alpha and $x^2$ plus $x *bold* y$.\n\n[[file:missing.png]]\n#+begin_export odt\n<office:scripts>run</office:scripts>\n#+end_export\n";
    let result = archive(source, 16777216).unwrap();
    for expected in ["rich form", "entity/math", "literal link"] {
        assert!(
            result.diagnostics().iter().any(|d| d.contains(expected)),
            "{expected}: {:?}",
            result.diagnostics()
        );
    }
    let f = Fixture::new();
    let path = f.0.join("rich.odt");
    std::fs::write(&path, result.bytes()).unwrap();
    verify(&path, false);
    let check = std::process::Command::new("python3")
        .args([
            "-c",
            r#"import sys
from pathlib import Path
from scripts.configuration_ui.odt_document import Document, text
d = Document(Path(sys.argv[1]).read_bytes()); body = text(d.content)
for source in ['#+ATTR_ODT: :onclick execute', '</text:p> & \\alpha',
               '[[file:missing.png]]', '$x *bold* y$', '<office:scripts>run</office:scripts>']:
    assert source in body, (source, body)
"#,
            path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );
    let long_math = format!("$x {}*bold* y$", " ".repeat(4096));
    let long = archive(&long_math, 16777216).unwrap();
    assert!(
        long.diagnostics()
            .iter()
            .any(|d| d.contains("math retained as literal"))
    );
    assert!(!f.0.join("missing.png").exists());
}
// Invalid XML, unresolved/cyclic notes, ragged cells and duplicate targets reject
// before any destination effect and preserve source history and current bytes.
#[test]
fn odt_failed_output() {
    for (name, source, expected) in [
        ("XML", "invalid \u{1} scalar\n", "ODT XML cannot represent"),
        ("note", "Missing [fn:x]\n", "Unresolved ODT footnote"),
        (
            "duplicate note",
            "[fn:x] one\n\n[fn:x] two\n",
            "Duplicate ODT footnote",
        ),
        (
            "cycle",
            "Text [fn:x]\n\n[fn:x] [fn:x]\n",
            "Cyclic ODT footnote",
        ),
        ("table", "| A | B |\n| one |\n", "Ragged ODT table"),
        (
            "target",
            "[[missing][label]]\n",
            "Unresolved ODT local target",
        ),
        (
            "duplicate",
            "<<same>> and <<same>>\n",
            "Duplicate ODT bookmark",
        ),
    ] {
        let f = Fixture::new();
        let path = f.write("source.org", source);
        let output = f.write("document.odt", "keep");
        let mut s = host(&f, Some(&path), false);
        let point = s.view().point;
        begin(&mut s, "buffer", "", "document.odt");
        assert!(s.echo().contains(expected), "{name}: {}", s.echo());
        assert_eq!(std::fs::read(output).unwrap(), b"keep", "{name}");
        assert_eq!(text(&s), source, "{name}");
        assert_eq!(s.view().point, point, "{name}");
        assert!(s.buffer().unwrap().history().is_empty(), "{name}");
    }
    let (f, mut s) = super::links::editor("org.limits.export_bytes=100", "Body\n", "");
    f.write("document.odt", "keep");
    begin(&mut s, "buffer", "", "document.odt");
    assert!(s.echo().contains("byte limit"), "{}", s.echo());
    assert_eq!(std::fs::read(f.0.join("document.odt")).unwrap(), b"keep");
}
// Read-only subtree selection keeps real source/view state, rebases outline
// levels and suppresses excluded metadata. Native packaging observes cancellation.
#[test]
fn odt_selected_readonly() {
    let f = Fixture::new();
    let source = "#+TITLE: Hidden\n* Parent\n** TODO Child :work:\nbody 界\n** Sibling\nkeep\n";
    let path = f.write("source.org", source);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    buffers.get_mut(id).unwrap().set_readonly(true);
    let policy = Org::default();
    let request = Request {
        backend: B::Odt,
        scope: Scope::Buffer,
        options: None,
        time: clock(&policy.time).unwrap(),
    };
    let snapshot = Work::new(&buffers, id, request, &policy)
        .unwrap()
        .run(&Cancellation::default())
        .unwrap();
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert!(snapshot.package(&cancelled).is_err());
    let view = View::new(buffers.get(id).unwrap());
    let mut s = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 12,
        },
    )
    .unwrap()
    .with_org_clock(clock);
    s.execute(Command::GotoLine);
    argument(&mut s, "3");
    let view = s.view().clone();
    begin(
        &mut s,
        "subtree",
        "toc:nil num:nil title:nil author:nil todo:nil tags:nil",
        "child.odt",
    );
    let check = std::process::Command::new("python3")
        .args([
            "-c",
            r#"import sys
from pathlib import Path
from scripts.configuration_ui.odt_document import Document, text, NS, key
d = Document(Path(sys.argv[1]).read_bytes())
assert d.texts('text:h') == ['Child']
assert d.content.find('.//text:h', NS).get(key('text', 'outline-level')) == '1'
assert d.meta.find('.//dc:title', NS) is None
for value in ['Hidden', 'Parent', 'Sibling', 'TODO', ':work:']: assert value not in text(d.content)
"#,
            f.0.join("child.odt").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );
    assert_eq!(s.view(), &view);
    assert_eq!(text(&s), source);
    assert!(s.buffer().unwrap().history().is_empty());
    assert!(!s.buffer().unwrap().is_modified());
}
// Declined/cancelled confirmation, source/target changes and aliases cannot
// authorize replacement. ODT stays independent of configured compiler tools.
#[test]
fn odt_publication_guards() {
    use std::os::unix::fs::PermissionsExt;
    let (f, mut s) =
        super::links::editor("org.tools.pdf=['./absent','{input}']", "* Head\nBody\n", "");
    let output = f.write("document.odt", "keep");
    begin(&mut s, "buffer", "toc:nil num:nil", "document.odt");
    argument(&mut s, "no");
    assert_eq!(std::fs::read(&output).unwrap(), b"keep");
    begin(&mut s, "buffer", "toc:nil num:nil", "document.odt");
    cancel(&mut s);
    settle(&mut s);
    assert_eq!(std::fs::read(&output).unwrap(), b"keep");
    begin(&mut s, "buffer", "toc:nil num:nil", "document.odt");
    f.write("document.odt", "external change");
    argument(&mut s, "yes");
    assert!(s.echo().contains("not published"), "{}", s.echo());
    assert_eq!(std::fs::read(&output).unwrap(), b"external change");
    std::fs::set_permissions(&output, std::fs::Permissions::from_mode(0o640)).unwrap();
    begin(&mut s, "buffer", "toc:nil num:nil", "document.odt");
    argument(&mut s, "yes");
    verify(&output, false);
    assert_eq!(
        std::fs::metadata(&output).unwrap().permissions().mode() & 0o777,
        0o640
    );
    begin(&mut s, "buffer", "", "document.odt");
    s.insert('X');
    let before = std::fs::read(&output).unwrap();
    argument(&mut s, "yes");
    assert!(s.echo().contains("source or view changed"), "{}", s.echo());
    assert_eq!(std::fs::read(&output).unwrap(), before);
    let source = std::fs::read(f.0.join("source.org")).unwrap();
    std::fs::hard_link(f.0.join("source.org"), f.0.join("alias.odt")).unwrap();
    for name in ["source.org", "alias.odt"] {
        begin(&mut s, "buffer", "", name);
        assert_eq!(std::fs::read(f.0.join(name)).unwrap(), source);
        assert!(!s.echo().contains("Exported Org document"));
    }
}
// Disk-only includes retain independent freshness authority until publication;
// the export cannot overwrite a checked input or use its stale observation.
#[test]
fn odt_include_guards() {
    let f = Fixture::new();
    let source = f.write("source.org", "#+INCLUDE: \"child.org\"\n");
    let child = f.write("child.org", "Included\n");
    let output = f.write("document.odt", "keep");
    let mut s = host(&f, Some(&source), false);
    begin(&mut s, "buffer", "toc:nil num:nil", "document.odt");
    std::fs::write(&child, "Changed\n").unwrap();
    argument(&mut s, "yes");
    assert!(s.echo().contains("not published"), "{}", s.echo());
    assert_eq!(std::fs::read(output).unwrap(), b"keep");
    begin(&mut s, "buffer", "", "child.org");
    assert!(s.echo().contains("input source"), "{}", s.echo());
    assert_eq!(std::fs::read_to_string(child).unwrap(), "Changed\n");
}
