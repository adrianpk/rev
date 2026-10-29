use super::*;
use rev::org::{
    Cancellation,
    tangle::{Plan, Selection, Work},
};
fn tangle(source: &str) -> Result<Plan, String> {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    Work::new(buffers.get(id).unwrap(), Selection::Buffer)?.run(&Cancellation::default())
}
// Normalized code bodies expand names recursively, retaining prefix characters,
// intentional relative indentation and ordinary per-target block separation.
#[test]
fn named_tangling() {
    let source = "#+NAME: leaf\n#+BEGIN_SRC text\n  alpha\n    beta\n#+END_SRC\n#+NAME: nested\n#+BEGIN_SRC text\n<<leaf>>\n#+END_SRC\n#+BEGIN_SRC text :tangle /example/one.txt :noweb yes\nif ready:\n    <<nested>>\n#+END_SRC\n#+BEGIN_SRC text :tangle /example/one.txt\nlast\n#+END_SRC\n#+BEGIN_SRC text :tangle /example/two.txt :noweb yes\n-- <<leaf>> suffix\n#+END_SRC\n";
    let plan = tangle(source).unwrap();
    assert_eq!(plan.targets().len(), 2);
    assert_eq!(
        plan.targets()[0].text(),
        "if ready:\n    alpha\n      beta\n\nlast\n"
    );
    assert_eq!(plan.targets()[1].text(), "-- alpha\n--   beta suffix\n");
}
// noweb-ref groups retain document order, separators and explicit prefix policy;
// disabled expansion leaves code-looking references literal without evaluation.
#[test]
fn noweb_controls() {
    let source = "#+BEGIN_SRC text :noweb-ref group\nfirst\n#+END_SRC\n#+BEGIN_SRC text :noweb-ref group\nsecond\n#+END_SRC\n#+BEGIN_SRC text :tangle /example/a :noweb tangle :noweb-sep \",\"\n<<group>>\n#+END_SRC\n#+BEGIN_SRC text :tangle /example/b :noweb no\n<<missing()>>\n#+END_SRC\n#+BEGIN_SRC text :tangle /example/c :noweb yes :noweb-prefix no\n-- <<group>>\n#+END_SRC\n";
    let plan = tangle(source).unwrap();
    for (file, expected) in
        plan.targets()
            .iter()
            .zip(["first,second\n", "<<missing()>>\n", "-- first\nsecond\n"])
    {
        assert_eq!(file.text(), expected);
    }
}
// Every expansion and destination declaration is checked before a file effect;
// malformed graphs, executable forms and traversal paths cannot produce a plan.
#[test]
fn tangle_rejection() {
    for (name, source) in [
        (
            "unresolved",
            "#+BEGIN_SRC sh :tangle /example/a :noweb yes\n<<missing>>\n#+END_SRC\n",
        ),
        (
            "evaluated",
            "#+BEGIN_SRC sh :tangle /example/a :noweb yes\n<<call()>>\n#+END_SRC\n",
        ),
        (
            "cycle",
            "#+NAME: loop\n#+BEGIN_SRC sh :tangle /example/a :noweb yes\n<<loop>>\n#+END_SRC\n",
        ),
        (
            "traversal",
            "#+BEGIN_SRC sh :tangle /example/../a\nexit 99\n#+END_SRC\n",
        ),
        (
            "dynamic path",
            "#+BEGIN_SRC sh :tangle (make-path)\nexit 99\n#+END_SRC\n",
        ),
        (
            "variables",
            "#+BEGIN_SRC sh :tangle /example/a :var x=1\nexit 99\n#+END_SRC\n",
        ),
        (
            "commented",
            "* COMMENT private\n#+BEGIN_SRC sh :tangle /example/a\nexit 99\n#+END_SRC\n",
        ),
    ] {
        assert!(tangle(source).is_err(), "{name}");
    }
}
// Current-block selection does not tangle neighboring blocks. A cancelled
// observation cannot return outputs, even when the source is otherwise valid.
#[test]
fn selected_tangling() {
    let source = "#+BEGIN_SRC text :tangle /example/a\nfirst\n#+END_SRC\n#+BEGIN_SRC text :tangle /example/b\nsecond\n#+END_SRC\n";
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let b = buffers.get(id).unwrap();
    let plan = Work::new(b, Selection::Block(Byte(source.find("second").unwrap())))
        .unwrap()
        .run(&Cancellation::default())
        .unwrap();
    assert_eq!(plan.targets().len(), 1);
    assert_eq!(plan.targets()[0].text(), "second\n");
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(
        Work::new(b, Selection::Buffer)
            .unwrap()
            .run(&cancel)
            .is_err()
    );
    assert_eq!(bytes(b), source.as_bytes());
    assert!(b.history().is_empty());
}
