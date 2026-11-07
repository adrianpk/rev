use crate::{
    catalog,
    support::{Result, cpu_ticks, peak_rss, source},
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::Command;
use rev::config::prepare;
use rev::context::{Catalog, Context, MAX_FILES, Name, SourcePosition, Store};
use rev::session::{Action, Session};
use rev::view::{Size, View};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

#[derive(Default)]
struct Turns {
    passes: usize,
    longest_us: u128,
    resize_us: u128,
    peak_fds: usize,
    peak_threads: usize,
}
fn descriptors() -> Result<usize> {
    Ok(fs::read_dir("/proc/self/fd")?.count())
}
fn threads() -> Result<usize> {
    Ok(fs::read_dir("/proc/self/task")?.count())
}
fn rss() -> Result<usize> {
    let status = fs::read_to_string("/proc/self/status")?;
    Ok(status
        .lines()
        .find_map(|line| line.strip_prefix("VmRSS:"))
        .ok_or("missing resident memory")?
        .split_whitespace()
        .next()
        .ok_or("missing resident memory value")?
        .parse()?)
}

// Drive the real cooperative owner and paint while IO runs. Resize uses the
// ordinary input boundary; the measurements include any result-adoption work.
fn pump(session: &mut Session, turns: &mut Turns, ready: impl Fn(&Session) -> bool) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let start = Instant::now();
        session.expire_input(Instant::now());
        session.poll_frame()?;
        turns.longest_us = turns.longest_us.max(start.elapsed().as_micros());
        turns.passes += 1;
        turns.peak_fds = turns.peak_fds.max(descriptors()?);
        turns.peak_threads = turns.peak_threads.max(threads()?);
        if ready(session) {
            return Ok(());
        }
        let start = Instant::now();
        session.handle(Event::Resize(
            if turns.passes.is_multiple_of(2) {
                80
            } else {
                100
            },
            24,
        ));
        turns.resize_us = turns.resize_us.max(start.elapsed().as_micros());
        assert!(
            Instant::now() < deadline,
            "native context deadline: {}",
            session.echo()
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}
fn session(root: &Path, state: PathBuf) -> Result<Session> {
    fs::create_dir_all(root)?;
    let path = root.join("config.toml");
    fs::write(&path, "tree.visible=false\ncompletion.automatic=false\n")?;
    let settings = prepare(&path);
    assert!(settings.diagnostic.is_none());
    let buffers = Buffers::with_settings(settings.snapshot)?;
    let view = View::new(buffers.get(buffers.initial())?);
    let mut editor = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )?;
    fs::write(root.join("retained.txt"), catalog::DOCUMENT)?;
    editor.load(&root.join("retained.txt"))?;
    editor.set_context_storage(state)?;
    Ok(editor)
}
fn cancel(editor: &mut Session) {
    editor.execute(Command::KeyboardQuit);
}
fn enter(editor: &mut Session) {
    editor.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
}
fn list(editor: &mut Session, turns: &mut Turns) -> Result<()> {
    editor.execute(Command::ProjectList);
    pump(editor, turns, |s| s.echo().starts_with("Select a Project"))
}
fn choose(editor: &mut Session, turns: &mut Turns, name: &str) -> Result<()> {
    list(editor, turns)?;
    editor.handle(Event::Paste(name.into()));
    editor.handle(Event::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)));
    enter(editor);
    pump(editor, turns, |s| {
        matches!(s.context(), Context::Project(_))
    })
}
fn seed(root: &Path, catalog: &Catalog) -> Result<PathBuf> {
    let state = root.join("state/contexts.toml");
    fs::create_dir_all(state.parent().unwrap())?;
    fs::write(&state, catalog.encode()?)?;
    Ok(state)
}

