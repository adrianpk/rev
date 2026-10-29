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
    let expected = "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<title>Manual &amp; Guide</title>\n<meta name=\"author\" content=\"Writer\">\n</head>\n<body>\n<main>\n<h1>Manual &amp; Guide</h1>\n<p>Writer</p>\n<h1 id=\"org-heading-0-1\">Heading 界</h1>\n<p>Body &lt;script&gt; &amp; <strong>bold</strong>.\n</p>\n</main>\n</body>\n</html>\n";
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
        "src=\"images/pic.png\"",
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

// Affiliated declarations affect only admitted HTML data. Captions retain
// inline semantics, identifiers stay unique, and image dimensions remain finite.
#[test]
fn html_export_attributes() {
    let source = "#+OPTIONS: toc:nil num:nil\n* Data\n#+NAME: records\n#+CAPTION: Recorded *totals*\n#+ATTR_HTML: :class striped :border 1\n| Name | Count |\n| Blue | 2 |\n\n#+CAPTION: A picture\n#+ATTR_HTML: :id picture :class \"hero image\" :alt \"Icon & mark\" :width 320 :height 50%\n[[file:images/a%20b.png]]\n\n[[records][table]] [[#picture][picture]] [[*Data][heading]]\n";
    let output = html(source).unwrap();
    for expected in [
        "<table border=\"1\" class=\"striped\">",
        "<caption>Recorded <strong>totals</strong>",
        "<img alt=\"Icon &amp; mark\" class=\"hero image\" height=\"50%\" id=\"picture\" loading=\"lazy\" src=\"images/a%20b.png\" width=\"320\">",
        "<figcaption>A picture",
        "href=\"#picture\"",
        "href=\"#org-heading-0-1\"",
        "href=\"#org-target-7265636f726473\"",
    ] {
        assert!(output.contains(expected), "missing {expected}: {output}");
    }
    for (name, source) in [
        ("event", "#+ATTR_HTML: :onclick run\nText\n"),
        ("style", "#+ATTR_HTML: :style hidden\nText\n"),
        ("geometry", "#+ATTR_HTML: :width 16385\n[[file:a.png]]\n"),
        ("wrong element", "#+ATTR_HTML: :alt text\nText\n"),
        (
            "duplicate",
            "#+ATTR_HTML: :id same\nOne\n\n#+ATTR_HTML: :id same\nTwo\n",
        ),
    ] {
        assert!(html(source).is_err(), "{name}");
    }
}
// The native command exports unsaved canonical source through the checked file
// owner without acknowledging a save or changing its editing state.
#[test]
fn html_export_file() {
    let f = Fixture::new();
    let path = f.write("source.org", "\u{feff}* Saved\r\nbody");
    let mut session = host(&f, Some(&path), false);
    session.handle(Event::Paste("#+TITLE: Unsaved\r\n".into()));
    settle(&mut session);
    let before = text(&session);
    let stamp = session.buffer().unwrap().stamp();
    let history = session.buffer().unwrap().history().len();
    let view = session.view().clone();
    action(&mut session, C::ExportHtml);
    for value in ["buffer", "toc:nil num:nil", "output.html"] {
        argument(&mut session, value);
    }
    assert!(
        session.echo().contains("Exported Org document"),
        "{}",
        session.echo()
    );
    let result = std::fs::read_to_string(f.0.join("output.html")).unwrap();
    assert!(result.contains("<title>Unsaved</title>"));
    assert!(result.contains("Saved</h1>"));
    assert_eq!(text(&session), before);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(session.buffer().unwrap().history().len(), history);
    assert_eq!(session.view(), &view);
    assert!(session.buffer().unwrap().is_modified());
    assert_eq!(
        std::fs::read_to_string(path).unwrap(),
        "\u{feff}* Saved\r\nbody"
    );
}
