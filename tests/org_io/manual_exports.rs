use super::*;
use rev::{
    config::{ExportBackend as B, Org},
    org::{
        Cancellation,
        export::{Request, Scope, Work},
    },
};
fn rendered(source: &str, backend: B) -> Result<String, String> {
    let f = Fixture::new();
    let path = f.write("manual.org", source);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let request = Request {
        backend,
        scope: Scope::Buffer,
        options: None,
        time: clock(&Org::default().time).unwrap(),
    };
    let snapshot =
        Work::new(&buffers, id, request, &Org::default())?.run(&Cancellation::default())?;
    Ok(snapshot.render(&Cancellation::default())?.text().into())
}
// Handwritten expectations cover native envelopes, hierarchy and escaped data;
// neither export path has authority to invoke makeinfo, groff or a shell.
#[test]
fn manual_document() {
    let source = "#+TITLE: Notes @ {home}\n#+OPTIONS: toc:nil num:nil\n* First\nBody *bold* and /italic/.\n** Child\n#+begin_src text\n@end example\n.so secret\n\\X'escape'\n#+end_src\n";
    let tex = rendered(source, B::Texinfo).unwrap();
    let man = rendered(source, B::Man).unwrap();
    for expected in [
        "\\input texinfo\n",
        "@settitle Notes @@ @{home@}",
        "@unnumbered First",
        "@unnumberedsec Child",
        "@strong{bold}",
        "@@end example\n.so secret\n@backslashchar{}X'escape'",
        "@bye\n",
    ] {
        assert!(tex.contains(expected), "{tex}");
    }
    for expected in [
        ".TH \"Notes @ {home}\" \"1\"",
        ".SH \"First\"",
        ".SS \"Child\"",
        "\\fBbold\\fP",
        "\\&.so secret",
        "\\[rs]X\\[aq]escape\\[aq]",
    ] {
        assert!(man.contains(expected), "{man}");
    }
    assert!(!man.contains("\n.so"));
}

// Generated node identities are separate from hostile visible labels. Named
// notes resolve once, repeat references retain identity and all data is escaped.
#[test]
fn manual_references() {
    let source = "#+OPTIONS: toc:nil num:nil\n* Start: @home\nSee [[*Start: @home][opening]], <<spot>> [[spot][place]] and [[https://example.org/a,b?q={x}][web, label]].\n\nText [fn:n] twice [fn:n] and [fn::inline *body*].\n\n[fn:n] A /note/.\n\n#+NAME: named\n| a | b |\n";
    for (backend, expected) in [
        (
            B::Texinfo,
            vec![
                "@ref{org-heading-",
                "@anchor{org-target-0}",
                "@ref{org-target-0,,place}",
                "@url{https://example.org/a@comma{}b?q=@{x@},web@comma{} label}",
                "@anchor{org-note-1}",
                " A @emph{note}.",
                "inline @strong{body}",
            ],
        ),
        (
            B::Man,
            vec![
                "opening (see Start: @home)",
                "place (see spot)",
                "web, label <https://example\\&.org/a,b?q={x}>",
                ".IP \"[1]\" 4",
                " A \\fInote\\fP\\&.",
                "inline \\fBbody\\fP",
            ],
        ),
    ] {
        let output = rendered(source, backend).unwrap();
        for value in expected {
            assert!(output.contains(value), "{value}: {output}");
        }
        assert_eq!(output.matches("inline ").count(), 1);
        assert!(!output.contains("[fn:"));
    }
}
// Unresolved and ambiguous relationships fail before any destination exists;
// unsupported kinds keep their literal spelling rather than inventing targets.
#[test]
fn manual_invalid_references() {
    for (name, source, error) in [
        (
            "missing note",
            "Text [fn:x].\n",
            "Unresolved manual footnote",
        ),
        (
            "cycle",
            "Text [fn:x].\n\n[fn:x] loop [fn:x]\n",
            "Cyclic manual footnote",
        ),
        (
            "duplicate note",
            "[fn:x] one\n\n[fn:x] two\n",
            "Duplicate manual footnote",
        ),
        (
            "missing target",
            "[[absent][where]]\n",
            "Unresolved manual heading or target",
        ),
        (
            "duplicate target",
            "<<spot>> <<spot>>\n",
            "Duplicate manual target",
        ),
        (
            "ambiguous heading",
            "* Same\n* Same\n[[*Same]]\n",
            "Ambiguous manual heading",
        ),
    ] {
        for backend in [B::Texinfo, B::Man] {
            assert!(
                rendered(source, backend).unwrap_err().contains(error),
                "{name}"
            );
        }
    }
    let source = "[[shell:touch marker]] and $x *bold* y$ and \\alpha.\n";
    for backend in [B::Texinfo, B::Man] {
        let output = rendered(source, backend).unwrap();
        assert!(output.contains("[[shell:touch marker]]"));
        assert!(output.contains("$x *bold* y$"));
        assert!(!output.contains("@strong{bold}"));
    }
}
