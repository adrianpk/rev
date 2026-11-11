use super::{fixture, support::*};
use rev::{
    buffer::edit::History,
    command::{BufferCommand, Command},
    session::Session,
};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

fn generate(s: &mut Session, instruction: &str) -> Result<()> {
    named(s, "ai-implement");
    assert!(
        s.frame()?
            .rows()
            .iter()
            .any(|row| row.text().contains("AI insert")),
        "instruction input was not admitted: {}",
        s.echo()
    );
    answer(s, instruction);
    wait(s, "proposal ready")
}
fn switch(s: &mut Session, name: &str) {
    s.execute(Command::Buffer(BufferCommand::Switch));
    answer(s, name);
}
fn until(s: &mut Session, predicate: impl Fn(&Session) -> bool) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(6);
    while !predicate(s) {
        if Instant::now() >= deadline {
            return Err(format!("native deadline: {}", s.echo()).into());
        }
        s.expire_input(Instant::now());
        s.poll_frame()?;
        std::thread::sleep(Duration::from_millis(2));
    }
    Ok(())
}

// Both profiles exercise native targets, real notifications, actual Pi discovery,
// private restart/reset and the existing single undo owner in one toy project.
pub(crate) fn run(root: &Path) -> Result<()> {
    let project = fixture::prepare(root, false)?;
    let source = project.join("main.zig");
    let first = first_session(root, &project, &source)?;
    let mut s = fixture::session(root, &source)?;
    let restored_pid = restore_context(&mut s, root, &project, &first)?;
    let reset_pid = reset_context(&mut s, root, &project, &first.history, &restored_pid)?;
    idle_measurement(&mut s, &reset_pid)?;
    reaped(std::slice::from_ref(&reset_pid))?;
    drop(s);
    exit_session(root, &project, &source)?;
    missing_backend(root, &source, first.external)?;
    println!("PASS native joint editing/reload/discovery/private lifecycle/missing Pi");
    Ok(())
}

fn context_switches(pid: &str) -> Result<u64> {
    let status = fs::read_to_string(format!("/proc/{pid}/status"))?;
    Ok(status
        .lines()
        .find_map(|line| line.strip_prefix("voluntary_ctxt_switches:"))
        .ok_or("missing context switches")?
        .trim()
        .parse()?)
}

struct First {
    pid: String,
    history: PathBuf,
    external: &'static str,
}

fn startup(root: &Path, source: &Path) -> Result<Session> {
    let started = Instant::now();
    let mut s = fixture::session(root, source)?;
    println!(
        "MEASURE native profile={} startup_us={} editor_peak_rss={}",
        std::env::var("REV_ACCEPTANCE_PROFILE")?,
        started.elapsed().as_micros(),
        peak_rss("self")?
    );
    assert!(
        children()?.is_empty(),
        "ordinary startup launched a subprocess"
    );
    idle(&mut s)?;
    assert!(!root.join("state/rev/ai/pi").exists());

    Ok(s)
}

fn first_result(s: &mut Session, project: &Path, source: &Path) -> Result<String> {
    // A dirty visiting snapshot wins over older disk bytes in the real tool.
    s.insert('x');
    let dirty = text(s);
    fs::write(project.join("expected-main"), &dirty)?;
    let started = Instant::now();
    generate(s, "ACCEPTANCE-FIRST: insert a harmless comment.")?;
    let pid = fixture::pid(project)?;
    assert!(text(s).contains(fixture::GENERATED));
    tool_provenance(project, source, &pid, started)?;
    s.execute(Command::AiDiscardResult);
    assert_eq!(text(s), dirty);
    s.execute(Command::History(History::Undo));
    assert_eq!(text(s), fixture::SOURCE);
    fs::write(project.join("expected-main"), fixture::SOURCE)?;

    Ok(pid)
}

fn tool_provenance(project: &Path, source: &Path, pid: &str, started: Instant) -> Result<()> {
    assert_eq!(fs::read_to_string(source)?, fixture::SOURCE);
    let tools: serde_json::Value =
        serde_json::from_slice(&fs::read(project.join("fixture-tools"))?)?;
    assert_eq!(tools["provenance"], "native editor snapshot");
    assert!(fs::read_to_string(project.join("fixture-limit"))?.contains("33554432"));
    println!(
        "MEASURE actual Pi first_result_ms={} external_pi_peak_rss={} editor_peak_rss={}",
        started.elapsed().as_millis(),
        peak_rss(pid)?,
        peak_rss("self")?
    );
    Ok(())
}

fn external_reload(
    s: &mut Session,
    project: &Path,
    source: &Path,
    pid: &str,
) -> Result<&'static str> {
    generate(
        s,
        "Prepare the proposal that external reload will invalidate.",
    )?;
    assert_eq!(
        fixture::pid(project)?,
        pid,
        "compatible invocation did not reuse child"
    );
    switch(s, "main.zig");
    let old = s.buffer()?.stamp();
    let external = "const needle: u8 = 8;\n";
    fs::write(project.join("replacement"), external)?;
    let started = Instant::now();
    fs::rename(project.join("replacement"), source)?;
    until(s, |s| text(s) == external)?;
    assert_ne!(s.buffer()?.stamp(), old);
    assert!(!s.buffer()?.is_modified());
    println!(
        "MEASURE native atomic_notification_reload_ms={}",
        started.elapsed().as_millis()
    );
    switch(s, "*AI Result*");
    s.execute(Command::AiApplyResult);
    assert!(s.echo().contains("stale"), "{}", s.echo());
    s.execute(Command::AiDiscardResult);
    assert_eq!(text(s), external);
    fs::write(project.join("expected-main"), external)?;

    Ok(external)
}

