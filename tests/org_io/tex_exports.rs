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
    let path = f.write("source.org", source);
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
// Handwritten TeX verifies the complete envelope and exact escaping without
// asking an optional compiler to stand in for native renderer semantics.
#[test]
fn tex_export_document() {
    let source = "#+TITLE: Notes & Plans\n#+AUTHOR: Writer\n#+OPTIONS: toc:nil num:nil\n* Heading\nBody *bold* and 50% #1.\n";
    let expected = "\\documentclass{article}\n\\usepackage[T1]{fontenc}\n\\usepackage[utf8]{inputenc}\n\\usepackage{textcomp}\n\\usepackage{amsmath}\n\\usepackage{hyperref}\n\\title{Notes \\& Plans}\n\\author{Writer}\n\\date{}\n\\begin{document}\n\\maketitle\n\\section*{Heading}\nBody \\textbf{bold} and 50\\% \\#1.\n\n\n\\end{document}\n";
    assert_eq!(rendered(source, B::Latex).unwrap(), expected);
}
// Frame-level declarations separate structural sections from slides and nested
// blocks. Source end delimiters remain escaped literal data inside the frame.
#[test]
fn tex_export_beamer() {
    let source = "#+OPTIONS: H:2 toc:nil num:nil\n* Group\n** Slide\nText\n*** Detail\n#+begin_src sh\n\\end{frame}\ntouch marker\n#+end_src\n";
    let output = rendered(source, B::Beamer).unwrap();
    for expected in [
        "\\documentclass{beamer}",
        "\\section*{Group}",
        "\\begin{frame}[fragile]{Slide}",
        "\\begin{block}{Detail}",
        "\\textbackslash{}end\\{frame\\}",
        "\\end{block}\n\\end{frame}",
    ] {
        assert!(output.contains(expected), "{expected}: {output}");
    }
    assert_eq!(output.matches("\\begin{frame}").count(), 1);
    assert_eq!(output.matches("\\end{frame}").count(), 1);
    assert!(rendered("#+OPTIONS: H:0\n* Slide\n", B::Beamer).is_err());
}
// Math is recognized before entity conversion, while literal code and unknown
// TeX controls cannot escape into a math fragment or compiler input directive.
#[test]
fn tex_export_math() {
    let source = "#+OPTIONS: toc:nil num:nil\nInline $x_1^2 + \\alpha$ and \\(\\frac{a}{b}\\).\n\n$$\\sum_{i=1}^{3} i$$\n\n~$literal$~ and $\\input{secret}$ and $^^5cinput{secret}$.\n";
    let output = rendered(source, B::Latex).unwrap();
    for expected in [
        "\\(x_1^2 + \\alpha\\)",
        "\\(\\frac{a}{b}\\)",
        "\\[\\sum_{i=1}^{3} i\\]",
        "\\texttt{\\$literal\\$}",
        "\\textbackslash{}input\\{secret\\}",
    ] {
        assert!(output.contains(expected), "{expected}: {output}");
    }
    assert!(!output.contains("\\input{secret}"));
    assert!(!output.contains("^^5c"));
}
// Lists retain their numeric start, table cells escape TeX delimiters, and notes
// retain bodies and references. Missing definitions reject the whole output.
#[test]
fn tex_export_data() {
    let source = "#+OPTIONS: toc:nil num:nil\n3. one\n4. two\n\n| Key | Value |\n|-----+-------|\n| A&B | 2_3 |\n\nText [fn:n] [fn:n] [fn::inline *body*].\n\n[fn:n] A *note*.\n";
    let output = rendered(source, B::Latex).unwrap();
    for expected in [
        "\\setcounter{enumi}{2}",
        "\\begin{tabular}{ll}",
        "A\\&B & 2\\_3",
        "\\footnote{ A \\textbf{note}.",
        "\\footnotemark[1]",
        "\\footnote{inline \\textbf{body}}",
    ] {
        assert!(output.contains(expected), "{expected}: {output}");
    }
    assert!(rendered("Missing [fn:x].\n", B::Latex).is_err());
}

// Read-only subtree export rebases headings and keeps unrelated metadata out of
// the selected body. Rendering cancellation and small budgets reject atomically.
#[test]
fn tex_export_selection_bounds() {
    let f = Fixture::new();
    let source = "#+TITLE: Hidden\n* Parent\n** TODO Child :work:\nbody\n** Sibling\nkeep\n";
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
    action(&mut s, C::ExportBeamer);
    for value in [
        "subtree",
        "toc:nil num:nil title:nil author:nil todo:nil tags:nil",
        "child.tex",
    ] {
        argument(&mut s, value);
    }
    let output = std::fs::read_to_string(f.0.join("child.tex")).unwrap();
    assert!(
        output.contains("\\begin{frame}[fragile]{Child}"),
        "{output}"
    );
    for hidden in ["Hidden", "Parent", "Sibling", "TODO", ":work:"] {
        assert!(!output.contains(hidden), "{hidden}");
    }
    assert_eq!(s.view(), &view);
    assert_eq!(text(&s), source);
    assert!(s.buffer().unwrap().history().is_empty());
    let mut policy = Org::default();
    policy.limits.export_bytes = 100;
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let snapshot = Work::new(
        &buffers,
        id,
        Request {
            backend: B::Latex,
            scope: Scope::Buffer,
            options: None,
            time: clock(&policy.time).unwrap(),
        },
        &policy,
    )
    .unwrap()
    .run(&Cancellation::default())
    .unwrap();
    assert!(snapshot.render(&Cancellation::default()).is_err());
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(snapshot.render(&cancel).is_err());
}
