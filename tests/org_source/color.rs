use rev::buffer::{Buffer, Buffers, Mode};
use rev::org::{Cancellation, ColorScan, Coverage, Limit, Limits, Role};
use rev::syntax::{Capture, Style};

pub(super) fn paint(buffer: &mut Buffer) {
    let colors = ColorScan::new(buffer, Limits::default())
        .unwrap()
        .run(&Cancellation::default());
    assert_eq!(colors.highlights().coverage(), Coverage::Complete);
    buffer.install_org_coloring(colors).unwrap();
}

fn roles(buffer: &Buffer) -> Vec<Capture> {
    buffer.org_captures(0..buffer.len_bytes())
}

// Styles use native ownership, including opaque bodies and closing delimiters;
// fake Org syntax inside a literal block cannot create heading or inline colors.
#[test]
fn source_roles() {
    let source = "* Heading *bold*\nSCHEDULED: <2026-10-28 Wed>\n:PROPERTIES:\n:ID: kept\n:END:\n#+TITLE: Sample\n- item\n| cell |\n#+TBLFM: $1=2\ntext [[file:a][link]] <<target>> [fn:x] \\alpha ~code~\n#+begin_src rust\n,* Fake [[file:b]]\n#+end_src\n# comment\n";
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.select_mode(Mode::Org).unwrap();
    let stamp = buffer.stamp();
    paint(buffer);
    let captures = roles(buffer);
    for (token, role) in [
        ("* Heading *bold*\n", Role::Heading),
        ("*bold*", Role::Emphasis),
        ("SCHEDULED: <2026-10-28 Wed>\n", Role::Planning),
        (":PROPERTIES:\n", Role::Drawer),
        (":ID: kept\n", Role::Property),
        ("#+TITLE: Sample\n", Role::Keyword),
        ("| cell |\n", Role::Table),
        ("#+TBLFM: $1=2\n", Role::Formula),
        ("[[file:a][link]]", Role::Link),
        ("<<target>>", Role::Target),
        ("[fn:x]", Role::Footnote),
        ("\\alpha", Role::Entity),
        ("~code~", Role::Code),
        ("#+begin_src rust\n", Role::Block),
        ("# comment\n", Role::Comment),
    ] {
        assert!(
            captures
                .iter()
                .any(|c| c.style == Style::Org(role) && &source[c.bytes.clone()] == token),
            "{role:?}: {captures:?}"
        );
    }
    let fake = source.find(",* Fake").unwrap();
    assert!(
        captures
            .iter()
            .filter(|c| c.bytes.contains(&fake))
            .all(|c| c.style == Style::Org(Role::Code))
    );
    assert_eq!(buffer.stamp(), stamp);
    assert_eq!(super::bytes(buffer), source.as_bytes());
    assert!(buffer.history().is_empty());
    assert!(!buffer.is_modified());
}

// An installed observation is never silently rebased after an edit or installed
// onto another buffer. Fresh current styles remain available after rejection.
#[test]
fn stale_styles() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("* One\nbody\n").unwrap();
    let other = buffers.create("* One\nbody\n").unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.select_mode(Mode::Org).unwrap();
    paint(buffer);
    let old = ColorScan::new(buffer, Limits::default()).unwrap();
    super::incremental::replace(buffer, 2..5, "Two");
    assert!(roles(buffer).is_empty());
    let stale = old.run(&Cancellation::default());
    paint(buffer);
    assert!(buffer.install_org_coloring(stale).is_err());
    assert!(!roles(buffer).is_empty());
    let foreign = ColorScan::new(buffer, Limits::default())
        .unwrap()
        .run(&Cancellation::default());
    assert!(
        buffers
            .get_mut(other)
            .unwrap()
            .install_org_coloring(foreign)
            .is_err()
    );
}

