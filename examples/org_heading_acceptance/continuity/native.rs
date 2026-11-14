use super::fixture::{self, Case, Colors, Format};
use crate::support::{self, Result};
use rev::command::{Command, History, Motion};
use rev::session::Session;
use std::path::Path;
use std::time::{Duration, Instant};

pub(super) fn run(root: &Path, format: Format) -> Result<()> {
    let started = Instant::now();
    refusals(root, format)?;
    for case in fixture::cases(format) {
        document(root, case, 0, 3)?;
    }
    for profile in ["non-modal", "modal"] {
        let case = Case {
            format,
            profile,
            indent: true,
            pretty: true,
            colors: Colors::Enabled,
        };
        for (bytes, headings) in [(50_000, 3), (500_000, 3), (0, 1003), (0, 3003)] {
            document(root, case, bytes, headings)?;
        }
        fidelity(root, case)?;
    }
    println!(
        "PASS continuity native {format:?}: elapsed_ms={}; all policy/color/profile states, 5 keys <=180ms, withheld publication, exact completion and source/history restoration",
        started.elapsed().as_millis()
    );
    Ok(())
}

fn document(root: &Path, case: Case, bytes: usize, headings: usize) -> Result<()> {
    let source = case.format.document(bytes, headings);
    for pattern in [Pattern::Body, Pattern::Title, Pattern::TwoRows] {
        let (mut editor, settings) = case.open(root, &source)?;
        burst(&mut editor, case, &settings, pattern)?;
        restore(&mut editor, &source)?;
    }
    println!(
        "MEASURE native document {case:?}: bytes={} lines={} headings={headings}",
        source.len(),
        source.lines().count()
    );
    Ok(())
}

#[derive(Clone, Copy, Debug)]
enum Pattern {
    Body,
    Title,
    TwoRows,
}

impl Pattern {
    fn row(self, key: usize) -> usize {
        match self {
            Self::Body => 4,
            Self::Title => 3,
            Self::TwoRows => 3 + key % 2,
        }
    }
}

fn place(editor: &mut Session, row: usize) {
    editor.execute(Command::GotoLine);
    editor.handle(crossterm::event::Event::Paste(row.to_string()));
    support::key(editor, crossterm::event::KeyCode::Enter);
    editor.execute(Command::Motion(Motion::LineEnd));
}

fn burst(
    editor: &mut Session,
    case: Case,
    settings: &rev::config::Snapshot,
    pattern: Pattern,
) -> Result<()> {
    let baseline = fixture::emitted(editor, case, settings)?;
    place(editor, pattern.row(0));
    let started = Instant::now();
    let mut states = Vec::new();
    for key in 0..5 {
        insert(editor, case, pattern, key)?;
        fixture::admitted(editor)?;
        let samples = fixture::emitted(editor, case, settings)?;
        assert_eq!(samples, baseline);
        states.push((
            editor.buffer()?.stamp().revision,
            editor.remaining_input(Instant::now()).is_some(),
            samples,
        ));
    }
    let elapsed = started.elapsed();
    assert!(
        elapsed <= Duration::from_millis(180),
        "five native keys took {elapsed:?}"
    );
    assert_eq!(
        editor.remaining_input(Instant::now()).is_some(),
        case.owns_work()
    );
    complete(editor, case)?;
    assert_eq!(fixture::emitted(editor, case, settings)?, baseline);
    assert!(editor.remaining_input(Instant::now()).is_none());
    println!(
        "MEASURE native burst {case:?} {pattern:?}: key_count=5 elapsed_us={} accepted=5 rejected=0 source_bytes={} observations={states:?}; owned publication requested={}, worker CPU completion not inferred",
        elapsed.as_micros(),
        editor.buffer()?.len_bytes(),
        case.owns_work()
    );
    Ok(())
}

fn insert(editor: &mut Session, case: Case, pattern: Pattern, key: usize) -> Result<()> {
    if key > 0 && matches!(pattern, Pattern::TwoRows) {
        place(editor, pattern.row(key));
    }
    editor.insert('z');
    if key == 0 {
        editor.frame()?;
        assert_eq!(
            editor.remaining_input(Instant::now()).is_some(),
            case.owns_work(),
            "existing observation slot follows active consumers"
        );
    }
    current_source(editor, pattern.row(key) - 1)?;
    Ok(())
}

fn restore(editor: &mut Session, source: &str) -> Result<()> {
    let groups = editor.buffer()?.history().len();
    assert!(groups > 0 && groups <= 5);
    for _ in 0..groups {
        editor.execute(Command::History(History::Undo));
    }
    assert_eq!(support::source(editor), source);
    support::settle(editor)?;
    assert!(!editor.buffer()?.is_modified());
    assert!(editor.buffer()?.history().is_empty());
    Ok(())
}

