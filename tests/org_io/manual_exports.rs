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
            "unexported target",
            "Link [[inside]].\n\n[fn:unused] <<inside>> note\n",
            "target was not exported",
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

// Full handwritten golden documents fix the complete native format, including
// lists, table cells, notes and hostile formatter-looking source blocks.
#[test]
fn manual_golden() {
    let source = include_str!("../fixtures/org-manual/document.org");
    for (backend, expected, format) in [
        (
            B::Texinfo,
            include_str!("../fixtures/org-manual/document.texi"),
            "texinfo",
        ),
        (
            B::Man,
            include_str!("../fixtures/org-manual/document.1"),
            "man",
        ),
    ] {
        let output = rendered(source, backend).unwrap();
        assert_eq!(output, expected, "{format}");
        let f = Fixture::new();
        let path = f.write("document", &output);
        let check = std::process::Command::new("python3")
            .args([
                "scripts/configuration_ui/manual_document.py",
                path.to_str().unwrap(),
                format,
            ])
            .output()
            .unwrap();
        assert!(
            check.status.success(),
            "{}",
            String::from_utf8_lossy(&check.stderr)
        );
        assert!(!f.0.join("marker").exists());
    }
}
fn begin(s: &mut Session, command: C, destination: &str) {
    action(s, command);
    for value in ["buffer", "toc:nil num:nil", destination] {
        argument(s, value);
    }
}
// Dirty BOM/CRLF source exports through both commands without acknowledging a
// save, changing view/revision/history or evaluating source. Undo remains normal.
#[test]
fn manual_publication() {
    for (command, backend, name) in [
        (C::ExportTexinfo, B::Texinfo, "output.texi"),
        (C::ExportMan, B::Man, "output.1"),
    ] {
        let f = Fixture::new();
        let source = "\u{feff}* Heading\r\nBody";
        let path = f.write("source.org", source);
        let mut s = host(&f, Some(&path), false);
        s.handle(Event::Paste("#+TITLE: Unsaved\r\n".into()));
        settle(&mut s);
        let before = text(&s);
        let stamp = s.buffer().unwrap().stamp();
        let history = s.buffer().unwrap().history().len();
        let view = s.view().clone();
        begin(&mut s, command, name);
        assert!(s.echo().contains("Exported Org document"), "{}", s.echo());
        let output = std::fs::read_to_string(f.0.join(name)).unwrap();
        assert!(output.contains("Unsaved") && output.contains("Body\n\n"));
        if backend == B::Texinfo {
            assert!(output.contains("@unnumbered Heading"));
            assert!(!output.contains("@contents"));
        } else {
            assert!(output.contains(".SH \"Heading\""));
        }
        assert_eq!(text(&s), before);
        assert_eq!(s.buffer().unwrap().stamp(), stamp);
        assert_eq!(s.buffer().unwrap().history().len(), history);
        assert_eq!(s.view(), &view);
        assert!(s.buffer().unwrap().is_modified());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), source);
        s.execute(Command::History(rev::command::History::Undo));
        assert_eq!(text(&s), "* Heading\r\nBody");
    }
}
// Every prompt remains cancellable. Observed replacement, disk input, source
// revision and hard-link alias guards are checked by the existing file owner.
#[test]
fn manual_publication_guards() {
    for command in [C::ExportTexinfo, C::ExportMan] {
        let (f, mut s) = super::links::editor(
            "org.tools.pdf=['./absent','{input}']",
            "#+INCLUDE: \"child.org\"\n* Heading\nBody\n",
            "",
        );
        f.write("child.org", "included\n");
        let output = f.write("document", "keep");
        for count in 0..4 {
            action(&mut s, command);
            for value in ["buffer", "toc:nil num:nil", "document"]
                .into_iter()
                .take(count)
            {
                argument(&mut s, value);
            }
            cancel(&mut s);
            settle(&mut s);
            assert_eq!(std::fs::read(&output).unwrap(), b"keep");
            assert!(!s.buffer().unwrap().is_modified());
        }
        begin(&mut s, command, "document");
        argument(&mut s, "no");
        assert_eq!(std::fs::read(&output).unwrap(), b"keep");
        begin(&mut s, command, "document");
        f.write("document", "changed");
        argument(&mut s, "yes");
        assert!(s.echo().contains("not published"), "{}", s.echo());
        assert_eq!(std::fs::read(&output).unwrap(), b"changed");
        begin(&mut s, command, "document");
        f.write("child.org", "changed child\n");
        argument(&mut s, "yes");
        assert!(s.echo().contains("not published"), "{}", s.echo());
        assert_eq!(std::fs::read(&output).unwrap(), b"changed");
        begin(&mut s, command, "document");
        s.insert('X');
        argument(&mut s, "yes");
        assert!(s.echo().contains("source or view changed"), "{}", s.echo());
        assert_eq!(std::fs::read(&output).unwrap(), b"changed");
        std::fs::hard_link(f.0.join("source.org"), f.0.join("alias")).unwrap();
        for name in ["source.org", "child.org", "alias"] {
            let bytes = std::fs::read(f.0.join(name)).unwrap();
            begin(&mut s, command, name);
            assert!(!s.echo().contains("Exported Org document"));
            assert_eq!(std::fs::read(f.0.join(name)).unwrap(), bytes);
        }
        assert!(!f.0.join("marker").exists());
    }
}
// Renderer cancellation, exact final byte bounds and unsupported controls fail
// atomically. Retained note bodies count toward the same finite output budget.
#[test]
fn manual_bounds() {
    for backend in [B::Texinfo, B::Man] {
        let f = Fixture::new();
        let path = f.write("source.org", "Text [fn:n].\n\n[fn:n] Note *body*.\n");
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.visit(&path).unwrap();
        let expected = rendered("Text [fn:n].\n\n[fn:n] Note *body*.\n", backend).unwrap();
        for limit in [expected.len(), expected.len() - 1] {
            let mut policy = Org::default();
            policy.limits.export_bytes = limit;
            let request = Request {
                backend,
                scope: Scope::Buffer,
                options: None,
                time: clock(&policy.time).unwrap(),
            };
            let snapshot = Work::new(&buffers, id, request, &policy)
                .unwrap()
                .run(&Cancellation::default())
                .unwrap();
            let result = snapshot.render(&Cancellation::default());
            if limit == expected.len() {
                assert_eq!(result.unwrap().text(), expected);
            } else {
                assert!(result.err().unwrap().contains("byte limit"));
            }
            let cancel = Cancellation::default();
            cancel.cancel();
            assert!(snapshot.render(&cancel).is_err());
        }
        for source in ["bad \u{1} scalar\n", "bad \u{1b}[31m scalar\n"] {
            assert!(rendered(source, backend).is_err());
        }
    }
    assert!(
        rendered(
            "* One\n** Two\n*** Three\n**** Four\n***** Five\n",
            B::Texinfo
        )
        .unwrap_err()
        .contains("four relative heading")
    );
}

