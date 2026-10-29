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

// Checked source evidence stays frozen across edits, undo and disk replacement.
// A new canonical visit supersedes disk evidence even before an edit occurs.
#[test]
fn export_source_freshness() {
    let f = Fixture::new();
    let root = f.write("root.org", "#+INCLUDE: \"child.org\"\n");
    let child = f.write("child.org", "disk\n");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&root).unwrap();
    let snapshot = work(&buffers, id, Scope::Buffer)
        .run(&Cancellation::default())
        .unwrap();
    snapshot.buffers_current(&buffers).unwrap();
    snapshot.files_current(&Cancellation::default()).unwrap();
    let cid = buffers.visit(&child).unwrap();
    assert!(snapshot.buffers_current(&buffers).is_err());
    let snapshot = work(&buffers, id, Scope::Buffer)
        .run(&Cancellation::default())
        .unwrap();
    let b = buffers.get_mut(cid).unwrap();
    let e = rev::buffer::edit::Replacement {
        source: b.span(0..0).unwrap(),
        text: "changed ".into(),
    };
    let plan = b.prepare(&[e], &[], None).unwrap();
    b.publish(plan, &mut []).unwrap();
    assert!(snapshot.buffers_current(&buffers).is_err());
    assert_eq!(snapshot.text().to_string(), "disk\n");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&root).unwrap();
    let snapshot = work(&buffers, id, Scope::Buffer)
        .run(&Cancellation::default())
        .unwrap();
    std::fs::write(&child, "replacement on disk\n").unwrap();
    assert!(snapshot.files_current(&Cancellation::default()).is_err());
    assert_eq!(snapshot.text().to_string(), "disk\n");
}
// Physical aliases must not bypass include cycle detection. Repeated noncyclic
// includes still count as occurrences even though evidence is retained once.
#[test]
fn export_include_aliases() {
    let f = Fixture::new();
    let root = f.write("root.org", "#+INCLUDE: \"alias.org\"\n");
    std::fs::hard_link(&root, f.0.join("alias.org")).unwrap();
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&root).unwrap();
    assert!(
        work(&buffers, id, Scope::Buffer)
            .run(&Cancellation::default())
            .err()
            .unwrap()
            .contains("cycle")
    );
    std::fs::remove_file(f.0.join("alias.org")).unwrap();
    f.write("child.org", "child\n");
    std::os::unix::fs::symlink("child.org", f.0.join("alias.org")).unwrap();
    let root = f.write(
        "repeated.org",
        "#+INCLUDE: \"child.org\"\n#+INCLUDE: \"alias.org\"\n",
    );
    let id = buffers.visit(&root).unwrap();
    let result = work(&buffers, id, Scope::Buffer)
        .run(&Cancellation::default())
        .unwrap();
    assert_eq!(result.text().to_string(), "child\nchild\n");
    assert_eq!(result.sources().len(), 2);
    let mut policy = Org::default();
    policy.limits.include_files = 1;
    assert!(
        Work::new(&buffers, id, request(Scope::Buffer), &policy)
            .unwrap()
            .run(&Cancellation::default())
            .is_err()
    );
}
// Limits reject the whole projection rather than dropping late includes or
// publishing a truncated macro result. Preparation never touches a destination.
#[test]
fn export_resource_bounds() {
    let f = Fixture::new();
    let root = f.write("root.org", "#+INCLUDE: \"child.org\"\n");
    f.write("child.org", "#+INCLUDE: \"leaf.org\"\n");
    f.write("leaf.org", "long leaf\n");
    let sentinel = f.write("destination/output.html", "preserve existing output");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&root).unwrap();
    for (name, mut limits) in [
        ("depth", Org::default().limits),
        ("files", Org::default().limits),
        ("bytes", Org::default().limits),
        ("total", Org::default().limits),
    ] {
        match name {
            "depth" => limits.include_depth = 1,
            "files" => limits.include_files = 1,
            "bytes" => limits.expansion_bytes = 3,
            _ => limits.agenda_total_bytes = 30,
        }
        let policy = Org {
            limits,
            ..Org::default()
        };
        assert!(
            Work::new(&buffers, id, request(Scope::Buffer), &policy)
                .unwrap()
                .run(&Cancellation::default())
                .is_err(),
            "{name}"
        );
    }
    let body = "x".repeat(500);
    let source = format!("#+MACRO: x {body}\n{{{{{{x}}}}}}\n\n{{{{{{x}}}}}}\n\n{{{{{{x}}}}}}\n");
    let path = f.write("large.org", &source);
    let id = buffers.visit(&path).unwrap();
    let mut policy = Org::default();
    policy.limits.expansion_bytes = 1024;
    assert!(
        Work::new(&buffers, id, request(Scope::Buffer), &policy)
            .unwrap()
            .run(&Cancellation::default())
            .err()
            .unwrap()
            .contains("expansion")
    );
    policy.limits.include_depth = 65;
    assert!(Work::new(&buffers, id, request(Scope::Buffer), &policy).is_err());
    assert_eq!(
        std::fs::read_to_string(sentinel).unwrap(),
        "preserve existing output"
    );
    assert!(!f.0.join("missing/output.html").exists());
}
// A region cannot promote literal block contents into active include directives.
// Empty/exact Unicode selections and file-global macro/options remain explicit.
#[test]
fn export_scope_boundaries() {
    let f = Fixture::new();
    let source = "#+MACRO: x scoped\n#+OPTIONS: toc:nil\n#+OPTIONS: author:nil\n#+begin_example\n#+INCLUDE: \"missing.org\"\n#+end_example\n* One\n{{{x}}}\n* Two\n#+INCLUDE: \"missing.org\"\n";
    let path = f.write("scope.org", source);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let b = buffers.get(id).unwrap();
    let start = source.find("#+INCLUDE:").unwrap();
    let end = start + "#+INCLUDE: \"missing.org\"\n".len();
    let region = Source::new(b).range(Byte(start)..Byte(end)).unwrap();
    assert!(
        work(&buffers, id, Scope::Region(region))
            .run(&Cancellation::default())
            .is_err()
    );
    let result = work(
        &buffers,
        id,
        Scope::Subtree(Byte(source.find("* One").unwrap())),
    )
    .run(&Cancellation::default())
    .unwrap();
    assert_eq!(result.text().to_string(), "* One\nscoped\n");
    assert!(!result.options().toc);
    assert!(!result.options().author);
    let mut selected = request(Scope::Subtree(Byte(source.find("* One").unwrap())));
    selected.options = Some(Options::default());
    let result = Work::new(&buffers, id, selected, &Org::default())
        .unwrap()
        .run(&Cancellation::default())
        .unwrap();
    assert!(result.options().toc);
    assert!(result.options().author);
    let foreign = Source::new(buffers.get(buffers.initial()).unwrap())
        .range(Byte(0)..Byte(0))
        .unwrap();
    assert!(
        Work::new(
            &buffers,
            id,
            request(Scope::Region(foreign)),
            &Org::default()
        )
        .is_err()
    );
}
// BOM/CRLF/EOF, affiliated attributes and literal Lisp stay in original evidence.
// The real marker path makes accidental code evaluation observable to this test.
#[test]
fn export_source_preservation() {
    let f = Fixture::new();
    let marker = f.0.join("executed");
    let output = f.write("output.html", "unchanged output");
    let source = format!(
        "\u{feff}#+TITLE: Source\r\n#+NAME: sample\r\n#+CAPTION: Table\r\n#+ATTR_HTML: :class report\r\n| 界 |\r\n#+begin_src sh\r\ntouch '{}'\r\n#+end_src\r\n# Local Variables:\r\n# eval: (write-region \"ran\" nil \"{}\")\r\n# End:",
        marker.display(),
        marker.display()
    );
    let path = f.write("source.org", &source);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    buffers.get_mut(id).unwrap().set_readonly(true);
    let before = buffers.get(id).unwrap().stamp();
    let result = work(&buffers, id, Scope::Buffer)
        .run(&Cancellation::default())
        .unwrap();
    assert!(result.sources()[0].has_bom());
    assert_eq!(
        result.sources()[0].text().to_string(),
        source.trim_start_matches('\u{feff}')
    );
    assert!(result.text().to_string().ends_with("# End:"));
    assert!(
        result
            .elements()
            .any(|n| n.kind == StructureKind::Table && n.affiliated.is_some())
    );
    assert_eq!(buffers.get(id).unwrap().stamp(), before);
    assert_eq!(buffers.get(id).unwrap().history().len(), 0);
    assert_eq!(std::fs::read_to_string(path).unwrap(), source);
    assert_eq!(std::fs::read_to_string(output).unwrap(), "unchanged output");
    assert!(!marker.exists());
}
// One owned Job can be cancelled/reaped without freeing an extra worker slot or
// creating files. Cancellation remains effective after work has finished too.
#[test]
fn export_owned_cancellation() {
    let f = Fixture::new();
    let path = f.write("root.org", "body\n");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(work(&buffers, id, Scope::Buffer).run(&cancel).is_err());
    let selected = work(&buffers, id, Scope::Buffer);
    let mut job = rev::org::Job::default();
    let b = buffers.get(id).unwrap();
    job.start(b.stamp(), move |cancel| selected.run(cancel))
        .unwrap();
    assert!(job.cancel());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    let result = loop {
        if let Some(result) = job.poll(b) {
            break result;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::yield_now();
    };
    assert!(matches!(result, Err(rev::org::JobError::Cancelled)));
    assert!(!job.is_active());
    assert_eq!(std::fs::read_to_string(path).unwrap(), "body\n");
}

// Invalid ingress and malformed or cut native forms cannot become usable
// includes. The complete source and existing destination survive every failure.
#[test]
fn export_invalid_includes() {
    let f = Fixture::new();
    let root = f.write("root.org", "#+INCLUDE: \"child.org\"\n");
    let output = f.write("output.txt", "preserved");
    for (name, bytes) in [
        ("UTF-8", vec![0xff]),
        ("NUL", b"bad\0text".to_vec()),
        ("block", b"#+begin_src sh\nunclosed\n".to_vec()),
    ] {
        let child = f.0.join("child.org");
        std::fs::write(&child, &bytes).unwrap();
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.visit(&root).unwrap();
        assert!(
            work(&buffers, id, Scope::Buffer)
                .run(&Cancellation::default())
                .is_err(),
            "{name}"
        );
        assert_eq!(std::fs::read(child).unwrap(), bytes);
    }
    f.write(
        "child.org",
        "#+begin_example\n#+INCLUDE: \"absent.org\"\n#+end_example\n",
    );
    let root = f.write("cut.org", "#+INCLUDE: \"child.org\" :lines \"2-3\"\n");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&root).unwrap();
    assert!(
        work(&buffers, id, Scope::Buffer)
            .run(&Cancellation::default())
            .is_err()
    );
    let root = f.write("directory.org", "#+INCLUDE: \".\"\n");
    let id = buffers.visit(&root).unwrap();
    assert!(
        work(&buffers, id, Scope::Buffer)
            .run(&Cancellation::default())
            .is_err()
    );
    std::os::unix::fs::symlink("absent.org", f.0.join("dangling.org")).unwrap();
    let root = f.write("dangling-root.org", "#+INCLUDE: \"dangling.org\"\n");
    let id = buffers.visit(&root).unwrap();
    assert!(
        work(&buffers, id, Scope::Buffer)
            .run(&Cancellation::default())
            .is_err()
    );
    assert_eq!(std::fs::read_to_string(output).unwrap(), "preserved");
    assert!(!f.0.join("absent.org").exists());
}

// Drawers can contain structurally recognized children for editing, but those
// children must never activate export includes, definitions or invocations.
#[test]
fn export_literal_drawers() {
    let f = Fixture::new();
    let source = "#+MACRO: x expanded\n* Task\n:LOGBOOK:\n#+INCLUDE: \"missing.org\"\n#+MACRO: x wrong\n{{{x}}}\n:END:\n{{{x}}}\n";
    let path = f.write("drawers.org", source);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let result = work(&buffers, id, Scope::Buffer)
        .run(&Cancellation::default())
        .unwrap();
    let text = result.text().to_string();
    assert!(text.contains("#+INCLUDE: \"missing.org\"\n#+MACRO: x wrong\n{{{x}}}"));
    assert!(text.ends_with("expanded\n"));
    assert_eq!(std::fs::read_to_string(path).unwrap(), source);
    assert!(!f.0.join("missing.org").exists());
}