fn fidelity(root: &Path, case: Case) -> Result<()> {
    let source = format!(
        "\u{feff}{}",
        case.format.document(0, 3).replace('\n', "\r\n")
    );
    let (mut editor, settings) = case.open(root, &source)?;
    place(&mut editor, 4);
    unicode(&mut editor, case.profile);
    fixture::emitted(&editor, case, &settings)?;
    let expected = source.replacen("Child body.", "Child body.界e\u{301}", 1);
    editor.execute(Command::SaveBuffer);
    assert_eq!(
        std::fs::read(root.join(format!("workspace/continuity.{}", case.format.name())))?,
        expected.as_bytes()
    );
    assert_eq!(editor.buffer()?.history().len(), 1);
    editor.execute(Command::History(History::Undo));
    assert_eq!(
        support::source(&editor),
        source.trim_start_matches('\u{feff}')
    );
    editor.execute(Command::SaveBuffer);
    assert_eq!(
        std::fs::read(root.join(format!("workspace/continuity.{}", case.format.name())))?,
        source.as_bytes()
    );
    editor.execute(Command::Motion(Motion::BufferStart));
    editor.execute(Command::Region(rev::command::RegionCommand::Set));
    editor.execute(Command::Motion(Motion::ForwardChar));
    editor.execute(Command::Linear(rev::command::LinearCommand::Copy));
    assert_eq!(
        editor.killed_text().unwrap().to_string(),
        case.format.marker().to_string()
    );
    println!(
        "PASS native fidelity {case:?}: Unicode BOM/CRLF save, one-action undo and canonical marker copy"
    );
    Ok(())
}

fn refusals(root: &Path, format: Format) -> Result<()> {
    let case = Case {
        format,
        profile: "non-modal",
        indent: true,
        pretty: true,
        colors: Colors::Enabled,
    };
    let extra = if format == Format::Org {
        "#+begin_example\nliteral\n#+end_example\n#+macro: ref definition\n"
    } else {
        "```text\nliteral\n```\n\n[ref]: /definition\n"
    };
    let source = format!("{}{extra}", format.document(0, 3));
    let marker = format.marker().to_string();
    let prefix = format!("{marker} Root");
    let bare = format!("{marker}Root");
    for change in [
        (&marker[..], ""),
        (&prefix, &bare),
        ("Child body.", "Child body.\nextra"),
        ("literal", "literalz"),
        ("definition", "definitionz"),
    ] {
        rejection(root, case, &source, change)?;
    }
    println!(
        "MEASURE native refusal {format:?}: accepted=0 rejected=5; prefix, separator, multiline, literal block and definition; source and one-step undo preserved"
    );
    Ok(())
}

fn rejection(root: &Path, case: Case, source: &str, change: (&str, &str)) -> Result<()> {
    let (mut editor, _) = case.open(root, source)?;
    let at = source.find(change.0).unwrap();
    editor.replace(at..at + change.0.len(), change.1)?;
    let frame = rev::display::Frame::project(editor.buffer()?, editor.view(), fixture::SIZE, "")?;
    assert!(frame.rows().iter().any(|row| {
        row.text()
            .contains(&format!("{0}{0} Child", case.format.marker()))
    }));
    assert_eq!(
        support::source(&editor),
        source.replacen(change.0, change.1, 1)
    );
    assert_eq!(editor.buffer()?.history().len(), 1);
    editor.execute(Command::History(History::Undo));
    assert_eq!(support::source(&editor), source);
    Ok(())
}

fn unicode(editor: &mut Session, profile: &str) {
    if profile == "modal" {
        support::key(editor, crossterm::event::KeyCode::Char('A'));
    }
    editor.handle(crossterm::event::Event::Paste("界e\u{301}".into()));
    if profile == "modal" {
        support::key(editor, crossterm::event::KeyCode::Esc);
    }
}

fn current_source(editor: &Session, row: usize) -> Result<()> {
    use rev::coordinates::{Cell, Coordinates};
    let buffer = editor.buffer()?;
    let coordinates = Coordinates::new(buffer)?;
    let start = coordinates.from_cell(Cell { row, column: 0 })?;
    let end = coordinates.line_end(start)?;
    let text = buffer.slice(&buffer.span(start.0..end.0)?)?.to_string();
    let frame = rev::display::Frame::project(buffer, editor.view(), fixture::SIZE, "")?;
    assert!(
        frame.rows()[row]
            .text()
            .contains(text.trim_start_matches(['*', '#', ' '])),
        "new source must be visible immediately"
    );
    Ok(())
}

// Source/disabled-color Org views request no automatic color/pretty work. Use
// the existing explicit control to finish exact analysis, then restore policy.
fn complete(editor: &mut Session, case: Case) -> Result<()> {
    use rev::command::OrgCommand;
    if !case.owns_work() {
        editor.execute(Command::Org(OrgCommand::PrettyHeadings));
        editor.frame()?;
        assert!(editor.remaining_input(Instant::now()).is_some());
    }
    support::settle(editor)?;
    if !case.owns_work() {
        editor.execute(Command::Org(OrgCommand::PrettyHeadings));
    }
    fixture::admitted(editor)?;
    if case.format == Format::Org {
        let buffer = editor.buffer()?;
        let index = buffer.org_index().ok_or("missing exact Org index")?;
        assert_eq!(index.stamp(), buffer.stamp());
        assert_eq!(index.coverage(), rev::org::Coverage::Complete);
        println!(
            "MEASURE exact Org: source_bytes={} indexed_rows={} covered_bytes={}",
            buffer.len_bytes(),
            index.len(),
            index.covered_bytes().0
        );
    }
    Ok(())
}
