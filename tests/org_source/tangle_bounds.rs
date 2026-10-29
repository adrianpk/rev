use super::tangling::tangle;
use super::*;
use rev::org::{
    Cancellation,
    tangle::{Selection, Work},
};
fn chain(count: usize) -> String {
    let mut source = String::new();
    for n in 0..count {
        let header = if n == 0 {
            " :tangle /example/a :noweb yes"
        } else {
            ""
        };
        let body = if n + 1 == count {
            "leaf".into()
        } else {
            format!("<<n{}>>", n + 1)
        };
        source.push_str(&format!(
            "#+NAME: n{n}\n#+BEGIN_SRC text{header}\n{body}\n#+END_SRC\n"
        ));
    }
    source
}
// The last admitted reference depth succeeds, while one extra dependency fails
// rather than emitting a partial file. Dense acyclic expansion also has a budget.
#[test]
fn tangle_graph_bounds() {
    assert_eq!(tangle(&chain(64)).unwrap().targets()[0].text(), "leaf\n");
    assert!(tangle(&chain(65)).err().unwrap().contains("64 levels"));
    let mut source = "#+NAME: n0\n#+BEGIN_SRC text\nx\n#+END_SRC\n".to_owned();
    for n in 1..16 {
        source.push_str(&format!(
            "#+NAME: n{n}\n#+BEGIN_SRC text\n<<n{}>>\n<<n{}>>\n#+END_SRC\n",
            n - 1,
            n - 1
        ));
    }
    source.push_str("#+BEGIN_SRC text :tangle /example/a :noweb yes\n<<n15>>\n#+END_SRC\n");
    assert!(tangle(&source).err().unwrap().contains("100000 steps"));
}
// File and source-block ceilings admit their exact boundary and reject the next
// item before returning any publication plan.
#[test]
fn tangle_count_bounds() {
    for (kind, count, expected) in [("files", 256, "256 files"), ("blocks", 512, "512 blocks")] {
        let mut source = String::new();
        for n in 0..count {
            let target = if kind == "files" || n == 0 {
                format!(" :tangle /example/{n}")
            } else {
                "".into()
            };
            source.push_str(&format!("#+BEGIN_SRC text{target}\nx\n#+END_SRC\n"));
        }
        assert!(tangle(&source).is_ok(), "{kind}");
        source.push_str("#+BEGIN_SRC text :tangle /example/extra\nx\n#+END_SRC\n");
        assert!(tangle(&source).err().unwrap().contains(expected), "{kind}");
    }
}
// Reusing a large fragment can exceed combined output without exceeding source
// bytes. Exact 8 MiB output passes; the next file rejects the whole plan.
#[test]
fn tangle_output_bound() {
    let chunk = "x".repeat(4 * 1024 * 1024 - 1);
    let mut source = format!("#+NAME: chunk\n#+BEGIN_SRC text\n{chunk}\n#+END_SRC\n");
    for target in ["one", "two"] {
        source.push_str(&format!(
            "#+BEGIN_SRC text :tangle /example/{target} :noweb yes\n<<chunk>>\n#+END_SRC\n"
        ));
    }
    let plan = tangle(&source).unwrap();
    assert_eq!(
        plan.targets().iter().map(|t| t.text().len()).sum::<usize>(),
        8 * 1024 * 1024
    );
    source.push_str("#+BEGIN_SRC text :tangle /example/three\nx\n#+END_SRC\n");
    assert!(tangle(&source).err().unwrap().contains("8 MiB"));
}
// A plan's source stamp cannot be reused after an ordinary canonical edit.
#[test]
fn tangle_source_revision() {
    let source = "#+BEGIN_SRC text :tangle /example/a\nx\n#+END_SRC\n";
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let b = buffers.get_mut(id).unwrap();
    let plan = Work::new(b, Selection::Buffer)
        .unwrap()
        .run(&Cancellation::default())
        .unwrap();
    assert!(plan.current(b).is_ok());
    let edit = Replacement {
        source: b.span(0..0).unwrap(),
        text: "note\n".into(),
    };
    let prepared = b.prepare(&[edit], &[], None).unwrap();
    b.publish(prepared, &mut []).unwrap();
    assert!(plan.current(b).is_err());
}

// Unsupported transformations fail explicitly; no-expand admits literal code
// with unevaluated variables, and inherited native controls still compose.
#[test]
fn tangle_header_controls() {
    for header in [
        ":comments link",
        ":shebang \"#!/bin/sh\"",
        ":tangle-mode (identity #o755)",
        ":noweb sometimes",
        ":mkdirp maybe",
        ":padline maybe",
    ] {
        let source = format!("#+BEGIN_SRC sh :tangle /example/a {header}\nraw\n#+END_SRC\n");
        assert!(tangle(&source).is_err(), "{header}");
    }
    let source = "#+PROPERTY: header-args:text :tangle /example/a :padline no\n#+BEGIN_SRC text :var x=(call) :no-expand yes :noweb yes\n<<call()>>\n#+END_SRC\n#+BEGIN_SRC text\nlast\n#+END_SRC\n";
    assert_eq!(
        tangle(source).unwrap().targets()[0].text(),
        "<<call()>>\nlast\n"
    );
}