// Read-only selections rebase actual ancestry, respect explicit task/metadata
// options and keep view, source and undo state. Region envelopes contain no peer.
#[test]
fn manual_selected_scope() {
    let source =
        "#+TITLE: Hidden\n* Parent\n** TODO Child :work:\nregion界 tail\n** Sibling\nkeep\n";
    for command in [C::ExportTexinfo, C::ExportMan] {
        let f = Fixture::new();
        let path = f.write("source.org", source);
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.visit(&path).unwrap();
        buffers.get_mut(id).unwrap().set_readonly(true);
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
        action(&mut s, command);
        for value in [
            "subtree",
            "toc:nil num:nil title:nil author:nil todo:nil tags:nil",
            "child",
        ] {
            argument(&mut s, value);
        }
        assert!(s.echo().contains("Exported Org document"), "{}", s.echo());
        let output = std::fs::read_to_string(f.0.join("child")).unwrap();
        assert!(output.contains("Child") && output.contains("region界 tail"));
        for hidden in ["Hidden", "Parent", "Sibling", "TODO", ":work:"] {
            assert!(!output.contains(hidden), "{hidden}");
        }
        assert_eq!(s.view(), &view);
        s.execute(Command::GotoLine);
        argument(&mut s, "4");
        s.handle(Event::Key(KeyEvent::new(
            KeyCode::Char(' '),
            KeyModifiers::CONTROL,
        )));
        for _ in 0..7 {
            s.handle(Event::Key(KeyEvent::new(
                KeyCode::Right,
                KeyModifiers::NONE,
            )));
        }
        let view = s.view().clone();
        action(&mut s, command);
        for value in ["region", "", "region"] {
            argument(&mut s, value);
        }
        assert!(s.echo().contains("Exported Org document"), "{}", s.echo());
        let output = std::fs::read_to_string(f.0.join("region")).unwrap();
        assert!(output.contains("region界"));
        assert!(
            !output.contains("tail") && !output.contains("Child") && !output.contains("Hidden")
        );
        assert_eq!(s.view(), &view);
        assert_eq!(text(&s), source);
        assert!(s.buffer().unwrap().history().is_empty());
    }
}
