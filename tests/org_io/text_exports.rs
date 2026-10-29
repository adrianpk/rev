use super::*;
use rev::{
    config::{ExportBackend as B, Org},
    org::{
        Cancellation,
        export::{Request, Scope, Work},
    },
};
fn request(backend: B) -> Request {
    Request {
        backend,
        scope: Scope::Buffer,
        options: None,
        time: clock(&Org::default().time).unwrap(),
    }
}
fn rendered(source: &str, backend: B) -> Result<String, String> {
    let f = Fixture::new();
    let path = f.write("source.org", source);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let snapshot =
        Work::new(&buffers, id, request(backend), &Org::default())?.run(&Cancellation::default())?;
    Ok(snapshot.render(&Cancellation::default())?.text().into())
}
const SOURCE: &str = "#+TITLE: Manual\n#+AUTHOR: Writer\n#+OPTIONS: toc:nil num:nil\n* TODO Plan :work:\nVisit [[file:guide.txt][the guide]] and *bold* [fn:n].\n\n- one\n- two\n\n| Name | Count |\n|------+-------|\n| Blue | 2 |\n\n[fn:n] A note.\n";
// Handwritten expected documents independently specify heading, table, list,
// link and footnote semantics for the four native formats.
#[test]
fn text_export_documents() {
    let md = "# Manual\n\nWriter\n\n# TODO Plan :work:\nVisit [the guide](<file:guide.txt>) and **bold** [^n].\n\n- one\n- two\n\n| Name | Count |\n| --- | --- |\n| Blue | 2 |\n\n[^n]: A note.\n";
    assert_eq!(rendered(SOURCE, B::Markdown).unwrap(), md);
    let plain = "Manual\n\nWriter\n\nTODO Plan :work:\nVisit the guide (file:guide.txt) and bold [n].\n\n- one\n- two\n\n| Name | Count |\n| Blue | 2 |\n\n[n] A note.\n";
    for backend in [B::Text, B::Utf8] {
        assert_eq!(rendered(SOURCE, backend).unwrap(), plain);
    }
    assert_eq!(rendered(SOURCE, B::Org).unwrap(), SOURCE);
}
// ASCII conversion fails with the exact unrepresentable scalar; UTF-8 formats
// retain it instead of substituting replacement text or writing partial output.
#[test]
fn text_export_unicode() {
    let source = "#+OPTIONS: toc:nil num:nil\n* Heading 界\nbody é\n";
    let error = rendered(source, B::Text).unwrap_err();
    assert!(error.contains("U+754C"), "{error}");
    assert_eq!(rendered(source, B::Utf8).unwrap(), "Heading 界\nbody é\n");
    assert_eq!(
        rendered(source, B::Markdown).unwrap(),
        "# Heading 界\nbody é\n"
    );
    assert_eq!(rendered(source, B::Org).unwrap(), source);
}
// Literal source remains inert and embedded fences cannot terminate its native
// Markdown code block. Ordinary raw HTML is escaped as paragraph text.
#[test]
fn text_export_blocks() {
    let source = "#+OPTIONS: toc:nil num:nil\n* Code\n#+begin_src sh\ntouch marker\n```\n#+end_src\n\nA <tag> and ~a`b~.\n";
    let result = rendered(source, B::Markdown).unwrap();
    assert!(
        result.contains("````sh\ntouch marker\n```\n````\n"),
        "{result}"
    );
    assert!(result.contains("A \\<tag\\> and ``a`b``."), "{result}");
}
// Inline notes retain their bodies, and a region cannot silently export a
// reference whose definition was omitted from its selected source.
#[test]
fn text_export_notes() {
    let source = "#+OPTIONS: toc:nil num:nil\nText [fn::inline *note*].\n";
    let result = rendered(source, B::Markdown).unwrap();
    assert!(result.contains("Text [^inline-1]."));
    assert!(result.contains("[^inline-1]: inline **note**"));
    assert!(rendered("Missing [fn:x].\n", B::Markdown).is_err());
}

