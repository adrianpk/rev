use super::habits::{SOURCE, clock};
use super::*;
use rev::command::{Scroll, Split};
use rev::org::time::{ZoneEnvironment, ZoneSelection, ZonedOperation};
use std::sync::atomic::{AtomicUsize, Ordering};
use unicode_width::UnicodeWidthStr;
fn visible(s: &mut Session, needle: &str) -> bool {
    s.frame()
        .unwrap()
        .rows()
        .iter()
        .any(|r| r.text().contains(needle))
}
fn marks(s: &mut Session) -> usize {
    s.frame()
        .unwrap()
        .rows()
        .iter()
        .filter(|r| r.band() == rev::display::Band::Document)
        .map(|r| r.text().matches('*').count())
        .sum()
}

// The read-only graph occupies one selected pane; peers, exact BOM/CRLF source
// and native cursor/cell bounds survive narrow or suspended rendering sizes.
#[test]
fn habit_narrow_panes() {
    let source = format!(
        "\u{feff}{}",
        SOURCE
            .replace("Due habit", "Due 界e\u{301} habit")
            .replace('\n', "\r\n")
    );
    let (f, session) = editor("", "notes.org", &source);
    let mut s = session.with_org_clock(clock);
    s.execute(Command::Split(Split::Right));
    let peer = s
        .windows()
        .order()
        .iter()
        .copied()
        .find(|id| *id != s.windows().selected_id())
        .unwrap();
    let peer_view = s.windows().get(peer).unwrap().clone();
    let before = text(&s);
    action(&mut s, OrgCommand::HabitGraph);
    for (width, height) in [(120, 12), (25, 9), (8, 4), (1, 1), (100, 12)] {
        s.handle(Event::Resize(width, height));
        let frame = s.frame().unwrap();
        for row in frame.rows() {
            assert_eq!(row.text().width(), usize::from(width));
        }
        if let Some(cursor) = frame.cursor() {
            assert!(cursor.column < width && cursor.row < height)
        }
    }
    assert_eq!(s.windows().get(peer).unwrap().point, peer_view.point);
    s.handle(Event::Mouse(crossterm::event::MouseEvent {
        kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
        column: 1,
        row: 2,
        modifiers: KeyModifiers::NONE,
    }));
    s.handle(Event::Paste("blocked".into()));
    s.execute(Command::BackwardDeleteChar);
    assert_eq!(text(&s), before);
    assert!(s.buffer().unwrap().history().is_empty());
    assert!(!s.buffer().unwrap().is_modified());
    assert_eq!(
        std::fs::read(f.0.join("notes.org")).unwrap(),
        source.as_bytes()
    );
}
static CAPTURES: AtomicUsize = AtomicUsize::new(0);
fn advancing(_: &rev::config::Time) -> Result<ZonedOperation, String> {
    let n = CAPTURES.fetch_add(1, Ordering::SeqCst);
    ZonedOperation::capture(
        &ZoneSelection::Utc,
        &ZoneEnvironment {
            localtime: "unused".into(),
            zoneinfo: "unused".into(),
            tz: None,
        },
        || Ok(std::time::UNIX_EPOCH + std::time::Duration::from_secs(1709303400 + n as u64 * 86400)),
    )
}
// Only explicit refresh captures a new day or completion observation. Dirty
// source is not autosaved; repeated paints own no clock reads or polling loop.
#[test]
fn habit_dirty_idle() {
    let (_f, session) = editor("org.habit.show_all_today=true", "notes.org", SOURCE);
    let mut s = session.with_org_clock(advancing);
    action(&mut s, OrgCommand::HabitGraph);
    let initial = marks(&mut s);
    assert_eq!(CAPTURES.load(Ordering::SeqCst), 1);
    let end = s.buffer().unwrap().len_chars();
    s.replace(
        end..end,
        "- State \"DONE\" from \"TODO\" [2024-03-01 Fri 14:00]\n",
    )
    .unwrap();
    settle(&mut s);
    assert_eq!(marks(&mut s), initial);
    assert!(visible(&mut s, "stale: g refresh"));
    let source = text(&s);
    let history = s.buffer().unwrap().history().len();
    let view = s.view().clone();
    for _ in 0..5 {
        s.frame().unwrap();
        assert_eq!(s.remaining_input(Instant::now()), None);
    }
    assert_eq!(CAPTURES.load(Ordering::SeqCst), 1);
    s.handle(key(KeyCode::Char('g')));
    settle(&mut s);
    assert_eq!(CAPTURES.load(Ordering::SeqCst), 2);
    assert!(visible(&mut s, "Habit agenda 2024-03-02"));
    assert_eq!(marks(&mut s), initial + 1);
    assert_eq!(text(&s), source);
    assert_eq!(s.buffer().unwrap().history().len(), history);
    assert_eq!(s.view(), &view);
}
// Color-independent glyphs use the actual native writer in an isolated process,
// preserving the shared test binary's color cache and source/worker boundaries.
#[test]
fn habit_no_color() {
    if std::env::var_os("REV_HABIT_COLOR_CHILD").is_none() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "habit_views::habit_no_color", "--nocapture"])
            .env("REV_HABIT_COLOR_CHILD", "1")
            .env("NO_COLOR", "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let (_f, session) = editor("", "notes.org", SOURCE);
    let mut s = session.with_org_clock(clock);
    action(&mut s, OrgCommand::HabitGraph);
    let frame = s.frame().unwrap();
    let mut bytes = Vec::new();
    frame.write(&mut bytes).unwrap();
    let output = String::from_utf8(bytes).unwrap();
    assert!(!output.contains("38;2;") && !output.contains("48;2;"));
    assert!(
        frame
            .rows()
            .iter()
            .any(|r| r.text().contains('*') && r.text().contains('!') && r.text().contains('+'))
    );
    assert_eq!(text(&s), SOURCE);
    assert!(s.buffer().unwrap().history().is_empty());
    assert_eq!(s.remaining_input(Instant::now()), None);
    s.handle(control('g'));
    action(&mut s, OrgCommand::Agenda);
    argument(&mut s, "current");
    settle(&mut s);
    let frame = s.frame().unwrap();
    let mut bytes = Vec::new();
    frame.write(&mut bytes).unwrap();
    let output = String::from_utf8(bytes).unwrap();
    assert!(!output.contains("38;2;") && !output.contains("48;2;"));
    assert!(frame.rows().iter().any(|r| r.text().contains("Org agenda")));
    assert_eq!(text(&s), SOURCE);
    assert!(s.buffer().unwrap().history().is_empty());
    assert_eq!(s.remaining_input(Instant::now()), None);
    s.handle(key(KeyCode::Char('q')));
    action(&mut s, OrgCommand::TodoList);
    argument(&mut s, "");
    argument(&mut s, "current");
    settle(&mut s);
    let frame = s.frame().unwrap();
    let mut bytes = Vec::new();
    frame.write(&mut bytes).unwrap();
    let output = String::from_utf8(bytes).unwrap();
    assert!(!output.contains("38;2;") && !output.contains("48;2;"));
    assert!(
        frame
            .rows()
            .iter()
            .any(|r| r.text().contains("Org TODO agenda"))
    );
    action(&mut s, OrgCommand::AgendaColumns);
    argument(&mut s, "ITEM,TODO");
    settle(&mut s);
    let frame = s.frame().unwrap();
    let mut bytes = Vec::new();
    frame.write(&mut bytes).unwrap();
    let output = String::from_utf8(bytes).unwrap();
    assert!(!output.contains("38;2;") && !output.contains("48;2;"));
    assert!(frame.rows().iter().any(|r| r.text().contains("ITEM=")));
    assert_eq!(text(&s), SOURCE);
    assert!(s.buffer().unwrap().history().is_empty());
}
// The selected-buffer graph row bound rejects complete excess requests; no
// silent truncation or source rewrite can turn the next row into an absent habit.
#[test]
fn habit_row_bound() {
    for (name, count, admit) in [("limit", 128, true), ("over", 129, false)] {
        let source =
            "* TODO H\nSCHEDULED: <2024-02-28 Wed .+1d>\n:PROPERTIES:\n:STYLE: habit\n:END:\n"
                .repeat(count);
        let (_f, session) = editor("", "notes.org", &source);
        let mut s = session.with_org_clock(clock);
        action(&mut s, OrgCommand::HabitGraph);
        assert_eq!(
            visible(&mut s, "Habit agenda 2024-03-01"),
            admit,
            "{name}: {}",
            s.echo()
        );
        if !admit {
            assert!(s.echo().contains("128 graph rows"), "{}", s.echo());
        }
        assert_eq!(text(&s), source);
        assert!(s.buffer().unwrap().history().is_empty());
        s.execute(Command::Scroll(Scroll::NextLine));
    }
}
// A source change before worker publication revokes the original request. The
// display never installs rows whose retained source/view authority is stale.
#[test]
fn habit_stale_request() {
    let (_f, session) = editor("", "notes.org", SOURCE);
    let mut s = session.with_org_clock(clock);
    s.execute(Command::Org(OrgCommand::HabitGraph));
    s.replace(0..0, "# external\n").unwrap();
    settle(&mut s);
    assert!(!visible(&mut s, "Habit agenda"));
    assert_eq!(text(&s), format!("# external\n{SOURCE}"));
    assert_eq!(s.buffer().unwrap().history().len(), 1);
}
