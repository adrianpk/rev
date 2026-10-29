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