pub(super) fn run(root: &Path) -> Result<()> {
    fs::create_dir_all(root)?;
    let fds = descriptors()?;
    let workers = threads()?;
    let before = rss()?;
    ordinary(&root.join("ordinary"))?;
    session_bytes(&root.join("session"))?;
    boundary(&root.join("boundary"))?;
    let deadline = Instant::now() + Duration::from_secs(3);
    while descriptors()? != fds || threads()? != workers {
        assert!(
            Instant::now() < deadline,
            "context owner did not retire descriptors/threads"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    println!(
        "MEASURE ownership: fds_before={fds} fds_after={} threads_before={workers} threads_after={} rss_before_kib={before} rss_after_kib={} peak_rss={}",
        descriptors()?,
        threads()?,
        rss()?,
        peak_rss("self")?
    );
    Ok(())
}

fn session_bytes(root: &Path) -> Result<()> {
    let start = Instant::now();
    let (catalog, charged) = catalog::session_boundary(&root.join("roots"))?;
    let bytes = catalog.encode()?;
    let state = seed(root, &catalog)?;
    assert_eq!(Store::open(&state)?.1, catalog);
    assert_eq!(fs::read(&state)?, bytes);
    println!(
        "MEASURE session byte boundary: files=512 charged_session_bytes={charged} catalog_bytes={} encoded_capacity={} elapsed_us={} rss_kib={} peak_rss={}; oversized candidate rejected without truncation",
        bytes.len(),
        bytes.capacity(),
        start.elapsed().as_micros(),
        rss()?,
        peak_rss("self")?
    );
    Ok(())
}

fn ordinary(root: &Path) -> Result<()> {
    let start = Instant::now();
    let catalog = catalog::ordinary(&root.join("roots"))?;
    let bytes = catalog.encode()?;
    assert_eq!(catalog.projects.len() + catalog.workspaces.len(), 50);
    assert_eq!(
        catalog
            .sessions
            .iter()
            .map(|s| s.snapshot.as_ref().unwrap().files.len())
            .sum::<usize>(),
        1000
    );
    let state = seed(root, &catalog)?;
    let mut editor = session(root, state.clone())?;
    let stamp = editor.buffer()?.stamp();
    let mut turns = Turns::default();
    list(&mut editor, &mut turns)?;
    assert_eq!(editor.buffer()?.stamp(), stamp);
    assert_eq!(source(&editor), catalog::DOCUMENT);
    cancel(&mut editor);
    assert_eq!(fs::read(&state)?, bytes);
    quiet(&mut editor)?;
    println!(
        "MEASURE ordinary: artifacts=50 session_files=1000 encoded_bytes={} encoded_capacity={} elapsed_us={} service_passes={} longest_pass_us={} resize_us={} rss_kib={} peak_rss={}",
        bytes.len(),
        bytes.capacity(),
        start.elapsed().as_micros(),
        turns.passes,
        turns.longest_us,
        turns.resize_us,
        rss()?,
        peak_rss("self")?
    );
    Ok(())
}

fn boundary(root: &Path) -> Result<()> {
    let start = Instant::now();
    let before = rss()?;
    let base = catalog::boundary(&root.join("roots"))?;
    let bytes = base.encode()?;
    let encoded_rss = rss()?;
    let state = seed(root, &base)?;
    let (store, observed) = Store::open(&state)?;
    assert_eq!(observed, base);
    drop(observed);
    catalog::overflow(&base)?;
    assert_eq!(fs::read(&state)?, bytes);
    let mut candidate = base.clone();
    candidate.projects[0].name = Name::new("Measured")?;
    candidate.projects[0].revision = candidate.projects[0].revision.next()?;
    let prepared = base.prepare(candidate)?;
    let prepared_rss = rss()?;
    let candidate_bytes = prepared.candidate().encode()?.len();
    let publication = store.publish(&prepared)?;
    assert!(publication.durability_warning.is_none());
    assert_eq!(Store::open(&state)?.1, publication.catalog);
    println!(
        "MEASURE boundary storage: projects={} workspaces={} max_roots={} max_files={} source_encoded_bytes={} source_vec_capacity={} candidate_encoded_bytes={candidate_bytes} rss_before_kib={before} rss_encoded_kib={encoded_rss} rss_prepared_kib={prepared_rss} rss_published_kib={} peak_rss={} elapsed_us={}",
        base.projects.len(),
        base.workspaces.len(),
        base.workspaces[0].roots.len(),
        MAX_FILES,
        bytes.len(),
        bytes.capacity(),
        rss()?,
        peak_rss("self")?,
        start.elapsed().as_micros()
    );
    let id = publication.catalog.projects[0].id.clone();
    let source_root = publication.catalog.projects[0]
        .root
        .path
        .as_path()
        .to_owned();
    drop((base, prepared, publication, store, bytes));
    println!(
        "MEASURE released storage fixture copies: rss_kib={} peak_rss={}",
        rss()?,
        peak_rss("self")?
    );
    let mut editor = session(root, state.clone())?;
    let mut turns = Turns::default();
    canceled(&mut editor, &mut turns, &state)?;
    let mut rounds = Vec::new();
    for _ in 0..2 {
        if editor.dashboard_visible() {
            cancel(&mut editor);
        }
        rounds.push(round(&mut editor, &mut turns, &state, &id)?);
    }
    quiet(&mut editor)?;
    cancel(&mut editor);
    choose(&mut editor, &mut turns, "Measured")?;
    let extra = root.join("extra.txt");
    fs::write(&extra, "loose source")?;
    editor.load(&extra)?;
    let retained = (
        editor.buffer()?.stamp(),
        editor.context_generation(),
        fs::read(&state)?,
    );
    editor.execute(Command::ContextClose);
    assert!(
        editor.echo().contains("file capacity exceeded"),
        "{}",
        editor.echo()
    );
    assert_eq!(editor.context(), &Context::Project(id));
    assert_eq!(editor.buffer()?.stamp(), retained.0);
    assert_eq!(editor.context_generation(), retained.1);
    assert_eq!(fs::read(&state)?, retained.2);
    assert_eq!(source(&editor), "loose source");
    assert!(!editor.dashboard_visible());
    for i in 0..MAX_FILES {
        assert_eq!(
            fs::read(source_root.join(format!("file-{i:03}")))?,
            catalog::DOCUMENT.as_bytes()
        );
    }
    assert_eq!(fs::read(&extra)?, b"loose source");
    println!(
        "MEASURE boundary session: files=512 rounds_restore_capture_us_rss_kib={rounds:?} service_passes={} longest_pass_us={} resize_us={} peak_fds={} peak_threads={} rss_kib={} peak_rss={}; 513th capture rejected with unchanged state/source",
        turns.passes,
        turns.longest_us,
        turns.resize_us,
        turns.peak_fds,
        turns.peak_threads,
        rss()?,
        peak_rss("self")?
    );
    Ok(())
}

fn canceled(editor: &mut Session, turns: &mut Turns, state: &Path) -> Result<()> {
    let retained = (
        editor.buffer()?.stamp(),
        editor.context_generation(),
        fs::read(state)?,
    );
    let mut samples = Vec::new();
    for _ in 0..3 {
        editor.execute(Command::ProjectList);
        cancel(editor);
        pump(editor, turns, |s| {
            s.remaining_input(Instant::now()).is_none()
        })?;
        assert_eq!(editor.buffer()?.stamp(), retained.0);
        assert_eq!(editor.context_generation(), retained.1);
        assert_eq!(fs::read(state)?, retained.2);
        samples.push(rss()?);
    }
    list(editor, turns)?;
    editor.handle(Event::Paste("Measured".into()));
    editor.handle(Event::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)));
    enter(editor);
    pump(editor, turns, |s| {
        s.echo().starts_with("Preparing Project files")
    })?;
    cancel(editor);
    pump(editor, turns, |s| {
        s.remaining_input(Instant::now()).is_none()
    })?;
    assert_eq!(editor.buffer()?.stamp(), retained.0);
    assert_eq!(editor.context_generation(), retained.1);
    assert!(matches!(editor.context(), Context::Standalone));
    assert_eq!(fs::read(state)?, retained.2);
    println!(
        "MEASURE canceled catalog reads: cycles=3 retained_rss_kib={samples:?} after_staged_cancel_kib={} peak_rss={}; source generation/stamp/state preserved",
        rss()?,
        peak_rss("self")?
    );
    Ok(())
}
fn quiet(editor: &mut Session) -> Result<()> {
    let mut turns = Turns::default();
    pump(editor, &mut turns, |s| {
        s.remaining_input(Instant::now()).is_none()
    })?;
    assert_eq!(
        editor.expire_input(Instant::now() + Duration::from_secs(10)),
        Action::Wait
    );
    let before = cpu_ticks("self")?;
    std::thread::sleep(Duration::from_millis(350));
    let ticks = cpu_ticks("self")?.saturating_sub(before);
    assert!(ticks <= 1, "idle native context consumed {ticks} CPU ticks");
    assert!(editor.remaining_input(Instant::now()).is_none());
    println!("MEASURE native idle: interval_ms=350 cpu_ticks={ticks} pending_timer=false");
    Ok(())
}

