use super::resources::{Native, Result, wait};
use std::{fs, path::Path, time::Instant};

// Exactly two PTYs cover real selectors, a generated native result, a local
// binding, cancellation and crash retirement. Lower-level variants stay native.
#[test]
fn terminal_journeys() -> Result<()> {
    for profile in ["non-modal", "modal"] {
        journey(profile)?;
    }
    Ok(())
}

fn journey(profile: &str) -> Result<()> {
    let mut native = Native::start(profile, Some("journey"), false, false)?;
    let pid = create_result(&mut native, profile)?;
    cancel_pending(&mut native, profile, &pid)?;
    retire_crash(&mut native, profile)?;
    restore_result(&mut native)?;
    save_result(&mut native, profile)
}

fn create_result(native: &mut Native, profile: &str) -> Result<String> {
    assert!(!native.peer_path("starts").exists());
    fs::write(native.peer_path("action"), "result")?;
    native.command(profile, "field.notes.collect")?;
    native
        .terminal
        .frame(|f| f.contains("Plugin result") && f.contains("result text"))?;
    wait(|| native.peer_path("generated").exists())?;
    Ok(fs::read_to_string(native.peer_path("starts"))?
        .trim()
        .to_owned())
}

fn cancel_pending(native: &mut Native, profile: &str, pid: &str) -> Result<()> {
    fs::write(native.peer_path("action"), "hold")?;
    native.terminal.send(b"\x1b[19~")?; // Actual xterm F8, resolved only in the result buffer.
    wait(|| fs::read_to_string(native.peer_path("calls")).is_ok_and(|s| s.lines().count() == 2))?;
    let began = Instant::now();
    native
        .terminal
        .send(if profile == "modal" { b"i!\x1b" } else { b"!" })?;
    native.terminal.frame(|f| f.contains("!result text"))?;
    println!(
        "MEASURE plugins journey={profile} pending_input_us={}",
        began.elapsed().as_micros()
    );
    native.terminal.send(b"\x07")?;
    wait(|| !Path::new(&format!("/proc/{pid}")).exists())?;
    Ok(())
}

fn retire_crash(native: &mut Native, profile: &str) -> Result<()> {
    fs::write(native.peer_path("action"), "crash")?;
    native.command(profile, "field.notes.collect")?;
    native
        .terminal
        .frame(|f| f.contains("field.notes") && (f.contains("exited") || f.contains("EOF")))?;
    assert_eq!(
        fs::read_to_string(native.peer_path("starts"))?
            .lines()
            .count(),
        2
    );
    Ok(())
}

fn restore_result(native: &mut Native) -> Result<()> {
    native.terminal.send(b"\x18bsource.txt\r")?;
    native.terminal.frame(|f| f.contains("source text"))?;
    native.terminal.send(b"\x18bPlugin result\r")?;
    native.terminal.frame(|f| f.contains("!result text"))?;
    Ok(())
}

fn save_result(native: &mut Native, profile: &str) -> Result<()> {
    native.command(profile, "write-file")?;
    native.terminal.frame(|f| f.contains("Write file:"))?;
    native.terminal.send(b"result.txt\r")?;
    let saved = native.fixture.0.join("workspace/result.txt");
    native
        .terminal
        .frame(|_| fs::read_to_string(&saved).is_ok_and(|s| s == "!result text\n"))?;
    native.terminal.idle()?;
    native.terminal.finish()?;
    assert_eq!(
        fs::read_to_string(native.peer_path("starts"))?
            .lines()
            .count(),
        2
    );
    assert_eq!(fs::read_to_string(saved)?, "!result text\n");
    Ok(())
}
