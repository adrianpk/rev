use super::{metrics::threads, support::*};
use rev::{
    command::{Command, History},
    session::Session,
};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

const BYTES: usize = 8 * 1024 * 1024;
const LINE: &str = "a界0123456789x\n";

// Exercise the default byte bound with actual tr, a native atomic replacement,
// retained output and ordinary undo; measure the same process after it is idle.
pub(crate) fn run(root: &Path) -> Result<()> {
    let original = LINE.repeat(BYTES / LINE.len());
    assert_eq!(original.len(), BYTES);
    let mut editor = prepare(root, &original)?;
    let before = threads()?;
    let started = Instant::now();
    editor.execute(Command::ShellFilterRegion);
    let capture = started.elapsed();
    assert!(capture < Duration::from_secs(5));
    let started = Instant::now();
    answer(&mut editor, "tr a A");
    settle(&mut editor)?;
    let publication = started.elapsed();
    let retained = verify(&mut editor, &original)?;
    idle_measure(&mut editor, before)?;
    println!(
        "MEASURE finite shell: input_bytes={BYTES} retained_stdout_bytes={retained} snapshot_us={} collect_publish_us={} peak_rss={}",
        capture.as_micros(),
        publication.as_micros(),
        peak_rss("self")?
    );
    Ok(())
}

fn prepare(root: &Path, original: &str) -> Result<Session> {
    let mut editor = open(root, "non-modal", "resources", "disk source\n", false)?;
    editor.replace(0..editor.buffer()?.len_chars(), original)?;
    whole(&mut editor);
    settle(&mut editor)?;
    Ok(editor)
}

fn verify(editor: &mut Session, original: &str) -> Result<usize> {
    assert_eq!(source(editor), original.replace('a', "A"));
    assert_eq!(editor.buffer()?.history().len(), 2);
    history_bytes(editor)?;
    let retained = output(editor)?;
    assert_eq!(retained.len(), BYTES);
    assert_eq!(retained, original.replace('a', "A"));
    switch(editor, "source.txt");
    let started = Instant::now();
    editor.execute(Command::History(History::Undo));
    settle(editor)?;
    assert_eq!(source(editor), original);
    assert_eq!(editor.buffer()?.history().len(), 1);
    assert_eq!(
        fs::read_to_string(editor.buffer()?.file().unwrap().path())?,
        "disk source\n"
    );
    println!(
        "MEASURE finite shell undo_us={}",
        started.elapsed().as_micros()
    );
    Ok(retained.len())
}

fn history_bytes(editor: &Session) -> Result<()> {
    let buffer = editor.buffer()?;
    let entry = buffer.history().back().ok_or("missing filter history")?;
    let changes = entry.changes();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].inserted().len_bytes(), BYTES);
    assert_eq!(changes[0].removed().len_bytes(), BYTES);
    println!("MEASURE native history inserted_bytes={BYTES} removed_bytes={BYTES}");
    Ok(())
}

fn idle_measure(editor: &mut Session, before: usize) -> Result<()> {
    idle(editor)?;
    assert_eq!(
        threads()?,
        before,
        "completed shell worker was not released"
    );
    let cpu = cpu_ticks("self")?;
    std::thread::sleep(Duration::from_millis(350));
    let ticks = cpu_ticks("self")?.saturating_sub(cpu);
    assert!(ticks <= 1, "idle native owner consumed {ticks} CPU ticks");
    assert!(editor.remaining_input(Instant::now()).is_none());
    println!(
        "MEASURE native idle: 350ms cpu_ticks={ticks} threads_before={before} threads_after={}",
        threads()?
    );
    Ok(())
}