fn save_apply(s: &mut Session, source: &Path, external: &str) -> Result<()> {
    generate(s, "Insert one comment after the fresh reload.")?;
    s.execute(Command::AiSaveCommand);
    answer(s, "joint-comment");
    assert!(s.echo().contains("saved and callable"), "{}", s.echo());
    switch(s, "main.zig");
    let history_len = s.buffer()?.history().len();
    switch(s, "*AI Result*");
    s.execute(Command::AiApplyResult);
    assert!(s.echo().contains("result applied"), "{}", s.echo());
    assert!(text(s).contains(fixture::GENERATED));
    assert_eq!(s.buffer()?.history().len(), history_len + 1);
    assert_eq!(fs::read_to_string(source)?, external);
    s.execute(Command::History(History::Undo));
    assert_eq!(text(s), external);
    assert!(!s.buffer()?.is_modified());
    Ok(())
}

fn first_session(root: &Path, project: &Path, source: &Path) -> Result<First> {
    let mut s = startup(root, source)?;
    let pid = first_result(&mut s, project, source)?;
    let external = external_reload(&mut s, project, source, &pid)?;
    save_apply(&mut s, source, external)?;
    let history = fixture::history(root)?;
    s.execute(Command::AiDisconnect);
    until(&mut s, |_| !Path::new("/proc").join(&pid).exists())?;
    reaped(std::slice::from_ref(&pid))?;
    drop(s);

    Ok(First {
        pid,
        history,
        external,
    })
}

fn restore_context(s: &mut Session, root: &Path, project: &Path, first: &First) -> Result<String> {
    // Actual Pi reconstruction must contain earlier instruction messages, not
    // merely return the same filename/provenance label from native metadata.
    named(s, "ai-joint-comment");
    wait(s, "proposal ready")?;
    let restored_pid = fixture::pid(project)?;
    assert_ne!(restored_pid, first.pid);
    assert_eq!(fixture::history(root)?, first.history);
    let context = fixture::context(project)?;
    assert!(context["users"].as_u64().is_some_and(|n| n >= 4) && context["prior"] == true);
    Ok(restored_pid)
}

fn reset_context(
    s: &mut Session,
    root: &Path,
    project: &Path,
    history: &Path,
    restored_pid: &str,
) -> Result<String> {
    s.execute(Command::AiDiscardResult);
    s.execute(Command::AiResetSession);
    until(s, |s| {
        !Path::new("/proc").join(restored_pid).exists()
            && s.remaining_input(Instant::now()).is_none()
    })?;
    generate(s, "Fresh reset context only.")?;
    let reset_pid = fixture::pid(project)?;
    assert_ne!(fixture::history(root)?, history);
    assert!(!history.exists());
    assert_eq!(fixture::context(project)?["users"], 1);
    s.execute(Command::AiDiscardResult);
    Ok(reset_pid)
}

fn idle_measurement(s: &mut Session, reset_pid: &str) -> Result<()> {
    idle(s)?;
    let cpu = cpu_ticks(reset_pid)?;
    let switches = context_switches(reset_pid)?;
    std::thread::sleep(Duration::from_millis(350));
    let ticks = cpu_ticks(reset_pid)?.saturating_sub(cpu);
    let switches = context_switches(reset_pid)?.saturating_sub(switches);
    println!(
        "MEASURE Pi idle 350ms cpu_ticks={ticks} voluntary_context_switches={switches} external_pi_peak_rss={}; kernel wakeup count unavailable",
        peak_rss(reset_pid)?
    );
    Ok(())
}

fn exit_session(root: &Path, project: &Path, source: &Path) -> Result<()> {
    let mut s = fixture::session(root, source)?;
    named(&mut s, "ai-joint-comment");
    wait(&mut s, "proposal ready")?;
    let exit_pid = fixture::pid(project)?;
    drop(s);
    reaped(&[exit_pid])?;

    Ok(())
}

fn missing_configuration(root: &Path) -> Result<()> {
    let config = root.join("config/rev/config.toml");
    let mut settings = fs::read_to_string(&config)?;
    let command = serde_json::to_string(
        std::env::current_exe()?
            .to_str()
            .ok_or("non UTF-8 executable")?,
    )?;
    settings = settings.replace(
        &format!("command={command}"),
        "command='/missing/rev-acceptance-pi'",
    );
    fs::write(config, settings)?;
    Ok(())
}

// A missing optional backend never prevents ordinary edit/undo, and fails
// explicitly without falling through to another executable/provider.
fn missing_backend(root: &Path, source: &Path, external: &str) -> Result<()> {
    missing_configuration(root)?;
    let mut s = fixture::session(root, source)?;
    s.insert('z');
    assert_ne!(text(&s), external);
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), external);
    named(&mut s, "ai-joint-comment");
    wait(&mut s, "unavailable")?;
    assert_eq!(text(&s), external);
    assert_eq!(fs::read_to_string(source)?, external);
    assert!(children()?.is_empty());
    Ok(())
}
