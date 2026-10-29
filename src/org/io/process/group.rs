use std::{
    io::{ErrorKind, Read},
    os::{
        fd::OwnedFd,
        unix::{fs::PermissionsExt, net::UnixStream},
    },
    process::{Child, Command, ExitStatus, Stdio},
    time::{Duration, Instant},
};
/// The fixed Unix utility sends a signal only to the group created by this
/// operation. It is not selected through source text, startup argv or PATH.
pub(super) fn available() -> Result<(), String> {
    let metadata = std::fs::metadata("/bin/kill")
        .map_err(|e| format!("Owned process groups require /bin/kill: {e}"))?;
    if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
        return Err("Owned process groups require executable /bin/kill".into());
    }
    Ok(())
}
pub(super) fn stop(pid: u32) -> Result<(), String> {
    let (mut read, write) = UnixStream::pair().map_err(|e| e.to_string())?;
    read.set_nonblocking(true).map_err(|e| e.to_string())?;
    let mut child = Command::new("/bin/kill")
        .args(["-KILL", "--", &format!("-{pid}")])
        .env("LC_ALL", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(OwnedFd::from(write)))
        .spawn()
        .map_err(|e| format!("Stop owned process group: {e}"))?;
    let status = wait(&mut child)?;
    result(&mut read, status)
}
fn wait(child: &mut Child) -> Result<ExitStatus, String> {
    let deadline = Instant::now() + Duration::from_millis(250);
    let result = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Err(e) => break Err(format!("Wait for group termination: {e}")),
            Ok(None) if Instant::now() >= deadline => {
                break Err("Process-group termination timed out".into());
            }
            _ => std::thread::sleep(Duration::from_millis(2)),
        }
    };
    if result.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    result
}
fn result(read: &mut UnixStream, status: ExitStatus) -> Result<(), String> {
    let mut bytes = [0u8; 1024];
    let count = match read.read(&mut bytes) {
        Ok(n) => n,
        Err(e) if e.kind() == ErrorKind::WouldBlock => 0,
        Err(e) => return Err(e.to_string()),
    };
    let message = String::from_utf8_lossy(&bytes[..count]);
    if status.success() || message.contains("No such process") {
        Ok(())
    } else {
        Err(format!("Stop owned process group: {message}"))
    }
}
