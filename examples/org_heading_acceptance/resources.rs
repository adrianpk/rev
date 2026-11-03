use super::support::{Result, cpu_ticks, named, open, peak_rss, settle, source};
use rev::session::Action;
use rev::session::Session;
use std::path::Path;
use std::time::{Duration, Instant};

pub(super) fn run(root: &Path) -> Result<()> {
    let mut text = String::new();
    for section in 0..512 {
        text.push_str(&format!(
            "{} Heading {section}\n",
            "*".repeat(section % 8 + 1)
        ));
        text.push_str(&"body 界 e\u{301} ordinary source remains unchanged\n".repeat(32));
    }
    let started = Instant::now();
    let mut editor = open(root, "non-modal", &text)?;
    settle(&mut editor)?;
    let startup = started.elapsed();
    assert!(startup < Duration::from_secs(5));
    assert!(editor.frame()?.rows()[0].text().contains("◉ Heading 0"));
    let projection = projections(&mut editor)?;
    settle(&mut editor)?;
    assert_eq!(source(&editor), text);
    assert!(editor.buffer()?.history().is_empty());
    let refresh = refresh(&mut editor, &text)?;
    assert!(editor.remaining_input(Instant::now()).is_none());
    assert_eq!(
        editor.expire_input(Instant::now() + Duration::from_secs(10)),
        Action::Wait
    );
    let (ticks, rss) = idle_ticks()?;
    println!(
        "MEASURE finite Org: bytes={} lines={} headings=512 startup_us={} 64_toggle_projections_us={} edit_undo_us={refresh} idle_350ms_cpu_ticks={ticks} peak_rss={}; no idle deadline, presentation adds no history, exact source restored by one undo",
        text.len(),
        text.lines().count(),
        startup.as_micros(),
        projection.as_micros(),
        rss
    );
    Ok(())
}

fn projections(editor: &mut Session) -> Result<Duration> {
    let started = Instant::now();
    for _ in 0..64 {
        named(editor, "org-pretty-headings-mode");
        editor.frame()?;
    }
    let elapsed = started.elapsed();
    assert!(elapsed < Duration::from_secs(5));
    Ok(elapsed)
}

fn refresh(editor: &mut Session, original: &str) -> Result<u128> {
    let started = Instant::now();
    editor.insert('*');
    let size = rev::view::Size {
        width: 80,
        height: 24,
    };
    let stale = rev::display::Frame::project(editor.buffer()?, editor.view(), size, "")?;
    assert!(stale.rows()[0].text().contains("** Heading 0"));
    settle(editor)?;
    assert!(editor.frame()?.rows()[0].text().contains("○ Heading 0"));
    assert_eq!(editor.buffer()?.history().len(), 1);
    editor.execute(rev::command::Command::History(rev::command::History::Undo));
    settle(editor)?;
    assert!(editor.frame()?.rows()[0].text().contains("◉ Heading 0"));
    let buffer = editor.buffer()?;
    assert!(!buffer.is_modified());
    assert!(buffer.history().is_empty());
    assert_eq!(source(editor), original);
    assert!(started.elapsed() < Duration::from_secs(5));
    Ok(started.elapsed().as_micros())
}

fn idle_ticks() -> Result<(u64, String)> {
    let cpu = cpu_ticks("self")?;
    std::thread::sleep(Duration::from_millis(350));
    let ticks = cpu_ticks("self")?.saturating_sub(cpu);
    assert!(ticks <= 1, "idle native owner consumed {ticks} CPU ticks");
    Ok((ticks, peak_rss("self")?))
}
