use std::{
    fs::File,
    io::{ErrorKind, Read},
    os::{
        fd::OwnedFd,
        unix::{ffi::OsStrExt, fs::PermissionsExt, net::UnixStream},
    },
    path::Path,
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
pub(super) fn stop(pid: u32, deadline: Instant) -> Result<(), String> {
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
    let status = wait(&mut child, deadline)?;
    result(&mut read, status)
}
fn wait(child: &mut Child, deadline: Instant) -> Result<ExitStatus, String> {
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

/// Linux process states distinguish exited zombies from members still handling
/// SIGKILL, including members that have already closed both output streams.
/// Observation never signals a process or waits for an unrelated group.
pub(crate) fn running(group: u32, deadline: Instant) -> Result<bool, String> {
    let processes =
        std::fs::read_dir("/proc").map_err(|e| format!("Observe owned process group: {e}"))?;
    for entry in processes {
        if Instant::now() >= deadline {
            return Err("Observe owned process-group shutdown timed out".into());
        }
        let entry = entry.map_err(|e| format!("Observe owned process group: {e}"))?;
        if !entry.file_name().as_bytes().iter().all(u8::is_ascii_digit) {
            continue;
        }
        if let Some(bytes) = stat(&entry.path().join("stat"))?
            && live_member(&bytes, group)?
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn stat(path: &Path) -> Result<Option<Vec<u8>>, String> {
    let mut bytes = Vec::new();
    let read = File::open(path).and_then(|file| file.take(4097).read_to_end(&mut bytes));
    match read {
        Ok(_) if bytes.len() <= 4096 => Ok(Some(bytes)),
        Ok(_) => Err("Process observation exceeds its 4 KiB stat bound".into()),
        // A task can disappear between enumeration, open and read. Linux
        // reports either ENOENT or ESRCH (3) for those races.
        Err(e) if e.kind() == ErrorKind::NotFound || e.raw_os_error() == Some(3) => Ok(None),
        Err(e) => Err(format!("Observe owned process-group member: {e}")),
    }
}

fn live_member(bytes: &[u8], group: u32) -> Result<bool, String> {
    // comm can contain ')' and non-UTF-8 bytes. Only the fields following its
    // last delimiter have the ASCII state/parent/group format.
    let end = bytes
        .iter()
        .rposition(|byte| *byte == b')')
        .ok_or("Process observation lacks comm delimiter")?;
    let tail = std::str::from_utf8(&bytes[end + 1..])
        .map_err(|e| format!("Process observation fields: {e}"))?;
    let mut fields = tail.split_whitespace();
    let state = fields.next().ok_or("Process observation lacks state")?;
    let field = fields.nth(1).ok_or("Process observation lacks group")?;
    // procfs pgrp is signed; a task exiting during observation can report -1.
    // Nonpositive values cannot identify the positive group we created.
    let observed = field.parse::<i32>().map_err(|e| {
        format!(
            "Process observation group {:?}: {e}",
            field.chars().take(32).collect::<String>()
        )
    })?;
    Ok(
        observed > 0
            && i64::from(observed) == i64::from(group)
            && !matches!(state, "Z" | "X" | "x"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // procfs exposes pgrp as signed %d. Nonpositive groups cannot identify an
    // owned child; malformed fields still reject rather than hiding live work.
    #[test]
    fn signed_group_fields() {
        for (name, bytes, expected) in [
            ("live member", b"123 (name) R 7 42 0\n".as_slice(), Ok(true)),
            (
                "other group",
                b"123 (name) S 7 43 0\n".as_slice(),
                Ok(false),
            ),
            (
                "negative group",
                b"123 (name) R 7 -1 0\n".as_slice(),
                Ok(false),
            ),
            ("zero group", b"123 (name) S 7 0 0\n".as_slice(), Ok(false)),
            ("zombie", b"123 (name) Z 7 42 0\n".as_slice(), Ok(false)),
            ("dead", b"123 (name) X 7 42 0\n".as_slice(), Ok(false)),
            (
                "complex comm",
                b"123 (name )\n\xff) R 7 42 0\n".as_slice(),
                Ok(true),
            ),
            (
                "malformed group",
                b"123 (name) R 7 invalid 0\n".as_slice(),
                Err(()),
            ),
            ("missing group", b"123 (name) R 7\n".as_slice(), Err(())),
        ] {
            assert_eq!(live_member(bytes, 42).map_err(|_| ()), expected, "{name}");
        }
    }
}
