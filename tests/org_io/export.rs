use super::Fixture;
use rev::{
    buffer::{BufferId, Buffers},
    config::{ExportBackend, Org},
    coordinates::Byte,
    org::{
        Cancellation, Source, StructureKind,
        export::{Options, Request, Scope, Work},
        time::{ZoneEnvironment, ZoneSelection, ZonedOperation},
    },
};
use std::time::UNIX_EPOCH;
fn request(scope: Scope) -> Request {
    Request {
        backend: ExportBackend::Html,
        scope,
        options: None,
        time: ZonedOperation::capture(&ZoneSelection::Utc, &ZoneEnvironment::system(), || {
            Ok(UNIX_EPOCH)
        })
        .unwrap(),
    }
}
fn work(buffers: &Buffers, id: BufferId, scope: Scope) -> Work {
    Work::new(buffers, id, request(scope), &Org::default()).unwrap()
}
// The export layer retains canonical unsaved bytes, native structure and a single
// captured time. It cannot save or alter source history while preparing data.
#[test]
fn export_snapshot_contract() {
    let f = Fixture::new();
    let original = "\u{feff}#+TITLE: Native 界\r\n#+OPTIONS: toc:nil num:nil\r\n* Heading\r\nbody";
    let path = f.write("source.org", original);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let edit = rev::buffer::edit::Replacement {
        source: buffer.span(buffer.len_chars()..buffer.len_chars()).unwrap(),
        text: " unsaved".into(),
    };
    let plan = buffer.prepare(&[edit], &[], None).unwrap();
    buffer.publish(plan, &mut []).unwrap();
    let stamp = buffer.stamp();
    let snapshot = work(&buffers, id, Scope::Buffer)
        .run(&Cancellation::default())
        .unwrap();
    assert!(snapshot.text().to_string().ends_with("body unsaved"));
    assert_eq!(
        snapshot.sources()[0].version(),
        &rev::org::export::SourceVersion::Buffer(stamp)
    );
    assert_eq!(snapshot.metadata().title.as_deref(), Some("Native 界"));
    assert!(!snapshot.options().toc);
    assert!(!snapshot.options().numbered);
    assert!(
        snapshot
            .elements()
            .any(|n| matches!(n.kind, StructureKind::Heading { level: 1 }))
    );
    assert_eq!(snapshot.request().time.instant().unix_seconds(), 0);
    assert_eq!(buffers.get(id).unwrap().stamp(), stamp);
    assert_eq!(buffers.get(id).unwrap().history().len(), 1);
    assert_eq!(std::fs::read_to_string(path).unwrap(), original);
}
// Region and subtree scope retain exact UTF-8 boundaries; a source range from
// another revision or buffer never becomes export authority over this buffer.
#[test]
fn export_scope_contract() {
    let f = Fixture::new();
    let path = f.write(
        "scope.org",
        "#+TITLE: File\n* One\n界 body\n** Child\ntext\n* Two\nnext\n",
    );
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get(id).unwrap();
    let source = Source::new(buffer);
    let text = source
        .text(&source.range(Byte(0)..Byte(buffer.len_bytes())).unwrap())
        .unwrap()
        .to_string();
    let start = text.find("界").unwrap();
    let region = source.range(Byte(start)..Byte(start + 3)).unwrap();
    let result = work(&buffers, id, Scope::Region(region))
        .run(&Cancellation::default())
        .unwrap();
    assert_eq!(result.text().to_string(), "界");
    let result = work(&buffers, id, Scope::Subtree(Byte(start)))
        .run(&Cancellation::default())
        .unwrap();
    assert_eq!(
        result.text().to_string(),
        "* One\n界 body\n** Child\ntext\n"
    );
    assert_eq!(result.metadata().title.as_deref(), Some("File"));
    assert!(
        work(&buffers, id, Scope::Subtree(Byte(0)))
            .run(&Cancellation::default())
            .is_err()
    );
}
// Typed boolean options and the fixed backend registry reject unsupported forms
// instead of interpreting Lisp, a plugin name or a tool request.
#[test]
fn export_options_contract() {
    for value in ["toc:2", "num:(eval x)", "unknown:t", "toc"] {
        assert!(Options::parse(value).is_err(), "{value}");
    }
    assert!(!Options::parse("tags:0 title:nil").unwrap().tags);
    let f = Fixture::new();
    let path = f.write("disabled.org", "body");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let mut policy = Org::default();
    policy.export.backends = vec![ExportBackend::Org];
    policy.export.default_format = ExportBackend::Org;
    assert!(Work::new(&buffers, id, request(Scope::Buffer), &policy).is_err());
}

