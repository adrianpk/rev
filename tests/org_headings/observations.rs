use super::*;

// Native literal ownership excludes star-like lines, escaped examples, lists
// and ordinary operators. Source/example content uses native comma escaping,
// as the source-block editor does; the frame never guesses structural ownership.
#[test]
fn literal_contexts() {
    let source = "* Real\n#+begin_src text\n,** Literal\n,* Escaped\n#+end_src\n#+begin_example\n,*** Example\n#+end_example\n- * item\na * b\n****No separator\n";
    let (_fixture, mut session) = editor("syntax.enabled=false", source);
    settle(&mut session);
    let frame = session.frame().unwrap();
    assert!(body(&frame, 0).starts_with("◉ Real"));
    for (row, line) in source.lines().enumerate().skip(1) {
        assert!(
            body(&frame, row).starts_with(line),
            "row {row}: {:?}",
            body(&frame, row)
        );
    }
    assert_eq!(text(session.buffer().unwrap()), source);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Absent, byte-limited, node-limited and canceled native observations retain
// readable source. Only complete current evidence may authorize decoration.
#[test]
fn partial_evidence() {
    for (name, limits, cancelled) in [
        ("bytes", Limits::new(4, 200, 20).unwrap(), false),
        ("nodes", Limits::new(4096, 1, 20).unwrap(), false),
        ("cancelled", Limits::default(), true),
    ] {
        let (_fixture, mut buffers, view) = setup("", "* Root\n** Child\n");
        let buffer = buffers.get_mut(view.point.stamp.buffer).unwrap();
        let plain = Frame::project(buffer, &view, SIZE, "").unwrap();
        assert!(body(&plain, 0).starts_with("* Root"));
        let cancel = Cancellation::default();
        if cancelled {
            cancel.cancel();
        }
        let coloring = ColorScan::new(buffer, limits).unwrap().run(&cancel);
        assert_ne!(
            coloring.highlights().coverage(),
            Coverage::Complete,
            "{name}"
        );
        buffer.install_org_coloring(coloring).unwrap();
        let partial = Frame::project(buffer, &view, SIZE, "").unwrap();
        assert!(body(&partial, 0).starts_with("* Root"), "{name}");
        assert!(body(&partial, 1).starts_with("** Child"), "{name}");
        assert_eq!(observe(buffer, Limits::default()), Coverage::Complete);
        let current = Frame::project(buffer, &view, SIZE, "").unwrap();
        assert!(body(&current, 0).starts_with("◉ Root"), "{name}");
        assert!(body(&current, 1).starts_with("○ Child"), "{name}");
    }
}
