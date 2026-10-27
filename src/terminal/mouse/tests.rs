use std::io::{self, Write};

use super::Reporting;

const SGR_ON: &[u8] = b"\x1b[?1006h";
const BUTTONS_ON: &[u8] = b"\x1b[?1002h";
const BUTTONS_OFF: &[u8] = b"\x1b[?1002l";
const SGR_OFF: &[u8] = b"\x1b[?1006l";

#[derive(Default)]
struct Fault {
    bytes: Vec<u8>,
    attempts: Vec<Vec<u8>>,
    reject: Vec<&'static [u8]>,
    partial: bool,
}

impl Write for Fault {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.attempts.push(bytes.to_vec());
        if self.partial && bytes == BUTTONS_ON {
            self.partial = false;
            self.bytes.extend_from_slice(&bytes[..2]);
            return Ok(2);
        }
        if let Some(index) = self.reject.iter().position(|pattern| bytes == *pattern) {
            self.reject.remove(index);
            return Err(io::Error::other("injected write failure"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

// Exact native mode commands exclude all-motion, RXVT and redundant tracking.
// Repeated enable/disable is inert; re-enable acquires a fresh matched pair.
#[test]
fn reporting_modes() {
    let mut reporting = Reporting::default();
    let mut output = Vec::new();
    reporting.set(&mut output, false).unwrap();
    assert!(output.is_empty());
    for _ in 0..2 {
        reporting.set(&mut output, true).unwrap();
    }
    assert_eq!(output, [SGR_ON, BUTTONS_ON].concat());
    for _ in 0..2 {
        reporting.set(&mut output, false).unwrap();
    }
    let cycle = [SGR_ON, BUTTONS_ON, BUTTONS_OFF, SGR_OFF].concat();
    assert_eq!(output, cycle);
    reporting.set(&mut output, true).unwrap();
    reporting.set(&mut output, false).unwrap();
    assert_eq!(output, cycle.repeat(2));
}

// Even a partially written mode owns its undo before the write starts. Failure
// at either acquisition step immediately restores every potentially active mode.
#[test]
fn acquisition_rollback() {
    for (name, reject, partial, undo) in [
        ("format", SGR_ON, false, vec![SGR_OFF]),
        ("tracking", BUTTONS_ON, false, vec![BUTTONS_OFF, SGR_OFF]),
        (
            "partial tracking",
            b"?1002h",
            true,
            vec![BUTTONS_OFF, SGR_OFF],
        ),
    ] {
        let mut output = Fault {
            reject: vec![reject],
            partial,
            ..Fault::default()
        };
        let mut reporting = Reporting::default();
        assert!(reporting.set(&mut output, true).is_err(), "{name}");
        assert!(output.bytes.ends_with(&undo.concat()), "{name}");
        let before = output.bytes.clone();
        reporting.set(&mut output, false).unwrap();
        assert_eq!(output.bytes, before, "rollback left an obligation: {name}");
        reporting.set(&mut output, true).unwrap();
        assert!(
            output.bytes.ends_with(&[SGR_ON, BUTTONS_ON].concat()),
            "{name}"
        );
        reporting.set(&mut output, false).unwrap();
    }
}

// One failed restoration cannot skip the other mode. A retry sends precisely
// the failed obligations and a third restore has no effect.
#[test]
fn restoration_retry() {
    for rejected in [vec![BUTTONS_OFF], vec![SGR_OFF], vec![BUTTONS_OFF, SGR_OFF]] {
        let mut output = Fault::default();
        let mut reporting = Reporting::default();
        reporting.set(&mut output, true).unwrap();
        output.reject = rejected.clone();
        let error = reporting.set(&mut output, false).unwrap_err().to_string();
        assert!(error.contains("disable"));
        assert_eq!(&output.attempts[2..], &[BUTTONS_OFF, SGR_OFF]);
        output.attempts.clear();
        reporting.set(&mut output, false).unwrap();
        assert_eq!(output.attempts, rejected);
        let before = output.bytes.clone();
        reporting.set(&mut output, false).unwrap();
        assert_eq!(output.bytes, before);
    }
}

// Failed acquisition plus failed rollback retains both original and cleanup
// diagnostics. Later cleanup can still release every obligation exactly once.
#[test]
fn failed_rollback() {
    let mut output = Fault {
        reject: vec![BUTTONS_ON, BUTTONS_OFF, SGR_OFF],
        ..Fault::default()
    };
    let mut reporting = Reporting::default();
    let error = reporting.set(&mut output, true).unwrap_err().to_string();
    for context in ["enable mouse button", "disable mouse button", "disable SGR"] {
        assert!(error.contains(context), "{error}");
    }
    reporting.set(&mut output, false).unwrap();
    assert!(output.bytes.ends_with(&[BUTTONS_OFF, SGR_OFF].concat()));
}
