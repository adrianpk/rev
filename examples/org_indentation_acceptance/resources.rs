use super::native::prepare;
use crate::support::{self, Result, cpu_ticks, named, open, peak_rss, settle, source};
use crossterm::event::Event;
use rev::command::{Command, Motion};
use rev::session::{Action, Session};
use std::path::Path;
use std::time::{Duration, Instant};

pub(super) fn run(root: &Path) -> Result<()> {
    let mut large = support::document();
    for section in 0..512 {
        large.push_str(&format!(
            "{} Section {section}\n",
            "*".repeat(section % 8 + 1)
        ));
        large.push_str(&"body\t界 e\u{301} ordinary source remains unchanged\n".repeat(32));
    }
    case(root, "large", &large)?;
    case(root, "deep", &nested(256))?;
    case(root, "over-limit", &nested(32768))
}

fn nested(depth: usize) -> String {
    format!(
        "{} Deep\n{}\n",
        "*".repeat(depth),
        "token 界 e\u{301} ".repeat(4096)
    )
}

fn case(root: &Path, name: &str, text: &str) -> Result<()> {
    println!(
        "START resource {name}: bytes={} scalars={}",
        text.len(),
        text.chars().count()
    );
    let started = Instant::now();
    let mut editor = open(root, "non-modal", text)?;
    prepare(&mut editor)?;
    assert!(started.elapsed() < Duration::from_secs(5));
    let startup = started.elapsed();
    println!("MEASURE {name} startup_us={}", startup.as_micros());
    observation(&editor, name)?;
    let reflow = reflows(&mut editor)?;
    assert_eq!(source(&editor), text);
    assert!(editor.buffer()?.history().is_empty());
    assert!(editor.remaining_input(Instant::now()).is_none());
    assert_eq!(
        editor.expire_input(Instant::now() + Duration::from_secs(10)),
        Action::Wait
    );
    let ticks = idle()?;
    println!(
        "MEASURE {name}: bytes={} scalars={} startup_us={} 16_toggle_resize_reflows_us={} idle_350ms_cpu_ticks={ticks} peak_rss={}; exact source, no history and no idle deadline",
        text.len(),
        text.chars().count(),
        startup.as_micros(),
        reflow.as_micros(),
        peak_rss("self")?
    );
    Ok(())
}

fn observation(editor: &Session, name: &str) -> Result<()> {
    match name {
        "deep" => capped(editor),
        "over-limit" => literal(editor),
        _ => Ok(()),
    }
}

fn literal(editor: &Session) -> Result<()> {
    let buffer = editor.buffer()?;
    assert!(buffer.org_indentation());
    let fragments = editor.view().wrapped_rows(buffer, 6, 3)?;
    assert_eq!(fragments.len(), 3);
    assert!(
        fragments
            .iter()
            .all(|fragment| fragment.margin == 0 && !fragment.source.is_empty())
    );
    println!(
        "MEASURE incomplete coverage: 32768 stars exceed native depth=256; literal margins=0 with finite source progress"
    );
    Ok(())
}

fn capped(editor: &Session) -> Result<()> {
    let buffer = editor.buffer()?;
    assert!(buffer.org_indentation());
    let fragments = editor.view().wrapped_rows(buffer, 6, 3)?;
    assert_eq!(fragments.len(), 3);
    assert!(fragments.iter().all(|fragment| fragment.margin == 4));
    println!(
        "MEASURE deep cap: 256 source stars, width=6, virtual_margin=4, source cells remain available"
    );
    Ok(())
}

fn reflows(editor: &mut Session) -> Result<Duration> {
    let started = Instant::now();
    for width in [6, 40, 80, 512].into_iter().cycle().take(16) {
        named(editor, "org-indent-mode");
        editor.handle(Event::Resize(width, 24));
        editor.execute(Command::Motion(Motion::BufferEnd));
        settle(editor)?;
        editor.execute(Command::Motion(Motion::BufferStart));
        settle(editor)?;
    }
    let elapsed = started.elapsed();
    assert!(elapsed < Duration::from_secs(5));
    Ok(elapsed)
}

fn idle() -> Result<u64> {
    let before = cpu_ticks("self")?;
    std::thread::sleep(Duration::from_millis(350));
    let ticks = cpu_ticks("self")?.saturating_sub(before);
    assert!(ticks <= 1, "idle owner consumed {ticks} CPU ticks");
    Ok(ticks)
}