fn editor(source: &str) -> (Fixture, Session) {
    let f = Fixture::new();
    let path = f.write("source.org", source);
    let session = host(&f, Some(&path), false);
    (f, session)
}
fn export_file(session: &mut Session, command: C, scope: &str, options: &str, path: &str) {
    action(session, command);
    argument(session, scope);
    argument(session, options);
    argument(session, path);
}
// Compiled commands publish the current unsaved source without acknowledging a
// save or changing its source association, history, point or inactive peer view.
#[test]
fn text_export_file() {
    let source = "\u{feff}* Heading 界\r\nbody";
    let (f, mut session) = editor(source);
    session.handle(Event::Paste("#+TITLE: Unsaved\r\n".into()));
    settle(&mut session);
    let before = text(&session);
    let stamp = session.buffer().unwrap().stamp();
    let history = session.buffer().unwrap().history().len();
    let point = session.view().point;
    assert!(session.buffer().unwrap().is_modified());
    export_file(
        &mut session,
        C::ExportMarkdown,
        "buffer",
        "toc:nil num:nil",
        "output.md",
    );
    assert!(
        session.echo().contains("Exported Org document"),
        "{}",
        session.echo()
    );
    let output = std::fs::read_to_string(f.0.join("output.md")).unwrap();
    assert!(output.starts_with("# Unsaved\n\n# Heading 界"), "{output}");
    assert_eq!(text(&session), before);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(session.buffer().unwrap().history().len(), history);
    assert_eq!(session.view().point, point);
    assert!(session.buffer().unwrap().is_modified());
    assert_eq!(
        std::fs::read_to_string(f.0.join("source.org")).unwrap(),
        source
    );
}
// Replacement consent is tied to its observed target. Decline, cancellation and
// an external replacement preserve the destination; successful writes keep mode.
#[test]
fn text_export_replacement() {
    use std::os::unix::fs::PermissionsExt;
    let (f, mut session) = editor("* Heading\nbody\n");
    let target = f.write("output.txt", "keep");
    std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o640)).unwrap();
    export_file(
        &mut session,
        C::ExportUtf8,
        "buffer",
        "toc:nil num:nil",
        "output.txt",
    );
    assert!(session.echo().contains("C-g"));
    argument(&mut session, "no");
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "keep");
    export_file(
        &mut session,
        C::ExportUtf8,
        "buffer",
        "toc:nil num:nil",
        "output.txt",
    );
    cancel(&mut session);
    settle(&mut session);
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "keep");
    export_file(
        &mut session,
        C::ExportUtf8,
        "buffer",
        "toc:nil num:nil",
        "output.txt",
    );
    std::fs::write(&target, "external change").unwrap();
    argument(&mut session, "yes");
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "external change");
    export_file(
        &mut session,
        C::ExportUtf8,
        "buffer",
        "toc:nil num:nil",
        "output.txt",
    );
    argument(&mut session, "yes");
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "Heading\nbody\n");
    assert_eq!(
        std::fs::metadata(target).unwrap().permissions().mode() & 0o777,
        0o640
    );
    assert_eq!(session.buffer().unwrap().history().len(), 0);
}
// Includes are read-only inputs even when disk-only or aliased. Changed disk
// evidence during confirmation cannot be used for a subsequent publication.
#[test]
fn text_export_input_guards() {
    let (f, mut session) = editor("#+INCLUDE: \"child.org\"\n");
    let child = f.write("child.org", "included body\n");
    let target = f.write("output.md", "keep");
    export_file(
        &mut session,
        C::ExportMarkdown,
        "buffer",
        "toc:nil num:nil",
        "output.md",
    );
    std::fs::write(&child, "changed child\n").unwrap();
    argument(&mut session, "yes");
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "keep");
    assert!(
        session.echo().contains("not published"),
        "{}",
        session.echo()
    );
    for path in ["source.org", "child.org", "alias.org"] {
        if path == "alias.org" {
            std::fs::hard_link(&child, f.0.join(path)).unwrap();
        }
        let before = std::fs::read(f.0.join(path)).unwrap();
        export_file(&mut session, C::ExportOrg, "buffer", "", path);
        assert_eq!(std::fs::read(f.0.join(path)).unwrap(), before);
        assert!(
            !session.echo().contains("Exported Org document"),
            "{}",
            session.echo()
        );
    }
}
// Unicode conversion and missing parents fail before creating output. A read-only
// source remains exportable, while cancelling any prompt leaves no destination.
#[test]
fn text_export_failed_output() {
    let (f, mut session) = editor("* 界\nbody\n");
    let target = f.write("output.txt", "keep");
    export_file(
        &mut session,
        C::ExportText,
        "buffer",
        "toc:nil num:nil",
        "output.txt",
    );
    assert!(session.echo().contains("U+754C"), "{}", session.echo());
    assert_eq!(std::fs::read_to_string(target).unwrap(), "keep");
    export_file(
        &mut session,
        C::ExportUtf8,
        "buffer",
        "",
        "missing/output.txt",
    );
    assert!(!f.0.join("missing").exists());
    action(&mut session, C::ExportUtf8);
    cancel(&mut session);
    settle(&mut session);
    assert!(!f.0.join("cancelled.txt").exists());
    let before = text(&session);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&f.0.join("source.org")).unwrap();
    buffers.get_mut(id).unwrap().set_readonly(true);
    let view = View::new(buffers.get(id).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 12,
        },
    )
    .unwrap()
    .with_org_clock(clock);
    export_file(
        &mut session,
        C::ExportUtf8,
        "buffer",
        "toc:nil num:nil",
        "readonly.txt",
    );
    assert!(
        session.echo().contains("Exported Org document"),
        "{}",
        session.echo()
    );
    assert_eq!(text(&session), before);
}