fn round(
    editor: &mut Session,
    turns: &mut Turns,
    state: &Path,
    id: &rev::context::ProjectId,
) -> Result<(u128, u128, usize)> {
    let restore = Instant::now();
    choose(editor, turns, "Measured")?;
    assert_eq!(editor.context(), &Context::Project(id.clone()));
    assert!(
        editor
            .buffer()?
            .file()
            .unwrap()
            .path()
            .ends_with("file-511")
    );
    assert_eq!(
        SourcePosition::capture(editor.buffer()?, editor.view().point.position)?,
        SourcePosition { line: 2, column: 4 }
    );
    let restore_us = restore.elapsed().as_micros();
    let close = Instant::now();
    editor.execute(Command::ContextClose);
    pump(editor, turns, |s| {
        s.dashboard_visible() && matches!(s.context(), Context::Standalone)
    })?;
    let capture_us = close.elapsed().as_micros();
    let catalog = Store::open(state)?.1;
    let saved = catalog
        .session(&catalog.projects[0].session_ref)
        .unwrap()
        .snapshot
        .as_ref()
        .unwrap();
    assert_eq!(saved.files.len(), MAX_FILES);
    assert!(
        saved
            .files
            .iter()
            .all(|f| f.cursor == SourcePosition { line: 2, column: 4 })
    );
    assert_eq!(
        saved.active_file.as_ref(),
        saved.files.last().map(|f| &f.file)
    );
    drop(catalog);
    Ok((restore_us, capture_us, rss()?))
}
