use super::*;
use rev::{
    config::{ExportBackend, Org},
    org::{
        Cancellation,
        export::{Request, Scope, Work},
    },
};
fn html(source: &str) -> Result<String, String> {
    let f = Fixture::new();
    let path = f.write("source.org", source);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let request = Request {
        backend: ExportBackend::Html,
        scope: Scope::Buffer,
        options: None,
        time: clock(&Org::default().time).unwrap(),
    };
    let snapshot =
        Work::new(&buffers, id, request, &Org::default())?.run(&Cancellation::default())?;
    Ok(snapshot.render(&Cancellation::default())?.text().into())
}
// Handwritten HTML specifies real document boundaries and escaped Unicode text;
// it is independent of the renderer's node traversal and formatting helpers.
#[test]
fn html_export_document() {
    let source = "#+TITLE: Manual & Guide\n#+AUTHOR: Writer\n#+OPTIONS: toc:nil num:nil\n* Heading 界\nBody <script> & *bold*.\n";
    let expected = "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<title>Manual &amp; Guide</title>\n<meta name=\"author\" content=\"Writer\">\n</head>\n<body>\n<main>\n<h1>Manual &amp; Guide</h1>\n<p>Writer</p>\n<h1 id=\"org-heading-1\">Heading 界</h1>\n<p>Body &lt;script&gt; &amp; <strong>bold</strong>.\n</p>\n</main>\n</body>\n</html>\n";
    assert_eq!(html(source).unwrap(), expected);
}
// Lists, cells, notes, local images and code retain their data. Executable link
// and export-block source must never become executable HTML or file effects.
#[test]
fn html_export_native_forms() {
    let source = "#+OPTIONS: toc:nil num:nil\n* Forms\n- one\n- two\n\n| Name | Count |\n|------+-------|\n| Blue | 2 |\n\nSee [[https://example.test/a?x=1&y=2][*site*]] [fn:n].\n\n[[file:images/pic.png]]\n\n[fn:n] A *note*.\n\n#+begin_src sh\ntouch marker\n<script>alert(1)</script>\n#+end_src\n#+begin_export html\n<script>exported()</script>\n#+end_export\n[[javascript:alert(1)][unsafe]]\n";
    let output = html(source).unwrap();
    for expected in [
        "<ul>",
        "<li>",
        "<p>one",
        "<td>Blue</td>",
        "<td>2</td>",
        "href=\"https://example.test/a?x=1&amp;y=2\"",
        "<strong>site</strong>",
        "<img src=\"images/pic.png\"",
        "A <strong>note</strong>.",
        "<pre><code>touch marker\n&lt;script&gt;",
        "&lt;script&gt;exported()&lt;/script&gt;",
        "[[javascript:alert(1)][unsafe]]",
    ] {
        assert!(output.contains(expected), "missing {expected}: {output}");
    }
    assert!(!output.contains("<script>"));
    assert!(!output.contains("href=\"javascript:"));
}
// A missing or duplicate footnote rejects the complete document; no reference
// can silently point at a body which the selected source did not include.
#[test]
fn html_export_notes() {
    assert!(html("Missing [fn:x].\n").is_err());
    assert!(html("[fn:x] One\n\n[fn:x] Two\n").is_err());
    let output = html("Text [fn::inline *body*].\n").unwrap();
    assert!(output.contains("inline <strong>body</strong>"));
    assert!(output.contains("aria-label=\"Back to reference\""));
}
