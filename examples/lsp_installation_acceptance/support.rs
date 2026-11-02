use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::{config, session::Session};
use std::path::Path;
use std::time::{Duration, Instant};

pub(crate) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub(crate) fn key(s: &mut Session, code: KeyCode) {
    s.handle(Event::Key(KeyEvent::new(code, KeyModifiers::NONE)));
}

pub(crate) fn answer(s: &mut Session, text: &str) {
    for c in text.chars() {
        key(s, KeyCode::Char(c));
    }
    key(s, KeyCode::Enter);
}

pub(crate) fn named(s: &mut Session, command: &str) {
    let (code, modifiers) = if s.interaction_profile() == config::InteractionProfile::Modal {
        (KeyCode::Char(':'), KeyModifiers::NONE)
    } else {
        (KeyCode::Char('x'), KeyModifiers::ALT)
    };
    s.handle(Event::Key(KeyEvent::new(code, modifiers)));
    answer(s, command);
}

pub(crate) fn wait(s: &mut Session, message: &str) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(12);
    while !s.echo().contains(message) {
        if Instant::now() >= deadline {
            return Err(format!("expected {message}: {}", s.echo()).into());
        }
        s.expire_input(Instant::now());
        s.poll_frame()?;
        std::thread::sleep(Duration::from_millis(2));
    }
    Ok(())
}

pub(crate) fn text(s: &Session) -> String {
    let b = s.buffer().unwrap();
    b.slice(&b.span(0..b.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

pub(crate) fn cpu_ticks(pid: &str) -> Result<u64> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat"))?;
    let fields: Vec<_> = stat
        .rsplit_once(')')
        .ok_or("invalid stat")?
        .1
        .split_whitespace()
        .collect();
    Ok(fields[11].parse::<u64>()? + fields[12].parse::<u64>()?)
}

pub(crate) fn peak_rss(pid: &str) -> Result<String> {
    let status = std::fs::read_to_string(format!("/proc/{pid}/status"))?;
    Ok(status
        .lines()
        .find_map(|s| s.strip_prefix("VmHWM:"))
        .ok_or("missing peak RSS")?
        .trim()
        .into())
}

pub(crate) fn children() -> Result<Vec<String>> {
    let mut pids = std::collections::BTreeSet::new();
    for task in std::fs::read_dir("/proc/self/task")? {
        let file = task?.path().join("children");
        pids.extend(
            std::fs::read_to_string(file)?
                .split_whitespace()
                .map(str::to_owned),
        );
    }
    Ok(pids.into_iter().collect())
}

pub(crate) fn reaped(pids: &[String]) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(5);
    while pids
        .iter()
        .any(|pid| Path::new(&format!("/proc/{pid}")).exists())
    {
        if Instant::now() >= deadline {
            return Err("owned server was not reaped".into());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    Ok(())
}

pub(crate) fn idle(s: &mut Session) -> Result<()> {
    s.expire_input(Instant::now());
    assert!(s.remaining_input(Instant::now()).is_none());
    let before = cpu_ticks("self")?;
    std::thread::sleep(Duration::from_millis(350));
    let ticks = cpu_ticks("self")?.saturating_sub(before);
    assert!(ticks <= 1, "idle native owner used {ticks} ticks");
    println!(
        "MEASURE management idle: 350ms cpu_ticks={ticks} peak_rss={}",
        peak_rss("self")?
    );
    Ok(())
}