// Index reuse saves the unaffected prefix while grammar is re-observed from the
// snapshot: a tail edit can change the meaning of an earlier block delimiter.
#[test]
fn suffix_recovery() {
    let source = "* Before\nuntouched\n* Root\nintro\n#+begin_example\n,* Fake\n#+end_example\n* Tail\nlast\n";
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.select_mode(Mode::Org).unwrap();
    paint(buffer);
    let start = source.find(",* Fake").unwrap();
    super::incremental::replace(buffer, start..start + 1, "");
    let retained = buffer.take_org_index().unwrap();
    let colors = ColorScan::resume(retained, buffer)
        .unwrap()
        .run(&Cancellation::default());
    assert!(colors.highlights().reindexed_bytes() < buffer.len_bytes());
    let fresh = ColorScan::new(buffer, Limits::default())
        .unwrap()
        .run(&Cancellation::default());
    assert_eq!(
        colors.highlights().captures(buffer, 0..buffer.len_bytes()),
        fresh.highlights().captures(buffer, 0..buffer.len_bytes())
    );
    buffer.install_org_coloring(colors).unwrap();
    assert!(
        roles(buffer)
            .iter()
            .any(|c| c.style == Style::Org(Role::Malformed))
    );
}

// Finite source/capture bounds and cancellation expose incomplete coverage;
// no partial UTF-8 style can escape, and source bytes remain intact.
#[test]
fn finite_colors() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("* 界 *bold*\nbody\n").unwrap();
    let buffer = buffers.get(id).unwrap();
    for (name, limits, coverage) in [
        (
            "bytes",
            Limits::new(4, 200_000, 256).unwrap(),
            Coverage::Incomplete(Limit::AnalysisBytes),
        ),
        (
            "nodes",
            Limits::new(8_388_608, 1, 256).unwrap(),
            Coverage::Incomplete(Limit::SourceNodes),
        ),
    ] {
        let colors = ColorScan::new(buffer, limits)
            .unwrap()
            .run(&Cancellation::default());
        assert_eq!(colors.highlights().coverage(), coverage, "{name}");
        for c in colors.highlights().captures(buffer, 0..buffer.len_bytes()) {
            assert!("* 界 *bold*\nbody\n".is_char_boundary(c.bytes.start));
            assert!("* 界 *bold*\nbody\n".is_char_boundary(c.bytes.end));
        }
    }
    let cancel = Cancellation::default();
    cancel.cancel();
    let colors = ColorScan::new(buffer, Limits::default())
        .unwrap()
        .run(&cancel);
    assert_eq!(colors.highlights().coverage(), Coverage::Cancelled);
}

// All literal block kinds color original body bytes as one opaque semantic role,
// including fake metadata, links and comma quoting. Read-only is not an obstacle
// to derived presentation and never becomes authority to edit canonical source.
#[test]
fn literal_colors() {
    let mut buffers = Buffers::new().unwrap();
    for (name, opening, closing, role) in [
        ("source", "src rust", "src", Role::Code),
        ("example", "example", "example", Role::Code),
        ("export", "export html", "export", Role::Code),
        ("comment", "comment", "comment", Role::Comment),
    ] {
        let body = ",* Fake\n:PROPERTIES:\n[[file:a]] *bold*\n";
        let source = format!("* Root\n#+begin_{opening}\n{body}#+end_{closing}\n* Tail\n");
        let id = buffers.create(&source).unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        buffer.select_mode(Mode::Org).unwrap();
        buffer.set_readonly(true);
        let stamp = buffer.stamp();
        paint(buffer);
        let start = source.find(body).unwrap();
        let captures = buffer.org_captures(start..start + body.len());
        assert_eq!(
            captures,
            [Capture {
                bytes: start..start + body.len(),
                style: Style::Org(role)
            }],
            "{name}"
        );
        assert_eq!(super::bytes(buffer), source.as_bytes(), "{name}");
        assert_eq!(buffer.stamp(), stamp, "{name}");
        assert!(
            buffer.is_readonly() && buffer.history().is_empty(),
            "{name}"
        );
    }
}