// Native include directives resolve relative to each observed source, prefer
// unsaved canonical buffers and preserve all literal source-block contents.
#[test]
fn export_include_sources() {
    let f = Fixture::new();
    let original = "#+TITLE: Includes\n#+INCLUDE: \"sub/child.org\"\n#+begin_src sh\n#+INCLUDE: \"absent.org\"\ntouch marker\n#+end_src\n";
    let path = f.write("root.org", original);
    let child = f.write(
        "sub/child.org",
        "* Child\n#+INCLUDE: \"leaf.org\" :lines \"2-3\"\n",
    );
    f.write("sub/leaf.org", "skip\nretained 界\nskip too\n");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let cid = buffers.visit(&child).unwrap();
    let b = buffers.get_mut(cid).unwrap();
    let e = rev::buffer::edit::Replacement {
        source: b.span(2..7).unwrap(),
        text: "Unsaved".into(),
    };
    let plan = b.prepare(&[e], &[], None).unwrap();
    b.publish(plan, &mut []).unwrap();
    let result = work(&buffers, id, Scope::Buffer)
        .run(&Cancellation::default())
        .unwrap();
    let text = result.text().to_string();
    assert!(text.contains("* Unsaved\nretained 界\n"), "{text}");
    assert!(text.contains("#+INCLUDE: \"absent.org\"\ntouch marker"));
    assert_eq!(result.sources().len(), 3);
    assert_eq!(
        result.sources()[1].version(),
        &rev::org::export::SourceVersion::Buffer(buffers.get(cid).unwrap().stamp())
    );
    assert_eq!(std::fs::read_to_string(path).unwrap(), original);
    assert!(!f.0.join("marker").exists());
    assert_eq!(buffers.get(cid).unwrap().history().len(), 1);
}
// Text-only macros admit bounded positional substitution and nested expansion.
// Unknown names remain literal with diagnostics; code, links and drawers are inert.
#[test]
fn export_text_macros() {
    let f = Fixture::new();
    let source = "#+TITLE: {{{greet(World)}}}\n#+MACRO: greet Hello $1\n#+MACRO: outer {{{greet($1)}}}!\n* Heading\n{{{outer(界)}}} /emphasis/ {{{unknown}}}\n~{{{greet(code)}}}~ [[file:{{{unknown}}}][link]]\n#+begin_example\n{{{greet(literal)}}}\n#+end_example\n";
    let path = f.write("macros.org", source);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let result = work(&buffers, id, Scope::Buffer)
        .run(&Cancellation::default())
        .unwrap();
    let text = result.text().to_string();
    assert!(
        text.contains("Hello 界! /emphasis/ {{{unknown}}}"),
        "{text}"
    );
    assert!(!text.contains("#+MACRO:"));
    assert!(text.contains("~{{{greet(code)}}}~"));
    assert!(text.contains("{{{greet(literal)}}}"));
    assert!(text.contains("file:{{{unknown}}}"));
    assert_eq!(result.metadata().title.as_deref(), Some("Hello World"));
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(std::fs::read_to_string(path).unwrap(), source);
}
// A cycle, malformed/unsupported directive or evaluation request fails closed.
// Data-local and block code have no evaluator or subprocess boundary here.
#[test]
fn export_expansion_failures() {
    let f = Fixture::new();
    f.write("child.org", "body\n");
    for (name, source) in [
        ("missing", "#+INCLUDE: \"absent.org\"\n"),
        ("cycle", "#+INCLUDE: \"root.org\"\n"),
        ("option", "#+INCLUDE: \"child.org\" :minlevel 1\n"),
        ("network", "#+INCLUDE: \"https://example.invalid/data\"\n"),
        (
            "eval",
            "#+MACRO: bad (eval (shell-command \"touch marker\"))\n{{{bad}}}\n",
        ),
        (
            "recursive macro",
            "#+MACRO: a {{{b}}}\n#+MACRO: b {{{a}}}\n{{{a}}}\n",
        ),
        ("argument", "#+MACRO: greet Hello $1\n{{{greet}}}\n"),
    ] {
        let path = f.write("root.org", source);
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.visit(&path).unwrap();
        assert!(
            work(&buffers, id, Scope::Buffer)
                .run(&Cancellation::default())
                .is_err(),
            "{name}"
        );
        assert_eq!(std::fs::read_to_string(path).unwrap(), source);
        assert!(!f.0.join("marker").exists());
    }
}
