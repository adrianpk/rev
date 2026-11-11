use crate::native::{control, keys, settings};
use crate::support::{Result, cpu_ticks, idle, open, peak_rss, settle, source};
use crossterm::event::Event;
use rev::{coordinates::Scalar, input::EditingState};
use std::path::Path;
use std::time::{Duration, Instant};

pub(super) fn run(root: &Path, binary: &Path) -> Result<()> {
    counts(root)?;
    block(root)?;
    burst(root)?;
    let bytes = std::fs::metadata(binary)?.len();
    let baseline = 18_931_160_u64;
    #[expect(
        clippy::cast_precision_loss,
        reason = "Only the informational ratio uses floating point; byte counts and deltas stay exact."
    )]
    let ratio = bytes as f64 / baseline as f64;
    println!(
        "MEASURE modal executable: bytes={bytes} recorded_lsp_baseline_bytes={baseline} delta_bytes={} ratio={:.6}; same normal release flags, separate server excluded",
        i128::from(bytes) - i128::from(baseline),
        ratio
    );
    println!("MEASURE native modal driver peak_rss={}", peak_rss("self")?);
    Ok(())
}

fn counts(root: &Path) -> Result<()> {
    let text = "word 界 e\u{301} tail\r\n".repeat(16_384);
    let mut editor = open(root, "large.txt", &format!("|{text}"), &settings(root)?)?;
    let started = Instant::now();
    keys(&mut editor, "4294967295w")?;
    assert_eq!(editor.view().point.position, Scalar(text.chars().count()));
    keys(&mut editor, "4294967295b")?;
    assert_eq!(editor.view().point.position, Scalar(0));
    keys(&mut editor, "4294967295d2w")?;
    assert_eq!(source(&editor), text);
    assert_eq!(editor.view().point.position, Scalar(0));
    assert!(editor.buffer()?.history().is_empty());
    assert!(started.elapsed() < Duration::from_secs(5));
    editor.handle(Event::Resize(18, 8));
    settle(&mut editor)?;
    assert_eq!(editor.view().point.position, Scalar(0));
    assert_eq!(editor.editing_state(), Some(EditingState::Normal));
    println!(
        "MEASURE large counts: bytes={} logical_lines=16384 elapsed_us={}; u32-max source-bounded traversal, overflow rejection, narrow resize and no edit",
        text.len(),
        started.elapsed().as_micros()
    );
    Ok(())
}

fn block(root: &Path) -> Result<()> {
    let text = "\t界e\u{301}tail\r\n".repeat(4096);
    let mut editor = open(root, "block.txt", &format!("|{text}"), &settings(root)?)?;
    let started = Instant::now();
    control(&mut editor, 'v');
    keys(&mut editor, "2l4095jd")?;
    assert_eq!(source(&editor), "tail\r\n".repeat(4096));
    assert_eq!(editor.buffer()?.history().len(), 1);
    keys(&mut editor, "u")?;
    assert_eq!(source(&editor), text);
    assert!(started.elapsed() < Duration::from_secs(5));
    println!(
        "MEASURE logical block: rows=4096 columns=3 bytes={} elapsed_us={}; tabs and wide/combining graphemes, one atomic delete and exact undo",
        text.len(),
        started.elapsed().as_micros()
    );
    Ok(())
}

fn burst(root: &Path) -> Result<()> {
    let mut editor = open(root, "burst.txt", "|abcdefgh\nnext\n", &settings(root)?)?;
    let started = Instant::now();
    for _ in 0..64 {
        keys(&mut editor, "lh")?;
    }
    assert_eq!(editor.view().point.position, Scalar(0));
    assert_eq!(source(&editor), "abcdefgh\nnext\n");
    assert!(editor.buffer()?.history().is_empty());
    idle(&mut editor);
    let burst = started.elapsed();
    let cpu = cpu_ticks("self")?;
    std::thread::sleep(Duration::from_millis(350));
    let idle_ticks = cpu_ticks("self")?.saturating_sub(cpu);
    assert!(
        idle_ticks <= 1,
        "idle native owner used {idle_ticks} CPU ticks"
    );
    idle(&mut editor);
    println!(
        "MEASURE finite modal burst: events=128 elapsed_us={} idle_350ms_cpu_ticks={idle_ticks}; source/point/history exact and no idle deadline",
        burst.as_micros()
    );
    Ok(())
}
